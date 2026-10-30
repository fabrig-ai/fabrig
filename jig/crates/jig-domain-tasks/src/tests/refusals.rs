use super::revisions::{Harness, task};
use crate::{Event, MessageKind, Party, Request};
use alloc::boxed::Box;
use skein_lib::Wall;

fn refused_news(number: u64) -> crate::Word {
    crate::Word {
        number,
        from: Party::Task(1),
        kind: MessageKind::News { subscription: 10, class: crate::NewsClass::Wakes },
        words: Box::new([]),
        at: Wall::EPOCH,
        hits: 1,
        eligible: false,
    }
}

#[test]
fn merging_refused_news_retires_its_refusal_and_relays_the_new_hint() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let row = &mut crate::domain::task_mut(&mut h.domain, 1).expect("task").record;
    row.attempt = 1;
    row.phase = crate::Phase::Active(crate::Active::Claimed { attempt: 1 });
    row.last_message = 1;
    row.inbox = Box::new([refused_news(1)]);
    row.refused = Box::new([1]);
    row.subscriptions = Box::new([crate::Subscription {
        number: 10,
        kind: crate::SubscriptionKind::Topic { connector: 0, topic: 10 },
    }]);
    let out = h.send(Event::Notice { task: 1, word: refused_news(2) });
    assert!(h.domain.task(1).expect("task").refused.is_empty());
    let mut relayed = false;
    for request in &out {
        if let Request::Relay { word, .. } = request {
            relayed |= word.number == 2;
        }
    }
    assert!(relayed);
}

#[test]
fn an_answers_refusals_are_kept_without_a_turn_or_charge() {
    let mut h = Harness::new();
    h.make([task(1, &[])].into());
    let row = &mut crate::domain::task_mut(&mut h.domain, 1).expect("task").record;
    row.attempt = 1;
    row.phase = crate::Phase::Active(crate::Active::Claimed { attempt: 1 });
    row.last_message = 1;
    row.inbox = Box::new([refused_news(1)]);
    let out = h.send(Event::Refusals { task: 1, attempt: 1, refused: Box::new([1]), offered: Some(1) });
    let row = h.domain.task(1).expect("task");
    assert_eq!(row.refused.as_ref(), &[1]);
    assert_eq!(row.inbox.len(), 1);
    assert_eq!((row.turn, row.run_spent), (0, 0));
    let mut kept = false;
    for request in &out {
        if let Request::Save { record: crate::Stored::Live(row) } = request {
            kept |= row.refused.as_ref() == [1];
        }
    }
    assert!(kept, "answer refusal row is durable");
}

#[test]
fn only_live_batch_counts_name_freeing_room() {
    use crate::{Capacity, CapacityAction, CapacityCause, LiveRoom, Problem, Refusal};
    let batch = CapacityAction::Batch { first: Some(2), members: 1 };
    for why in [Refusal::Live, Refusal::Project, Refusal::Busy, Refusal::Tree, Refusal::AuthorityShape] {
        for cause in [
            CapacityCause::Tasks { limit: 4 },
            CapacityCause::ProjectTasks { limit: 2 },
            CapacityCause::Tree { limit: 8 },
            CapacityCause::CreationFlights { limit: 1 },
        ] {
            for action in [batch, CapacityAction::Release, CapacityAction::Proposal { number: 1 }] {
                for project in [None, Some(7)] {
                    let expected = match (&why, cause, action, project) {
                        (Refusal::Live, CapacityCause::Tasks { .. }, CapacityAction::Batch { .. }, _) => {
                            Some(LiveRoom::Deployment)
                        }
                        (
                            Refusal::Project,
                            CapacityCause::ProjectTasks { .. },
                            CapacityAction::Batch { .. },
                            Some(p),
                        ) => Some(LiveRoom::Project(p)),
                        _ => None,
                    };
                    let capacity = Capacity { project, by: Party::Person(9), action, cause };
                    let problem = Problem::capacity(None, why.clone(), capacity);
                    assert_eq!(problem.freeing_count(), expected, "{problem:?}");
                }
            }
        }
        assert_eq!(Problem::new(None, why).freeing_count(), None, "bare refusals name no room");
    }
}
