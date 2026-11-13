//! Pure document contracts over current init output. No peer or filesystem is involved.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "ordinary tests edit bounded public JSON fixtures with serde"
)]
#![expect(clippy::default_numeric_fallback, reason = "JSON fixture integers have no Rust arithmetic unit")]
use super::{Documents, Paths, Peers, SEED, rewrite, steps};
use crate::Refusal;
use alloc::{boxed::Box, vec, vec::Vec};
use core::num::NonZeroU16;
use serde_json::{Value, json};

fn fixture() -> (Value, Value, Paths, Peers) {
    let port = NonZeroU16::new;
    let answer = temper_init::files(
        &temper_init::Facts {
            sign_in: false,
            binary: b"/fixture/temper",
            git: Some(b"/usr/bin/git"),
            claude: None,
            login: Some(b"person"),
            ports: temper_init::Ports {
                clients: port(8367).expect("port"),
                owned: port(8368).expect("port"),
                forge: port(8369).expect("port"),
                llm: port(8370).expect("port"),
                webhooks: port(8371).expect("port"),
            },
        },
        &temper_init::Directories { config: b"/source/config", data: b"/source/data" },
        &[7; 32],
        &[9; 32],
    )
    .expect("init fixture");
    let mut temper = Value::Null;
    let mut agent = Value::Null;
    for file in answer.files {
        if let temper_init::Content::Bytes { bytes, .. } = file.content {
            match file.name {
                b"temper.json" => temper = serde_json::from_slice(&bytes).expect("temper JSON"),
                b"agent.json" => agent = serde_json::from_slice(&bytes).expect("agent JSON"),
                _ => {}
            }
        }
    }
    let paths = Paths {
        config: b"/effective/config".as_slice().into(),
        data: b"/effective/data".as_slice().into(),
        cache: b"/cache".as_slice().into(),
        bundle: b"/effective/config/bundle-loopback".as_slice().into(),
        claude: b"/fixture/fake-claude".as_slice().into(),
    };
    let peers = Peers {
        github: "127.0.0.1:9101".parse().expect("peer"),
        oauth: "127.0.0.1:9102".parse().expect("peer"),
        llm: "127.0.0.1:9103".parse().expect("peer"),
        forgejo: "127.0.0.1:9104".parse().expect("peer"),
    };
    (temper, agent, paths, peers)
}
fn write(t: &Value, a: &Value, p: &Paths, peers: &Peers) -> Result<Documents, Refusal> {
    rewrite(&serde_json::to_vec(t).expect("temper bytes"), &serde_json::to_vec(a).expect("agent bytes"), p, peers)
}
fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("effective JSON")
}

#[test]
fn welcome_steps_name_only_the_four_held_peers_and_share_the_seed() {
    let (_, _, _, peers) = fixture();
    assert_eq!(SEED, 7);
    for config in [b"/effective/config".as_slice(), b"/effective/config/"] {
        let given = steps(config, b"/effective/data/", &peers, None, false);
        assert_eq!(given.len(), 4);
        for (step, name, command, document, listen) in [
            (
                &given[0],
                b"fake github".as_slice(),
                b"forge".as_slice(),
                b"/effective/config/github.json".as_slice(),
                peers.github,
            ),
            (
                &given[2],
                b"fake model".as_slice(),
                b"llm".as_slice(),
                b"/effective/config/llm.json".as_slice(),
                peers.llm,
            ),
            (
                &given[1],
                b"fake issuer".as_slice(),
                b"oauth".as_slice(),
                b"/effective/config/oauth.json".as_slice(),
                peers.oauth,
            ),
            (
                &given[3],
                b"fake forgejo".as_slice(),
                b"forge".as_slice(),
                b"/effective/config/forgejo.json".as_slice(),
                peers.forgejo,
            ),
        ] {
            assert_eq!(step.name(), name);
            assert_eq!(step.kind(), temper_forge_setup::Kind::Long);
            let mut expected = vec![b"fake".as_slice(), command, b"--document", document, b"--seed", b"7"];
            if name == b"fake github" {
                expected.extend([b"--state".as_slice(), b"/effective/data/github-forge.state"]);
            } else if name == b"fake forgejo" {
                expected.extend([b"--state".as_slice(), b"/effective/data/forgejo-forge.state"]);
            }
            assert_eq!(step.arguments(), expected.into_iter().map(Box::<[u8]>::from).collect::<Vec<_>>());
            assert_eq!(step.listeners(), &[listen]);
            assert_eq!(step.ready(), None);
        }
    }
}

