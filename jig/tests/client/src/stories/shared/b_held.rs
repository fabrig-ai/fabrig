//! An existing stopped conversation (jig/docs/design/client/domain/testing.md, 4–5).
use crate::{Scripted, joint::engine::Driver};
use jig_client_domain::contract as c;
use skein_lib::{Time, Wall};

pub fn scripted(front: &mut Scripted) {
    let mut task = crate::stories::conversations::chat(1, b"Hello", Wall::EPOCH);
    task.requester = c::Requester::Person(1);
    let reason = c::ChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 1,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
        hold_wait_since: Some(Wall::EPOCH),
    };
    task.hold = Some(Box::new(reason.clone()));
    task.chip.phase = c::Phase::Held;
    task.chip.details.phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 };
    task.chip.details.details.hold = Some(reason);
    task.context.as_mut().expect("task context").runtime = c::Runtime::Held { was: None };
    task.acts = crate::test::list([
        c::Offered { effect: None, act: c::ActKind::Release, admission: c::Admission::Allowed },
        c::Offered { effect: None, act: c::ActKind::Message, admission: c::Admission::Allowed },
    ]);
    front
        .engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            jig_client_scripted_engine::Change::Object {
                project: 1,
                key: c::ObjectKey::Task(1),
                value: c::ReadValue::Task(Box::new(task)),
            },
        )
        .expect("held conversation fits");
}
pub fn core(driver: &mut Driver, now: Time, wall: Wall) {
    use jig_fake_parties::{Act, Ask, Party, Role, Task};
    driver.scripted_party(
        Party::new(
            1,
            Role::Owner,
            1_000_000,
            Box::new([
                Act::SignIn { provider: 0, subject: 7_u64.to_be_bytes().into() },
                Act::Request { key: [81; 16], ask: Ask::Chat { words: b"Hello".as_slice().into() } },
                Act::Request { key: [82; 16], ask: Ask::Stop { task: Task::Number(1) } },
            ]),
        ),
        now,
        wall,
    );
}

const REQUESTER: u64 = 1;
const PROJECT: u32 = 1;
const PARENT: u64 = 1;
const DELEGATE: u64 = 2;
const HOLD_REVISION: u64 = 1;

/// Seed the same delegate and its current person-held escalation as the core.
pub(super) fn scripted_delegate(front: &mut Scripted, parent: &mut c::Task) {
    use jig_client_scripted_engine::{Change, Rows, Source};
    parent.flags.ended_by = c::EndedBy::Requester;
    let mut task = crate::stories::task();
    task.chip.task = DELEGATE;
    task.chip.title = c::DocumentText::complete(crate::stories::text(b"Held delegate"));
    task.requester = c::Requester::Task(PARENT);
    task.lineage = crate::test::list([PARENT]);
    task.acts =
        crate::test::list([c::Offered { effect: None, act: c::ActKind::Cancel, admission: c::Admission::Allowed }]);
    task.lineage_chips = crate::test::list([parent.chip.clone()]);
    let hold = c::ChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Budget,
        revision: HOLD_REVISION,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: Some(c::HoldEscalation::Waiting {
            holder: c::EscalationHolder::Person(REQUESTER),
            entry: 1,
            since: Wall::EPOCH,
        }),
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
        hold_wait_since: Some(Wall::EPOCH),
    };
    task.hold = Some(Box::new(hold.clone()));
    task.chip.phase = c::Phase::Held;
    task.chip.details.phase = c::ChipPhase::Held { reason: hold.reason, revision: HOLD_REVISION };
    task.chip.details.details.hold = Some(hold);
    task.context.as_mut().expect("delegate context").runtime = c::Runtime::Held { was: None };
    let acts =
        crate::test::list([c::Offered { effect: None, act: c::ActKind::LeaveHeld, admission: c::Admission::Allowed }]);
    let chip = task.chip.clone();
    for (key, value) in [
        (c::ObjectKey::Task(DELEGATE), c::ReadValue::Task(Box::new(task.clone()))),
        (
            c::ObjectKey::Escalation(DELEGATE),
            c::ReadValue::Escalation(Box::new(c::Escalation { task: Box::new(task), acts })),
        ),
    ] {
        front
            .engine
            .act(Time::ZERO, Wall::EPOCH, Change::Object { project: PROJECT, key, value })
            .expect("delegate fits");
    }
    parent.delegates = crate::test::list([c::LiveDelegate { chip, takeover_budget: 1 }]);
    let mut frame = crate::stories::frame(REQUESTER);
    frame.needs.escalations = 1;
    frame.needs_total = 1;
    front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("frame fits");
    front
        .engine
        .act(
            Time::ZERO,
            Wall::EPOCH,
            Change::Source(Source {
                party: REQUESTER,
                read: c::Read::Inbox {
                    group: c::InboxReadGroup::Needs,
                    project: None,
                    kind: None,
                    goal: None,
                    after: None,
                },
                rows: Rows::Inbox(vec![crate::stories::inbox_entry(
                    c::ObjectKey::Escalation(DELEGATE),
                    PROJECT,
                    c::InboxGroup::Escalations,
                    1,
                    Wall::EPOCH,
                    None,
                )]),
            }),
        )
        .expect("escalation inbox fits");
}
/// The decision's record names the person who asked, through a delegate.
pub fn a_delegate_s_escalation_record_names_the_person_who_asked<F: crate::Front<crate::Test>>(
    world: &mut crate::World<crate::Test, F>,
) {
    use super::b_conversations::{WAIT, finish, sign_in};
    use crate::harness::Seed;
    use skein_fake_person::{Find, Step};
    use skein_ui::Role;
    world.seed(Seed::ChatProject { party: REQUESTER, project: PROJECT });
    world.seed(Seed::Project(super::b_seed::Facts::DelegateEscalation));
    let client = world.client(REQUESTER, crate::stories::inbox());
    world.person(
        client,
        sign_in(vec![
            Step::See { find: Find::text(b"Held delegate"), within: WAIT },
            Step::See { find: Find::exactly_named(Role::Button, b"Leave it held"), within: WAIT },
        ]),
    );
    finish(world);
    world.person(
        client,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Leave it held") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"Leave held".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Escalation decided"), within: WAIT },
        ],
    );
    finish(world);
    let closer = world.tab(client);
    super::b_proposals::open(world, closer, Find::text(b"Signed in as"));
    finish(world);
    world.go(closer, jig_client_test_domain::Address::Jig(jig_client_view::address::Address::task(DELEGATE)));
    super::b_proposals::open(world, closer, Find::exactly_named(Role::Button, b"Cancel"));
    finish(world);
    world.person(
        closer,
        vec![
            Step::Press { find: Find::exactly_named(Role::Button, b"Cancel") },
            Step::Type { find: Find::named(Role::TextArea, b"Your reason"), words: b"Done".as_slice().into() },
            Step::Press { find: Find::exactly_named(Role::Button, b"Confirm") },
            Step::See { find: Find::text(b"Cancelled"), within: WAIT },
        ],
    );
    finish(world);
    // The application names the record so the page reads the ended escalation afresh.
    world.event(
        client,
        jig_client_test_domain::Event::Jig(jig_client_view::Event::Named {
            objects: crate::test::list([c::ObjectKey::Escalation(DELEGATE)]),
            sections: crate::test::list([]),
        }),
    );
    world.person(
        client,
        vec![
            Step::See { find: Find::text(b"Ended escalation"), within: WAIT },
            Step::See { find: Find::text(b"Requester task: 1"), within: WAIT },
            Step::See { find: Find::text(b"Decision person: 1"), within: WAIT },
        ],
    );
    finish(world);
}
