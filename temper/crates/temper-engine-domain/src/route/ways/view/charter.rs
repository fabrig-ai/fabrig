//! A charter card reads its order and wake policy (temper/docs/design/client/pages/chat/conversation.md, 3).
use super::{fact, not_drawn};
use crate::{Domain, ways};
use alloc::boxed::Box;
use skein_lib::{Decimal, bytes};

pub(super) fn draw(domain: &Domain, project: u32, party: u64, number: u32) -> ways::View {
    let Some(charter) = domain.core.current.charter(number) else {
        return not_drawn(&[b"no charter ", Decimal::of(u64::from(number)).as_bytes(), b"."]);
    };
    let inputs = super::super::inputs(domain, project, party, &[], None, None);
    let path = ways::derive(&inputs);
    let mut name = bytes::concat(&[b"charter ", Decimal::of(u64::from(number)).as_bytes()]);
    for stop in &path.stops {
        let named = match stop.opens {
            ways::Opens::Charter(charter) => charter == number,
            ways::Opens::Gate { .. } => stop.step == ways::Step::Review && inputs.reviewer == number,
            ways::Opens::Nothing
            | ways::Opens::Rule { .. }
            | ways::Opens::Tier(_)
            | ways::Opens::Queue
            | ways::Opens::Turn => false,
        };
        if named {
            name.clone_from(&stop.card.name);
            break;
        }
    }
    let policy = &charter.policy;
    ways::View::Charter {
        charter: number,
        card: ways::Card {
            who: b"an agent".as_slice().into(),
            name,
            facts: Box::new([
                fact(b"Models", &models(domain, charter)),
                fact(
                    b"Can",
                    ways::can(
                        policy.inspect,
                        policy.modify,
                        policy.shell,
                        policy.agents,
                        policy.tools.0 & jig_domain_core::ToolFamilies::JIG.settings != 0,
                    ),
                ),
                fact(b"Wakes", &wakes(charter.wake)),
                ways::Fact {
                    label: b"Asked to".as_slice().into(),
                    value: ways::first_sentence(&policy.instructions, ways::ASKED),
                },
            ]),
        },
    }
}

fn models(domain: &Domain, charter: &jig_domain_core::Charter) -> Box<[u8]> {
    let mut words = Box::from(b"".as_slice());
    for model in core::iter::once(&charter.policy.model).chain(charter.policy.alternatives.iter()) {
        let mut name = model.name.clone();
        for (at, configured) in domain.core.settings.models.iter().enumerate() {
            if configured.name == model.name {
                let number = u32::try_from(at).expect("model index").checked_add(1).expect("model number");
                match domain.core.current.value(jig_domain_core::settings::Setting::Model(number)) {
                    Some(jig_domain_core::settings::Value::Model(row)) => {
                        if let Some(shown) = &row.shown {
                            name.clone_from(shown);
                        }
                    }
                    Some(
                        jig_domain_core::settings::Value::Overlay(_) | jig_domain_core::settings::Value::Charter(_)
                        | jig_domain_core::settings::Value::Template(_)
                        | jig_domain_core::settings::Value::Policy(_)
                        | jig_domain_core::settings::Value::Rules(_)
                        | jig_domain_core::settings::Value::Application(_),
                    )
                    | None => {}
                }
            }
        }
        words = bytes::concat(&[&words, if words.is_empty() { b"" } else { b", " }, &name]);
        if let Some(effort) = model.default_effort() {
            words = bytes::concat(&[&words, " · ".as_bytes(), effort]);
        }
    }
    if charter.rest {
        words = bytes::concat(&[&words, b", then the rest"]);
    }
    words
}

fn wakes(policy: jig_domain_tasks::WakePolicy) -> Box<[u8]> {
    use jig_domain_tasks::ResultsWake;
    if policy == jig_domain_tasks::WakePolicy::DEFAULT {
        return b"at once on words, results, failures and questions".as_slice().into();
    }
    let mut words = bytes::concat(&[
        b"words: ",
        &wake_rule(policy.words),
        b"; notices: ",
        &wake_rule(policy.notices),
        b"; news: ",
        &wake_rule(policy.news),
    ]);
    let results = match policy.results {
        ResultsWake::Never => b"never".as_slice(),
        ResultsWake::Each => b"each".as_slice(),
        ResultsWake::LastOrFailure => b"last or failure".as_slice(),
    };
    words = bytes::concat(&[&words, b"; results: ", results]);
    for (name, enabled) in [
        (b"questions".as_slice(), policy.questions),
        (b"answers".as_slice(), policy.answers),
        (b"timers".as_slice(), policy.timers),
    ] {
        words = bytes::concat(&[&words, b"; ", name, if enabled { b": at once" } else { b": never" }]);
    }
    words
}

fn wake_rule(rule: jig_domain_tasks::WakeRule) -> Box<[u8]> {
    use jig_domain_tasks::WakeRule;
    match rule {
        WakeRule::Never => b"never".as_slice().into(),
        WakeRule::Immediate => b"at once".as_slice().into(),
        WakeRule::Batch { count, age } => bytes::concat(&[
            b"batch ",
            Decimal::of(u64::from(count)).as_bytes(),
            b" or after ",
            Decimal::of(age.as_nanos()).as_bytes(),
            b" ns",
        ]),
        WakeRule::Paced { span } => bytes::concat(&[b"paced every ", Decimal::of(span.as_nanos()).as_bytes(), b" ns"]),
    }
}
