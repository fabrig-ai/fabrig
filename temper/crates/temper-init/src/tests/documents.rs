use super::fixtures::{self, DIRECTORIES, cases, content, parse, text};
use crate::{files, template};
use serde_json::{Value, json};
use skein_lib::bytes;

fn set(value: &mut Value, path: &str, replacement: Value) {
    *value.pointer_mut(path).expect("the member exists in the owned example") = replacement;
}

#[test]
fn written_workers_outlast_an_engine_restart_below_the_fleet_grace() {
    let facts = fixtures::facts();
    let boxed = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).unwrap();
    let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).unwrap();
    for files in [boxed.files.as_ref(), bare.as_ref()] {
        let document = parse(file_bytes(files, b"temper.json"));
        let limits = &document["engine"]["limits"];
        let profile = jig_protocol_config::profile(match limits["profile"].as_str().unwrap() {
            "small" => jig_protocol_config::Profile::Small,
            "standard" => jig_protocol_config::Profile::Standard,
            other => panic!("unknown profile {other}"),
        });
        let grace = profile.core.fleet.grace.as_millis();
        let engine = &limits["overrides"];
        let turn = engine["hub.turn_bytes"].as_u64().unwrap();
        let resume = (turn + 4).max(4_194_304);
        let window = resume + (u64::try_from(size_of::<Box<[u8]>>()).unwrap() - 4) * (resume / 4);
        assert_eq!(engine["core.resume_bytes"], resume);
        assert_eq!(engine["hub.transcript_bytes"], window);
        assert_eq!(document["application"]["limits"]["overrides"]["root.journal.transcript_bytes"], resume);
        for worker in document["workers"].as_array().unwrap() {
            assert_eq!(
                worker["graces"],
                json!({"grace_ms":300_000_u64,"agent_stop_ms":5_000_u64,"delivery_ms":1_000_u64,"save_ms":1_000_u64})
            );
            assert_eq!(worker["limits"]["overrides"]["hub.transcript_bytes"], window);
            let bytes = serde_json::to_vec(worker).unwrap();
            let document = jig_protocol_config::document(
                &bytes,
                &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
            )
            .unwrap();
            let worker =
                jig_protocol_config::worker(&document, 0, &jig_protocol_config::Directories::working()).unwrap();
            let stop =
                jig_protocol_config::stop_bound(&worker.graces, worker.limits.link.ping, worker.limits.link.silence)
                    .unwrap()
                    .as_millis();
            assert_eq!(stop, 326_000);
            assert!(stop < grace);
        }
    }
}

#[test]
fn the_box_has_room_for_reference_creating_executors() {
    let answered = files(&fixtures::facts(), &DIRECTORIES, &[0; 32], &[0xab; 32]).unwrap();
    let written = parse(content(&answered, b"temper.json"));
    let reference = parse(include_bytes!("../../../../docs/examples/reference/settings.json"));
    let mut charters = vec![1, 2, 4, 5, 9, 10, 11, 12, 13, 14, 15, 16];
    for item in reference["items"].as_array().unwrap() {
        if item["setting"]["kind"] == "charter" {
            charters.push(item["setting"]["number"].as_u64().unwrap());
        }
    }
    charters.sort_unstable();
    charters.dedup();
    assert_eq!(charters, [1, 2, 4, 5, 9, 10, 11, 12, 13, 14, 15, 16]);
    let bytes = serde_json::to_vec(&written["engine"]["limits"]).unwrap();
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
    )
    .unwrap();
    let parsed = jig_protocol_config::overrides::parse(&document, 0, &jig_protocol_config::Path::root());
    assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
    let limits = parsed.limits.core;
    jig_protocol_config::check_effective_limits(&limits).expect("deployment receiving limits fit the codecs");
    assert_eq!(limits.authority.executors, 21);
    assert_eq!(limits.tasks.executor_kinds, 21);
    assert_eq!(limits.documents.lines, 16);
    assert!(usize::try_from(limits.documents.lines).unwrap() >= charters.len());
    assert!(limits.documents.lines >= limits.tasks.charters);
    // The closer adds procedure p6 beside p2, p3 and p4 on each configured connector.
    let procedures = 4 * written["engine"]["connectors"].as_array().unwrap().len();
    assert!(usize::try_from(limits.documents.lines).unwrap() >= procedures);
    // The reference omits the trial, drafter and general standing role from delegation.
    let mut delegated = 0;
    for number in &charters {
        if ![10, 12, 13].contains(number) {
            delegated += 1;
        }
    }
    let executors = delegated + procedures + 2 + 3; // Merge/close jobs and owner/maintainer/member roles.
    assert_eq!(executors, 18);
    assert!(executors <= usize::try_from(limits.authority.executors).unwrap());
    assert!(executors <= usize::try_from(limits.tasks.executor_kinds).unwrap());
}

