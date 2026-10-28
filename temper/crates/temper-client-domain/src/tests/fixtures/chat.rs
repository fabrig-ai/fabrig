use super::*;
use crate::pages::{Request, chat::Chat};
use jig_client::{Client, Event, Handle, Told, Watchable, contract as c};
pub const FIRST: u64 = 1;
pub const TASK: u64 = 9;
pub const ATTEMPT: u64 = 23;
pub const POSITION: u64 = 3;
pub const LABEL: u32 = 7;
const ONE: u32 = 1;
const OUTPUTS: u32 = 64;
const EXCHANGES: u32 = 8;
pub fn task() -> c::Task {
    use jig_client_protocol_wire::v19 as w;
    let mut record = jig_client_test_records::task_record().into_parts();
    record.chip = jig_client_test_records::owner_waiting_chip_record();
    let record = w::Task::new(&w::CEILINGS, record).expect("checked task fixture");
    let link = jig_client_protocol_link::Limits::new(limits().client.domain, EXCHANGES).expect("link limits");
    let mut task = jig_client_protocol_link::translate::documents::task(record, &link).expect("task document");
    task.chip.task = TASK;
    task.label = LABEL;
    task.project = PROJECT;
    task.activity = POSITION;
    task.acts = List::with_capacity(ONE);
    task.acts.push(c::Offered { act: c::ActKind::Stop, admission: c::Admission::Allowed, effect: None }).expect("stop");
    phase(&mut task, c::OwnerChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }));
    task
}
pub fn phase(task: &mut c::Task, phase: c::OwnerChipPhase) {
    task.chip.phase = phase.compact();
    let c::ChipPhase::Owner { phase: target, .. } = task.chip.details.as_mut() else { panic!("owner chip") };
    *target = phase;
}
pub struct Fixture {
    pub chat: Chat,
    pub client: Client,
    pub out: Queue<Request>,
    pub watch: Token,
    pub me: Token,
}
pub fn opened() -> Fixture {
    let limits = limits();
    let mut client = Client::unlinked(&limits.client, SEED).expect("client");
    let mut emitted = Queue::with_capacity(OUTPUTS);
    for event in [
        Event::StartUnlinked { kept: List::with_capacity(0) },
        Event::SignIn { with: jig_client::With::Transport },
        Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
    ] {
        client.step(Time::ZERO, Wall::EPOCH, event, &mut emitted);
    }
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut chat = Chat::new(TASK, FIRST, &limits, &mut out);
    let _read = out.pop();
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        Event::Watch { handle: chat.document, subject: Watchable::Jig(c::Subject::Task { task: TASK }) },
        &mut emitted,
    );
    let mut watch = None;
    let mut me = None;
    while let Some(request) = emitted.pop() {
        if let jig_client::Request::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Task { .. }) },
        } = request
        {
            watch = Some(token);
        } else if let jig_client::Request::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Me) },
        } = request
        {
            me = Some(token);
        }
    }
    let watch = watch.expect("task opened");
    deliver(&mut client, watch, task());
    while out.pop().is_some() {}
    chat.told(&client, Told::Changed { handle: chat.document }, &mut out);
    Fixture { chat, client, out, watch, me: me.expect("automatic me watch") }
}
pub fn deliver(client: &mut Client, watch: Token, task: c::Task) {
    client.step(
        Time::ZERO,
        Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(task))) },
        },
        &mut Queue::with_capacity(OUTPUTS),
    );
}
pub fn handles(out: &mut Queue<Request>) -> Box<[Handle]> {
    let mut handles = List::with_capacity(OUTPUTS);
    while let Some(request) = out.pop() {
        if let Request::Client(Event::Unwatch { handle }) = request {
            handles.push(handle).expect("handles fit");
        }
    }
    handles.into_boxed()
}
pub const DELEGATE: u64 = 12;
pub const WAKE_SUMMARY: &[u8] =
    b"Begin the work your brief describes.\n\n2: Result from task 2:\n1d0b14e4db018fed000000000000000300000000";