#[test]
fn init_targets_change_together_and_preserve_registration_terms() {
    let (mut temper, agent, paths, peers) = fixture();
    temper["engine"]["deployment"]["seed"] = 99.into();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite init");
    let effective = json(&docs.temper);
    assert!(effective["application"].get("charter").is_none());
    assert!(effective["application"].get("landing").is_none());
    let effective_agent = json(&docs.agent);
    let forge = &effective["application"]["machine"]["github"];
    assert_eq!(forge["api"], json!({"socket":"127.0.0.1:9101"}));
    assert_eq!(forge["remote"], "http://127.0.0.1:9101");
    assert_eq!(forge["web"], "http://127.0.0.1:9101");
    assert_eq!(forge["transport"], "plaintext");
    let document = crate::read::document(&docs.temper).expect("effective document");
    let app = super::member(&document, 0, b"application", &jig_protocol_config::Path::root()).expect("application");
    let at = super::member(&document, app, b"machine", &jig_protocol_config::Path::root()).expect("machine");
    let mut problems = temper_forge_setup::problem::Problems::empty();
    let machine = temper_forge_setup::machine::read(
        &document,
        at,
        &temper_forge_setup::limits::small(),
        &jig_protocol_config::Directories::working(),
        &mut problems,
    )
    .expect("ordinary machine format");
    assert!(problems.is_empty());
    let github = machine.github.expect("fake GitHub");
    assert!(!github.tls);
    assert_eq!(github.git.as_deref(), Some(b"/effective/data/demo/bin/git".as_slice()));
    let peer: Value = serde_json::from_slice(&docs.forgejo).expect("peer document");
    assert_eq!(peer["git_program"], "/effective/data/demo/bin/git");

    let sign_in = &effective["engine"]["llm"]["providers"][0]["sources"]["sign_in"];
    for (name, expected) in [
        ("issuer", "http://127.0.0.1:9102"),
        ("authorization", "http://127.0.0.1:9102/api/accounts/authorize"),
        ("token", "http://127.0.0.1:9102/api/accounts/oauth/token"),
        ("revocation", "http://127.0.0.1:9102/api/accounts/oauth/revoke"),
        ("keys", "http://127.0.0.1:9102/.well-known/jwks.json"),
    ] {
        assert_eq!(sign_in[name], expected, "{name}");
    }
    for name in ["client", "name_hint", "scopes", "must_grant", "resource", "redirect_port", "redirect_path"] {
        assert_eq!(sign_in[name], temper["engine"]["llm"]["providers"][0]["sources"]["sign_in"][name], "{name}");
    }
    assert_eq!(sign_in["transport"], "plaintext");
    assert!(sign_in.get("server_name").is_none());
    assert_eq!(effective["engine"]["llm"]["endpoints"][0]["address"], json!({"socket":"127.0.0.1:9103"}));
    assert_eq!(effective_agent["endpoints"][0]["address"], "127.0.0.1:9103");
    for endpoint in [&effective["engine"]["llm"]["endpoints"][0], &effective_agent["endpoints"][0]] {
        assert_eq!(endpoint["transport"], "plaintext");
        assert!(endpoint.get("server_name").is_none());
    }
    assert_eq!(effective["engine"]["deployment"]["seed"], 99);
    assert_eq!(effective["application"]["clients"]["bundle"], "/effective/config/bundle-loopback");
    assert_eq!(effective["workers"][0]["agent"]["configuration"], "/effective/config/agent.json");
    assert_eq!(
        effective["workers"][0]["agent"]["arguments"],
        json!(["internal", "agent", "/effective/config/agent.json"])
    );
    assert_eq!(effective_agent["endpoints"][2], agent["endpoints"][3]);
    assert_eq!(docs, write(&temper, &agent, &paths, &peers).expect("deterministic rewrite"));
}

#[test]
fn generated_peers_use_existing_formats_and_the_first_added_key() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite init");
    let forge = crate::forge::Document::decode(&docs.github).expect("existing forge format");
    let llm = crate::llm::Document::decode(&docs.llm).expect("existing model format");
    assert_eq!(json(&docs.github)["github"]["app"]["setup_url"], "http://127.0.0.1:8367/forges/1/github/setup");
    assert_eq!(forge.listen(), peers.github);
    assert_eq!(llm.listen(), peers.llm);
    assert_eq!(llm.key(), b"/effective/data/llm.key");
    assert_eq!(llm.added_keys(), &[Box::<[u8]>::from(b"/effective/data/signed-in.key".as_slice())]);
    assert_eq!(json(&docs.github)["github"]["accounts"].as_array().expect("accounts").len(), 1);
    assert_eq!(json(&docs.github)["repositories"].as_array().expect("repositories").len(), 1);
    assert_eq!(json(&docs.llm)["otherwise"], json!(["Hello from the local fake model."]));
    assert_eq!(
        json(&docs.oauth),
        json!({"listen":{"socket":"127.0.0.1:9102"},"redirect_port":1455,"redirect_path":"auth/callback"})
    );
}