#[test]
fn the_documents_enable_the_printed_link_and_the_approval() {
    for source in [fixtures::OWNED, template::TEMPER, template::BARE] {
        let value = parse(source);
        assert_eq!(value["engine"]["clients"]["link"], true, "engine.clients.link");
        assert_eq!(value["engine"]["clients"]["approval"], true, "engine.clients.approval");
    }
}

#[test]
fn the_box_runs_four_agents_and_keeps_eight_streams_and_four_calls_beside_them() {
    for source in [fixtures::OWNED, template::TEMPER, template::BARE] {
        let value = parse(source);
        let limits = &value["engine"]["limits"]["overrides"];
        assert_eq!(value["workers"][0]["slots"], 4_u32, "workers[0].slots");
        for name in ["checkout.workspaces", "git_component.operations"] {
            assert_eq!(value["application"]["limits"]["overrides"][name], 4_u32, "{name}");
        }
        for name in ["fleet.slots", "hub.slots"] {
            assert!(limits.get(name).is_none(), "engine overrides {name}");
        }
        assert!(value["workers"][0]["limits"]["overrides"].get("hub.slots").is_none(), "worker overrides hub.slots");
        for name in ["clients.streams", "clients_http.streams"] {
            assert_eq!(limits[name], 8_u32, "{name}");
        }
        for name in ["clients.exchanges", "clients_http.calls"] {
            assert_eq!(limits[name], 12_u32, "{name}");
        }
    }
}

