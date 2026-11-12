//! Subscription rows in document order (client/pages/team.md, 1, What wakes it).
use jig_client::contract as c;
use skein_lib::{Duration, List, Wall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interest {
    Timer { at: Wall, period: Option<Duration>, wakes: bool },
    Task { target: u64, held: bool, result: bool },
    Topics { connector: u16, count: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row {
    pub interest: Interest,
    pub class: c::WakeRule,
}

#[must_use]
pub fn rows(subscriptions: &c::Subscriptions, policy: &c::WakePolicy) -> List<Row> {
    let mut rows: List<Row> = List::with_capacity(subscriptions.entries.len());
    for subscription in &subscriptions.entries {
        let (interest, class) = match subscription.kind {
            c::SubscriptionKind::Timer { at, period } => {
                (Interest::Timer { at, period, wakes: policy.timers }, c::WakeRule::Never)
            }
            c::SubscriptionKind::Task { target, held, result } => {
                (Interest::Task { target, held, result }, policy.notices)
            }
            c::SubscriptionKind::Topic { connector, .. } => {
                let mut grouped = false;
                for row in rows.iter_mut() {
                    match &mut row.interest {
                        Interest::Topics { connector: existing, count } if *existing == connector => {
                            *count = count.checked_add(1).expect("topics fit subscriptions");
                            grouped = true;
                            break;
                        }
                        Interest::Timer { .. } | Interest::Task { .. } | Interest::Topics { .. } => {}
                    }
                }
                if grouped {
                    continue;
                }
                (Interest::Topics { connector, count: 1 }, policy.news)
            }
        };
        rows.push(Row { interest, class }).expect("no more rows than subscriptions");
    }
    rows
}
