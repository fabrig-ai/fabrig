//! Charter-driven schema generation and typed finish/deliver inputs.

use skein_fake_llm_domain::api::{Line, Script};
use skein_lib::Token;
use smith_domain::{llm, run};
use smith_protocol_llm_world::client_peer;
use smith_protocol_llm_world::codec::{self as codec};
use smith_protocol_llm_world::fixture::limits;

fn field(name: &[u8], max: u32) -> run::outcome::FieldRule {
    run::outcome::FieldRule { name: name.into(), max }
}

fn contract() -> run::outcome::OutcomeSpec {
    run::outcome::OutcomeSpec {
        change: Some(run::outcome::ChangeSpec { checks_must_pass: true, fields: Box::new([field(b"ticket", 12)]) }),
        verdicts: Box::new([run::outcome::VerdictRule {
            name: b"approve".as_slice().into(),
            text_max: 40,
            fields: Box::new([field(b"source", 20)]),
            items: run::outcome::ItemSpec {
                min: 1,
                max: 2,
                kinds: Box::new([run::outcome::ItemRule {
                    kind: b"observation".as_slice().into(),
                    fields: Box::new([field(b"path", 32)]),
                }]),
            },
        }]),
        report: Some(run::outcome::TextSpec { max: 80, fields: Box::new([field(b"source", 20)]) }),
        failure: Some(run::outcome::TextSpec { max: 60, fields: Box::new([field(b"cause", 24)]) }),
    }
}

fn served(ask: llm::Decoded) -> run::Ask {
    match ask {
        llm::Decoded::Served { ask } => ask,
        llm::Decoded::Builtin { .. } | llm::Decoded::Owned { .. } | llm::Decoded::Invalid { .. } => {
            panic!("typed run ask")
        }
    }
}

fn text_contract(fields: Box<[run::outcome::FieldRule]>) -> run::outcome::OutcomeSpec {
    run::outcome::OutcomeSpec {
        change: None,
        verdicts: Box::new([]),
        report: Some(run::outcome::TextSpec { max: 80, fields: fields.clone() }),
        failure: Some(run::outcome::TextSpec { max: 60, fields }),
    }
}

#[test]
fn text_finish_without_fields_decodes_for_a_fieldless_rule() {
    let rules = text_contract(Box::new([]));
    for input in [br#"{"form":"report","text":"done"}"#.as_slice(), br#"{"form":"failure","reason":"blocked"}"#] {
        let run::Ask::Finish { outcome } = served(codec::decode_finish(input, &rules, &limits())) else {
            panic!("finish ask")
        };
        assert_eq!(run::outcome::judge(&rules, &outcome), Ok(()));
    }
}

#[test]
fn text_finish_with_empty_fields_still_decodes_for_a_fieldless_rule() {
    let rules = text_contract(Box::new([]));
    for input in [
        br#"{"form":"report","text":"done","fields":{}}"#.as_slice(),
        br#"{"form":"failure","reason":"blocked","fields":{}}"#,
    ] {
        let run::Ask::Finish { outcome } = served(codec::decode_finish(input, &rules, &limits())) else {
            panic!("finish ask")
        };
        assert_eq!(run::outcome::judge(&rules, &outcome), Ok(()));
    }
}

#[test]
fn text_finish_without_required_fields_names_the_missing_field() {
    let rules = text_contract(Box::new([field(b"source", 20)]));
    for input in [br#"{"form":"report","text":"done"}"#.as_slice(), br#"{"form":"failure","reason":"blocked"}"#] {
        assert!(matches!(
            codec::decode_finish(input, &rules, &limits()),
            llm::Decoded::Invalid { problem: llm::Problem::Missing { field } } if field.as_ref() == b"fields"
        ));
    }
}

#[test]
fn text_finish_fieldless_schema_omits_the_fields_property_and_requirement() {
    let schema = codec::finish_schema(&text_contract(Box::new([])), limits().client.request).expect("text schema");
    let schema: serde_json::Value = serde_json::from_slice(&schema).expect("schema JSON");
    assert_eq!(
        schema,
        serde_json::json!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {"form": {"const": "report"}, "text": {"type": "string", "maxLength": 80}},
                    "required": ["form", "text"]
                },
                {
                    "type": "object",
                    "properties": {"form": {"const": "failure"}, "reason": {"type": "string", "maxLength": 60}},
                    "required": ["form", "reason"]
                }
            ]
        })
    );
}

#[test]
fn text_finish_schema_keeps_declared_fields_required() {
    let rules = text_contract(Box::new([field(b"source", 20)]));
    let schema = codec::finish_schema(&rules, limits().client.request).expect("text schema");
    let schema: serde_json::Value = serde_json::from_slice(&schema).expect("schema JSON");
    let branches = schema["oneOf"].as_array().expect("text forms");
    for branch in branches {
        assert_eq!(
            branch["properties"]["fields"],
            serde_json::json!({
                "type": "object",
                "properties": {"source": {"type": "string", "minLength": 1, "maxLength": 20}},
                "required": ["source"]
            })
        );
        assert!(branch["required"].as_array().expect("required properties").contains(&serde_json::json!("fields")));
    }
}