/// The fake box must write and revise the same document as the demo model
/// (temper/docs/design/client/web/journeys.md, 9).
#[test]
fn the_fake_box_keeps_the_demos_two_document_turns() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite init");
    crate::llm::Document::decode(&docs.llm).expect("document scripts fit the model format");
    let model = json(&docs.llm);
    let scripts = model["cued"].as_array().expect("fake box cues");
    let demo = temper["application"]["demo"]["llm"]["cued"].as_array().expect("demo cues");
    for cue in ["Write this up as a draft beside us.", "Add what happens to a running goal."] {
        let mut expected = demo.iter().find(|script| script["cue"] == cue).expect("demo document cue").clone();
        // The signed-in box tries its model as T1 before its first chat, T2.
        expected["turns"][0]["lines"][0]["arguments"]["scope"]["goal"] = json!(2);
        if cue == "Add what happens to a running goal." {
            // The numbers before the first chat's document are taken, so it writes note 3.
            expected["turns"][0]["lines"][0]["arguments"]["name"] = json!(3);
        }
        let found = scripts.iter().find(|script| script["cue"] == cue).expect("fake box document cue");
        assert_eq!(*found, expected, "write and revision calls agree: {cue}");
    }
}

/// The TUI sends the chosen words, not the item's number or reason.
#[test]
fn the_fake_box_writes_and_settles_the_farewell_draft() {
    const OPENING: &str = "Plan a farewell line for hello.\n\nWrite this up as a draft beside us.";
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite init");
    let model = crate::llm::Document::decode(&docs.llm).expect("design scripts fit the model format");
    // An open chat cues from its request; a restored chat may retain its brief
    // in system text. The browser's own request still cues the demo.
    for (system, request, title) in [
        ("", OPENING, "Farewell line"),
        (OPENING, OPENING, "Farewell line"),
        ("", "Write this up as a draft beside us.", "Away mode"),
    ] {
        let written = if title == "Away mode" { "Note 3 revision 1 written" } else { "Note 2 revision 1 written" };
        let (first, question) = document_reply(&model, system, request, written);
        let body = first["body"].as_str().expect("Markdown draft");
        assert_eq!(body.lines().next(), Some(alloc::format!("# {title}").as_str()));
        assert_eq!(first["scope"], json!({"kind":"goal", "project":1, "goal":2}));
        assert!(first.get("name").is_none(), "first write creates the note");
        if title == "Away mode" {
            continue;
        }
        assert!(body.lines().filter(|line| line.starts_with("## ")).count() >= 3);
        assert!(body.contains("README.md"));
        // temper-fake has no TUI dependency: check the parser's trailing-list
        // contract here, including the two nonempty words/reason pairs.
        let question = core::str::from_utf8(&question).expect("reply UTF-8");
        let (_, choices) = question.trim_end().rsplit_once("\n\n").expect("trailing list");
        assert_eq!(
            choices.lines().collect::<Vec<_>>(),
            [
                "1. At the end of README.md — it closes the page",
                "2. In its own file — the greeting stays alone",
                "3. Tell me something else",
            ]
        );
        let (revision, ready) = document_reply(&model, system, "At the end of README.md", "Note 2 revision 2 written");
        assert_eq!(revision["scope"], first["scope"]);
        assert_eq!(revision["description"], first["description"]);
        assert_eq!(revision["name"], 2);
        assert_eq!(revision["recalled"], 1);
        let start = usize::try_from(revision["first"].as_u64().expect("first line")).expect("line fits") - 1;
        let removed = usize::try_from(revision["remove"].as_u64().expect("removed lines")).expect("lines fit");
        let replacement = revision["body"].as_str().expect("replacement lines");
        let mut lines = body.lines().collect::<Vec<_>>();
        let before = lines[..start].iter().rposition(|line| line.starts_with("## ")).expect("inside a section");
        assert_eq!(lines[before], "## Open questions");
        assert!(lines[start..start + removed].iter().all(|line| !line.starts_with('#')));
        assert!(!replacement.lines().any(|line| line.starts_with('#')), "no section boundary added");
        lines.splice(start..start + removed, replacement.lines());
        assert!(lines.contains(&"Put the farewell line at the end of README.md."));
        assert!(!lines.contains(&"Where should the farewell line go?"));
        assert_eq!(
            ready.as_ref(),
            b"I settled the location at the end of README.md. The draft is ready; nothing blocks the work."
        );
    }
}

