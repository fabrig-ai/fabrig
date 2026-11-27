//! A front host reads Ctrl+C at a drawn point in a chat's life (temper/docs/design/client/tui/box.md, 7 and 11).
use crate::journey::fixture::{printed_link, signed_in, stream_bytes, watch_where};
use jig_client::{Event, Message, Request, Streamed};
use jig_client_protocol_wire::{self as w, Record as _};
use jig_client_test_records as records;
use skein_lib::{Duration, List, Rng, Secret, Writer};
use skein_ui::Role;
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_shell_tui::{
    host::{Duties, Host, Report, Stop},
    testing::quit::{CHAT, Hosts, PROJECT, Read},
};
use temper_client_tui::BoxStage;

const LABEL: u32 = 8;
const SCREEN: Size = Size { rows: 24, columns: 80 };
const FIRST_ACTIVITY: u64 = 7;
// Each family fits half the sampled runs, leaving room in the simulated layer's growth allowance.
const SEEDS_PER_FAMILY: u64 = 29;
const LIFE: [Read; 9] = [
    Read::Starting,
    Read::Working,
    Read::Writing,
    Read::Tool,
    Read::Retrying,
    Read::Delegating,
    Read::ParkedOnDelegate,
    Read::Woken,
    Read::Parked,
];
// With the landing held, the last read is the chat parked on its still-live delegate.
const HELD_READS: usize = 7;
// Starting is the first read that counts as work.
const FIRST_WORKING: usize = 1;

#[test]
fn ctrl_c_uses_the_last_read_chat_facts() {
    let mut cuts = [false; LIFE.len() + 1];
    let mut held_cuts = [false; HELD_READS + 1];
    let hello = hello();
    let hosts = Hosts::new(SCREEN);
    let documents = LIFE
        .iter()
        .enumerate()
        .map(|(at, &read)| {
            let mut row = chat_row(read, FIRST_ACTIVITY + u64::try_from(at).expect("activity"));
            row.parts = List::from_boxed(Box::new([w::RowPart::Unknown]));
            chat_document(&row)
        })
        .collect::<Vec<_>>();
    for seed in 1..=2 * SEEDS_PER_FAMILY {
        let (life, first, covered) = if seed <= SEEDS_PER_FAMILY {
            (LIFE.as_slice(), 0, cuts.as_mut_slice())
        } else {
            (&LIFE[..HELD_READS], FIRST_WORKING, held_cuts.as_mut_slice())
        };
        let cut = first + draw_cut(seed, &mut covered[first..]);
        let last = life[..cut].last();
        // Section 7's oracle: the empty prefix and a parked chat with no delegate are idle.
        let asks = match last {
            None | Some(Read::WaitingForWords | Read::Parked) => false,
            Some(
                Read::Starting
                | Read::Working
                | Read::Writing
                | Read::Tool
                | Read::Retrying
                | Read::Delegating
                | Read::WaitingOnDelegate
                | Read::ParkedOnDelegate
                | Read::Woken,
            ) => true,
        };
        let (mut host, watch) = host_with_live_chats(&hosts, seed, &hello);
        for (at, &read) in life.iter().enumerate() {
            if at == cut {
                ctrl_c(&mut host, asks, seed, last);
            }
            read_document(&mut host, watch, &documents[at], seed, read);
        }
        if cut == life.len() {
            ctrl_c(&mut host, asks, seed, last);
        }
    }
    assert!(cuts.iter().all(|&hit| hit), "seeds 1..={SEEDS_PER_FAMILY} did not cover all lifetime cuts: {cuts:?}");
    assert!(
        held_cuts[FIRST_WORKING..].iter().all(|&hit| hit),
        "seeds {}..={} did not cover all held cuts: {held_cuts:?}",
        SEEDS_PER_FAMILY + 1,
        2 * SEEDS_PER_FAMILY
    );
}

