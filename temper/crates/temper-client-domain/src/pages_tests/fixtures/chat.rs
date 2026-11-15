use super::*;
use crate::pages::{Request, chat::Chat};
use jig_client::{Client, Event, Told, Watchable, contract as c};
pub const FIRST: u64 = 1;
pub const TASK: u64 = 9;
pub const ATTEMPT: u64 = 23;
pub const POSITION: u64 = 3;
pub const LABEL: u32 = 7;
const ONE: u32 = 1;
const OUTPUTS: u32 = 64;
const EXCHANGES: u32 = 8;
pub fn task() -> c::Task {
    use jig_client_protocol_wire::v24 as w;
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
    phase(&mut task, c::ChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT }));
    task
}
pub fn phase(task: &mut c::Task, phase: c::ChipPhase) {
    task.chip.phase = phase.compact();
    let c::ChipPhaseFacts { phase: target, .. } = task.chip.details.as_mut();
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
        client.step_unlinked(Time::ZERO, Wall::EPOCH, event, &mut emitted);
    }
    let mut out = Queue::with_capacity(OUTPUTS);
    let mut chat = Chat::new(TASK, crate::pages::chat::document::Mode::Chat, FIRST, &limits, &mut out);
    let _read = out.pop();
    client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Watch { handle: chat.document, subject: Watchable::Jig(c::Subject::Task { task: TASK }) },
        &mut emitted,
    );
    let mut watch = None;
    let mut me = None;
    while let Some(request) = emitted.pop() {
        if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Task { .. }) },
        } = request
        {
            watch = Some(token);
        } else if let jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Open { watch: token, subject: Watchable::Jig(c::Subject::Me) },
        } = request
        {
            me = Some(token);
        }
    }
    let watch = watch.expect("task opened");
    deliver(&mut client, watch, task());
    while out.pop().is_some() {}
    chat.told(Time::ZERO, &client, Told::Changed { handle: chat.document }, &mut out);
    Fixture { chat, client, out, watch, me: me.expect("automatic me watch") }
}
pub fn deliver(client: &mut Client, watch: Token, task: c::Task) {
    client.step_unlinked(
        Time::ZERO,
        Wall::EPOCH,
        Event::Up {
            up: jig_client::Up::Watched { watch, event: WatchEvent::Delivered(Delivery::Task(Box::new(task))) },
        },
        &mut Queue::with_capacity(OUTPUTS),
    );
}
pub fn document_text(bytes: &[u8]) -> DocumentText {
    let bound = u32::try_from(bytes.len()).expect("fixture text fits");
    DocumentText::complete(c::Text::new(Box::from(bytes), bound).expect("fixture text"))
}