fn document_reply(model: &crate::llm::Document, system: &str, request: &str, written: &str) -> (Value, Box<[u8]>) {
    use skein_fake_llm_domain::api::{Finish, Message, Part, Role};
    let mut query = skein_fake_llm_domain::api::Query {
        credential: Some(0),
        model: b"gpt-6.1-sol".as_slice().into(),
        system: system.as_bytes().into(),
        messages: Box::new([Message {
            role: Role::User,
            parts: Box::new([Part::Text { text: request.as_bytes().into() }]),
        }]),
        tools: Box::new([]),
        choice: skein_fake_llm_domain::api::ToolChoice::None,
        max_tokens: 8192,
        caching: skein_fake_llm_domain::api::Caching::Unscoped,
    };
    let call = document_answer(model, query.clone());
    assert_eq!(call.finish, Finish::ToolCalls);
    let [Part::ToolCall { id, name, arguments }] = call.parts.as_ref() else { panic!("one note call: {call:?}") };
    assert_eq!(name.as_ref(), b"note");
    let note = json(arguments);
    let mut messages = query.messages.into_vec();
    messages.push(Message { role: Role::Assistant, parts: call.parts.clone() });
    messages.push(Message {
        role: Role::User,
        parts: Box::new([Part::ToolOutput { id: id.clone(), output: written.as_bytes().into(), is_error: false }]),
    });
    query.messages = messages.into_boxed_slice();
    let reply = document_answer(model, query);
    assert_eq!(reply.finish, Finish::Stop);
    let [Part::Text { text }] = reply.parts.as_ref() else { panic!("one text reply: {reply:?}") };
    (note, text.clone())
}

fn document_answer(
    model: &crate::llm::Document,
    query: skein_fake_llm_domain::api::Query,
) -> skein_fake_llm_domain::api::Answer {
    use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
    let mut domain = model.domain(SEED);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: crate::llm::LIMITS };
    let mut out = Queue::with_capacity(1);
    skein_fake_llm_domain::step(
        &mut domain,
        &env,
        skein_fake_llm_domain::Event::Call { reply_to: ReplyTo::new(Token::new(1)), call: Token::new(2), query },
        &mut out,
    );
    if out.is_empty() {
        skein_fake_llm_domain::fire(&mut domain, &env, &mut out);
    }
    match out.pop().expect("model answer") {
        skein_fake_llm_domain::Request::Reply { result, .. } => result.expect("successful answer"),
        skein_fake_llm_domain::Request::Piece { .. } | skein_fake_llm_domain::Request::Hold { .. } => {
            panic!("whole answer")
        }
    }
}

#[test]
fn forgejo_alone_seeds_the_two_copy_sources() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite init");
    let document = crate::forge::Document::decode(&docs.forgejo).expect("source document");
    let (forge, config) = document.seed(SEED);
    let expected = [
        (
            b"https://code.example.invalid/acme/widget".as_slice(),
            b"acme/widget".as_slice(),
            b"main".as_slice(),
            &[b"main".as_slice()][..],
        ),
        (
            b"https://code.example.invalid/acme/gadget".as_slice(),
            b"acme/gadget".as_slice(),
            b"trunk".as_slice(),
            &[b"topic".as_slice(), b"trunk".as_slice()][..],
        ),
    ];
    assert_eq!(document.sources().len(), expected.len());
    assert_eq!(config.sources.len(), expected.len());
    for (source, (address, full, default, branches)) in config.sources.iter().zip(expected) {
        assert_eq!(source.address.as_ref(), address);
        assert_eq!(source.repository.as_ref(), full);
        assert!(!source.private);
        assert_eq!(forge.domain.metadata(full).expect("source metadata").default_branch.as_ref(), default);
        assert_eq!(forge.domain.branches(full).iter().map(|(name, _)| name.as_ref()).collect::<Vec<_>>(), branches);
        for branch in branches {
            let head = forge.domain.branch(full, branch).expect("seeded branch");
            let object = forge.domain.object(head).expect("source commit");
            assert!(object.parent.is_none(), "one source commit");
            assert!(object.tree.contains_key(b".github/workflows/ci.yml".as_slice()));
            assert!(!object.tree.contains_key(temper_forge_setup::STARTER_PATH));
        }
    }
    assert_eq!(crate::forge::Document::decode(&docs.github).expect("GitHub document").sources(), []);
}