#[test]
fn the_owned_example_offers_openai_keys_and_chatgpts_sign_in() {
    let owned = parse(fixtures::OWNED);
    assert_eq!(
        owned["engine"]["llm"]["providers"],
        json!([{
            "name": "openai", "number": 1_u32, "signed_in_plan": true,
            "manage_usage": "https://chatgpt.com/settings/usage",
            "keep": ["gpt-*"],
            "drop": ["*audio*", "*image*", "*realtime*", "*search*", "*transcribe*", "*tts*", "*-20??-??-??"],
            "endpoint": "openai", "sources": {"keys": true, "sign_in": {
                "issuer": "https://auth.openai.com", "client": "dynamic_agent_client", "name_hint": "temper",
                "authorization": "https://auth.openai.com/api/accounts/authorize",
                "token": "https://auth.openai.com/api/accounts/oauth/token",
                "revocation": "https://auth.openai.com/api/accounts/oauth/revoke",
                "keys": "https://auth.openai.com/.well-known/jwks.json",
                "transport": "tls", "server_name": "auth.openai.com",
                "scopes": ["openid", "profile", "email", "offline_access", "resource.invoke", "chatgpt.tokens.use.direct"],
                "must_grant": ["chatgpt.tokens.use.direct"], "resource": "https://api.openai.com/v1",
                "redirect_path": "auth/callback", "redirect_port": 1_455_u16
            }}
        }])
    );
    let endpoint = &owned["engine"]["llm"]["endpoints"][0];
    assert_eq!(endpoint["name"], "openai");
    assert_eq!(endpoint["provider"], "openai");
    assert_eq!(endpoint["dialect"], 3_u32);
    assert_eq!(endpoint["address"], json!({"host": "api.openai.com", "port": 443_u16}));
    assert_eq!(endpoint["transport"], "tls");
    assert_eq!(endpoint["server_name"], "api.openai.com");
    assert_eq!(endpoint["headers"], parse(b"[]"));
    assert!(owned["engine"]["llm"].get("models").is_none());
    let agent = parse(fixtures::AGENT);
    let endpoint = &agent["endpoints"][0];
    assert_eq!(endpoint["name"], "openai");
    assert_eq!(endpoint["provider"], "openai");
    assert_eq!(endpoint["dialect"], 3_u32);
    assert_eq!(endpoint["address"], "api.openai.com:443");
    assert_eq!(endpoint["transport"], "tls");
    assert_eq!(endpoint["server_name"], "api.openai.com");
    assert_eq!(endpoint["headers"], parse(b"[]"));
    assert_eq!(endpoint["limits"]["reasoning_item"], 65_536_u32);
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the whole default document against the owned example")]
fn the_default_keeps_the_owned_structure_with_stock_models() {
    let owned = parse(fixtures::OWNED);
    assert_eq!(owned["engine"]["memory_bytes"], 1_073_741_824_u64);
    for worker in owned["workers"].as_array().unwrap() {
        assert!(worker.get("memory_bytes").is_none());
    }
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
        let actual = parse(content(&answered, b"temper.json"));
        let members = actual["application"]["demo"].as_object().unwrap();
        assert_eq!(members.len(), 1, "{row}");
        assert!(members.contains_key("llm"), "{row}");
        for name in ["forges", "identities", "setup"] {
            assert!(actual["application"].get(name).is_none(), "{row}: {name}");
        }
        let ports = [
            "/application/clients/listener/socket",
            "/application/machine/fake/listen/socket",
            "/application/machine/owned/listen/socket",
            "/application/demo/llm/listen/socket",
            "/application/webhooks/listener/socket",
        ]
        .map(|path| actual.pointer(path).expect("written listener").as_str().expect("socket"));
        for (index, port) in ports.iter().enumerate() {
            assert!(!ports[..index].contains(port), "{row}: five distinct ports");
        }
        assert_eq!(
            actual["application"]["machine"]["owned"]["listen"]["socket"],
            format!("127.0.0.1:{}", facts.ports.owned),
            "{row}"
        );
        let mut expected = owned.clone();
        expected["workers"][0]["graces"]["grace_ms"] = json!(300_000_u64);
        expected["engine"]["acts"] =
            json!([{ "connector": 1_u16, "may": ["remove_project", "remove_account", "adopt"] }]);
        assert_ne!(actual["engine"]["owners"].as_array().unwrap().len(), 0);
        // The startup turn_bounds writer holds these to each document's own
        // agent; init adds a demo endpoint, so smith derives a different T.
        let template = parse(include_bytes!("../../template/temper.json"));
        for pointer in ["/engine/limits/overrides", "/workers/0/limits/overrides", "/application/limits/overrides"] {
            for (name, bytes) in template.pointer(pointer).expect("template overrides").as_object().expect("limit map")
            {
                if name.starts_with("hub.turn")
                    || name == "hub.transcript_bytes"
                    || name == "core.resume_bytes"
                    || name.starts_with("root.loads.")
                    || name == "root.journal.transcript_bytes"
                    || name == "io.output"
                {
                    expected.pointer_mut(pointer).expect("owned overrides")[name] = bytes.clone();
                }
            }
        }
        expected["application"]["limits"]["overrides"]["forge.client.repositories"] = json!(3_u32);
        expected["application"]["limits"]["overrides"]["forge.repositories"] = json!(3_u32);
        expected["application"]["limits"]["overrides"]["forge.output"] = json!(82_u32);
        expected["application"]["welcome"] = json!(true);
        expected["engine"]["owners"][0]["subject"] = json!("ada@localhost");
        expected["engine"]["clients"]["provider"] = json!({
            "key": "provider.key", "addresses": [1_u32],
            "clients": [{"id": "forgejo", "secret": null,
                "redirect_uri": format!("http://127.0.0.1:{}/user/oauth2/temper/callback", facts.ports.owned)}]
        });
        let openai = owned["engine"]["llm"]["providers"][0].clone();
        for (path, value) in [
            (
                "/engine/llm/providers",
                json!([
                    openai,
                    anthropic("providers"),
                    {"name": "demo", "number": 3_u64, "signed_in_plan": false,
                     "endpoint": "demo", "sources": {"keys": true}}
                ]),
            ),
            (
                "/engine/llm/endpoints",
                json!([
                    owned["engine"]["llm"]["endpoints"][0],
                    anthropic("endpoints"),
                    anthropic("direct"),
                    {"name": "demo", "number": 3_u64, "dialect": 3_u64, "provider": "demo",
                     "address": {"socket": format!("127.0.0.1:{}", facts.ports.llm)},
                     "transport": "plaintext", "headers": parse(b"[]"), "connect_ms": 10_000_u64, "handshake_ms": 10_000_u64}
                ]),
            ),
            (
                "/application/machine",
                json!({
                    "owned": {
                        "forgejo": "fetched", "runner": {
                            "binary": "fetched", "labels": ["temper-host:host"], "capacity": 1_u32,
                            "path": "/usr/local/bin:/usr/bin:/bin"},
                        "listen": {"socket": format!("127.0.0.1:{}", facts.ports.owned)},
                        "remote": format!("http://127.0.0.1:{}", facts.ports.owned),
                        "person": {"login": "ada", "email": "ada@localhost"}, "sign_in": "forgejo"
                    },
                    "fake": {
                        "listen": {"socket": format!("127.0.0.1:{}", facts.ports.forge)},
                        "remote": format!("http://127.0.0.1:{}", facts.ports.forge),
                        "git": "demo/bin/git"
                    },
                    "github": null
                }),
            ),
            ("/application/demo", demo(facts.ports.llm.get())),
            ("/application/webhooks", json!({"listener": {"socket": format!("127.0.0.1:{}", facts.ports.webhooks)}})),
            ("/application/clients/listener/socket", json!(format!("127.0.0.1:{}", facts.ports.clients))),
            ("/application/clients/bundle", Value::Null),
            (
                "/application/workers/0/git",
                match facts.git {
                    Some(git) => json!(text(git)),
                    None => Value::Null,
                },
            ),
            ("/engine/projects", parse(b"[]")),
            ("/engine/hosts/workers/0/digest", json!("worker.digest")),
            ("/engine/deployment/directory", json!("deployment")),
            ("/workers/0/secret", json!("worker.secret")),
            ("/workers/0/agent/program", json!(text(facts.binary))),
            ("/workers/0/agent/configuration", json!("/c/temper/agent.json")),
            ("/workers/0/agent/arguments/2", json!("/c/temper/agent.json")),
        ] {
            set(&mut expected, path, value);
        }
        for member in ["charters", "templates", "policies", "rules"] {
            expected["engine"].as_object_mut().unwrap().remove(member);
            assert!(actual["engine"].get(member).is_none(), "{row}: {member}");
        }
        assert_eq!(actual, expected, "{row}");
    }
}

