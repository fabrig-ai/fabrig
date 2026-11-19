//! The upgrade tool's engine and report over a copy of the floor's fake disk.
use crate::{fixtures, golden};
use jig_protocol_store::{Event, KeptMap, Older, OpenRefusal, Undecoded};

use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_oauth_accounts as accounts;
use skein_shell::Host;
use std::path::Path;
use temper_playground::machine::{self, SignalHost};

struct Tool(temper_service::Service, Time);
impl Host for Tool {
    fn iterate(&mut self, now: Time, wall: Wall) {
        self.1 = now;
        temper_service::iterate(&mut self.0, now, wall);
        for submit in self.0.submissions().iter() {
            if let skein_io::kernel::Op::Listen { .. }
            | skein_io::kernel::Op::Connect { .. }
            | skein_io::kernel::Op::Spawn { .. } = &submit.kind
            {
                panic!("upgrade tool made an effect: {:?}", submit.kind);
            }
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
        self.0.check_wait_boundary(self.1);
    }
    fn exit(&self) -> Option<Exit> {
        self.is_empty().then_some(Exit::Code(render(&self.0).1))
    }
}
fn render(service: &temper_service::Service) -> (String, u8) {
    temper_engine_shell::upgrade::lines(
        "temper",
        service.upgrade_report().expect("tool report"),
        &temper_shell::directories::resolve(&[], &[(b"HOME".as_slice().into(), b"/home/test".as_slice().into())], b"/")
            .expect("absolute directories")
            .0,
        b"/state",
    )
}
#[derive(Default)]
struct Judge {
    done: bool,
}
impl skein_world::Referee<SignalHost<Tool>> for Judge {
    fn watch(&mut self, _: Time, procs: &[SignalHost<Tool>]) -> Result<(), String> {
        self.done = procs.iter().all(Host::is_empty);

        Ok(())
    }
    fn passed(&self) -> bool {
        self.done
    }
}
thread_local! { static CONFIGURATION: std::cell::RefCell<Option<temper_playground::documents::Configuration>> = const { std::cell::RefCell::new(None) }; }
fn start(inherited: &skein_world::Inherited) -> SignalHost<Tool> {
    let mut document = CONFIGURATION.with(|bytes| bytes.borrow().as_ref().expect("configured rehearsal").clone());
    let given = temper_playground::documents::given(&document);
    temper_playground::documents::settings::strip_models(&mut document);
    let mut runtime = temper_service::AccountsRuntime { providers: List::with_capacity(2), trust: None };
    for provider in document["world_settings"]["llm"]["providers"].as_array().expect("LLM providers") {
        if provider["sources"]["sign_in"].is_object() {
            let endpoint = || accounts::Endpoint {
                address: ([127, 0, 0, 1], 7104).into(),
                transport: accounts::Transport::Plaintext,
            };
            let mut origins = List::with_capacity(1);
            origins
                .push(accounts::Origin { uri: b"http://127.0.0.1:7104".as_slice().into(), endpoint: endpoint() })
                .expect("sign-in origin");
            runtime
                .providers
                .push((
                    u32::try_from(provider["number"].as_u64().expect("provider number")).expect("u32"),
                    endpoint(),
                    origins,
                ))
                .expect("two providers");
        }
    }
    let files = fixtures::files();
    document.as_object_mut().expect("world document").remove("world_settings");
    let mut service = temper_service::startup::restoring_with_limits(
        &serde_json::to_vec(&document).expect("configuration"),
        &jig_protocol_config::Directories::working(),
        &files,
        Some(&temper_playground::documents::agent()),
        &fixtures::seeds(),
        Some(runtime),
        &given,
        &document.jig,
        &document.limits,
    )
    .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))));
    let root = |name: &[u8]| inherited.roots.iter().find(|(key, _)| key.as_ref() == name).expect("tool root").1;
    service
        .rehearse_store(root(b"source"), root(b"scratch"), 1000, jig_protocol_store::Reach::Whole)
        .expect("tool roots adopted");
    let owners = service.wait_state().engine.expect("upgrade engine");
    assert!(
        owners.workers.closed && owners.setup.closed && owners.forge.closed,
        "upgrade closes unstarted owners before store I/O: {owners:?}"
    );
    if let Some(accounts) = owners.accounts {
        assert!(accounts.empty && !accounts.work_pending && !accounts.opened, "unstarted accounts close: {accounts:?}");
    }
    SignalHost::new(Tool(service, Time::ZERO), Some(inherited.signal))
}

/// Every floor record decodes and restores after the id jobs.
pub fn golden_check(directory: &Path) {
    golden_check_with(directory, golden::read_configuration(directory), Copy::Whole);
}

/// A malformed value is named before any plan or restore.
pub fn broken_check(directory: &Path) {
    golden_check_with(directory, golden::read_configuration(directory), Copy::Broken);
}
/// A floorless copy refuses before the decoding walk.
pub fn floorless_check(directory: &Path) {
    golden_check_with(directory, golden::read_configuration(directory), Copy::Floorless);
}
#[derive(Clone, Copy)]
enum Copy {
    Whole,
    Broken,
    Floorless,
}

/// A deployment's remote workers and owned Forgejo must stay unstarted.
pub fn deployment_check(directory: &Path) {
    golden_check_with(directory, deployment_document(directory), Copy::Whole);
}