#[test]
fn every_contract_form_yields_a_bounded_json_schema() {
    let all = contract();
    let forms = [
        run::outcome::OutcomeSpec { change: all.change.clone(), verdicts: Box::new([]), report: None, failure: None },
        run::outcome::OutcomeSpec { change: None, verdicts: all.verdicts.clone(), report: None, failure: None },
        run::outcome::OutcomeSpec { change: None, verdicts: Box::new([]), report: all.report.clone(), failure: None },
        run::outcome::OutcomeSpec { change: None, verdicts: Box::new([]), report: None, failure: all.failure.clone() },
    ];
    for specification in &forms {
        let schema = codec::finish_schema(specification, limits().client.request)
            .expect("contract-derived schema fits the dialect");
        let document = skein_llm::Json::from_bytes(&schema, &limits().client.native().request_document())
            .expect("scripted peer accepts a complete JSON object schema");
        assert_eq!(document.document().token(0).map(|record| record.kind), Some(skein_json::Kind::ObjectStart));
        assert!(skein_lib::bytes::find(&schema, b"\"maxLength\"").is_some());
    }
    let all_schema = codec::finish_schema(&all, limits().client.request).expect("four forms");
    for literal in [b"approve".as_slice(), b"observation", b"maxItems", b"minItems", b"source"] {
        assert!(skein_lib::bytes::find(&all_schema, literal).is_some());
    }
    let delivery =
        codec::deliver_schema(all.change.as_ref().expect("separate grant"), 4096).expect("mid-run change fields");
    assert!(skein_lib::bytes::find(&delivery, b"ticket").is_some());
}