fn demo(port: u16) -> Value {
    json!({
        "llm": {
            "listen": {
                "socket": format!("127.0.0.1:{port}")
            },
            "provider": 3,
            "key": "demo/llm.key",
            "asked": [
                {
                    "cue": "Add CSV export to widget",
                    "turns": csv_chat()
                },
                {
                    "cue": "Add a dark theme to widget",
                    "turns": [
                        demo_turn("I would give widget a dark theme that follows the system's preference, with its colours as variables in one place so both themes stay in step. I am the scripted model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done.")
                    ]
                },
                {
                    "cue": "Fix widget's empty search state",
                    "turns": [
                        demo_turn("I would have widget say when a search finds nothing, name the words searched for and offer to clear them, in place of an empty table that looks broken. I am the scripted model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done.")
                    ]
                }
            ],
            "cued": [
                {
                    "cue": "## Task\n\nExport widget's table as CSV",
                    "turns": csv_coding()
                },
                {
                    "cue": "Review this change at the named head",
                    "turns": [scripted_turn(json!([{"call": "finish", "arguments": {
                        "form": "verdict", "label": "1", "text": "Checked widget CSV export.", "fields": {}, "items": parse(b"[]")
                    }}]))]
                },
                {"cue": "Write this up as a draft beside us.", "turns": chat_document(false)},
                {"cue": "Add what happens to a running goal.", "turns": chat_document(true)}
            ],
            "otherwise": [
                demo_turn("I am the scripted model, the playground’s: I answer only the suggested requests, from a script. To ask for other work, connect a model from Accounts: a ChatGPT plan, a Claude subscription or an API key.")
            ]
        }
    })
}

// The chat-document walk establishes literal project 1, chat 1 and note 1.
fn chat_document(revised: bool) -> Value {
    let body = concat!(
        "# Away mode\n",
        "\n",
        "Draft, 2026-11-05, with the user.\n",
        "\n",
        "## Purpose\n",
        "\n",
        "Away mode lets the person step away from ongoing work.\n",
        "Agents keep working within the authority already granted.\n",
        "They leave a clear account of progress for the person's return.\n",
        "\n",
        "The person chooses when to leave and when to return.\n",
        "Away mode does not grant extra authority.\n",
        "\n",
        "## When the person returns\n",
        "\n",
        "Show the work completed while the person was away.\n",
        "Keep the decisions that need the person in one place.\n",
        "Make each question name the goal and the choice it needs.\n",
        "\n",
        "The person can answer questions before starting more work.\n",
        "The conversation and its document remain available.\n",
        "\n",
        "## Open questions\n",
        "\n",
        "- Where should the person switch away mode on?\n",
        "- Should it apply to one project or every project?\n",
        "- How should urgent questions be shown on return?\n",
        "\n",
        "These choices remain for the person to settle.\n",
    );
    let mut arguments = json!({
        "scope": {"kind": "goal", "project": 1_u32, "goal": 1_u64},
        "description": "Away mode", "body": body
    });
    let sentence = if revised {
        arguments["name"] = json!(1_u64);
        arguments["recalled"] = json!(1_u32);
        arguments["body"] = json!(body.replace("## When the person returns", "## A running goal\n\nA running goal continues within its existing authority and budget.\nWhen it needs the person's decision, it waits with a clear question.\nOther work that can proceed independently keeps moving.\nAway mode never supplies an answer on the person's behalf.\n\n## When the person returns"));
        "I added how a running goal continues and waits for your decisions."
    } else {
        "I wrote an Away mode draft with its purpose, return flow and open questions."
    };
    json!([scripted_turn(json!([{"call": "note", "arguments": arguments}])), demo_turn(sentence)])
}

