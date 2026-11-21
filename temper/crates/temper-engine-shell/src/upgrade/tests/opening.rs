//! A stopped deployment reaches the opening without a socket (protocol/binaries.md, 2).
use super::*;
use skein_io::kernel::{Complete, Exit, Op, Submit};
use skein_lib::{Queue, Time, Wall};
use skein_shell::Host;
use temper_engine_domain_forge::{Address, Bot, ForgeRecord, ForgeState, Kind, Mode, Place, Stored};
use temper_engine_service as service;

struct Tool(service::Service);
impl Host for Tool {
    fn iterate(&mut self, now: Time, wall: Wall) {
        service::iterate(&mut self.0, now, wall);
        if self.0.upgrade_report().is_some_and(|report| report.restored) {
            assert!(self.0.engine_component().domain().ready(), "the rehearsal reached decisions opening");
        }
        for submit in self.0.submissions().iter() {
            assert!(
                !matches!(submit.kind, Op::Listen { .. } | Op::Connect { .. } | Op::Spawn { .. }),
                "rehearsal effect: {:?}",
                submit.kind
            );
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.0.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.0.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.0.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.0.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        self.0.next_policy_deadline()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn operations(&self) -> u32 {
        self.0.operations()
    }
    fn drain(&mut self) {
        while self.0.notes().pop().is_some() {}
        while self.0.listening().pop().is_some() {}
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(0))
    }
}

fn check(directories: &temper_shell::directories::Directories) -> (String, u8) {
    let source = crate::path(&directories.data).join("state");
    let scratch = crate::path(&directories.cache).join("check");
    std::fs::create_dir(&scratch).unwrap();
    let (mut service, _) = crate::stopped::prepare_rehearsal(directories, "test-build", |store| {
        store
            .adopt_rehearsal(
                skein_shell::open_root(&source).unwrap(),
                skein_shell::open_root(&scratch).unwrap(),
                skein_shell::effective_user(),
            )
            .map_err(|_| "adopt rehearsal".to_owned())
    })
    .unwrap();
    let store = service.engine_component().store();
    store.ask(jig_protocol_store::Request::Get {
        owner: skein_lib::Token::new(77),
        key: jig_protocol_store::Get::Application(
            temper_protocol_store::keys::encode(&temper_engine_domain_forge::Key::Forge(1), u32::MAX).unwrap(),
        ),
    });
    let mut saved = Queue::uncapped();
    let mut found = false;
    while let Some(event) = store.events().pop() {
        if let jig_protocol_store::Event::Got { owner, row } = event {
            assert_eq!(owner, skein_lib::Token::new(77));
            assert!(row.is_some(), "forge in the copy");
            found = true;
        } else {
            saved.push(event);
        }
    }
    assert!(found);
    while let Some(event) = saved.pop() {
        store.events().push(event);
    }
    let mut tool = Tool(service);
    let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: tool.operations() }).unwrap();
    assert_eq!(skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut tool), Exit::Code(0));
    let report = tool.0.upgrade_report().expect("check report");
    assert!(report.done);
    assert_eq!(std::fs::read_dir(scratch).unwrap().count(), 0, "scratch emptied");
    super::super::lines("temper-engine", report, directories, source.as_os_str().as_bytes())
}

// Patch the image with the start's store-job adapter, preserving its wires and worker openings.
struct Seed {
    store: service::store::Store,
    completions: Queue<Complete>,
    last: u64,
    writes: Vec<jig_protocol_store::Write>,
}
impl Seed {
    fn header(&mut self) {
        self.store.ask(jig_protocol_store::Request::Get {
            owner: skein_lib::Token::new(1),
            key: jig_protocol_store::Get::Jig(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)),
        });
    }
}
impl Host for Seed {
    #[expect(clippy::wildcard_enum_match_arm, reason = "the fixture accepts only its store operation's terminals")]
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.store.complete(complete);
        }
        self.store.clock(now, wall);
        self.store.iterate();
        while let Some(event) = self.store.events().pop() {
            use jig_protocol_store::{Event, Request, Row, Write};
            match event {
                Event::Opened { last, pending, .. } => {
                    self.last = last;
                    if pending {
                        self.store.ask(Request::Switch { owner: skein_lib::Token::new(1) });
                    } else {
                        self.header();
                    }
                }
                Event::Switched { .. } => self.header(),
                event @ (Event::Job { .. } | Event::Site { .. }) => assert!(self.store.opening_site(event)),
                Event::Got {
                    row:
                        Some(Row::Jig(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(mut header)))),
                    ..
                } => {
                    self.last += 1;
                    header.commits = self.last;
                    let mut writes = std::mem::take(&mut self.writes);
                    writes.push(Write::Save(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
                        header,
                    ))));
                    self.store.ask(Request::Commit { number: self.last, writes: writes.into_boxed_slice() });
                }
                Event::Committed { number, .. } => {
                    if self.writes.is_empty() && number == self.last {
                        self.store.ask(Request::Get {
                            owner: skein_lib::Token::new(1),
                            key: jig_protocol_store::Get::Application(
                                temper_protocol_store::keys::encode(
                                    &temper_engine_domain_forge::Key::Forge(1),
                                    u32::MAX,
                                )
                                .unwrap(),
                            ),
                        });
                    }
                }
                Event::Got { row: Some(Row::Application { value, .. }), .. } => {
                    assert!(!value.is_empty(), "seeded forge record");
                    self.store.close();
                }
                Event::Census { .. } | Event::Ran { .. } => {}
                other => panic!("fixture store: {other:?}"),
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.store.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty() || self.store.work_pending(now)
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.store.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.store.closed()
    }
    fn operations(&self) -> u32 {
        64
    }
    fn drain(&mut self) {}
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(0))
    }
}
fn seed(
    path: &Path,
    writes: Vec<jig_protocol_store::Write>,
    jig: &jig_protocol_config::Limits,
    own: &service::limits::Limits,
) {
    let root = service::limits::engine(jig, own).unwrap();
    let limits = service::store::limits(&root, &root.forge, &jig.store).unwrap();
    let mut store = service::store::Store::new(&root, &limits, &[], &crate::serve::seeds(None, None).unwrap());
    store.adopt(skein_shell::open_root(path).unwrap(), skein_shell::effective_user()).unwrap();
    let mut seed = Seed { store, completions: Queue::uncapped(), last: 0, writes };
    let mut kernel = skein_shell::Kernel::open(skein_shell::Config { operations: seed.operations() }).unwrap();
    assert_eq!(skein_shell::drive(&mut kernel, &skein_shell::Clock::new(), &mut seed), Exit::Code(0));
    assert!(seed.writes.is_empty(), "fixture writes committed");
}

