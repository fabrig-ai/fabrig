//! Names and bounded words shared by person answers (domain/agent.md, 8.4.1).
use super::super::join;
use alloc::boxed::Box;
use jig_domain_core::{documents as d, settings::CatalogueModel};
use jig_domain_tasks as tasks;
use skein_lib::{Decimal, Wall, bytes};
use temper_engine_domain::SeeNames;

pub(super) fn number(value: u64) -> Box<[u8]> {
    Decimal::of(value).as_bytes().into()
}
pub(super) fn short(value: &[u8], cut: bool) -> Box<[u8]> {
    let end = bytes::find(value, b"\n").unwrap_or(value.len()).min(bytes::find(value, b"\r").unwrap_or(value.len()));
    let first = value.get(..end).expect("first line");
    let cut = cut || first.len() < value.len() || first.len() > 160;
    let room = if cut { 157 } else { 160 };
    let prefix = bytes::utf8_prefix(first, room);
    join(&[prefix, if cut { "…".as_bytes() } else { b"" }])
}
pub(super) fn person(number_value: u64, names: &SeeNames) -> Box<[u8]> {
    for name in &names.people {
        if name.number == number_value && !name.name.is_empty() {
            return short(&name.name, false);
        }
    }
    join(&[b"person ", &number(number_value)])
}
pub(super) fn task_name(noun: &[u8], chip: &d::Chip) -> Box<[u8]> {
    join(&[noun, b" ", &number(chip.number), b", ", &short(&chip.title.bytes, chip.title.cut)])
}
pub(super) fn kind(label: u32, contract: Option<&d::ContractLine>, executor: tasks::Executor) -> Box<[u8]> {
    match contract {
        Some(d::ContractLine::Change { .. }) => return b"a change".as_slice().into(),
        Some(
            d::ContractLine::Job { .. }
            | d::ContractLine::Approval { .. }
            | d::ContractLine::Report { .. }
            | d::ContractLine::Verdict { .. },
        )
        | None => {}
    }
    match executor {
        tasks::Executor::Procedure { connector: temper_client_protocol_wire::FORGE, code: 2 } => {
            return b"a change".as_slice().into();
        }
        tasks::Executor::Procedure { .. }
        | tasks::Executor::Agent { .. }
        | tasks::Executor::Person(_)
        | tasks::Executor::Job { .. } => {}
    }
    match label {
        1 => b"a chat".as_slice().into(),
        2 => b"a goal".as_slice().into(),
        other => join(&[b"label ", &number(u64::from(other))]),
    }
}
pub(super) fn executor(
    value: tasks::Executor,
    llm: Option<&tasks::LlmChoice>,
    names: &SeeNames,
    catalogue: &[CatalogueModel],
) -> Box<[u8]> {
    match value {
        tasks::Executor::Agent { charter } => {
            let mut text = join(&[b"an agent, charter ", &number(u64::from(charter))]);
            for name in &names.charters {
                if name.number == u64::from(charter) && !name.name.is_empty() {
                    text = join(&[&text, b", ", &short(&name.name, false)]);
                    break;
                }
            }
            if let Some(llm) = llm {
                let wanted = u32::from(llm.main.model).checked_add(1).expect("model index fits");
                for model in catalogue {
                    if model.number == wanted {
                        text = join(&[&text, b", on ", &short(&model.model.name, false)]);
                        break;
                    }
                }
            }
            text
        }
        tasks::Executor::Procedure { .. } => b"a procedure".as_slice().into(),
        tasks::Executor::Person(address) => person_address(address, names),
        tasks::Executor::Job { kind } => join(&[b"a job, kind ", &number(u64::from(kind))]),
    }
}
fn person_address(address: tasks::PersonAddress, names: &SeeNames) -> Box<[u8]> {
    match address {
        tasks::PersonAddress::Person(n) => person(n, names),
        tasks::PersonAddress::Role(n) => join(&[b"a person with ", &role(jig_domain_people::Role::Policy { role: n })]),
    }
}
pub(super) fn role(value: jig_domain_people::Role) -> Box<[u8]> {
    use jig_domain_people::Role;
    match value {
        Role::Owner => b"owner".as_slice().into(),
        Role::Maintainer => b"maintainer".as_slice().into(),
        Role::Member => b"member".as_slice().into(),
        Role::Observer => b"observer".as_slice().into(),
        Role::Policy { role } => join(&[b"role ", &number(u64::from(role))]),
    }
}
pub(super) fn state(chip: &d::Chip, waits: &d::WaitsOn, names: &SeeNames) -> Box<[u8]> {
    let phase = match &chip.phase {
        d::PhaseLine::Waiting { .. } => b"waiting".as_slice(),
        d::PhaseLine::Active(active) => match active {
            tasks::Active::Idle => b"idle".as_slice(),
            tasks::Active::Due => b"ready",
            tasks::Active::Preparing => b"preparing",
            tasks::Active::Claimed { .. } => b"starting",
            tasks::Active::Running { .. } => b"working",
            tasks::Active::BackingOff { .. } => b"waiting to retry",
        },
        d::PhaseLine::Person { .. } => b"waiting for a person",
        d::PhaseLine::Closing(_) => b"closing",
        d::PhaseLine::Held { .. } => b"held",
        d::PhaseLine::Ended => match &chip.details.ending {
            Some(d::Ending::Done(_)) => b"done".as_slice(),
            Some(d::Ending::Failed(_)) => b"failed",
            Some(d::Ending::Cancelled { .. }) => b"cancelled",
            None => b"ended",
        },
    };
    match waits {
        d::WaitsOn::Nothing => phase.into(),
        d::WaitsOn::News => join(&[phase, b", waiting for news"]),
        d::WaitsOn::Delegates => join(&[phase, b", waiting for delegates"]),
        d::WaitsOn::Dependencies { count, .. } => {
            join(&[phase, b", waiting on ", &number(u64::from(*count)), b" dependencies"])
        }
        d::WaitsOn::Party { whom, .. } => {
            let who = match whom {
                jig_domain_people::Whom::Person(n) => person(*n, names),
                jig_domain_people::Whom::Role { role: n, .. } => role(jig_domain_people::Role::Policy { role: *n }),
            };
            join(&[phase, b", waiting on ", &who])
        }
    }
}
pub(super) fn time(wall: Wall) -> Box<[u8]> {
    let date = wall.civil();
    let minutes = wall.as_secs().div_euclid(60);
    join(&[
        &number(u64::from(date.year)),
        b"-",
        &two(u64::from(date.month)),
        b"-",
        &two(u64::from(date.day)),
        b" ",
        &two(minutes.div_euclid(60).rem_euclid(24)),
        b":",
        &two(minutes.rem_euclid(60)),
    ])
}
fn two(value: u64) -> Box<[u8]> {
    join(&[if value < 10 { b"0" } else { b"" }, &number(value)])
}