// The template delegates playground/widget: forge 1, repository 1, tag 4294967297.
fn csv_chat() -> Value {
    // The stock's scripted change takes 5K, with K = 4C at Fable's cache-write rate.
    let c = (1_048_576_u64 * 12_500 + 8192 * 50_000).div_ceil(1_000_000);
    let k = c.checked_mul(4).unwrap();
    let change_spend = k.checked_mul(5).unwrap();
    json!([
        scripted_turn(json!([
            {"text": "I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge."},
            {"call": "delegate", "arguments": {"batch": [{
                "executor": {"kind": "procedure", "connector": 1, "code": 2},
                "spec": {"words": "Export widget's table as CSV", "parameters": [
                    {"name": 1, "kind": "resource", "connector": 1, "resource": 4_294_967_297_u64},
                    {"name": 2, "kind": "bytes", "value": "main"}
                ]},
                "contract": {"kind": "change", "connector": 1, "change_kind": 1, "words": 128},
                "authority": {"tools": 1, "grants": [demo_grant(2), demo_grant(3), demo_grant(4), demo_grant(10), demo_grant(11), demo_grant(12)],
                    "delegation": {"kinds": [{"kind": "agent", "number": 2}, {"kind": "agent", "number": 4}], "tasks": 4, "depth": 2},
                    "budget": {"spend": change_spend}, "notes": 0}
            }]}}
        ])),
        scripted_turn(json!([{"call": "wait", "arguments": {}}])),
        scripted_turn(json!([
            {"text": "CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change."},
            {"call": "wait", "arguments": {}}
        ]))
    ])
}

fn demo_grant(kind: u16) -> Value {
    json!({"connector": 1_u16, "kind": kind, "segments": ["forge"], "terminal": "open", "last": ""})
}

fn csv_coding() -> Value {
    let old = "fn main() {\n    for (name, count) in TABLE {\n        println!(\"{name:<8}{count}\");\n    }\n}\n";
    let new = concat!(
        "fn main() {\n",
        "    let csv = std::env::args().any(|arg| arg == \"--csv\");\n",
        "    if csv {\n",
        "        println!(\"name,count\");\n",
        "    }\n",
        "    for (name, count) in TABLE {\n",
        "        if csv {\n",
        "            println!(\"{name},{count}\");\n",
        "        } else {\n",
        "            println!(\"{name:<8}{count}\");\n",
        "        }\n",
        "    }\n",
        "}\n"
    );
    json!([
        scripted_turn(json!([{"call": "read", "arguments": {"path": "src/main.rs"}}])),
        scripted_turn(json!([{"call": "edit", "arguments": {
            "path": "src/main.rs", "edits": [{"old": old, "new": new}]
        }}])),
        scripted_turn(json!([{"call": "finish", "arguments": {"form": "change", "fields": {
            "title": "Add CSV export to widget",
            "body": "widget --csv prints the table as CSV: a header line, then a line for each row. This was a scripted change."
        }}}]))
    ])
}

fn scripted_turn(lines: Value) -> Value {
    let mut bytes = 0_usize;
    for line in lines.as_array().unwrap() {
        let length = match line["text"].as_str() {
            Some(text) => text.len(),
            None => line["call"]
                .as_str()
                .unwrap()
                .len()
                .checked_add(serde_json::to_vec(&line["arguments"]).unwrap().len())
                .unwrap(),
        };
        bytes = bytes.checked_add(length).unwrap();
    }
    json!({"lines": lines, "finish": "tool_calls", "tokens": bytes.div_ceil(4)})
}

fn demo_turn(text: &str) -> Value {
    json!({"lines": [{"text": text}], "finish": "stop", "tokens": text.len().div_ceil(4)})
}

fn demo_agent(port: u16) -> Value {
    json!({"name": "demo", "number": 3_u64, "dialect": 3_u64, "provider": "openai",
           "address": format!("127.0.0.1:{port}"), "transport": "plaintext",
           "connect": 10_000_u64, "handshake": 10_000_u64, "headers": parse(b"[]"),
           "limits": { "window": 1_048_576_u64, "output": 8_192_u64,
                       "reasoning_item": 2_048_u64, "head": 60_000_u64, "idle": 30_000_u64}})
}

#[test]
fn the_agent_profile_holds_a_run_and_twenty_five_sub_agents() {
    for source in [template::AGENT, fixtures::AGENT] {
        let agent = parse(source);
        assert_eq!(agent["profile"]["declared"]["conversations"], 26_u32);
    }
}