#[test]
fn a_bad_finish_is_corrected_and_each_declared_form_is_typed() {
    let limits = limits();
    let contract = contract();
    let invalid = codec::decode_finish(br#"{"form":"report","fields":{"source":"test"}}"#, &contract, &limits);
    assert!(matches!(
        invalid,
        llm::Decoded::Invalid { problem: llm::Problem::Missing { field } } if field.as_ref() == b"text"
    ));
    let cases = [
        br#"{"form":"change","fields":{"ticket":"abc"}}"#.as_slice(),
        br#"{"form":"verdict","label":"approve","text":"looks good","fields":{"source":"test"},"items":[{"kind":"observation","fields":{"path":"a.rs"}}]}"#,
        br#"{"form":"report","text":"done","fields":{"source":"test"}}"#,
        br#"{"form":"failure","reason":"blocked","fields":{"cause":"remote"}}"#,
    ];
    for input in cases {
        let ask = served(codec::decode_finish(input, &contract, &limits));
        let run::Ask::Finish { outcome } = ask else { panic!("finish ask") };
        assert_eq!(run::outcome::judge(&contract, &outcome), Ok(()), "{}", String::from_utf8_lossy(input));
    }
}

#[test]
fn delivery_fields_decode_without_interpreting_host_names() {
    let limits = limits();
    let change = contract().change.expect("separate change rule");
    let malformed = codec::decode_deliver(br#"{"fields":{"ticket":12}}"#, &change, &limits);
    assert!(matches!(
        malformed,
        llm::Decoded::Invalid { problem: llm::Problem::WrongType { field } } if field.as_ref() == b"ticket"
    ));
    let ask = served(codec::decode_deliver(br#"{"fields":{"ticket":"abc","extra":"kept"}}"#, &change, &limits));
    let run::Ask::Deliver { change } = ask else { panic!("delivery ask") };
    assert_eq!(change.fields.len(), 2);
    assert_eq!(change.fields[1].name.as_ref(), b"extra");
    assert_eq!(change.fields[1].value.as_ref(), b"kept");
}

#[test]
fn contract_schemas_reach_the_scripted_byte_peer_in_both_dialects() {
    let bounds = limits();
    let rules = contract();
    for configuration in &client_peer::configurations() {
        let prompt = llm::Prompt {
            effort: None,
            purpose: llm::Purpose::Ordinary { guard: None },
            choice: smith_domain::llm::ToolChoice::Auto,
            namespace: smith_domain::tools::Namespace::One,
            affinity: smith_domain::llm::Affinity { key: [0; 16], thread: 0 },
            asker: llm::Asker::Main,
            endpoint: llm::Endpoint(1),
            model: b"model".as_slice().into(),
            system: b"@contract-schema".as_slice().into(),
            tools: smith_domain::tools::Grants { inspect: false, modify: false, shell: false },
            served: Box::new([llm::Served::Finish, llm::Served::Deliver]),
            messages: Box::new([llm::Message {
                role: llm::Role::User,
                content: Box::new([llm::Block::Text { text: b"Finish.".as_slice().into(), replay: None }]),
            }]),
            max_tokens: 32,
        };
        let receiving = smith_protocol_llm::fixture::receiving(&bounds.client, 4096).expect("fixture receiving bound");
        let prepared = codec::prepare_for_contract(
            codec::Input {
                owner: Token::new(91),
                endpoint_name: prompt.endpoint,
                prompt,
                endpoint: configuration.endpoint.clone(),
                credential: skein_llm::Credential {
                    value: configuration.credential.value.clone(),
                    account_id: configuration.credential.account_id.clone(),
                    scheme: skein_llm::Scheme::Bearer,
                },

                receiving,
            },
            &rules,
            rules.change.as_ref(),
            &bounds,
        )
        .expect("contract schema admitted before provider work");
        let scripts = Box::new([Script {
            cue: b"@contract-schema".as_slice().into(),
            turns: Box::new([smith_protocol_llm_world::fixture::says(
                Box::new([Line::Text { text: b"ready".as_slice().into() }]),
                1,
            )]),
        }]);
        let mut peer = skein_llm_world::fake::Exchange::prepared(
            prepared.client,
            configuration.endpoint.clone(),
            Some(skein_llm::Credential {
                value: configuration.credential.value.clone(),
                account_id: configuration.credential.account_id.clone(),
                scheme: skein_llm::Scheme::Bearer,
            }),
            bounds.client,
            scripts,
        );
        peer.start();
        peer.run();
        let [query] = peer.queries.as_slice() else { panic!("one provider query") };
        assert_eq!(query.tools.len(), 2);
        for offered in &query.tools {
            let schema = skein_llm::Json::from_bytes(&offered.parameters, &bounds.client.native().request_document())
                .expect("scripted peer accepts the whole schema document");
            assert_eq!(schema.document().token(0).map(|record| record.kind), Some(skein_json::Kind::ObjectStart));
        }
        assert!(skein_lib::bytes::find(&query.tools[0].parameters, b"observation").is_some());
        assert!(skein_lib::bytes::find(&query.tools[1].parameters, b"ticket").is_some());
    }
}

#[test]
fn root_finish_uses_the_admitted_contract_and_corrects_the_fixture_grammar() {
    use smith_protocol_llm_world::{Job, World, fixture};
    for configuration in client_peer::configurations() {
        let mut settings = fixture::settings(903);
        settings.job = Job::Reporting;
        let mut bounds = limits();
        bounds.client.http.request = 16384;
        bounds.client.request = 16384;
        let receiving = smith_protocol_llm::fixture::receiving(&bounds.client, settings.limits.decoded_call_bytes)
            .expect("receiving bounds");
        settings.limits.session.completion_bytes = receiving.max_completion_bytes;
        settings.limits.session.completion_blocks = receiving.max_completion_blocks;
        settings.limits.session.failure_bytes = receiving.max_failure_bytes;
        let mut charter = smith_agent_world::scripted_charter(&settings);
        charter.outcome.report = Some(run::outcome::TextSpec { max: 80, fields: Box::new([field(b"evidence", 20)]) });
        let scripts = Box::new([Script {
            cue: b"@report".as_slice().into(),
            turns: [
                br#"{"report":"fixture grammar","evidence":"peer"}"#.as_slice(),
                br#"{"form":"report","text":"production grammar","fields":{"evidence":"peer"}}"#,
            ]
            .into_iter()
            .map(|arguments| {
                smith_protocol_llm_world::fixture::calls(
                    Box::new([Line::Call { name: b"finish".as_slice().into(), arguments: arguments.into() }]),
                    1,
                )
            })
            .collect(),
        }]);
        let mut world = World::with_workspace_protocol_charter(
            settings,
            None,
            None,
            skein_fake_checkout::Checkout::new(),
            (configuration, bounds),
            scripts,
            charter,
        );
        world.run(100_000);
        let run::Answer::Accepted { outcome: run::outcome::Declared::Report(report), .. } = world.answer() else {
            panic!("production finish accepted: {:?}", world.answer())
        };
        assert_eq!(report.text.as_ref(), b"production grammar");
        assert_eq!(report.fields[0].name.as_ref(), b"evidence");
        assert_eq!(world.completions().len(), 2);
        assert!(
            world.prompts()[0].tools.iter().any(|tool| tool.name.as_ref() == b"finish"
                && skein_lib::bytes::find(&tool.parameters, b"evidence").is_some())
        );
        assert!(world.turns().iter().flat_map(|turn| &turn.messages).flat_map(|message| &message.content).any(|block| matches!(block, smith_domain::session::llm::Block::ToolResult { result: smith_domain::session::llm::Returned::Invalid { problem: llm::Problem::Missing { field } }, .. } if field.as_ref() == b"form")));
    }
}