#[test]
fn a_live_change_and_uncertain_write_reach_the_opening_without_network() {
    let (_scratch, directories) = deployment();
    let state = crate::path(&directories.data).join("state");
    // The settled story keeps a live change and its run, unlike the floor's ended change.
    std::fs::remove_dir_all(&state).unwrap();
    let image = skein_scratch::paths::tree().join("tests/one-process/golden/settled");
    temper_deployment_fixtures::copy(&image.join("state"), &state);
    for line in std::fs::read_to_string(image.join("modes.txt")).unwrap().lines() {
        let fields: Vec<_> = line.split(' ').collect();
        if fields[0] == "d"
            && let Some(path) = fields[3].strip_prefix("state/")
        {
            std::fs::create_dir_all(state.join(path)).unwrap();
            std::fs::set_permissions(state.join(path), std::fs::Permissions::from_mode(0o700)).unwrap();
        }
    }
    let document = temper_config_shell::read(&directories, temper_config_shell::Shape::EngineAlone).unwrap().document;
    let settings = crate::serve::settings(&document, &directories, &service::stock::Given::Stock);
    let mut own = service::limits::stock();
    service::limits::derive(&mut own, temper_service_core::derived::STOCK).unwrap();
    let forge = Stored::Forge(ForgeRecord {
        number: 1,
        kind: Kind::Forgejo,
        mode: Mode::Connected,
        state: ForgeState::Kept,
        place: Place {
            api: Address { host: b"127.0.0.1".as_slice().into(), port: 1, tls: None },
            git: b"http://127.0.0.1:1".as_slice().into(),
            web: None,
        },
        git_program: None,
        read_only: false,
        bot: Bot {
            login: b"bot".as_slice().into(),
            name: b"Bot".as_slice().into(),
            email: b"bot@example.invalid".as_slice().into(),
        },
        asked: false,
        first: None,
        person: None,
    });
    let forge = jig_protocol_store::Write::Put {
        key: temper_protocol_store::keys::encode(&temper_engine_domain_forge::Key::Forge(1), u32::MAX).unwrap(),
        value: temper_protocol_store::records::encode(&forge),
    };
    // This is a deployment fixture, not a playground: restarting must keep its project and forge.
    let playground = jig_protocol_store::Write::Remove {
        key: temper_protocol_store::keys::encode(&temper_engine_domain_forge::Key::Playground, u32::MAX).unwrap(),
    };
    seed(&state, vec![playground, forge], &settings.limits, &own);
    let before = temper_deployment_fixtures::tree(&state);
    let (text, code) = check(&directories);
    assert_eq!(code, 0, "{text}");
    let lines: Vec<_> = text.lines().collect();
    let opening = lines
        .iter()
        .position(|line| line.contains("registry commit, jobs and restores pass; opened in "))
        .expect(&text);
    let millis = lines[opening].split("opened in ").nth(1).unwrap().strip_suffix(" ms").unwrap();
    assert!(millis.parse::<u64>().is_ok(), "{text}");
    assert!(lines[opening + 1].starts_with("temper-engine upgrade: forge 1 would catch up: "), "{text}");
    assert!(lines[opening + 1].contains("1 uncertain write"), "{text}");
    assert!(lines[opening + 1].contains("fresh reads"), "CatchUp was routed before the snapshot: {text}");
    // Its old worker statement differs from this binary's limits. The check's Wires
    // commit erases that statement, so this refusal proves the pre-commit copy was read.
    assert!(text.contains("worker local may not stay: limits:"), "{text}");
    assert!(text.ends_with("workers may stay: no\n"), "{text}");
    assert_eq!(temper_deployment_fixtures::tree(&state), before);
}