#[test]
fn the_agent_configuration_is_the_owned_examples_but_its_programs() {
    let owned = parse(fixtures::AGENT);
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
        let mut expected = owned.clone();
        let programs = match facts.claude {
            Some(claude) => json!({"claude_code": {"path": text(claude), "directory": "/d/temper/programs"}}),
            None => json!({"claude_code": {"path": "fetched", "directory": "/d/temper/programs"}}),
        };
        set(&mut expected, "/programs", programs);
        set(
            &mut expected,
            "/endpoints",
            json!([
                owned["endpoints"][0],
                anthropic_agent("endpoints"),
                anthropic_agent("direct"),
                demo_agent(facts.ports.llm.get())
            ]),
        );
        expected["endpoints"][0]["limits"] = agent_limits("openai");
        assert_eq!(parse(content(&answered, b"agent.json")), expected, "{row}");
    }
}

#[test]
fn facts_are_written_as_json_text() {
    for (row, binary, escaped) in [
        (
            "quote and backslash",
            b"/opt/tem\"per\\bin/temper".as_slice(),
            b"\"/opt/tem\\\"per\\\\bin/temper\"".as_slice(),
        ),
        (
            "control bytes and UTF-8",
            b"/opt/\n\t\x01/\xc3\xa9/temper".as_slice(),
            b"\"/opt/\\n\\t\\u0001/\xc3\xa9/temper\"".as_slice(),
        ),
    ] {
        let facts = crate::Facts { binary, ..fixtures::facts() };
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid text paths");
        let document = content(&answered, b"temper.json");
        assert!(bytes::find(document, escaped).is_some(), "{row}: JSON escaped the path");
        let parsed = parse(document);
        assert_eq!(parsed["engine"]["llm"]["endpoints"][3]["address"], json!({"socket": "127.0.0.1:8370"}), "{row}");
        assert_eq!(parse(content(&answered, b"agent.json"))["endpoints"][3], demo_agent(8370), "{row}");
        assert_eq!(
            parsed
                .pointer("/workers/0/agent/program")
                .expect("the worker has an agent program")
                .as_str()
                .expect("a JSON path is text")
                .as_bytes(),
            binary,
            "{row}"
        );
    }
}

#[test]
fn no_placeholder_is_left() {
    for (row, facts) in cases() {
        let answered = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("valid facts");
        for name in [b"temper.json".as_slice(), b"agent.json".as_slice()] {
            assert_eq!(bytes::find(content(&answered, name), b"\"$"), None, "{row}: {name:?}");
        }
    }
    for (row, source, expected) in [
        (
            "temper",
            template::TEMPER,
            [
                (b"\"$binary\"".as_slice(), 1_u32),
                (b"\"$agent\"".as_slice(), 2),
                (b"\"$git\"".as_slice(), 1),
                (b"\"$listener\"".as_slice(), 1),
                (b"\"$forge\"".as_slice(), 1),
                (b"\"$forge_git\"".as_slice(), 1),
                (b"\"$owned\"".as_slice(), 1),
                (b"\"$owned_git\"".as_slice(), 1),
                (b"\"$login\"".as_slice(), 1),
                (b"\"$email\"".as_slice(), 2),
                (b"\"$provider\"".as_slice(), 1),
                (b"\"$owned_sign_in\"".as_slice(), 1),
                (b"\"$llm\"".as_slice(), 2),
                (b"\"$webhooks\"".as_slice(), 1),
            ]
            .as_slice(),
        ),
        ("agent", template::AGENT, [(b"\"$programs\"".as_slice(), 1_u32), (b"\"$llm\"".as_slice(), 1)].as_slice()),
        (
            "bare",
            template::BARE,
            [
                (b"\"$binary\"".as_slice(), 1_u32),
                (b"\"$agent\"".as_slice(), 2),
                (b"\"$git\"".as_slice(), 1),
                (b"\"$listener\"".as_slice(), 1),
                (b"\"$webhooks\"".as_slice(), 1),
            ]
            .as_slice(),
        ),
        ("bare agent", template::BARE_AGENT, [(b"\"$programs\"".as_slice(), 1_u32)].as_slice()),
    ] {
        // Exact token counts catch missing, extra and misspelled placeholders.
        let mut count = 0_u32;
        for (token, wanted) in expected {
            assert_eq!(bytes::count(source, token, u32::MAX), *wanted, "{row}: {token:?}");
            count = count.checked_add(*wanted).expect("the table's token count fits");
        }
        assert_eq!(bytes::count(source, b"\"$", u32::MAX), count, "{row}: only known tokens");
        parse(source);
    }
}