#[test]
fn target_selection_uses_ids_and_names_and_removes_stale_tls() {
    // Fill every TLS removal slot alongside all welcome git/program edits.
    const TLS: [&str; 4] = ["server_name", "trust", "trust_der", "identity_file"];
    let (mut temper, mut agent, paths, peers) = fixture();
    for pointer in ["/engine/llm/providers/0/sources/sign_in", "/engine/llm/endpoints/0"] {
        for member in TLS {
            temper.pointer_mut(pointer).expect("rewrite target")[member] = "/stale/root.der".into();
        }
    }
    for member in TLS {
        agent["endpoints"][0][member] = "/stale/root.der".into();
    }
    temper["engine"]["llm"]["providers"].as_array_mut().expect("providers").reverse();
    agent["endpoints"].as_array_mut().expect("endpoints").reverse();
    let docs = write(&temper, &agent, &paths, &peers).expect("structural rewrite");
    let effective = json(&docs.temper);
    let effective_agent = json(&docs.agent);
    assert_eq!(effective["engine"]["llm"]["providers"][0], temper["engine"]["llm"]["providers"][0]);
    assert!(effective["engine"]["llm"]["endpoints"][0].get("trust").is_none());
    assert!(effective_agent["endpoints"][2].get("trust_der").is_none());
    assert_eq!(effective_agent["endpoints"][2]["address"], "127.0.0.1:9103");
    for target in [
        &effective["engine"]["llm"]["providers"][2]["sources"]["sign_in"],
        &effective["engine"]["llm"]["endpoints"][0],
        &effective_agent["endpoints"][2],
    ] {
        for member in TLS {
            assert!(target.get(member).is_none(), "removed {member}: {target}");
        }
    }
}

#[test]
fn malformed_targets_have_named_refusals() {
    let (temper, agent, paths, peers) = fixture();
    for (pointer, replacement, words) in [
        ("/engine/llm/providers/0/sources/sign_in/issuer", json!(false), "wrong value kind"),
        ("/engine/llm/providers/0/sources/sign_in/client", json!("other"), "expected the shipped registration"),
        ("/engine/llm/providers/0/sources/sign_in/redirect_port", json!(0), "expected shipped callback 1455"),
        (
            "/workers/0/agent/arguments",
            json!(["other", "/source/config/agent.json"]),
            "expected agent and its configuration",
        ),
        (
            "/workers/0/agent/arguments/2",
            json!("/other/agent.json"),
            "configuration and argv must name the same agent document",
        ),
        ("/engine/llm/endpoints/0/address", json!(null), "wrong value kind"),
    ] {
        let mut input = temper.clone();
        *input.pointer_mut(pointer).expect("fixture pointer") = replacement;
        let refusal = write(&input, &agent, &paths, &peers).expect_err("bad target refused");
        assert_eq!(refusal.words, words.as_bytes(), "{pointer}: {}", String::from_utf8_lossy(&refusal.line()));
        assert!(!refusal.path.segments.is_empty(), "{pointer}");
    }
}

