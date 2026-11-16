//! The upgrade tool's engine and report over a copy of the baseline's fake disk.
use crate::{fixtures, golden};
use skein_fake_machine::{How, Is, Machine, Opened};
use skein_io::kernel::{Complete, Exit, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_oauth_accounts as accounts;
use skein_shell::Host;
use std::{collections::BTreeMap, path::Path};
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
thread_local! { static CONFIGURATION: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) }; }
fn start(inherited: &skein_world::Inherited) -> SignalHost<Tool> {
    let configuration = CONFIGURATION.with(|bytes| bytes.borrow().clone());
    let mut document: serde_json::Value = serde_json::from_slice(&configuration).expect("configuration JSON");
    let given = temper_playground::documents::given(&document);
    temper_playground::documents::settings::strip_models(&mut document);
    let mut runtime = temper_service::AccountsRuntime { providers: List::with_capacity(2), trust: None };
    for provider in document["engine"]["llm"]["providers"].as_array().expect("LLM providers") {
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
    let mut files = fixtures::files();
    for worker in document["engine"]["hosts"]["workers"].as_array().expect("configured workers") {
        let name = worker["digest"].as_str().expect("digest path").as_bytes();
        if !files.entries.iter().any(|(path, _)| path.as_ref() == name) {
            // A SHA-256 digest has 64 hexadecimal digits.
            const DIGEST_BYTES: usize = 64;
            files
                .entries
                .push((name.into(), skein_lib::Secret::new(Box::new([b'0'; DIGEST_BYTES]))))
                .expect("remote worker fixture files");
        }
    }
    let mut service = temper_service::startup::restoring_with(
        &serde_json::to_vec(&document).expect("configuration"),
        &jig_protocol_config::Directories::working(),
        &files,
        Some(&temper_playground::documents::agent()),
        &fixtures::seeds(),
        Some(runtime),
        &given,
    )
    .unwrap_or_else(|why| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why))));
    let root = |name: &[u8]| inherited.roots.iter().find(|(key, _)| key.as_ref() == name).expect("tool root").1;
    service.rehearse_store(root(b"source"), root(b"scratch"), 1000).expect("tool roots adopted");
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

type Contents = BTreeMap<Vec<u8>, Option<Vec<u8>>>;
fn contents(disk: &mut Machine, index: u32) -> Contents {
    let root = disk.reopen_root(index);
    let mut found = BTreeMap::new();
    collect(disk, root, &[], &mut found);
    disk.close(root);
    found
}
fn collect(disk: &mut Machine, root: Opened, prefix: &[u8], found: &mut Contents) {
    loop {
        let page = disk.list(root, 32, 4096).expect("fixture directory");
        if page.is_empty() {
            break;
        }
        for (kind, name) in page {
            let path = [prefix, name.as_ref()].concat();
            let how = match kind {
                Is::Directory => How::DirectoryNoFollow,
                Is::File => How::ReadNoFollow,
                other => panic!("fixture {other:?}"),
            };
            let child = disk.open(root, &name, how).expect("fixture child");
            match kind {
                Is::Directory => {
                    found.insert(path.clone(), None);
                    collect(disk, child, &[path.as_slice(), b"/"].concat(), found);
                }
                Is::File => {
                    let size = u32::try_from(disk.stat(child).size).expect("fixture size");
                    found.insert(path, Some(disk.read(child, 0, size).expect("fixture bytes")));
                }
                other => panic!("fixture {other:?}"),
            }
            disk.close(child);
        }
    }
}

/// Rehearse the production sets; the floor refuses before records are decoded.
pub fn golden_check(directory: &Path) {
    golden_check_with(directory, golden::read_configuration(directory));
}

/// A deployment's remote workers and owned Forgejo must stay unstarted.
pub fn deployment_check(directory: &Path) {
    golden_check_with(
        directory,
        serde_json::to_vec(&deployment_document(directory)).expect("deployment configuration"),
    );
}

fn deployment_document(directory: &Path) -> serde_json::Value {
    let mut document: serde_json::Value =
        serde_json::from_slice(&golden::read_configuration(directory)).expect("golden configuration");
    document["workers"] = serde_json::json!([]);
    document["application"]["workers"] = serde_json::json!([]);
    document["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7103"}]);
    document["engine"]["hosts"]["workers"] = serde_json::json!([
        {"name":"worker-a", "digest":"secrets/worker-a.digest"},
        {"name":"worker-b", "digest":"secrets/worker-b.digest"},
        {"name":"worker-c", "digest":"secrets/worker-c.digest"}
    ]);
    document["application"]["machine"] = serde_json::json!({"owned": {
        "listen":{"socket":"127.0.0.1:3000"}, "remote":"http://127.0.0.1:3000",
        "forgejo":"/opt/forgejo/forgejo", "runner":null,
        "person":{"login":"person", "email":"person@example.invalid"}, "sign_in":null
    }});
    document
}

/// Configured sign-in sources must not keep the rehearsal waiting for its start mark.
pub fn accounts_check(directory: &Path) {
    let mut document = deployment_document(directory);
    document["engine"]["providers"] = serde_json::json!([
        {"number":1,"kind":"link"}, {"number":2,"kind":"deployment"}, {"number":3,"kind":"application"}
    ]);
    document["engine"]["acts"] = serde_json::json!([{"connector":1,"may":["remove_account","adopt"]}]);
    // The second endpoint and third identity provider add two settings versions.
    document["engine"]["limits"]["overrides"]["core.settings.versions"] = 10.into();
    let mut endpoint = document["engine"]["llm"]["endpoints"][0].clone();
    endpoint["name"] = "second".into();
    endpoint["number"] = 2.into();
    endpoint["provider"] = "golden-provider".into();
    document["engine"]["llm"]["endpoints"].as_array_mut().expect("endpoints").push(endpoint);
    document["engine"]["llm"]["providers"][1]["endpoint"] = "second".into();
    document["engine"]["llm"]["models"][1]["endpoint"] = "second".into();
    for provider in document["engine"]["llm"]["providers"].as_array_mut().expect("providers") {
        provider["signed_in_plan"] = true.into();
        provider["manage_usage"] = "https://accounts.example.invalid/usage".into();
        provider["sources"] = serde_json::json!({"keys":true,"sign_in":{
            "issuer":"http://127.0.0.1:7104", "client":"test-client", "scopes":["openid","read"],
            "authorization":"http://127.0.0.1:7104/authorize", "token":"http://127.0.0.1:7104/token",
            "keys":"http://127.0.0.1:7104/keys", "transport":"plaintext", "must_grant":["read"],
            "redirect_path":"callback"
        }});
    }
    golden_check_with(directory, serde_json::to_vec(&document).expect("accounts configuration"));
}

fn golden_check_with(directory: &Path, configuration: Vec<u8>) {
    let mut disk = golden::files::load(directory);
    let before = contents(&mut disk, 0);
    // The fixture client's root is the tool's disposable scratch, not a running client.
    CONFIGURATION.with(|bytes| *bytes.borrow_mut() = configuration);
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
    assert_eq!(code, 1, "{lines}");
    assert!(
        lines.contains("holds a store older than this build opens: it has no floor, or one below this build's"),
        "{lines}"
    );
    assert!(
        !lines.contains("decoded ")
            && !lines.contains("record not decodable:")
            && !lines.contains("patched ")
            && !lines.contains("patches and restores pass"),
        "{lines}"
    );
    let mut disk = outcome.machine.0.borrow_mut();
    assert_eq!(contents(&mut disk, 0), before, "every source path and byte unchanged");
    assert!(contents(&mut disk, 2).is_empty(), "close emptied scratch");
}
