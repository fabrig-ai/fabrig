//! Machine templates and both demo shapes through startup (protocol/configuration.md, 3).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests construct JSON fixtures and inspect startup diagnostics"
)]
use super::fixtures;
use crate::startup::{Problems, What, lines};
use jig_protocol_config::{self as config, Path};
use temper_engine_domain_forge::{self as forge, Api, ApiTransport, Bot, ForgeRecord, ForgeState, Machine, Mode};

const GITHUB: &str = r#"{"api":null,"transport":null,"git":null,"web":null}"#;
const OWNED: &str = r#"{"bot":{"login":"temper","email":"temper@example.test"},"mode":{"owned":{"forgejo":"fetched","runner":null,"listen":{"socket":"127.0.0.1:8368"},"person":{"login":"ada","email":"ada@example.test"}}}}"#;
const FAKE: &str = r#"{"bot":{"login":"temper","email":"temper@example.test"},"mode":{"fake":{"listen":{"socket":"127.0.0.1:8369"},"git":"demo/bin/git","owner":"demo","repositories":[{"name":"widget","landing":"main"}]}}}"#;
// protocol/configuration.md: a project name holds at most 64 UTF-8 bytes.
const PROJECT_NAME_BYTES: usize = 64;
const PROJECT: &[u8] = br#"{"name":"Playground","policy":1}"#;