#[test]
fn duplicate_targets_and_members_are_refused() {
    let (mut temper, agent, paths, peers) = fixture();
    let target = temper["engine"]["llm"]["providers"][0].clone();
    temper["engine"]["llm"]["providers"].as_array_mut().expect("providers").push(target);
    assert_eq!(write(&temper, &agent, &paths, &peers).expect_err("duplicate provider").words, b"target must be unique");
    let agent = serde_json::to_vec(&agent).expect("agent bytes");
    assert_eq!(
        rewrite(br#"{"engine":{},"engine":{}}"#, &agent, &paths, &peers).expect_err("duplicate member").words,
        b"duplicate"
    );
}

#[test]
fn input_byte_bound_is_checked_before_any_output() {
    let (temper, agent, paths, peers) = fixture();
    let mut source: Vec<u8> = serde_json::to_vec(&temper).expect("temper bytes");
    let bound = usize::try_from(crate::DOCUMENT_BYTES).expect("document bound");
    source.resize(bound, b' ');
    let agent = serde_json::to_vec(&agent).expect("agent bytes");
    rewrite(&source, &agent, &paths, &peers).expect("document exactly at its byte bound");
    source.push(b' ');
    assert_eq!(rewrite(&source, &agent, &paths, &peers).expect_err("past document bound").words, b"past its bound");
}

#[test]
fn peer_and_effective_path_bounds_refuse_before_rewriting() {
    let (temper, agent, paths, peers) = fixture();
    for address in ["127.0.0.1:0", "127.0.0.1:1455", "192.0.2.1:9101"] {
        let mut input = peers;
        input.github = address.parse().expect("peer fixture");
        assert!(write(&temper, &agent, &paths, &input).is_err(), "{address}");
    }
    for value in [b"relative".as_slice(), b"/effective/../source", b"/bad\0path"] {
        let mut input = paths.clone();
        input.config = value.into();
        assert!(write(&temper, &agent, &input, &peers).is_err(), "{value:?}");
    }
    let mut input = paths;
    input.data = vec![b'/'; 4097].into();
    write(&temper, &agent, &input, &peers).expect_err("effective data path past its byte bound");
}

#[test]
fn adapter_and_machine_kinds_are_ordinary_and_no_forges_are_added() {
    for present in [false, true] {
        let (temper, mut agent, paths, peers) = fixture();
        if present {
            agent["programs"] = json!({"claude_code":{"path":"/source/claude","directory":"/source/programs"}});
        }
        let docs = write(&temper, &agent, &paths, &peers).expect("ordinary rewrite");
        let effective = json(&docs.temper);
        let effective_agent = json(&docs.agent);
        assert!(effective["application"].get("forges").is_none());
        assert!(effective["application"].get("setup").is_none());
        for kind in ["owned", "fake"] {
            assert_eq!(effective["application"]["machine"][kind], temper["application"]["machine"][kind]);
        }
        assert_eq!(
            effective_agent["programs"]["claude_code"],
            json!({"path":"/fixture/fake-claude","directory":"/effective/data/programs"})
        );
        assert!(effective["engine"]["llm"]["providers"][0]["sources"].get("programs").is_none());
        let document = crate::forge::Document::decode(&docs.forgejo).expect("existing Forgejo schema");
        assert_eq!(document.listen(), peers.forgejo);
        assert_eq!(document.admin_token(), b"/effective/data/forgejo-admin.token");
    }
}

#[test]
fn backend_rewrite_keeps_creating_list_capacity() {
    let standard = jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core.documents.lines;
    for (receiving, expected) in [(Some(2), 3), (Some(8), 8), (None, standard)] {
        let (mut temper, agent, paths, peers) = fixture();
        let overrides = temper["engine"]["limits"]["overrides"].as_object_mut().unwrap();
        match receiving {
            Some(receiving) => overrides.insert("documents.lines".into(), json!(receiving)),
            None => overrides.remove("documents.lines"),
        };
        let docs = write(&temper, &agent, &paths, &peers).expect("backend rewrite");
        let effective = json(&docs.temper);
        assert_eq!(effective["engine"]["limits"]["overrides"]["documents.lines"], json!(expected));
        assert_eq!(effective["engine"]["limits"]["overrides"]["accounts.credentials"], json!(4));
    }
}

fn assert_program_provider(llm: &Value, temper: &Value) {
    let provider = llm["providers"]
        .as_array()
        .expect("providers")
        .iter()
        .find(|provider| provider["number"] == 2)
        .expect("program's provider");
    assert_eq!(
        provider,
        &json!({
            "name":"anthropic", "number":2, "signed_in_plan":false,
            "manage_usage":"https://claude.ai/settings/usage",
            "endpoint":"fake-claude", "keep":["claude-*"], "drop":["*-20??????"], "sources":{"programs":["claude_code"]}
        })
    );
    let openai = &llm["providers"][0];
    assert_eq!(openai["endpoint"], temper["engine"]["llm"]["providers"][0]["endpoint"]);
    assert!(openai["sources"].get("programs").is_none());
    assert!(provider.get("models").is_none());
    assert!(provider["sources"]["programs"].as_array().expect("program adapters").contains(&json!("claude_code")));
}

#[test]
fn program_offer_has_a_runnable_model_route_in_both_documents() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("program route rewrite");
    let admitted = admit(&docs.temper);
    assert_eq!(
        admitted.limits.core.documents.lines,
        jig_protocol_config::profile(jig_protocol_config::Profile::Standard).core.documents.lines,
        "the rewrite preserves the profile's creating lists"
    );
    let effective = json(&docs.temper);
    let effective_agent = json(&docs.agent);
    let llm = &effective["engine"]["llm"];
    assert_program_provider(llm, &temper);
    assert!(llm.get("models").is_none());
    let stock = temper_engine_service::stock::defaults(&admitted.llm, admitted.limits.core.tasks.result_bytes);
    let model = stock.models.iter().find(|entry| entry.model.name.as_ref() == super::PROGRAM_MODEL.as_bytes()).unwrap();
    assert_eq!(model.model.endpoint, 2);
    let engine_endpoint = llm["endpoints"]
        .as_array()
        .expect("engine endpoints")
        .iter()
        .find(|endpoint| endpoint["name"] == "fake-claude")
        .expect("engine program endpoint");
    let agent_endpoint = effective_agent["endpoints"]
        .as_array()
        .expect("agent endpoints")
        .iter()
        .find(|endpoint| endpoint["name"] == "fake-claude")
        .expect("agent program endpoint");
    for endpoint in [engine_endpoint, agent_endpoint] {
        assert_eq!(endpoint["program"], "claude_code");
        assert_eq!(endpoint["provider"], "anthropic");
        assert_eq!(endpoint["dialect"], 2, "Anthropic’s endpoint dialect");
        assert_eq!(endpoint["number"], 2);
    }
    let agent_limits = &agent_endpoint["limits"];
    for field in ["window", "output"] {
        let actual = if field == "window" { model.model.window } else { model.model.output };
        assert_eq!(json!(actual), agent_limits[field], "program quantities agree: {field}");
    }
    let stock = temper_engine_service::stock::defaults(&admitted.llm, admitted.limits.core.tasks.result_bytes);
    for charter in &stock.charters {
        assert_eq!(charter.policy.model.name.as_ref(), super::PROGRAM_MODEL.as_bytes());
        let names: Vec<_> = stock
            .models
            .iter()
            .filter(|entry| entry.model.provider == 2)
            .map(|entry| entry.model.name.as_ref())
            .collect();
        assert_eq!(names, [b"claude-opus-5-5".as_slice(), b"claude-sonnet-5-5", b"claude-fable-5-1"]);
    }
}