#[test]
fn the_person_keeps_a_forgejo_login_or_falls_back() {
    use crate::Person::{FellBack, Found};
    for (row, login, written, person) in [
        ("valid", Some(b"ada".as_slice()), "ada", Found),
        ("case kept", Some(b"Ada".as_slice()), "Ada", Found),
        ("reserved", Some(b"admin".as_slice()), "person", FellBack),
        ("reserved case", Some(b"Admin".as_slice()), "person", FellBack),
        ("reserved pattern", Some(b"x.keys".as_slice()), "person", FellBack),
        ("pattern case", Some(b"X.GPG".as_slice()), "person", FellBack),
        ("pattern is suffix", Some(b"x.keys.other".as_slice()), "x.keys.other", Found),
        ("invalid start", Some(b"-ada".as_slice()), "person", FellBack),
        (
            "at bound",
            Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_slice()),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            Found,
        ),
        ("past bound", Some(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".as_slice()), "person", FellBack),
        ("bot", Some(b"temper".as_slice()), "person", FellBack),
        ("admin account", Some(b"temper-admin".as_slice()), "person", FellBack),
        ("org", Some(b"work".as_slice()), "person", FellBack),
        ("bot case", Some(b"Temper".as_slice()), "person", FellBack),
        ("empty", Some(b"".as_slice()), "person", FellBack),
        ("nontext", Some(b"\xff".as_slice()), "person", FellBack),
        ("absent", None, "person", FellBack),
    ] {
        let facts = crate::Facts { login, ..fixtures::facts() };
        let answer = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("login never refuses init");
        assert_eq!(answer.person, person, "{row}");
        let document = parse(content(&answer, b"temper.json"));
        assert_eq!(
            document["application"]["machine"]["owned"]["person"],
            json!({"login": written, "email": format!("{written}@localhost")}),
            "{row}"
        );
    }
}

#[test]
fn the_template_seeds_no_project_and_no_settings_members() {
    let document = parse(template::TEMPER);
    assert_eq!(document["engine"]["projects"], parse(b"[]"));
    for member in ["charters", "templates", "policies", "rules"] {
        assert!(document["engine"].get(member).is_none());
    }
    assert!(document["application"].get("landing").is_none());
    assert!(document["application"].get("charter").is_none());
}

#[test]
fn the_bare_document_is_the_box_less_its_parts() {
    for (row, facts) in cases() {
        let boxed = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).expect("boxed files");
        let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).expect("bare files");
        let mut expected = parse(content(&boxed, b"temper.json"));
        assert_eq!(expected["application"]["welcome"], true, "{row}");
        expected["application"].as_object_mut().unwrap().remove("welcome");
        expected["application"]["machine"] = json!({"github": null});
        expected["application"]["demo"] = Value::Null;
        expected["engine"]["clients"].as_object_mut().unwrap().remove("provider");
        expected["engine"]["owners"][0]["subject"] = json!("owner-a");
        expected["engine"].as_object_mut().unwrap().remove("acts");
        for member in ["providers", "endpoints"] {
            expected["engine"]["llm"][member].as_array_mut().unwrap().retain(|item| item["name"] != "demo");
        }
        let mut agent = parse(content(&boxed, b"agent.json"));
        agent["endpoints"].as_array_mut().unwrap().retain(|item| item["name"] != "demo");
        let names = [b"worker.secret".as_slice(), b"worker.digest", b"agent.json", b"temper.json"];
        assert_eq!(bare.len(), names.len(), "{row}");
        for (file, name) in bare.iter().zip(names) {
            assert_eq!(file.name, name, "{row}");
            let source = fixtures::file(&boxed.files, name);
            assert_eq!(file.directory, source.directory, "{row}");
            let crate::Content::Bytes { bytes, private } = &file.content else { panic!("bare files hold bytes") };
            assert_eq!(bytes::find(bytes, b"\"$"), None, "{row}: no placeholder");
            match name {
                b"temper.json" => {
                    assert_eq!(parse(bytes), expected, "{row}");
                    assert!(!private);
                }
                b"agent.json" => {
                    assert_eq!(parse(bytes), agent, "{row}");
                    assert!(!private);
                }
                _ => assert_eq!(file, source, "{row}: shared secret and digest"),
            }
        }
    }
}

// protocol/configuration/stock.md: the seven declarations, including their document order.

fn agent_limits(endpoint: &str) -> Value {
    json!({"window":1_048_576_u32,"output":8192,"reasoning_item":65_536_u32,
        "head":60_000_u32,"idle":if endpoint == "openai" {300_000_u32} else {120_000_u32}})
}

