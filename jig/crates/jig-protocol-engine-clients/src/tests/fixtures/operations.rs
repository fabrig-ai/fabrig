use super::*;

pub fn answer(on: w::AnswerOn) -> w::Ask {
    w::Ask::Answer(w::Answer { project: 3, on })
}
pub fn decide(on: w::DecideOn) -> w::Ask {
    w::Ask::Decide(w::Decide { project: 3, on })
}
pub fn empty_amendment(tracked: Option<w::Tracking>) -> w::Amendment {
    w::Amendment {
        llm: None,
        addressed: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        instructions: None,
        tracked,
    }
}
pub fn priority_amend() -> w::Ask {
    let tracked = w::Tracking::Tracked(w::TrackingTracked { priority: u32::MAX });
    w::Ask::Amend(Box::new(w::Amend {
        project: 3,
        task: 8,
        revision: None,
        change: empty_amendment(Some(tracked)),
        reason: Box::new([]),
    }))
}
pub fn template_change() -> w::Ask {
    let value = w::TemplateOverride {
        template: 2,
        executor: Some(w::TemplateExecutor::Chosen),
        budget: Some(w::TemplateBudget::Given),
        deadline: Some(w::TemplateDeadline::None),
        priority: Some(u32::MAX),
    };
    let mut overrides = List::with_capacity(1);
    overrides.push(value).expect("one override");
    let edit = w::PolicyEdit::Templates(w::PolicyEditTemplates { overrides });
    w::Ask::ChangePolicy(w::ChangePolicy { project: 3, edit })
}

/// An amendment retaining every field, with the supplied explanation.
#[must_use]
pub fn person_unchanged(reason: &[u8]) -> jig_domain_people::Amendment {
    jig_domain_people::Amendment {
        tracked: None,
        addressed: None,
        instructions: None,
        spec: None,
        wake: None,
        dependencies: None,
        authority: None,
        llm: None,
        reason: reason.into(),
    }
}

pub fn compound(project: u32, values: &[w::Operation]) -> w::Ask {
    let mut encoded = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        let mut out = Writer::new(
            usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
        );
        value.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
        encoded.push(out.finish()).unwrap();
    }
    w::Ask::Compound(w::Compound { project, operations: pieces(encoded.as_slice()) })
}

pub fn encode_ask(ask: &w::Ask) -> Box<[u8]> {
    let mut out = Writer::new(skein_lib::length::usize_of(
        ask.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    ask.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("bounded ask");
    out.finish()
}
pub fn keyed(owner: w::Owner, version: u32, request: Box<[u8]>) -> Box<[u8]> {
    let request = match owner {
        w::Owner::Jig => request,
        w::Owner::Family(_) => family_request(request),
    };
    let value = w::Keyed { key: [5; 16], owner, version, request, value: None };
    let mut out = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("bounded keyed");
    out.finish()
}

pub fn pieces(values: &[Box<[u8]>]) -> Box<[u8]> {
    let mut size = 0_usize;
    for value in values {
        size = size.checked_add(value.len()).unwrap().checked_add(4).unwrap();
    }
    let mut out = Writer::new(size);
    for value in values {
        skein_codec::piece::write(&mut out, value).unwrap();
    }
    out.finish()
}
