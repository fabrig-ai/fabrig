//! An existing stopped conversation (jig/docs/design/client/domain/testing.md, 4–5).
use crate::{Scripted, joint::engine::Driver};
use jig_client_domain::contract as c;
use skein_lib::{Time, Wall};

pub fn scripted(front: &mut Scripted) {
    let mut task = crate::stories::conversations::chat(1, b"Hello", Wall::EPOCH);
    task.requester = c::OwnerRequester::Person(1);
    let reason = c::OwnerChipHold {
        placed: None,
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 1,
        before: c::OwnerPriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::OwnerFailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
        hold_wait_since: Some(Wall::EPOCH),
    };
    task.hold = Some(Box::new(reason.clone()));
    task.chip.phase = c::Phase::Held;
    task.chip.details.phase = c::OwnerChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 };
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