pub(super) fn money(cents: u64) -> Box<[u8]> {
    join(&[b"$", &number(cents.div_euclid(100)), b".", &two(cents.rem_euclid(100))])
}
pub(super) fn spent(numbers: &tasks::Numbers) -> Box<[u8]> {
    money(numbers.spent.checked_add(numbers.spent_below).expect("spend bounded by budget"))
}
pub(super) fn hold(value: tasks::Hold) -> Box<[u8]> {
    let words = match value {
        tasks::Hold::Exceeded(_) => b"a run limit was reached".as_slice(),
        tasks::Hold::Failures(_) => b"the retry allowance was exhausted",
        tasks::Hold::Stalled => b"the executor stopped making progress",
        tasks::Hold::Dependency(n) => return join(&[b"dependency task ", &number(n), b" did not succeed"]),
        tasks::Hold::Stopped => b"work was stopped",
        tasks::Hold::Placed { .. } => b"a hold was placed",
        tasks::Hold::Drift => b"the forge changed under the work",
        tasks::Hold::Effects | tasks::Hold::EffectFailed => b"an effect failed",
        tasks::Hold::Uncertain { .. } => b"an effect's outcome is uncertain",
        tasks::Hold::Budget => b"the budget is exhausted",
        tasks::Hold::Deadline => b"the deadline was reached",
        tasks::Hold::HoldsWaited => b"waiting for resources took too long",
        tasks::Hold::ResourceUnavailable => b"a resource is unavailable",
        tasks::Hold::ResourceAccess => b"the repository's access changed",
        tasks::Hold::PoolLost { .. } => b"the pool lost its allocation",
        tasks::Hold::Procedure => b"the procedure needs a decision",
        tasks::Hold::Circle => b"the work forms a circle of waits",
    };
    words.into()
}
pub(super) fn result(value: &d::Ending) -> Box<[u8]> {
    match value {
        d::Ending::Done(value) => join(&[b"done", &result_words(value)]),
        d::Ending::Failed(words) => join(&[b"failed: ", &short(&words.bytes, words.cut)]),
        d::Ending::Cancelled { reason, result, .. } => {
            let kept = match result {
                Some(value) => result_words(value),
                None => b"".as_slice().into(),
            };
            join(&[b"cancelled: ", &short(&reason.bytes, reason.cut), &kept])
        }
    }
}
fn result_words(value: &d::ResultValue) -> Box<[u8]> {
    let words = match value {
        d::ResultValue::Report(words)
        | d::ResultValue::Failure(words)
        | d::ResultValue::Verdict { words, .. }
        | d::ResultValue::Change { words, .. } => Some(words),
        d::ResultValue::Approved { words, .. } => words.as_ref(),
    };
    match words {
        Some(words) => join(&[b": ", &short(&words.bytes, words.cut)]),
        None => b"".as_slice().into(),
    }
}