#[test]
fn ctrl_c_asks_after_the_delegate_wait_is_drawn_with_passing_facts_unread() {
    const PASSING: [Read; 4] = [Read::Writing, Read::Working, Read::Tool, Read::Delegating];
    // Every omission mask, with the key either at the wait or after the chat parks.
    const CASES: usize = (1 << PASSING.len()) * 2;
    let mut covered = [false; CASES];
    let hosts = Hosts::new(SCREEN);
    let hello = hello();
    let left_off = chat_document(&chat_row(Read::Starting, FIRST_ACTIVITY));
    let starting = task_document(Read::Starting, FIRST_ACTIVITY);
    let passing: [_; PASSING.len()] = core::array::from_fn(|at| {
        task_document(PASSING[at], FIRST_ACTIVITY + 1 + u64::try_from(at).expect("activity"))
    });
    let waiting_at = FIRST_ACTIVITY + 1 + u64::try_from(PASSING.len()).expect("passing reads");
    let waiting = task_document(Read::WaitingOnDelegate, waiting_at);
    let parked = task_document(Read::ParkedOnDelegate, waiting_at + 1);
    for seed in 1..=u64::try_from(CASES).expect("number of cases") {
        let case = draw_cut(seed, &mut covered);
        let omit = case / 2;
        let (mut host, watch) = host_with_open_chat(&hosts, seed, &hello, &left_off);
        read_document(&mut host, watch, &starting, seed, Read::Starting);
        for (bit, (&read, fact)) in PASSING.iter().zip(&passing).enumerate() {
            if omit & (1 << bit) == 0 {
                read_document(&mut host, watch, fact, seed, read);
            }
        }
        read_document(&mut host, watch, &waiting, seed, Read::WaitingOnDelegate);
        assert!(
            skein_lib::bytes::find(&host.frame().text(), b"waiting on 1 delegate").is_some(),
            "seed {seed}, omitted mask {omit:04b}: delegate wait not drawn"
        );
        if case.is_multiple_of(2) {
            ctrl_c(&mut host, true, seed, Some(&Read::WaitingOnDelegate));
        }
        read_document(&mut host, watch, &parked, seed, Read::ParkedOnDelegate);
        if !case.is_multiple_of(2) {
            ctrl_c(&mut host, true, seed, Some(&Read::ParkedOnDelegate));
        }
    }
    assert!(covered.iter().all(|&hit| hit));
}

fn draw_cut(seed: u64, covered: &mut [bool]) -> usize {
    let span = covered.len();
    let drawn = Rng::new(seed).below(u64::try_from(span).expect("number of cuts"));
    let drawn = usize::try_from(drawn).expect("drawn cut");
    // Start at the drawn cut. Until all placements have been tested, skip any already covered.
    let cut = (drawn..span).chain(0..drawn).find(|&cut| !covered[cut]).unwrap_or(drawn);
    covered[cut] = true;
    cut
}

fn ctrl_c(host: &mut Host, asks: bool, seed: u64, last: Option<&Read>) {
    let duties = host.report(Report::Key(Key::Control(b'c')));
    assert!(!duties.close, "seed {seed} after {last:?}: closed instead of asking or stopping");
    assert_eq!(duties.stop, if asks { None } else { Some(Stop::First) }, "seed {seed} after {last:?}");
    let shown = host.tree().nodes().iter().any(|node| {
        node.role == Role::Menu && node.name.as_deref().is_some_and(|name| name.starts_with(b"Stop the box?"))
    });
    assert_eq!(shown, asks, "seed {seed} after {last:?}: question shown");
}

fn host_with_live_chats(hosts: &Hosts, seed: u64, hello: &[u8]) -> (Host, u64) {
    let (host, duties) = signed_front(hosts, seed, hello);
    let watch = watch_where(&duties, |subject| {
        matches!(subject, w::Subject::Tasks(tasks) if tasks.query.project == Some(PROJECT) && tasks.query.label == Some(LABEL))
    }).unwrap_or_else(|| panic!("seed {seed}: no project chat watch"));
    (host, watch)
}

fn host_with_open_chat(hosts: &Hosts, seed: u64, hello: &[u8], left_off: &w::Owned) -> (Host, u64) {
    let (mut host, duties) = signed_front(hosts, seed, hello);
    let watch = watch_where(&duties, |subject| {
        matches!(subject, w::Subject::Tasks(tasks) if tasks.query.project.is_none() && tasks.query.label == Some(LABEL))
    }).unwrap_or_else(|| panic!("seed {seed}: no left-off chat watch"));
    read_document(&mut host, watch, left_off, seed, Read::Starting);
    host.frame();
    host.report(Report::Key(Key::Up));
    host.frame();
    let duties = host.report(Report::Key(Key::Enter));
    let watch = watch_where(&duties, |subject| matches!(subject, w::Subject::Task(task) if task.task == CHAT))
        .unwrap_or_else(|| {
            let frame = host.frame().text();
            panic!(
                "seed {seed}: no open chat watch ({} duties)\n{}",
                duties.asked.len(),
                String::from_utf8_lossy(&frame)
            )
        });
    (host, watch)
}

