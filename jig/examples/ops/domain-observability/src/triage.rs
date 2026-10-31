//! Observability owns the triage result contract (examples.md, 5 and 7.1).
use alloc::boxed::Box;
use skein_lib::List;

/// The closed result shape is handed to its owner, without its explanation.
#[must_use]
pub(crate) fn watch_result(child: u64, result: &ResultKind, limits: &crate::Limits) -> crate::WatchMessage {
    let verdict = match result {
        ResultKind::Verdict { code, fields } if *code == limits.vocabulary.noise_verdict && fields.is_empty() => {
            Some(crate::TriageVerdict::Noise)
        }
        ResultKind::Verdict { code, fields } if *code == limits.vocabulary.incident_verdict => {
            triage_fields(fields, limits)
        }
        ResultKind::Verdict { .. } => None,
        ResultKind::Ended => return crate::WatchMessage::Ended { child },
    };
    match verdict {
        Some(verdict) => crate::WatchMessage::Triage { child, verdict },
        None => crate::WatchMessage::InvalidTriage { child },
    }
}

// Ops's declared machine-field vocabulary. These are explicit result values,
// never task numbers derived from report text or from the alerts they cover.
fn triage_fields(fields: &[ResultField], limits: &crate::Limits) -> Option<crate::TriageVerdict> {
    if fields.len() != 4 {
        return None;
    }
    let mut alerts = None;
    let mut environment = None;
    let mut service = None;
    let mut severity = None;
    for field in fields {
        match field.name.as_ref() {
            b"alerts" if alerts.is_none() => {
                let mut numbers = List::with_capacity(limits.alerts_per_batch);
                let mut start = 0;
                for (end, byte) in field.value.iter().enumerate() {
                    if *byte == b' ' {
                        numbers.push(skein_lib::bytes::decimal(field.value.get(start..end)?)?).ok()?;
                        start = end.checked_add(1)?;
                    }
                }
                numbers.push(skein_lib::bytes::decimal(field.value.get(start..)?)?).ok()?;
                alerts = Some(numbers.into_boxed());
            }
            b"environment" if environment.is_none() => environment = Some(field.value.clone()),
            b"service" if service.is_none() => service = Some(field.value.clone()),
            b"severity" if severity.is_none() => {
                severity = Some(u8::try_from(skein_lib::bytes::decimal(&field.value)?).ok()?);
            }
            _ => return None,
        }
    }
    let environment = environment?;
    let service = service?;
    if environment.is_empty()
        || service.is_empty()
        || u32::try_from(environment.len()).ok()? > limits.name_bytes
        || u32::try_from(service.len()).ok()? > limits.name_bytes
    {
        return None;
    }
    Some(crate::TriageVerdict::Incident {
        alerts: alerts?,
        service: crate::Service::new(environment, service),
        severity: severity?,
    })
}

/// Classify the watch's offered inbox and decode only its delegate results.
#[must_use]
pub fn watch_messages(inbox: Box<[WatchInput]>, limits: &crate::Limits) -> Box<[crate::WatchMessage]> {
    let mut messages = List::with_capacity(u32::try_from(inbox.len()).expect("offered inbox bound"));
    for input in inbox {
        let message = match input {
            WatchInput::News => Some(crate::WatchMessage::News),
            WatchInput::Result { child: Some(child), result } => Some(watch_result(child, &result, limits)),
            WatchInput::Result { child: None, .. } => None,
        };
        if let Some(message) = message {
            messages.push(message).expect("one message per word");
        }
    }
    messages.into_boxed()
}

/// The watch's configured spec carrier (examples.md, 5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WatchSpec {
    pub service: crate::Service,
    pub template: u16,
}

/// Decode the watch's service and template without inferring either.
#[must_use]
pub fn watch_spec(spec: &Spec, vocabulary: &crate::Vocabulary) -> Option<WatchSpec> {
    let mut environment = None;
    let mut service = None;
    let mut template = None;
    for parameter in &spec.parameters {
        match parameter {
            Parameter::Bytes { name, value } if *name == vocabulary.environment_parameter => {
                environment = Some(value.clone());
            }
            Parameter::Bytes { name, value } if *name == vocabulary.service_parameter => {
                service = Some(value.clone());
            }
            Parameter::Number { name, value } if *name == vocabulary.watch_template_parameter => {
                template = u16::try_from(*value).ok();
            }
            Parameter::Bytes { .. } | Parameter::Number { .. } | Parameter::Resource => {}
        }
    }
    Some(WatchSpec { service: crate::Service::new(environment?, service?), template: template? })
}

/// The watch's input values, translated by its root (domain/root.md, 7).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WatchInput {
    News,
    Result { child: Option<u64>, result: ResultKind },
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResultKind {
    Verdict { code: u32, fields: Box<[ResultField]> },
    Ended,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultField {
    pub name: Box<[u8]>,
    pub value: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Spec {
    pub parameters: Box<[Parameter]>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Parameter {
    Bytes { name: u32, value: Box<[u8]> },
    Number { name: u32, value: u64 },
    Resource,
}