fn deployment_document(directory: &Path) -> temper_playground::documents::Configuration {
    let mut document = golden::read_configuration(directory);
    document["workers"] = serde_json::json!([]);

    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7103"}]);
    document
}

/// Configured sign-in sources must not keep the rehearsal waiting for its start mark.
pub fn accounts_check(directory: &Path) {
    let mut document = deployment_document(directory);
    // The configured endpoints and identity providers coexist with the floor's kept settings versions.
    document.jig_over("core.settings.versions", 12);
    document["world_settings"]["llm"]["endpoints"][1]["name"] = "second".into();
    document["world_settings"]["llm"]["providers"][1]["endpoint"] = "second".into();
    document["world_settings"]["llm"]["models"][1]["endpoint"] = "second".into();
    for provider in document["world_settings"]["llm"]["providers"].as_array_mut().expect("providers") {
        provider["signed_in_plan"] = true.into();
        provider["manage_usage"] = "https://accounts.example.invalid/usage".into();
        provider["sources"] = serde_json::json!({"keys":true,"sign_in":{
            "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
            "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
            "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
            "redirect_path":"callback"
        }});
    }
    golden_check_with(directory, document, Copy::Whole);
}

fn golden_check_with(directory: &Path, configuration: temper_playground::documents::Configuration, copy: Copy) {
    let mut disk = golden::files::load(directory);
    let broken =
        jig_protocol_store::keys::encode(&jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment), 64)
            .expect("deployment key");
    match copy {
        Copy::Whole => {}
        Copy::Broken => golden::files::change_record(&mut disk, &broken, Some(&[0])),
        Copy::Floorless => golden::files::change_record(&mut disk, &[0, 13], None),
    }
    let before = golden::files::image_root(&mut disk, 0);
    // The fixture client's root is the tool's disposable scratch, not a running client.
    CONFIGURATION.with(|bytes| *bytes.borrow_mut() = Some(configuration));
    let mut world = skein_world::World::new(
        fixtures::SEED,
        skein_sim::Config::calm(),
        Judge::default(),
        skein_world::Memory::Unchecked,
    )
    .with_machine(machine::Machine::new(disk));
    world.spawn_restartable(
        vec![
            skein_world::StartupRoot { name: b"source".as_slice().into(), path: b"state".as_slice().into() },
            skein_world::StartupRoot { name: b"scratch".as_slice().into(), path: b"client".as_slice().into() },
        ],
        start,
    );
    let outcome = world.run();
    let [tool] = outcome.procs.as_slice() else { panic!("one upgrade tool") };
    let (lines, code) = render(&tool.0);
    eprintln!("{lines}");
    let report = tool.0.upgrade_report().expect("completed report");
    assert!(report.done, "report complete: {lines}");
    let ran: Vec<_> = report
        .events
        .iter()
        .filter_map(|event| {
            if let Event::Ran { of, job, counts, .. } = event {
                assert_eq!(*of, super::job_owner(job));
                assert!(counts.iter().all(|count| count.count == 0), "short floor ids: {lines}");
                Some(job.as_ref())
            } else {
                None
            }
        })
        .collect();
    match copy {
        Copy::Whole => {
            assert_eq!(
                ran,
                [
                    "escalation-deciders",
                    "test-ids-landing",
                    "test-ids-receipt",
                    "test-ids-tier",
                    "test-ids-verification"
                ]
            );
            assert_eq!(code, 0, "{lines}");
            assert!(report.restored, "{lines}");
            assert!(
                report.events.iter().any(|event| matches!(event, Event::Decoded { records: 293, .. })),
                "literal record count: {lines}"
            );
            assert!(lines.contains("registry commit, jobs and restores pass"), "{lines}");
        }
        Copy::Broken => {
            assert!(ran.is_empty(), "broken store before jobs: {lines}");
            assert_eq!(code, 1, "{lines}");
            assert!(!report.restored);
            assert!(report.events.iter().any(|event| matches!(event, Event::Undecodable { family, version: 0, at: Undecoded::Record { map: KeptMap::Store, key }, .. } if family.as_ref() == "jig-store-core" && key.as_ref() == broken.as_ref())), "broken record named: {lines}");
            assert!(!report.events.iter().any(|event| matches!(event, Event::Decoded { .. })), "{lines}");
        }
        Copy::Floorless => {
            assert!(ran.is_empty(), "floor before jobs: {lines}");
            assert_eq!(code, 1, "{lines}");
            assert!(!report.restored);
            assert_eq!(
                report.end,
                Some(temper_service::EngineEnd::Stopped(temper_service::EngineStop::StoreOpen {
                    why: OpenRefusal::OlderBuild { why: Older::Floor }
                })),
                "{lines}"
            );
            assert!(lines.contains(jig_protocol_store::FLOOR_BRIDGE), "{lines}");
            assert!(
                !report.events.iter().any(|event| matches!(event, Event::Decoded { .. } | Event::Undecodable { .. })),
                "floor before decoding: {lines}"
            );
        }
    }
    let mut disk = outcome.machine.0.borrow_mut();
    golden::files::unchanged(&before, &golden::files::image_root(&mut disk, 0));
    assert!(golden::files::image_root(&mut disk, 2).len() == 1, "close emptied scratch");
}