fn signed_front(hosts: &Hosts, seed: u64, hello: &[u8]) -> (Host, Duties) {
    let mut host = hosts.front(seed);
    host.report(Report::Stage(BoxStage::Running));
    let local = host.report(Report::Local { link: Some(Secret::new(printed_link())) });
    let exchange = local
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { exchange, message: Message::SignInLink(_), .. } = request {
                Some(*exchange)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("seed {seed}: local link exchange"));
    host.report(Report::Heard(Event::Reply { exchange, message: signed_in() }));
    let duties = host.report(Report::Heard(Event::Streamed { message: Streamed::Record(hello.into()) }));
    (host, duties)
}

fn hello() -> Box<[u8]> {
    stream_bytes(&records::opened(
        w::Hello {
            stream: [7; 16],
            served: List::from_boxed(Box::new([w::Served {
                owner: w::Owner::Jig,
                first: w::VERSION,
                last: w::VERSION,
            }])),
            heartbeat: Duration::from_secs(30),
        },
        me(),
    ))
}

fn me() -> w::Me {
    let template = w::Template {
        number: jig_client::shapes::CHAT,
        shown: records::wire_text(b"Chat"),
        executor: w::TemplateExecutor::Charter(w::TemplateCharter { value: 7 }),
        contract: w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: 32 }),
        shallower: 1,
        budget: w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap: None }),
        deadline: w::TemplateDeadline::None,
        tracked: None,
        flags: chat_flags(),
        label: LABEL,
        grace: None,
        llm: None,
        resources: List::with_capacity(0),
    };
    let project = w::MeProject {
        project: PROJECT,
        name: records::wire_text(b"Project"),
        role: w::HeldRole::Owner,
        templates: piece(&template),
        left_out: 0,
        defaults: List::with_capacity(0),
        public: None,
        creating: None,
        pool: None,
        models: List::with_capacity(0),
    };
    let mut me = records::frame_record();
    me.projects = piece(&project);
    me.retention = Duration::from_secs(1000);
    me.needs = w::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 };
    me.needs_total = 0;
    me.updates = w::Count { value: 0, limited: false };
    me
}

fn chat_flags() -> w::TaskFlags {
    w::TaskFlags { ended_by: w::EndedBy::Result, conversation: true, listed: true, retention: w::Retention::Kept }
}

fn chat_row(read: Read, at: u64) -> w::TaskLine {
    let (facts, waits_on) = read.facts(records::wire_owner_chip_details_row());
    let mut row = records::task_line_record();
    row.project = PROJECT;
    row.chip = records::chip_from_rows(&facts);
    row.flags = chat_flags();
    row.label = LABEL;
    row.tracked = None;
    row.requester = w::Requester::Person(w::RequesterPerson { value: 7 });
    row.waits_on = waits_on;
    row.changed = at;
    row.last_activity = at;
    row
}

fn chat_document(row: &w::TaskLine) -> w::Owned {
    document(&w::DeliveryValue::Tasks(w::Tasks { ended_mark: 0, rows: piece(row), more: false }))
}

fn task_document(read: Read, at: u64) -> w::Owned {
    let (facts, waits_on) = read.facts(records::wire_owner_chip_details_row());
    let mut task = records::task_record();
    task.project = PROJECT;
    task.label = LABEL;
    task.flags = chat_flags();
    task.chip = records::chip_from_rows(&facts);
    task.waits_on = waits_on;
    task.changed = at;
    task.last_activity = at;
    document(&w::DeliveryValue::Task(Box::new(task)))
}

fn document(value: &w::DeliveryValue) -> w::Owned {
    w::Owned { owner: w::Owner::Jig, version: w::VERSION, record: value.encode(&w::CEILINGS).expect("chat document") }
}

fn read_document(host: &mut Host, watch: u64, document: &w::Owned, seed: u64, read: Read) {
    let bytes = stream_bytes(&w::StreamEvent::Delivery(w::Delivery { watch, document: document.clone() }));
    let duties = host.report(Report::Heard(Event::Streamed { message: Streamed::Record(bytes) }));
    assert!(!duties.close && duties.stop.is_none(), "seed {seed}: {read:?} delivery stopped the box");
}

fn piece(record: &impl w::Record<Limits = w::Limits>) -> Box<[u8]> {
    let bytes = record.encode(&w::CEILINGS).expect("bounded fixture");
    let mut writer = Writer::new(bytes.len() + 4);
    skein_codec::piece::write(&mut writer, &bytes).expect("one measured row");
    writer.finish()
}
