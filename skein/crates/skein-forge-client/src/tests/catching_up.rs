use super::fixtures::{Harness, keep as kept, recovery};
use super::{REPO, configured};
use crate::{Domain, Event, RecoveryClock, Stored, Writer};
use skein_forge::api::{Answer, Op, Read, Repository};

#[test]
fn fresh_owed_counts_only_each_forges_pending_resources_and_listings() {
    let limits = crate::Limits { rate: 100, reserve: 0, ..super::LIMITS };
    let mut h = Harness::new(limits);
    h.domain = configured(&limits);
    let other = Repository { forge: 3, ..REPO };
    for repository in [REPO, other] {
        let mut live = recovery::live();
        live.watch = kept::watch(kept::issue(), false);
        live.watch.resource.repository = repository;
        h.step(Event::Restore { record: Stored::Live(live) });
        h.step(Event::Restore {
            record: Stored::Repository(crate::RepositoryRecord { repository, ..recovery::repository() }),
        });
    }
    assert_eq!(h.domain.fresh_owed(REPO.forge), 0);
    h.step(Event::Restored { clock: RecoveryClock::Monotonic });
    assert_eq!(h.domain.fresh_owed(REPO.forge), 0);
    h.step(Event::ReadAfresh);
    assert_eq!(h.domain.fresh_owed(REPO.forge), 2);
    assert_eq!(h.domain.fresh_owed(other.forge), 2);
    assert_eq!(h.domain.fresh_owed(4), 0);
    let mut owed = [2_u32, 2_u32];
    for _ in 0_u32..4 {
        let (read, repository) = h.sent();
        let answer = match read.op {
            Op::Read(Read::Items { .. }) => {
                Answer::Items { cut: false, items: Box::new([]), more: false, now: super::at(recovery::START_AT) }
            }
            Op::Read(Read::Item { .. }) => recovery::empty_item(),
            Op::Read(_) | Op::Write(_) => panic!("unexpected fresh read: {:?}", read.op),
        };
        h.answer(&read, 1, Ok(answer));
        let index = usize::from(repository.forge != REPO.forge);
        owed[index] -= 1;
        assert_eq!(h.domain.fresh_owed(REPO.forge), owed[0]);
        assert_eq!(h.domain.fresh_owed(other.forge), owed[1]);
    }
    assert_eq!(owed, [0, 0]);
    assert_eq!(h.domain.fresh_owed(REPO.forge), 0);
}

#[test]
fn finds_owed_counts_each_forges_attempt_until_its_first_look() {
    let mut h = kept::Harness::new();
    h.domain = Domain::configured(
        &h.env.limits,
        0,
        crate::Config {
            namespace: Box::from(&b"kit"[..]),
            writers: Box::new([Writer { forge: 2, author: 1 }, Writer { forge: 3, author: 1 }]),
        },
    )
    .unwrap();
    assert_eq!(h.domain.finds_owed(2), 0);
    h.event(Event::Restored { clock: RecoveryClock::Monotonic });
    let saved = recovery::uncertain_comment().store.get(&1).unwrap().clone();
    for (number, forge) in [(1, 2), (2, 3)] {
        let mut entry = saved.clone();
        entry.number = number;
        entry.repository.forge = forge;
        h.event(Event::Make { entry });
    }
    assert_eq!(h.domain.finds_owed(2), 1);
    assert_eq!(h.domain.finds_owed(3), 1);
    assert_eq!(h.domain.finds_owed(4), 0);
    h.event(Event::SettleOutbox);
    let find = h.one();
    h.answer(find, recovery::empty_item());
    assert_eq!(h.domain.finds_owed(2), 0);
    assert_eq!(h.domain.finds_owed(3), 1);
    let find = h.one();
    h.answer(find, recovery::empty_item());
    assert_eq!(h.domain.finds_owed(3), 0);
    assert_eq!(h.outbox_done, 1);
}