fn anthropic(member: &str) -> Value {
    match member {
        "providers" => {
            parse(br#"{"name":"anthropic","number":2,"signed_in_plan":false,"manage_usage":"https://claude.ai/settings/usage","endpoint":"anthropic","direct":"anthropic-api","keep":["claude-*"],"drop":["*-20??????"],"sources":{"programs":["claude_code"],"keys":true}}"#)
        }
        "endpoints" => {
            parse(br#"{"name":"anthropic","number":2,"dialect":2,"provider":"anthropic","program":"claude_code","address":{"host":"api.anthropic.com","port":443},"transport":"tls","headers":[],"connect_ms":10000,"handshake_ms":10000,"server_name":"api.anthropic.com"}"#)
        }
        "direct" => {
            let mut endpoint = anthropic("endpoints");
            endpoint["name"] = json!("anthropic-api");
            endpoint["number"] = json!(4_u32);
            endpoint.as_object_mut().unwrap().remove("program");
            endpoint
        }
        _ => panic!("known LLM member"),
    }
}
fn anthropic_agent(member: &str) -> Value {
    let mut endpoint = anthropic(member);
    endpoint["address"] = json!("api.anthropic.com:443");
    let object = endpoint.as_object_mut().unwrap();
    for (old, new) in [("connect_ms", "connect"), ("handshake_ms", "handshake")] {
        let value = object.remove(old).unwrap();
        object.insert(new.into(), value);
    }
    endpoint["limits"] = agent_limits("anthropic");
    endpoint
}
#[test]
fn every_deployment_offers_claude_found_or_fetched() {
    // protocol/setup/init.md, 3: OpenAI, Anthropic and demo fit in one offer.
    const PROVIDERS: u32 = 3;
    for (row, facts) in cases() {
        let boxed = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).unwrap();
        let bare = crate::bare(&fixtures::bare_facts(&facts), &DIRECTORIES, &[0; 32]).unwrap();
        for files in [boxed.files.as_ref(), bare.as_ref()] {
            let document = parse(file_bytes(files, b"temper.json"));
            let agent = parse(file_bytes(files, b"agent.json"));
            // Providers and endpoints keep their order beside the seven model declarations.
            for (member, at) in [("providers", 1), ("endpoints", 1)] {
                assert_eq!(document["engine"]["llm"][member][at], anthropic(member), "{row}: {member}");
            }
            assert!(document["engine"]["llm"].get("models").is_none());
            assert_eq!(document["engine"]["llm"]["endpoints"][2], anthropic("direct"), "{row}: direct");
            assert_eq!(agent["endpoints"][1], anthropic_agent("endpoints"), "{row}");
            assert_eq!(agent["endpoints"][2], anthropic_agent("direct"), "{row}");
            assert_eq!(
                agent["programs"]["claude_code"],
                json!({"path": text(facts.claude.unwrap_or(crate::FETCHED)), "directory": "/d/temper/programs"}),
                "{row}"
            );
            assert_eq!(agent["environment"], parse(b"[]"), "{row}");
            let bytes = serde_json::to_vec(&document["engine"]["limits"]).unwrap();
            let parsed = jig_protocol_config::document(
                &bytes,
                &jig_protocol_config::DocumentLimits { bytes: 65_536, tokens: 4096, depth: 16 },
            )
            .unwrap();
            let parsed = jig_protocol_config::overrides::parse(&parsed, 0, &jig_protocol_config::Path::root());
            assert!(parsed.problems.is_empty(), "{row}: {:?}", parsed.problems);
            assert!(parsed.limits.core.documents.lines >= PROVIDERS, "{row}");
        }
    }
}

fn file_bytes<'a>(files: &'a [crate::File], name: &[u8]) -> &'a [u8] {
    let crate::Content::Bytes { bytes, .. } = &fixtures::file(files, name).content else { panic!("document bytes") };
    bytes
}

#[test]
fn a_box_signs_its_person_in_with_a_public_forgejo_client() {
    let facts = fixtures::facts();
    let answer = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).unwrap();
    let document = parse(content(&answer, b"temper.json"));
    assert_eq!(document["engine"]["owners"][0]["subject"], "ada@localhost");
    assert_eq!(
        document["engine"]["clients"]["provider"],
        json!({
            "key": "provider.key", "addresses": [1_u32],
            "clients": [{"id": "forgejo", "secret": null,
                "redirect_uri": format!("http://127.0.0.1:{}/user/oauth2/temper/callback", facts.ports.owned)}]
        })
    );
    assert_eq!(document["application"]["machine"]["owned"]["sign_in"], "forgejo");
    assert!(document["application"]["sign_in"].is_null());
}

#[test]
fn a_box_without_a_key_keeps_its_provider_and_source_null() {
    let facts = crate::Facts { sign_in: false, ..fixtures::facts() };
    let answer = files(&facts, &DIRECTORIES, &[0; 32], &[0xab; 32]).unwrap();
    let document = parse(content(&answer, b"temper.json"));
    assert!(document["engine"]["clients"]["provider"].is_null());
    assert!(document["application"]["machine"]["owned"]["sign_in"].is_null());
    assert_eq!(document["engine"]["owners"][0]["subject"], "ada@localhost");
}
