use crate::*;
mod overlay;
use serde_json::{Value, json};

pub(crate) fn facts() -> Facts {
    Facts { login: "ada".into(), hostname: "host".into(), provider_key: true }
}
fn document(read: &Read) -> Value {
    serde_json::from_slice(&read.document).expect("start's JSON")
}

const WORKER_ENGINE: &str = "engine = \"127.0.0.1:7200\"\n";
// 32 UTF-8 bytes, the stock link.name bound (configuration/file.md, 3.1).
const HOST_NAME_AT_BOUND: &str = "0123456789abcdefghijklmnopqrsté";

fn long_host_name() -> Facts {
    Facts { hostname: format!("{HOST_NAME_AT_BOUND}x"), ..facts() }
}

#[test]
fn the_default_worker_name_keeps_every_byte_at_the_bound() {
    let facts = Facts { hostname: HOST_NAME_AT_BOUND.into(), ..facts() };
    let read = parse(WORKER_ENGINE.as_bytes(), Shape::WorkerAlone, &facts).expect("host name at the bound");
    assert_eq!(document(&read)["workers"][0]["name"], HOST_NAME_AT_BOUND);
}

#[test]
fn the_default_worker_name_past_the_bound_names_the_remedy() {
    let found = parse(WORKER_ENGINE.as_bytes(), Shape::WorkerAlone, &long_host_name())
        .expect_err("host name one byte past the bound");
    assert_eq!(
        found.lines().as_ref(),
        b"temper.toml: name (its default): the host's name is 33 bytes, past its bound of 32; give name explicitly\n"
    );
}

#[test]
fn an_explicit_worker_name_overrides_an_overlong_host_name() {
    let text = format!("{WORKER_ENGINE}name = \"w\"\n");
    let read = parse(text.as_bytes(), Shape::WorkerAlone, &long_host_name()).expect("explicit name");
    assert_eq!(document(&read)["workers"][0]["name"], "w");
}

#[test]
fn shapes_with_an_engine_ignore_an_overlong_host_name() {
    let facts = long_host_name();
    for (shape, text, names) in
        [(Shape::Standalone, "", vec!["local"]), (Shape::EngineAlone, "workers = \"127.0.0.1:7200\"\n", vec![])]
    {
        let read = parse(text.as_bytes(), shape, &facts).expect("the host does not name a worker in this shape");
        let value = document(&read);
        let workers = value["workers"].as_array().expect("workers array");
        assert_eq!(workers.iter().map(|worker| worker["name"].as_str().expect("name")).collect::<Vec<_>>(), names);
    }
}

