//! Infrastructure owns procedure parameters and resource holdings (examples.md, 3.2).
use crate::Vocabulary;
use alloc::boxed::Box;

/// Ops's procedure parameters are typed spec carriers, never root policy.
#[must_use]
fn spec_service(vocabulary: &Vocabulary, spec: &Spec) -> Option<crate::Service> {
    let mut environment = None;
    let mut name = None;
    for parameter in &spec.parameters {
        match parameter {
            Parameter::Bytes { name: position, value } if *position == vocabulary.environment_parameter => {
                environment = Some(value.clone());
            }
            Parameter::Bytes { name: position, value } if *position == vocabulary.name_parameter => {
                name = Some(value.clone());
            }
            Parameter::Bytes { .. } | Parameter::Number { .. } | Parameter::Resource => {}
        }
    }
    Some(crate::Service::new(environment?, name?))
}
#[must_use]
fn spec_number(spec: &Spec, number: u32) -> Option<u64> {
    for parameter in &spec.parameters {
        match parameter {
            Parameter::Number { name, value } => {
                if *name == number {
                    return Some(*value);
                }
            }
            Parameter::Bytes { .. } | Parameter::Resource => {}
        }
    }
    None
}
fn spec_environment(vocabulary: &Vocabulary, spec: &Spec) -> Option<crate::Environment> {
    let mut pool = None;
    let mut name = None;
    for parameter in &spec.parameters {
        match parameter {
            Parameter::Bytes { name: position, value } if *position == vocabulary.environment_parameter => {
                pool = Some(value.clone());
            }
            Parameter::Bytes { name: position, value } if *position == vocabulary.name_parameter => {
                name = Some(value.clone());
            }
            Parameter::Bytes { .. } | Parameter::Number { .. } | Parameter::Resource => {}
        }
    }
    Some(crate::Environment::new(pool?, name?))
}
#[must_use]
pub fn procedure(vocabulary: &Vocabulary, code: u32, spec: &Spec, deployment: [u8; 16]) -> Option<crate::Procedure> {
    if code == crate::PROVISION_CODE {
        return Some(crate::Procedure::Provision {
            environment: spec_environment(vocabulary, spec)?,
            until: spec_number(spec, vocabulary.environment_until_parameter)?,
            price: spec_number(spec, vocabulary.environment_price_parameter)?,
            deadline: spec_number(spec, vocabulary.environment_deadline_parameter)?,
        });
    }
    if code == crate::TEAR_DOWN_CODE {
        return Some(crate::Procedure::TearDown {
            environment: spec_environment(vocabulary, spec)?,
            created_by: crate::Key::procedure(
                deployment,
                spec_number(spec, vocabulary.creation_task_parameter)?,
                spec_number(spec, vocabulary.creation_purpose_parameter)?,
            ),
            deadline: spec_number(spec, vocabulary.environment_deadline_parameter)?,
        });
    }
    let service = spec_service(vocabulary, spec)?;
    match code {
        code if code == vocabulary.scale_procedure => Some(crate::Procedure::Scale {
            service,
            replicas: u32::try_from(spec_number(spec, vocabulary.replicas_parameter)?).ok()?,
            deadline: spec_number(spec, vocabulary.service_deadline_parameter)?,
        }),
        code if code == vocabulary.remediate_procedure => Some(crate::Procedure::Remediate {
            service,
            operation: spec_number(spec, vocabulary.operation_parameter)?,
            deadline: spec_number(spec, vocabulary.service_deadline_parameter)?,
        }),
        _ => None,
    }
}
#[must_use]
pub fn holdings(vocabulary: &Vocabulary, number: u16, executor: Executor, spec: &Spec) -> Option<Box<[Holding]>> {
    match executor {
        Executor::Procedure { connector, code } => {
            if connector == number {
                if code == crate::TEAR_DOWN_CODE {
                    return Some(Box::new([Holding::Write {
                        resource: crate::Resource::Environment(spec_environment(vocabulary, spec)?),
                        kind: vocabulary.environment_hold,
                    }]));
                }
                if code == crate::PROVISION_CODE {
                    let environment = spec_environment(vocabulary, spec)?;
                    return Some(Box::from([
                        Holding::Write {
                            resource: crate::Resource::Environment(environment.clone()),
                            kind: vocabulary.environment_hold,
                        },
                        Holding::Slot { pool: crate::Pool(environment.pool), kind: vocabulary.pool_slot_hold },
                    ]));
                }
            } else {
                return Some(Box::new([]));
            }
        }
        Executor::Other => return Some(Box::new([])),
    }
    match spec_service(vocabulary, spec) {
        Some(service) => Some(Box::new([Holding::Write {
            resource: crate::Resource::Service(service),
            kind: vocabulary.service_hold,
        }])),
        None => {
            if spec.parameters.is_empty() {
                Some(Box::new([]))
            } else {
                None
            }
        }
    }
}
/// Connector-local service/environment write and pool-slot kinds, also used at cold restore.
#[must_use]
pub fn hold_kinds(vocabulary: &Vocabulary) -> Box<[Kind]> {
    Box::new([
        Kind { kind: vocabulary.service_hold, hold: crate::Hold::ExclusiveWait },
        Kind { kind: vocabulary.environment_hold, hold: crate::Hold::ExclusiveWait },
        Kind { kind: vocabulary.pool_slot_hold, hold: crate::Hold::PooledWait },
    ])
}

/// The infrastructure connector's spec carrier (examples.md, 3.2).
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
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Executor {
    Procedure { connector: u16, code: u32 },
    Other,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Holding {
    Write { resource: crate::Resource, kind: u16 },
    Slot { pool: crate::Pool, kind: u16 },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Kind {
    pub kind: u16,
    pub hold: crate::Hold,
}
