//! An existing stopped conversation (jig/docs/design/client/domain/testing.md, 4–5).
use super::b_seed::{inbox_source, install};
use crate::{PROJECT, Scripted, joint::engine::Driver};
use jig_client_domain::contract as c;
use jig_client_scripted_engine::Change;
use jig_fake_parties::{Act, Ask, Party, Role, Task};
use skein_lib::{Time, Wall};

const OWNER: u64 = 1;
const PARENT: u64 = 1;
const DELEGATE: u64 = 2;
const HOLD_REVISION: u64 = 1;

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
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
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
    install(front, c::ObjectKey::Task(1), c::ReadValue::Task(Box::new(task)));
}
pub fn core(driver: &mut Driver, now: Time, wall: Wall) {
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

/// Seed a member's delegate held for its owner's decision.
pub(crate) fn scripted_delegate(front: &mut Scripted, parent: &mut c::Task) {
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
            // Scripted role recipients match policy holdings; this owner holds Owner.
            holder: c::EscalationHolder::Person(OWNER),
            entry: 1,
            since: Wall::EPOCH,
        }),
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
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
        install(front, key, value);
    }
    parent.delegates = crate::test::list([c::LiveDelegate { chip, takeover_budget: 1 }]);
    let mut frame = front.engine.truth().frames[&OWNER].clone();
    frame.needs.escalations = 1;
    frame.needs_total = 1;
    front.engine.act(Time::ZERO, Wall::EPOCH, Change::Me(frame)).expect("frame fits");
    inbox_source(
        front,
        OWNER,
        c::InboxReadGroup::Needs,
        vec![crate::stories::inbox_entry(
            c::ObjectKey::Escalation(DELEGATE),
            PROJECT,
            c::InboxGroup::Escalations,
            1,
            Wall::EPOCH,
            None,
        )],
    );
}