#[test]
fn the_examples_give_the_documents_and_pass_the_shapes_checks() {
    let read = parse(b"listen = \"127.0.0.1:7100\"\n", Shape::Standalone, &facts()).expect("box file");
    let value = document(&read);
    assert_eq!(
        value["engine"],
        json!({
            "hosts": {"listener": [{"in_process": "workers"}]},
            "owners": [{"provider": 1, "subject": "ada@localhost"}],
            "clients": {"provider": {"key": "provider.key", "clients": [], "addresses": [1]}}
        })
    );
    assert_eq!(
        value["workers"],
        json!([{"engine": {"in_process":"workers"}, "name":"local", "secret":"worker.secret", "slots":1}])
    );
    assert_eq!(
        value["application"],
        json!({
            "clients": {"listener": {"socket":"127.0.0.1:7100"}, "public": null, "bundle": null},
            "webhooks": {"listener": {"socket":"127.0.0.1:7101"}}, "sign_in": null
        })
    );
    let dirs = jig_protocol_config::Directories::working();
    temper_service::startup::named(&read.document, &dirs).expect("standalone decoders and cross checks");
    let without =
        parse(b"", Shape::Standalone, &Facts { provider_key: false, ..facts() }).expect("without provider file");
    assert!(document(&without)["engine"]["clients"].get("provider").is_none());
    temper_service::startup::named(&without.document, &dirs).expect("box without provider");
    // config.md, 4.3, with its workers' address on loopback (the shape's bound).
    let engine = parse(
        br#"listen = "127.0.0.1:7100"
public = "https://temper.example.com"
workers = "127.0.0.1:7200"
owner = "sam@example.com"
[sign_in.google]
client = "1234.apps.googleusercontent.com"
secret = "secrets/google.secret"
[[sign_in.members]]
address = "sam@example.com"
login = "sam"
"#,
        Shape::EngineAlone,
        &facts(),
    )
    .expect("engine file");
    assert_eq!(document(&engine)["engine"]["owners"], json!([{"provider":3,"subject":"sam@example.com"}]));
    assert_eq!(document(&engine)["engine"]["hosts"]["listener"], json!([{"socket":"127.0.0.1:7200"}]));
    assert_eq!(
        document(&engine)["application"]["sign_in"]["google"],
        json!({"provider":3,"client":"1234.apps.googleusercontent.com","secret":"secrets/google.secret","endpoints":null})
    );
    assert!(document(&engine)["engine"]["clients"].get("provider").is_none());
    assert_eq!(document(&engine)["workers"], json!([]));
    temper_engine_service::startup::named(&engine.document, &dirs).expect("engine decoders and cross checks");
    let worker =
        parse(b"engine = \"127.0.0.1:7200\"\nname = \"w1\"\n", Shape::WorkerAlone, &facts()).expect("worker file");
    assert_eq!(
        document(&worker),
        json!({"workers":[{"engine":{"socket":"127.0.0.1:7200"},"name":"w1","secret":"secrets/worker.secret","slots":1}],"application":{}})
    );
    temper_worker_service::startup::worker_part(&worker.document, &dirs, None, &[])
        .expect("worker decoders and checks");
    all_groups();
}

fn all_groups() {
    let dirs = jig_protocol_config::Directories::working();
    let full = parse(
        br#"owner = "sam@example.com"
[sign_in.google]
client = "google"
secret = "secrets/google.secret"
[[sign_in.members]]
address = "sam@example.com"
login = "sam"
[[sign_in.forges]]
forge = 1
client = "forgejo"
[openid]
[[openid.clients]]
id = "forgejo"
redirect = "https://git.example.com/callback"
[push]
key = "push.key"
subject = "mailto:operator@example.com"
[programs]
git = "/tools/git"
claude = "/tools/claude"
forgejo = "/tools/forgejo"
runner = "/tools/runner"
[environment]
Z_FIRST = "quotes\" and backslash\\"
A_SECOND = "line\nbreak"
[internal]
bundle = "/bundle"
[test.github]
api = "api.example.com:443"
transport = { server_name = "api.example.com", trust = "certs/root.pem" }
remote = "https://github.example.com"
web = "https://github.example.com"
git = "tools/git"
[test.google]
issuer = "https://issuer.example.com"
authorization = "https://issuer.example.com/auth"
token = "https://issuer.example.com/token"
keys = "https://issuer.example.com/keys"
transport = { server_name = "issuer.example.com" }
[test.push]
services = ["http://127.0.0.1:9000"]
"#,
        Shape::Standalone,
        &facts(),
    )
    .expect("all groups");
    let v = document(&full);
    assert_eq!(
        v["engine"]["clients"]["provider"],
        json!({"key":"provider.key","addresses":[3],"clients":[{"id":"forgejo","secret":null,"redirect_uri":"https://git.example.com/callback"}]})
    );
    assert_eq!(v["application"]["sign_in"]["forges"], json!([{"forge":1,"client":"forgejo"}]));
    assert_eq!(
        v["application"]["sign_in"]["google"]["endpoints"]["transport"],
        json!({"tls":{"server_name":"issuer.example.com","trust":null}})
    );
    assert_eq!(v["application"]["test"]["github"]["api"], json!({"host":"api.example.com","port":443}));
    assert_eq!(
        v["application"]["test"]["github"]["transport"],
        json!({"tls":{"server_name":"api.example.com","trust":"certs/root.pem"}})
    );
    assert_eq!(v["application"]["push"]["services"], json!(["http://127.0.0.1:9000"]));
    assert_eq!(
        v["application"]["environment"],
        json!([{"name":"Z_FIRST","value":"quotes\" and backslash\\"},{"name":"A_SECOND","value":"line\nbreak"}])
    );
    assert_eq!(v["application"]["clients"]["bundle"], "/bundle");
    assert!(v["application"].get("programs").is_none());
    assert_eq!(full.file.programs.runner, Some("/tools/runner".into()));
    temper_service::startup::named(&full.document, &dirs).expect("all groups pass today's decoders");
}