fn document(machine: Option<&str>) -> Box<[u8]> {
    let input = fixtures::document();
    let input = core::str::from_utf8(&input).unwrap();
    let input = input.replace("\"machine\":{\"owned\":null,\"github\":{\"api\":null,\"transport\":null,\"git\":null,\"web\":null},\"git_program\":null},", "");
    match machine {
        Some(machine) => input
            .replace("\"demo\":null", &format!("\"machine\":{machine},\"demo\":null"))
            .into_bytes()
            .into_boxed_slice(),
        None => input.into_bytes().into_boxed_slice(),
    }
}
fn machine(owned: &str, github: &str, git_program: &str) -> Box<str> {
    format!(r#"{{"owned":{owned},"github":{github},"git_program":{git_program}}}"#).into_boxed_str()
}
fn named(input: &[u8]) -> crate::startup::Named {
    crate::named(input, &config::Directories::working())
        .unwrap_or_else(|found| panic!("{}", core::str::from_utf8(&lines(&found)).unwrap()))
}
fn start(input: &[u8]) -> crate::Service {
    crate::startup(
        input,
        &config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        skein_lib::List::with_capacity(0),
    )
    .unwrap_or_else(|found| panic!("{}", core::str::from_utf8(&lines(&found)).unwrap()))
}
fn demo(input: &[u8], project: &[u8]) -> Box<[u8]> {
    let llm = br#"{"provider":3,"key":"demo/llm.key","asked":[{"cue":"Add CSV export to widget"}]}"#;
    let value = format!(
        r#"{{"forge":{FAKE},"project":{},"llm":{}}}"#,
        core::str::from_utf8(project).unwrap(),
        core::str::from_utf8(llm).unwrap()
    );
    fixtures::with(input, b"application.demo", value.as_bytes())
}
fn assert_problem(input: &[u8], path: Path, what: config::What) {
    let found = crate::named(input, &config::Directories::working()).unwrap_err();
    assert_eq!(found.problems.len(), 1, "{found:?}");
    let found = found.problems.get(0).unwrap();
    assert_eq!((&found.path, found.what), (&path, What::Jig(what)));
}
fn local_record(mode: Mode, port: u16, text: &str, playground: bool, git_program: Option<Box<[u8]>>) -> ForgeRecord {
    ForgeRecord {
        number: 0,
        kind: forge::Kind::Forgejo,
        mode,
        api: Api { host: b"127.0.0.1".as_slice().into(), port, transport: ApiTransport::Plaintext },
        git: format!("http://127.0.0.1:{port}").into_bytes().into_boxed_slice(),
        git_program,
        web: None,
        read_only: false,
        bot: Bot {
            login: b"temper".as_slice().into(),
            name: b"Temper".as_slice().into(),
            email: b"temper@example.test".as_slice().into(),
        },
        setup: Some(text.as_bytes().into()),
        first: None,
        playground,
        state: ForgeState::Kept,
    }
}

#[test]
fn a_document_with_no_forge_identity_or_setup_starts() {
    let settings = fixtures::engine();
    assert!(settings.forges.is_empty());
    assert!(settings.identities.is_empty());
    assert!(settings.setup.is_empty());
    assert!(settings.records_only.is_empty());
    let parsed = config::document(
        fixtures::APPLICATION,
        &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 },
    )
    .unwrap();
    for member in [b"forges".as_slice(), b"identities", b"setup"] {
        assert!(config::read::Object::new(&parsed, 0).unwrap().member(&parsed, member).is_none());
    }
    let input = document(None);
    let named = named(&input);
    assert!(named.forges.is_empty());
    assert!(named.identities.is_empty());
    assert_eq!(named.machine, Machine::bare());
    let service = start(&input);
    assert_eq!(service.engine_domain().unwrap().forges().rows.len(), 0);
    // Explicit empty lists remain accepted too.
    let input = fixtures::with(&fixtures::document_members::document(), b"application.forges", b"[]");
    let input = fixtures::with(&input, b"application.identities", b"[]");
    start(&input);
}
#[test]
fn the_machines_owned_body_becomes_an_owned_record_template() {
    // Compact body text has a 4096-byte store ceiling (contracts T3).
    let body_bytes = usize::try_from(temper_forge_setup::setup::BODY_BYTES).unwrap();
    let input = document(Some(&machine(OWNED, GITHUB, "null")));
    assert_eq!(named(&input).machine.owned, Some(local_record(Mode::Owned, 8368, OWNED, false, None)));
    start(&input);
    let public = OWNED.replace("\"forgejo\":", "\"public\":\"https://forge.example.test\",\"forgejo\":");
    let input = document(Some(&machine(&public, GITHUB, "null")));
    assert_eq!(named(&input).machine.owned.unwrap().git.as_ref(), b"http://127.0.0.1:8368");
    let empty = OWNED.replace("fetched", "");
    let padding = "x".repeat(body_bytes - empty.len());
    let body = OWNED.replace("fetched", &padding);
    let input = document(Some(&machine(&body, GITHUB, "null")));
    assert_eq!(named(&input).machine.owned.unwrap().setup.as_ref().unwrap().len(), body_bytes);
    let body = OWNED.replace("fetched", &(padding + "x"));
    assert_problem(
        &document(Some(&machine(&body, GITHUB, "null"))),
        Path::root().member(b"application").member(b"machine").member(b"owned"),
        config::What::Bound,
    );
}
#[test]
fn a_machine_owned_body_of_another_mode_is_refused_at_its_mode() {
    let input = document(Some(&machine(FAKE, GITHUB, "null")));
    assert_problem(
        &input,
        Path::root().member(b"application").member(b"machine").member(b"owned").member(b"mode"),
        config::What::Form,
    );
}
#[test]
fn the_machines_github_nulls_are_github_coms() {
    let input = document(Some(&machine("null", GITHUB, "null")));
    assert_eq!(named(&input).machine, Machine::bare());
}
#[test]
fn the_machines_github_members_replace_the_defaults() {
    for (api, transport, host, port, expected, origin_bytes) in [
        (r#"{"socket":"[::1]:8443"}"#, r#""plaintext""#, b"::1".as_slice(), 8443, ApiTransport::Plaintext, 18),
        (
            r#"{"host":"github.example.test","port":443}"#,
            r#"{"tls":{"server_name":"github.example.test","trust":null}}"#,
            b"github.example.test".as_slice(),
            443,
            ApiTransport::Tls { server_name: b"github.example.test".as_slice().into(), trust: None },
            31,
        ),
    ] {
        let github = format!(
            r#"{{"api":{api},"transport":{transport},"git":"https://git.example.test","web":"https://web.example.test"}}"#
        );
        let input = document(Some(&machine("null", &github, "null")));
        let mut record = ForgeRecord::github_com();
        record.api = Api { host: host.into(), port, transport: expected };
        record.git = b"https://git.example.test".as_slice().into();
        record.web = Some(b"https://web.example.test".as_slice().into());
        let machine = named(&input).machine;
        assert_eq!(machine.github, record);
        let writes = crate::startup::checks::record_writes(&fixtures::engine().limits, &[], &machine);
        assert_eq!(*writes.get(5).unwrap(), (Some(origin_bytes), b"Origin".as_slice(), None));
    }
}
#[test]
fn the_machines_git_program_is_the_github_templates_resolved_under_data() {
    // Data's six-byte prefix leaves 4090 bytes for a relative program at the path ceiling.
    const RELATIVE_PROGRAM_BYTES: usize = 4090;
    let input = demo(&document(Some(&machine(OWNED, GITHUB, r#""bin/git""#))), PROJECT);
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").unwrap();
    let machine = crate::named(&input, &directories).unwrap().machine;
    assert_eq!(machine.github.git_program.as_deref(), Some(b"/data/bin/git".as_slice()));
    assert_eq!(machine.owned.unwrap().git_program, None);
    assert_eq!(machine.playground.unwrap().git_program.as_deref(), Some(b"/data/demo/bin/git".as_slice()));
    let path = "x".repeat(RELATIVE_PROGRAM_BYTES);
    let input = document(Some(&self::machine("null", GITHUB, &format!("\"{path}\""))));
    assert_eq!(crate::named(&input, &directories).unwrap().machine.github.git_program.unwrap().len(), 4096);
    for (program, what) in [
        (format!("\"{path}x\""), config::What::Bound),
        (r#""bin/\u0000git""#.into(), config::What::Form),
        (r#""../git""#.into(), config::What::Relative),
    ] {
        let input = document(Some(&self::machine("null", GITHUB, &program)));
        let found = crate::named(&input, &directories).unwrap_err();
        assert_eq!(found.problems.len(), 1);
        let found = found.problems.get(0).unwrap();
        assert_eq!(
            (&found.path, found.what),
            (&Path::root().member(b"application").member(b"machine").member(b"git_program"), What::Jig(what))
        );
    }
}
#[test]
fn a_machine_member_unknown_is_refused() {
    for (input, path) in [
        (
            format!(r#"{{"owned":null,"github":{GITHUB},"git_program":null,"extra":true}}"#).into_boxed_str(),
            Path::root().member(b"application").member(b"machine").member(b"extra"),
        ),
        (
            machine("null", r#"{"api":null,"transport":null,"git":null,"web":null,"extra":true}"#, "null"),
            Path::root().member(b"application").member(b"machine").member(b"github").member(b"extra"),
        ),
    ] {
        assert_problem(&document(Some(&input)), path, config::What::Unknown);
    }
}
#[test]
fn a_demo_of_t4s_shape_reads_its_fake_body_and_project() {
    let input = demo(&document(None), PROJECT);
    assert_eq!(
        named(&input).machine.playground,
        Some(local_record(Mode::Fake, 8369, FAKE, true, Some(b"demo/bin/git".as_slice().into())))
    );
    let service = start(&input);
    let demo = service.engine.as_ref().unwrap().demo.as_ref().unwrap();
    assert_eq!((demo.project, demo.forge, demo.project_name.as_ref(), demo.project_policy), (0, 0, "Playground", 1));
    assert_eq!(
        (demo.owner.as_ref(), demo.name.as_ref(), demo.landing.as_ref()),
        (b"demo".as_slice(), b"widget".as_slice(), b"main".as_slice())
    );
    assert_eq!(demo.cues.as_ref(), &[Box::<[u8]>::from(b"Add CSV export to widget".as_slice())]);
    assert_eq!(demo.key_file.as_ref(), b"demo/llm.key");
    let domain = demo.domain(&[], &named(&input).machine, skein_lib::Duration::from_secs(120));
    assert_eq!(
        (domain.project_name.as_ref(), domain.project_policy, domain.adoption.host.as_ref()),
        ("Playground", 1, b"127.0.0.1:8369".as_slice())
    );
}
#[test]
fn a_demo_project_name_past_64_bytes_or_policy_0_is_refused() {
    let name = "x".repeat(PROJECT_NAME_BYTES);
    let input = demo(&document(None), format!(r#"{{"name":"{name}","policy":4294967295}}"#).as_bytes());
    start(&input);
    for (project, member, what) in [
        (format!(r#"{{"name":"{name}x","policy":1}}"#), b"name".as_slice(), config::What::Bound),
        (r#"{"name":"Playground","policy":0}"#.into(), b"policy".as_slice(), config::What::Form),
        (r#"{"name":"","policy":1}"#.into(), b"name".as_slice(), config::What::Form),
        (r#"{"name":"Playground","policy":4294967296}"#.into(), b"policy".as_slice(), config::What::Bound),
    ] {
        assert_problem(
            &demo(&document(None), project.as_bytes()),
            Path::root().member(b"application").member(b"demo").member(b"project").member(member),
            what,
        );
    }
}
#[test]
fn todays_demo_shape_reads_as_before() {
    let application = fixtures::document_members::demo_application();
    let input = fixtures::with(
        &fixtures::document_members::document(),
        b"application.demo",
        &fixtures::value(&application, b"demo"),
    );
    let input = fixtures::with(&input, b"application.setup", &fixtures::value(&application, b"setup"));
    let service = crate::startup(
        &input,
        &config::Directories::working(),
        &fixtures::files(),
        &fixtures::startup_agent(),
        &fixtures::seeds(),
        None,
        fixtures::worker_runtime(),
        fixtures::loopback_runtime(),
    )
    .unwrap();
    let demo = service.engine.as_ref().unwrap().demo.as_ref().unwrap();
    assert_eq!((demo.project, demo.forge, demo.project_name.as_ref(), demo.project_policy), (1, 1, "", 0));
    assert!(named(&input).machine.playground.is_none());
}
#[test]
fn forge_forges_is_an_override_and_must_fit_the_component() {
    let input = fixtures::with(
        &document(None),
        b"application.limits.overrides",
        br#"{"forge.forges":4,"forge_component.forges":4}"#,
    );
    start(&input);
    let input =
        fixtures::with(&input, b"application.limits.overrides", br#"{"forge.forges":4,"forge_component.forges":3}"#);
    let found: Problems = crate::named(&input, &config::Directories::working()).unwrap_err();
    assert_eq!(lines(&found).as_ref(), b"application.limits.overrides.forge_component.forges: must agree with application.limits.overrides.forge.forges\n");
}
#[test]
fn the_root_config_carries_the_machine() {
    // Owned, GitHub and playground each reserve one Origin diagnostic.
    const TEMPLATE_NOTES: u32 = 3;
    let input = demo(&document(Some(&machine(OWNED, GITHUB, "null"))), PROJECT);
    let mut service = start(&input);
    assert!(service.engine_domain().is_some());
    let acts = forge::demo_act_bound(&service.engine.as_ref().unwrap().env.limits.forge).unwrap();
    assert_eq!(service.notes().capacity(), TEMPLATE_NOTES + acts);
    assert_eq!(named(&input).machine.owned.unwrap().mode, Mode::Owned);
}