#[test]
fn managed_admin_name_accepts_the_actual_fake_seeded_admin_and_token() {
    use skein_forge::forgejo::document::{Document, User};
    use skein_forge_setup::{
        item::Item,
        reachable::{self, Reachable},
        reconcile::{Event, Need, Reply},
    };
    use skein_lib::{Queue, Secret};
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite");
    let limits = temper_forge_setup::limits::small();
    let entry = temper_forge_setup::setup::Entry {
        forge: 4,
        directory: Box::new([]),
        bot: temper_forge_setup::setup::Account {
            login: b"temper".as_slice().into(),
            email: b"temper@localhost".as_slice().into(),
        },
        mode: temper_forge_setup::setup::Mode::Managed { admin_token: None },
        asked: false,
    };
    let given = temper_forge_setup::given::values(&entry, None, &limits);
    assert_eq!(given.admin_name.as_ref(), b"admin");
    let token = crate::credential::credential(SEED, crate::credential::ADMIN_TOKEN);
    let forge = crate::forge::Document::decode(&docs.forgejo).expect("forge document");
    let (mut peer, _) = crate::forge::peer(&forge, &token, SEED).expect("seeded peer");
    let store = peer.setup_mut();
    let admin = store.find(b"temper-admin").expect("actual seeded admin");
    assert!(store.token_user(&token) == Some(admin.id), "seed token authenticates the seeded admin");
    let user =
        User { id: admin.id, login: admin.login.clone(), is_admin: admin.admin, full_name: admin.full_name.clone() };
    let mut machine = Reachable::new(&given, None, 1).expect("managed Given admitted");
    let mut out = Queue::uncapped();
    reachable::step(&mut machine, Event::Begin, &mut out);
    let Some(Need::Read(Item::AdminToken)) = out.pop() else { panic!("read the admin token") };
    reachable::step(&mut machine, Event::Read(Some(Secret::new(token.as_slice().into()))), &mut out);
    let Some(Need::Send { .. }) = out.pop() else { panic!("send the next ordinary request") };
    reachable::step(&mut machine, Event::Reply(Reply::Document(Box::new(Document::User(user)))), &mut out);
    let Some(Need::Send { .. }) = out.pop() else {
        panic!("ordinary reachability must accept the seeded admin despite its different login")
    };
    reachable::step(
        &mut machine,
        Event::Reply(Reply::Document(Box::new(Document::ApiSettings(skein_forge::forgejo::document::ApiSettings {
            max_response_items: 32,
            default_paging_num: 30,
        })))),
        &mut out,
    );
    let Some(Need::Send { .. }) = out.pop() else { panic!("send the next ordinary request") };
    reachable::step(&mut machine, Event::Reply(Reply::Rows(Box::new([]))), &mut out);
    let Some(Need::Done(skein_forge_setup::reconcile::Outcome::Reachable { more: false, .. })) = out.pop() else {
        panic!("the ordinary picker finishes")
    };
    peer.shutdown();
}

