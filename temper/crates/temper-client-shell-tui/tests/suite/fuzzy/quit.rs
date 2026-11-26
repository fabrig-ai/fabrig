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
    host::{Host, Report, Stop},
    testing::quit::{Hosts, PROJECT, Read},
};
use temper_client_tui::BoxStage;

const LABEL: u32 = 8;
const SCREEN: Size = Size { rows: 24, columns: 80 };
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
    let hello = stream_bytes(&records::opened(
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
    ));
    let hosts = Hosts::new(SCREEN);
    let documents = LIFE
        .iter()
        .enumerate()
        .map(|(at, &read)| {
            let record =
                chat_rows(read, u64::try_from(at).expect("activity") + 7).encode(&w::CEILINGS).expect("chat document");
            w::Owned { owner: w::Owner::Jig, version: w::VERSION, record }
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
        // Section 7's oracle: the empty prefix and only the parked read with no delegate are idle.
        let asks = match last {
            None | Some(Read::Parked) => false,
            Some(
                Read::Starting
                | Read::Working
                | Read::Writing
                | Read::Tool
                | Read::Retrying
                | Read::Delegating
                | Read::ParkedOnDelegate
                | Read::Woken,
            ) => true,
        };
        let (mut host, watch) = host_with_live_chats(&hosts, seed, &hello);
        for (at, &read) in life.iter().enumerate() {
            if at == cut {
                ctrl_c(&mut host, asks, seed, last);
            }
            let bytes = stream_bytes(&w::StreamEvent::Delivery(w::Delivery { watch, document: documents[at].clone() }));
            let duties = host.report(Report::Heard(Event::Streamed { message: Streamed::Record(bytes) }));
            assert!(!duties.close && duties.stop.is_none(), "seed {seed}: {read:?} delivery stopped the box");
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
    let watch = watch_where(&duties, |subject| {
        matches!(subject, w::Subject::Tasks(tasks) if tasks.query.project == Some(PROJECT) && tasks.query.label == Some(LABEL))
    }).unwrap_or_else(|| panic!("seed {seed}: no project chat watch"));
    (host, watch)
}

fn me() -> w::Me {
    let template = w::Template {
        number: jig_client::shapes::CHAT,
        name: records::wire_text(b"Chat"),
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

fn chat_rows(read: Read, at: u64) -> w::DeliveryValue {
    let (facts, waits_on) = read.facts(records::wire_owner_chip_details_row());
    let mut row = records::task_line_record();
    row.project = PROJECT;
    row.chip = records::chip_from_rows(&facts);
    row.flags = chat_flags();
    row.label = LABEL;
    row.tracked = None;
    row.requester = w::Requester::Person(w::RequesterPerson { value: 7 });
    row.waits_on = waits_on;
    row.parts = List::from_boxed(Box::new([w::RowPart::Unknown]));
    row.changed = at;
    row.last_activity = at;
    w::DeliveryValue::Tasks(w::Tasks { ended_mark: 0, rows: piece(&row), more: false })
}

fn piece(record: &impl w::Record<Limits = w::Limits>) -> Box<[u8]> {
    let bytes = record.encode(&w::CEILINGS).expect("bounded fixture");
    let mut writer = Writer::new(bytes.len() + 4);
    skein_codec::piece::write(&mut writer, &bytes).expect("one measured row");
    writer.finish()
}
