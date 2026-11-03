//! The engine world is the lowest tier joining retained tier facts to ctl's reads.
use jig_client::{self as client, contract as c};
use jig_domain_core::reads;
use skein_lib::{Env, List, Queue, Writer};
use temper_client_tools::{jig::call::Parsed, root};
use temper_engine_domain_world::forge_routes::World;
use temper_engine_domain_world::forge_routes::{Gate, StoryLimits};

/// Keep the baseline's projection at 128 bytes without changing other tier stories.
pub(super) fn world() -> World {
    let mut documents = jig_protocol_config::profile(jig_protocol_config::Profile::Small).core.documents;
    documents.text_bytes = 128;
    let options = StoryLimits {
        tier: Some(temper_engine_domain_world::tier_runs::Tier::default()),
        tasks: Some(12),
        spec_bytes: Some(4096),
        ..StoryLimits::default()
    };
    World::configured_limits(true, true, false, 1000, Gate::default(), Some(documents), options)
}

/// Read a tier task through ctl and require its complete execution words in asked.
pub(super) fn assert_words(world: &mut World, task: u64, expected: &str) {
    world.signed_in = None;
    world.sign_in();
    let printed = show(world, task);
    assert_eq!(printed.status, 0);
    let text = std::str::from_utf8(&printed.stdout).expect("ctl text");
    assert!(text.contains(&format!("asked: {expected}")), "missing full execution identity: {text}");
}

/// Submit ordinary task show through the tools root and real task document read.
pub(super) fn show(world: &mut World, task: u64) -> temper_client_tools::jig::Printed {
    show_format(world, task, false)
}

/// Read the same bounded task document as JSON through the tools root.
pub(super) fn show_json(world: &mut World, task: u64) -> temper_client_tools::jig::Printed {
    show_format(world, task, true)
}

fn show_format(world: &mut World, task: u64, json: bool) -> temper_client_tools::jig::Printed {
    let env = Env {
        limits: root::Limits::new(temper_client_tools::limits::client(), temper_client_tools::limits::tools()),
        now: world.env.now,
        wall: world.env.wall,
    };
    let mut root = root::Root::unlinked(&env.limits, temper_engine_domain_world::tier_runs::SEED).expect("ctl limits");
    let mut out = Queue::uncapped();
    for event in [
        client::Event::StartUnlinked { kept: List::with_capacity(0) },
        client::Event::SignIn { with: client::With::Transport },
        client::Event::Up { up: client::Up::Served { served: true, versions: None } },
    ] {
        root::step(&mut root, &env, root::Event::Client { event }, &mut out);
    }
    let number = task.to_string();
    let argv: &[&[u8]] =
        if json { &[b"task", b"show", number.as_bytes(), b"--json"] } else { &[b"task", b"show", number.as_bytes()] };
    let Parsed::Call { call, json } = temper_client_tools::table::parse(argv).expect("task show") else {
        panic!("call")
    };
    root::step(&mut root, &env, root::Event::Call { number: 1, call, json, value: None }, &mut out);
    // Each supplemental chunk consumes at least one source byte within ctl's text bound.
    for _ in 0..env.limits.tools.text {
        let request = out.pop().expect("ctl requests until printed");
        if let root::Request::Printed { printed, .. } = request {
            return printed;
        }
        let Some(event) = response(world, request) else { continue };
        root::step(&mut root, &env, root::Event::Client { event }, &mut out);
    }
    panic!("ctl task show did not finish");
}

fn source_read(what: &c::Read) -> reads::Read {
    match what {
        c::Read::Task { task } => reads::Read::Task { task: *task },
        c::Read::TaskText { task, field: c::TaskTextField::Specification, from } => {
            reads::Read::TaskText { task: *task, field: reads::TaskTextField::Specification, from: *from }
        }
        c::Read::Settings
        | c::Read::Setting(_)
        | c::Read::Revision(_)
        | c::Read::Revisions(_)
        | c::Read::Tasks { .. }
        | c::Read::Me
        | c::Read::Turn { .. }
        | c::Read::TaskText { .. }
        | c::Read::Proposal { .. }
        | c::Read::Escalation { .. }
        | c::Read::Question { .. }
        | c::Read::PersonTask { .. }
        | c::Read::Result { .. }
        | c::Read::Ended { .. }
        | c::Read::History { .. }
        | c::Read::Activity { .. }
        | c::Read::Inbox { .. }
        | c::Read::Spend { .. }
        | c::Read::Project { .. }
        | c::Read::Notes { .. }
        | c::Read::Deployment => panic!("unexpected supplemental read: {what:?}"),
    }
}

fn answer(world: &mut World, read: reads::Read) -> c::ReadValue {
    let answer = world.party_read(read);
    if let reads::Answer::TaskText(page) = answer {
        return c::ReadValue::TaskText(Box::new(c::TaskText {
            task: page.task,
            field: c::TaskTextField::Specification,
            from: page.from,
            total: page.total,
            next: page.next,
            text: c::Text::new(page.bytes, world.env.limits.documents.text_bytes).expect("bounded chunk"),
        }));
    }
    let wire =
        jig_protocol_engine_clients::translate::objects::document(&answer, &[], &jig_client_protocol_wire::CEILINGS)
            .unwrap_or_else(|problem| panic!("engine projection: {problem:?}, {answer:?}"));
    let mut bytes = Writer::new(usize::try_from(wire.measure()).expect("document fits"));
    wire.encode(&mut bytes).expect("measured document");
    let preset = temper_client_tools::limits::client().client;
    let limits = jig_client_protocol_link::Limits::new(preset.domain, preset.exchanges).expect("client limits");
    jig_client_protocol_link::translate::reads::value(&bytes.finish(), &limits).expect("client translation")
}

fn response(world: &mut World, request: root::Request) -> Option<client::Event> {
    Some(match request {
        root::Request::Client { request: client::Request::Down { down: client::Down::Open { watch, .. } } } => {
            let value = answer(world, reads::Read::Frame);
            let c::ReadValue::Me(me) = value else { panic!("me") };
            client::Event::Up {
                up: client::Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(me)) },
            }
        }
        root::Request::Client {
            request: client::Request::Down { down: client::Down::Read { read, what: client::Readable::Jig(what) } },
        } => client::Event::Up {
            up: client::Up::ReadDone {
                read,
                result: client::Readout::Jig(c::ReadResult::Value(answer(world, source_read(&what)))),
            },
        },

        root::Request::Client { .. }
        | root::Request::Ready
        | root::Request::Record { .. }
        | root::Request::Kept { .. }
        | root::Request::Progress { .. }
        | root::Request::Printed { .. }
        | root::Request::Fact { .. } => return None,
    })
}