#[test]
fn written_anthropic_entries_are_replaced_once() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite written Anthropic route");
    let effective = json(&docs.temper);
    let effective_agent = json(&docs.agent);
    let llm = &effective["engine"]["llm"];
    assert_eq!(llm["providers"].as_array().unwrap().iter().filter(|p| p["name"] == "anthropic").count(), 1);
    assert_program_provider(llm, &temper);
    for endpoints in [&llm["endpoints"], &effective_agent["endpoints"]] {
        assert!(!endpoints.as_array().unwrap().iter().any(|e| e["name"] == "anthropic-api"));
        let selected: Vec<_> = endpoints.as_array().unwrap().iter().filter(|e| e["number"] == 2).collect();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0]["name"], "fake-claude");
    }
    assert!(llm.get("models").is_none());
    for document in [&effective, &effective_agent] {
        assert!(document["engine"]["llm"].get("models").is_none());
        for endpoint in document.get("endpoints").unwrap_or(&document["engine"]["llm"]["endpoints"]).as_array().unwrap()
        {
            assert!(endpoint.get("models").is_none());
        }
    }
}

/// A fresh catalogue read must preserve every stock choice (temper/docs/design/protocol/configuration/settings.md, 1).
#[test]
fn the_templates_list_patterns_keep_every_stock_model() {
    for template in [
        include_bytes!("../../../temper-init/template/temper.json").as_slice(),
        include_bytes!("../../../temper-init/template/bare.json").as_slice(),
    ] {
        let (mut value, _, _, _) = fixture();
        value["engine"]["llm"]["providers"] = json(template)["engine"]["llm"]["providers"].clone();
        let admitted = admit(&serde_json::to_vec(&value).unwrap());
        let stock = temper_engine_service::stock::defaults(&admitted.llm, admitted.limits.core.tasks.result_bytes);
        for provider in &admitted.llm.providers {
            if provider.number == 3 {
                assert!(provider.list.is_none(), "the scripted provider reads no list");
                continue;
            }
            let patterns = provider.list.as_ref().expect("the real providers read their lists");
            let models: Vec<_> = stock.models.iter().filter(|row| row.model.provider == provider.number).collect();
            assert!(!models.is_empty(), "each provider has stock models");
            for row in models {
                assert!(
                    jig_domain_core::settings::catalogue::kept(&row.model.name, patterns),
                    "the template drops stock model {:?}",
                    row.model.name
                );
            }
        }
    }
}

fn admit(bytes: &[u8]) -> jig_protocol_config::Engine {
    let document = crate::read::document(bytes).unwrap();
    let engine = super::member(&document, 0, b"engine", &jig_protocol_config::Path::root()).unwrap();
    jig_protocol_config::engine(
        &document,
        engine,
        &jig_protocol_config::Directories::working(),
        jig_protocol_config::Defaults::Passed,
    )
    .unwrap()
}

#[test]
fn the_fake_list_preserves_openais_stock() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).unwrap();
    let fake = crate::llm::Document::decode(&docs.llm).unwrap();
    let admitted = admit(&docs.temper);
    let stock = temper_engine_service::stock::defaults(&admitted.llm, admitted.limits.core.tasks.result_bytes);
    let body = crate::wire::llm_config(fake.models()).list.unwrap();
    let listed = skein_llm::list::parse(skein_llm::Provider::OpenAi, &body, &skein_llm::list::Limits::DEFAULT).unwrap();
    for row in stock.models.iter().filter(|row| row.model.provider == 1) {
        assert!(
            listed.iter().any(|listed| listed.name == row.model.name),
            "the fake omits stock model {:?}",
            row.model.name
        );
    }
}

#[test]
fn box_forge_documents_declare_head_owned_ci() {
    let (temper, agent, paths, peers) = fixture();
    let docs = write(&temper, &agent, &paths, &peers).expect("rewrite init");
    for bytes in [&docs.forgejo, &docs.github] {
        let doc = json(bytes);
        assert_eq!(doc["owner"], "fake-person");
        assert_eq!(doc["repositories"][0]["name"], "widget");
        assert_eq!(doc["repositories"][0]["ci"], json!({"declared":".fake-forge/ci"}));
        assert_eq!(doc["made_ci"], json!({"declared":".fake-forge/ci"}));
        crate::forge::Document::decode(bytes).expect("generated forge document");
    }
}
