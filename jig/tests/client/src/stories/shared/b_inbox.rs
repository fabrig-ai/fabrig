//! Inbox row order on each front (jig/docs/design/client/domain/testing.md, 4–5).
use super::{
    b_conversations::{WAIT, finish, sign_in},
    b_seed::{Facts, inbox_source, install},
};
use crate::{Front, Scripted, Test, World, harness::Seed, joint::engine::Driver};
use jig_client_domain::contract as c;
use jig_client_scripted_engine::Change;
use jig_fake_workers::{Call, Script};
use skein_fake_person::{Find, Step};
use skein_lib::{Duration, Time, Wall};
use skein_ui::Role;

const PARTY: u64 = 1;
const PROJECT: u32 = 1;
const PROPOSER: u64 = 1;
// Each proposed goal reserves a task number before the next conversation starts.
const SECOND_PROPOSER: u64 = 3;
const OLDER_RESULT: u64 = 5;
const NEWER_RESULT: u64 = 6;
const SPACING: Duration = Duration::from_millis(20);
// Four 50 ms heartbeats let the three staged conversation requests settle.
const SEED_SETTLE: Duration = Duration::from_millis(200);
const GROUP_ROWS: u32 = 2;
/// Two conversations propose goals and two complete with reports.
pub const CONVERSATIONS: u32 = 4;

#[must_use]
pub fn scripts() -> Vec<Box<[Script]>> {
    let proposal = Box::new([
        Script::Call(Call::ProposeGoal { words: b"First need".as_slice().into(), budget: 2, priority: 0 }),
        Script::Park,
    ]);
    let mut scripts: Vec<Box<[Script]>> = vec![proposal.clone(), proposal];
    scripts.extend(
        [b"Older result".as_slice(), b"Newer result".as_slice()]
            .into_iter()
            .map(|report| Box::new([Script::Finish { report: report.into() }]) as Box<[Script]>),
    );
    scripts
}
pub fn core(driver: &mut Driver, now: Time, wall: Wall) {
    use jig_fake_parties::{Act, Ask, Party, Role};
    super::b_scenarios::core(driver, now, wall, Facts::InboxOrder);
    driver.scripted_party(
        Party::new(
            PROJECT,
            Role::Owner,
            1_100_000,
            Box::new([
                Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
                Act::Pause { for_: SPACING },
                Act::Request { key: [83; 16], ask: Ask::Chat { words: b"Second need".as_slice().into() } },
                Act::Pause { for_: SPACING },
                Act::Request { key: [81; 16], ask: Ask::Chat { words: b"Older result".as_slice().into() } },
                Act::Pause { for_: SPACING },
                Act::Request { key: [82; 16], ask: Ask::Chat { words: b"Newer result".as_slice().into() } },
            ]),
        ),
        now,
        wall,
    );
}
pub fn scripted(front: &mut Scripted) {
    let mut frame = crate::stories::frame(PARTY);
    frame.needs.proposals = GROUP_ROWS;
    frame.needs_total = GROUP_ROWS;
    frame.updates.value = GROUP_ROWS;
    front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("frame fits");
    let needs = vec![need(front, PROPOSER, 1), need(front, SECOND_PROPOSER, 2)];
    inbox_source(front, PARTY, c::InboxReadGroup::Needs, needs);
    let updates = vec![update(front, NEWER_RESULT, 4), update(front, OLDER_RESULT, 3)];
    inbox_source(front, PARTY, c::InboxReadGroup::Updates, updates);
}
fn need(front: &mut Scripted, task: u64, position: u64) -> c::InboxEntry {
    let key = c::ProposalKey { proposer: c::Proposer::Task(task), number: position };
    let mut proposal = crate::stories::proposal();
    proposal.key = key;
    proposal.waiting.holder = c::HistoryHolder::Person(PARTY);
    proposal.waiting.since = Wall::from_nanos(position);
    proposal.funding.clear();
    install(front, c::ObjectKey::Proposal(key), c::ReadValue::Proposal(Box::new(proposal)));
    crate::stories::inbox_entry(
        c::ObjectKey::Proposal(key),
        PROJECT,
        c::InboxGroup::Proposals,
        position,
        Wall::from_nanos(position),
        None,
    )
}
fn update(front: &mut Scripted, task: u64, position: u64) -> c::InboxEntry {
    let mut chip = crate::stories::task().chip;
    chip.task = task;
    chip.phase = c::Phase::Done;
    chip.details = Box::new(crate::stories::owner_chip(c::ChipPhase::Ended, None));
    let value = c::ResultValue::Report(c::DocumentText::complete(crate::stories::text(b"Finished")));
    chip.details.details.ending = Some(c::ChipEnding::Done(value.clone()));
    chip.details.details.ended_at = Some(Wall::from_nanos(position));
    let result = c::Result {
        task: chip,
        ending: c::Ending::Done,
        value: Some(value),
        at: Some(Wall::from_nanos(position)),
        details: None,
        acts: crate::test::list([]),
    };
    install(front, c::ObjectKey::Result(task), c::ReadValue::Result(Box::new(result)));
    crate::stories::inbox_entry(
        c::ObjectKey::Result(task),
        PROJECT,
        c::InboxGroup::Results,
        position,
        Wall::from_nanos(position),
        None,
    )
}
/// jig/docs/design/domain/engine.md, 11.2.
pub fn a_person_s_inbox_lists_needs_oldest_first_and_updates_newest_first<F: Front<Test>>(world: &mut World<Test, F>) {
    world.seed(Seed::ChatProject { party: PARTY, project: PROJECT });
    world.seed(Seed::Project(Facts::InboxOrder));
    let client = world.client(PARTY, crate::stories::inbox());
    world.person(
        client,
        sign_in(vec![
            Step::Pause { span: SEED_SETTLE },
            Step::See { find: Find::named(Role::Region, b"Needs you"), within: WAIT },
            Step::See { find: Find::named(Role::Region, b"Updates"), within: WAIT },
            Step::Press { find: Find::named(Role::Button, b"All kinds") },
            Step::See { find: Find::text(b"Result position:"), within: WAIT },
        ]),
    );
    finish(world);
    let keys: Vec<_> = world.tree(client).nodes().iter().filter_map(|node| node.key).collect();
    let needs: Vec<_> = keys.iter().filter(|key| key.kind == 51).map(|key| (key.a, key.b)).collect();
    let updates: Vec<_> = keys.iter().filter(|key| key.kind == 56).map(|key| key.a).collect();
    assert_eq!(needs, vec![(PROPOSER, 1), (SECOND_PROPOSER, 2)]);
    assert_eq!(updates, vec![NEWER_RESULT, OLDER_RESULT]);
}