#[test]
fn refusals_collect_the_keys_and_name_their_lines() {
    let cases = [
        (
            Shape::Standalone,
            "listn = \"127.0.0.1:7100\"\n",
            "temper.toml:1: listn: unknown; temper-box help config lists the keys",
        ),
        (
            Shape::Standalone,
            "engine = \"127.0.0.1:7200\"\n",
            "temper.toml:1: engine: a key of temper-worker's file, not temper-box's",
        ),
        (
            Shape::WorkerAlone,
            "engine = \"127.0.0.1:7200\"\nname = \"w1\"\nslots = 4\n",
            "temper.toml:3: slots: moved: a worker's places are its setting's; temper ctl worker places NAME N",
        ),
        (
            Shape::Standalone,
            "listen = \"127.0.0.1:65535\"\n",
            "temper.toml: webhooks (its default): listen uses port 65535; give webhooks explicitly",
        ),
        (
            Shape::Standalone,
            "listen = 7100\n",
            "temper.toml:1: listen: a socket address on loopback, such as \"127.0.0.1:7100\"",
        ),
        (
            Shape::WorkerAlone,
            "engine = \"127.0.0.1:7200\"\nname = \"012345678901234567890123456789012\"\n",
            "temper.toml:2: name: 1 to 32 bytes",
        ),
        (Shape::EngineAlone, "", "temper.toml: workers: required in temper-engine's file"),
        (
            Shape::EngineAlone,
            "workers = \"10.0.0.1:7200\"\n",
            "temper.toml:1: workers: a socket address on loopback, such as \"127.0.0.1:7100\"",
        ),
        (
            Shape::Standalone,
            "[[sign_in.members]]\naddress = \"sam@example.com\"\nlogin = \"sam\"\n",
            "temper.toml:1: sign_in.members: requires [sign_in.google]",
        ),
    ];
    for (shape, text, words) in cases {
        assert_eq!(parse(text.as_bytes(), shape, &facts()).expect_err(text).to_string(), words, "{text}");
    }
    let all = parse(b"listn = 1\nlisten = 7100\nslots = 4\n", Shape::Standalone, &facts()).expect_err("all problems");
    assert_eq!(all.problems.len(), 3);
    assert!(all.problems.iter().any(|s| s.starts_with("temper.toml:3: slots:")));
    for (text, line) in
        [("listen = \"127.0.0.1:7100\"\nlisten = \"127.0.0.1:7101\"\n", 2), ("# header\nlisten = [\n", 2)]
    {
        let why = parse(text.as_bytes(), Shape::Standalone, &facts()).expect_err("TOML syntax");
        assert!(why.to_string().starts_with(&format!("temper.toml:{line}:")), "{why}");
    }
    let explicit = parse(b"listen = \"127.0.0.1:65535\"\nwebhooks = \"127.0.0.1:7101\"\n", Shape::Standalone, &facts())
        .expect("explicit webhooks with highest listen port");
    assert_eq!(document(&explicit)["application"]["clients"]["listener"]["socket"], "127.0.0.1:65535");
    for count in [1, 32] {
        let text = format!("engine = \"127.0.0.1:7200\"\nname = \"{}\"\n", "n".repeat(count));
        assert!(parse(text.as_bytes(), Shape::WorkerAlone, &facts()).is_ok());
    }
}