pub fn with_delegate() -> Fixture {
    let mut fixture = opened();
    let mut task = task();
    task.acts = acts::offered(c::ActKind::Cancel, c::Admission::Allowed);
    task.delegates = List::with_capacity(ONE);
    let mut chip = task.chip.clone();
    chip.task = DELEGATE;
    task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).expect("delegate");
    deliver(&mut fixture.client, fixture.watch, task);
    let mut out = Queue::with_capacity(OUTPUTS);
    fixture.client.step(
        Time::ZERO,
        Wall::EPOCH,
        Event::Watch { handle: crate::pages::me_handle(), subject: Watchable::Jig(c::Subject::Me) },
        &mut out,
    );

    fixture.client.step(
        Time::ZERO,
        Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched {
                watch: fixture.me,
                event: WatchEvent::Delivered(Delivery::Me(Box::new(chats::me()))),
            },
        },
        &mut out,
    );
    while fixture.out.pop().is_some() {}
    fixture
}

pub fn delegate_call(task: u64) -> c::Call {
    let mut objects = List::with_capacity(ONE);
    objects.push(c::ObjectKey::Task(task)).expect("delegated task");
    c::Call {
        number: task,
        tool: acts::text(b"delegate"),
        input: acts::text(b"{}"),
        outcome: Some(acts::text(b"Delegated tasks: 2, 3")),
        failed: false,
        objects,
        parts: List::with_capacity(0),
    }
}
pub fn activity(f: &mut Fixture, tasks: &[u64], position: u64) {
    let mut blocks = List::with_capacity(u32::try_from(tasks.len()).expect("fixture calls fit"));
    for task in tasks {
        blocks.push(c::Block::Call(delegate_call(*task))).expect("call fits");
    }
    activity_blocks(f, c::Speaker::Assistant, blocks, position);
}
pub fn activity_blocks(f: &mut Fixture, speaker: c::Speaker, blocks: List<c::Block>, position: u64) {
    let mut messages = List::with_capacity(ONE);
    messages.push(c::Message { speaker, blocks }).expect("message");
    let turn = c::Turn {
        messages,
        used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        cut: false,
    };
    let mut rows = List::with_capacity(ONE);
    rows.push(c::ActivityLine {
        position,
        at: Wall::EPOCH,
        event: c::Activity::Turn {
            name: c::TurnName { attempt: ATTEMPT, number: u32::try_from(position).expect("fixture turn") },
            turn: c::TurnRead::Decoded(turn),
        },
    })
    .expect("turn");
    let mut task = task();
    task.activity = position;
    deliver(&mut f.client, f.watch, task);
    f.chat.told(&f.client, Told::Changed { handle: f.chat.document }, &mut f.out);
    f.chat.told(
        &f.client,
        Told::Page {
            handle: f.chat.activity,
            page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(c::ActivityPage {
                rows,
                before: None,
                after: None,
                cut: false,
            })))),
        },
        &mut f.out,
    );
}
pub fn result(task: &c::Task, ending: c::Ending, value: Option<c::ResultValue>) -> c::Result {
    c::Result {
        task: task.chip.clone(),
        ending,
        value,
        at: Some(Wall::EPOCH),
        details: None,
        acts: List::with_capacity(0),
    }
}
pub fn document_text(bytes: &[u8]) -> DocumentText {
    let bound = u32::try_from(bytes.len()).expect("fixture text fits");
    DocumentText::complete(c::Text::new(Box::from(bytes), bound).expect("fixture text"))
}
pub fn change() -> c::Task {
    let mut task = task();
    task.hold = None;
    task.result = None;
    task.result_contract = c::TaskContractDocument::Change { connector: FORGE, kind: 1, words: 64 };
    let c::ChipPhase::Owner { details, .. } = task.chip.details.as_mut() else { panic!("owner chip") };
    details.ending = None;
    task
}
pub fn change_value(words: &[u8]) -> c::ResultValue {
    c::ResultValue::Change {
        kind: 1,
        part: c::OwnerPart { connector: FORGE, kind: c::OwnerPartKind::Result, key: c::OwnerPartKey::Number(TASK) },
        words: document_text(words),
    }
}
