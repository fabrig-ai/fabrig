//! Selected restores keep their obligations over a job-made copy (protocol/store/opening.md, 2).
use jig_protocol_store::{Check, OpenRefusal};
use jig_store_world::{
    patches::{self, BASE, Records, SETS},
    root,
};

#[test]
fn selected_open_defers_a_missing_ended_owner_and_refusing_start_keeps_the_source() {
    use jig_domain_core::{Census, Key, Record, RestoreRange};
    use jig_protocol_store::{Restore, restore::Item};
    let position = Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE + 1, party: 1, through: 1 });
    let mut world = Records::story().committed_with(913, BASE, patches::limits(), core::slice::from_ref(&position));
    let before = patches::source(&mut world);
    world.owner_sets(SETS);
    world.open().expect("selected open leaves the upgraded copy pending");
    assert_eq!(world.start_open_failure(), None);
    assert_eq!(world.active(), b"1\n");
    let page = world
        .restore_page(
            Restore::Jig(Box::new([RestoreRange::Census(Census { questions: Box::new([]), results: Box::new([]) })])),
            None,
            8,
            65_536,
        )
        .expect("typed census");
    let [Item::MissingLink { record, target }] = page.items.as_ref() else { panic!("missing owner: {:?}", page.items) };
    assert_eq!(*record, position.key());
    assert_eq!(*target, Key::Tasks(jig_domain_tasks::Key::Ended(root::LIVE + 1)));
    patches::kept(&before, &patches::source(&mut world), &"no switch before link");
    // A root refusing the deferred obligation makes no Switch. A later
    // process opens another pending copy and hands up the same obligation.
    world.crash(914);
    world.open().expect("open defers links on every start");
    assert_eq!(world.start_open_failure(), None);
    assert_eq!(world.active(), b"1\n");
    let page = world
        .restore_page(
            Restore::Jig(Box::new([RestoreRange::Census(Census { questions: Box::new([]), results: Box::new([]) })])),
            None,
            8,
            65_536,
        )
        .expect("typed census after crash");
    assert!(matches!(page.items.as_ref(), [Item::MissingLink { record, target }]
        if *record == position.key() && *target == Key::Tasks(jig_domain_tasks::Key::Ended(root::LIVE + 1))));
    patches::kept(&before, &patches::source(&mut world), &"deferred link after crash");
}

#[test]
fn selected_open_still_refuses_an_undecodable_live_row_before_switch() {
    use jig_protocol_store::restore::{Cause, DecodeFailure, OpenFailure};
    let mut live = root::live_task(root::LIVE);
    let jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)) = &mut live else { panic!("live") };
    task.dependencies = Box::new([root::LIVE + 1]);
    let key = jig_protocol_store::keys::encode(&live.key(), 64).expect("task key");
    let mut world = Records::story().committed_with(915, BASE, patches::limits(), &[live]);
    let before = patches::source(&mut world);
    let mut lowered = patches::limits();
    lowered.core.tasks.dependencies = 0;
    world.owner_sets(SETS);
    world.restart_with_limits(916, lowered);
    assert_eq!(world.open(), None);
    assert_eq!(world.refusal(), Some(&OpenRefusal::Check { why: Check::Load { key: key.clone() } }));
    let Some(OpenFailure::Data(failure)) = world.start_open_failure() else { panic!("typed row failure") };
    assert_eq!(failure.cause, Cause::Decode(DecodeFailure::Value));
    assert_eq!(failure.row.as_ref().expect("live row").key, key);
    assert_eq!(world.active(), b"1\n");
    patches::kept(&before, &patches::source(&mut world), &"selected decode refusal");
}

#[test]
fn selected_open_preserves_present_ended_owner_corruption_for_the_census() {
    use jig_domain_core::{Census, Record, RestoreRange};
    use jig_protocol_store::{
        Restore,
        restore::{Cause, DecodeFailure, JoinFailure},
    };
    let mut ended = root::live_task(root::LIVE + 1);
    let Record::Tasks(jig_domain_tasks::Stored::Live(mut task)) = ended else { panic!("task") };
    task.spec.words = Box::from(b"past lowered limit".as_slice());
    task.phase = jig_domain_tasks::Phase::Ended(jig_domain_tasks::Ending::Done(jig_domain_tasks::TaskResult::Report {
        words: Box::new([]),
    }));
    task.ended_at = Some(skein_lib::Wall::EPOCH);
    ended = Record::Tasks(jig_domain_tasks::Stored::Ended(task));
    let position = Record::People(jig_domain_people::Stored::EndedRead { task: root::LIVE + 1, party: 1, through: 1 });
    let mut world = Records::story().committed_with(917, BASE, patches::limits(), &[ended, position]);
    let mut lowered = patches::limits();
    lowered.core.tasks.spec_bytes = 1;
    world.owner_sets(SETS);
    world.restart_with_limits(918, lowered);
    world.open().expect("owner decoding belongs to census");
    let failure = world
        .restore_page(
            Restore::Jig(Box::new([RestoreRange::Census(Census { questions: Box::new([]), results: Box::new([]) })])),
            None,
            8,
            65_536,
        )
        .expect_err("present owner is corrupt under lowered codec");
    assert!(matches!(failure.cause, Cause::Join { failure: JoinFailure::Decode(DecodeFailure::Value), .. }));
    assert_eq!(world.active(), b"1\n");
}