#[test]
fn keys_render_the_starts_words_at_explicit_default_and_array_keys() {
    let read = parse(b"# header\n# comment\n\npublic = \"http://example.com\"\n", Shape::Standalone, &facts())
        .expect("file gives the origin to the start");
    let found = temper_service::startup::named(&read.document, &jig_protocol_config::Directories::working())
        .expect_err("HTTP off loopback");
    let lines = temper_service::startup::lines(&found);
    let lines = std::str::from_utf8(&lines).expect("diagnostic");
    let words = lines.strip_prefix("application.clients.public: ").expect("origin's path").trim_end();
    assert_eq!(
        read.keys.render(
            &jig_protocol_config::Path::root().member(b"application").member(b"clients").member(b"public"),
            words
        ),
        format!("temper.toml:4: public: {words}")
    );
    assert_eq!(
        read.keys.render_member("application.webhooks.listener.socket", "invalid form"),
        "temper.toml: webhooks (its default): invalid form"
    );
    assert_eq!(read.keys.render_member("store", "bad record"), "store: bad record");
    assert_eq!(read.keys.render_member("application.push.key", "not read"), "application.push.key: not read");
    let members = parse(
        br#"[sign_in.google]
client = "google"
secret = "google.secret"
[[sign_in.members]]
address = "ada@localhost"
login = "ada"
[[sign_in.members]]
address = "sam@example.com"
login = "sam"
"#,
        Shape::Standalone,
        &facts(),
    )
    .expect("members");
    assert_eq!(
        members.keys.render_member("application.sign_in.members[1]", "past its bound"),
        "temper.toml:7: sign_in.members[1]: past its bound"
    );
    assert_eq!(
        members.keys.render_member("application.sign_in.members[1].login", "must be unique"),
        "temper.toml:9: sign_in.members[1].login: must be unique"
    );
}

#[test]
fn read_refuses_the_former_file_before_reading_and_uses_machine_facts() {
    use skein_shell::Unread;
    use std::io::ErrorKind;

    let absent = || Err(ErrorKind::NotFound.into());
    let fixed = facts();
    let read = |former: bool, text: Option<&[u8]>, shape| {
        machine::read_with(
            shape,
            || if former { Ok(()) } else { absent() },
            |max| {
                assert!(!former, "the former file is checked before reading");
                let text = text.ok_or(Unread::Absent)?;
                if text.len() > usize::try_from(max).expect("read bound fits usize") {
                    Err(Unread::TooLarge { max })
                } else {
                    Ok(text.into())
                }
            },
            || {
                assert!(!former && text.is_some(), "facts follow the configuration read");
                Ok(fixed.clone())
            },
        )
    };
    for shape in [Shape::Standalone, Shape::EngineAlone, Shape::WorkerAlone] {
        let words = if shape == Shape::Standalone {
            "temper-box init writes one".into()
        } else {
            format!("write it from {} help config", binary(shape))
        };
        assert_eq!(
            read(false, None, shape).expect_err("absent file").to_string(),
            format!("temper.toml: absent; {words}")
        );
    }
    for text in [None, Some(b"listen = 7100\n".as_slice())] {
        for shape in [Shape::Standalone, Shape::EngineAlone, Shape::WorkerAlone] {
            assert_eq!(
                read(true, text, shape).expect_err("former first").to_string(),
                format!(
                    "temper.json: moved: the configuration is temper.toml; write it from {} help config, then remove temper.json",
                    binary(shape)
                )
            );
        }
    }
    let large = vec![b' '; FILE_BYTES + 1];
    assert_eq!(
        read(false, Some(&large), Shape::Standalone).expect_err("large").to_string(),
        "temper.toml: past 1048576 bytes"
    );
    let text = b"listen = \"127.0.0.1:7100\"\n";
    let actual = read(false, Some(text), Shape::Standalone).expect("good file read");
    assert!(fixed.provider_key);
    let expected = parse(text, Shape::Standalone, &fixed).expect("same parse");
    assert_eq!(actual.file, expected.file);
    assert_eq!(actual.document, expected.document);
    assert_ne!(fixed.login, "");
    assert!(!fixed.hostname.ends_with('\n'));
}

#[test]
fn render_is_the_boxs_one_line_with_the_rows_comment() {
    let file = File { listen: Some("127.0.0.1:7100".parse().expect("socket")), ..File::default() };
    assert_eq!(render(&file, Shape::Standalone).as_ref(), b"# temper's configuration. `temper-box help config` says what each line does.\n# Everything else is changed by asking temper.\n\nlisten = \"127.0.0.1:7100\"   # where the client connects\n");
}
