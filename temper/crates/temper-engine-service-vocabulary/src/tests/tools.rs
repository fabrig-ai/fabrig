use jig_domain_core as core;
use jig_domain_host as host;
use skein_lib::{ReplyTo, Token, Writer};
use temper_engine_domain as root;

const TO: ReplyTo = ReplyTo::new(Token::new(9));
fn call(activation: u64, tool: &[u8], input: &[u8]) -> jig_domain_fleet::Call {
    let name = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation, completion: 2, position: 1 },
    )
    .unwrap();
    let mut writer = Writer::new(16);
    name.encode(&mut writer).unwrap();
    jig_domain_fleet::Call {
        contract: 1,
        name: writer.finish(),
        tool: tool.into(),
        input: input.into(),
        writes: false,
        deadline: skein_lib::Duration::from_secs(1),
    }
}
#[test]
fn a_call_is_decoded_into_the_roots_tool() {
    let event = crate::tools::decode(TO, 3, 7, &call(7, b"release", br#"{"target":5,"revision":1}"#));
    let root::Event::Worker(root::WorkerInput::DecodedHostCall { to, body }) = event else { panic!("decoded call") };
    assert_eq!(to, TO);
    assert_eq!(
        body,
        root::Call { completion: 2, position: 1, tool: root::Tool::Release { target: 5, revision: 1, again: None } }
    );
}
#[test]
fn a_call_of_another_activation_is_answered_an_error() {
    let event = crate::tools::decode(TO, 3, 8, &call(7, b"release", br#"{"target":5,"revision":1}"#));
    let root::Event::Worker(root::WorkerInput::RenderedHostCall { to, answer }) = event else {
        panic!("rendered error")
    };
    assert_eq!(to, TO);
    assert_eq!(answer, core::SettledAnswer::Host { error: true, body: b"malformed host call".as_slice().into() });
}
#[test]
fn the_largest_smith_call_name_fits_the_documentation_allowance() {
    let name = smith_channel::CallName::new(
        &smith_channel::CEILINGS,
        smith_channel::CallNameParts { activation: u64::MAX, completion: u32::MAX, position: u32::MAX },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(temper_engine_smith::CALL_NAME_BYTES).unwrap());
    name.encode(&mut writer).unwrap();
    let encoded = writer.finish();
    assert_eq!(encoded.len(), 16);
    assert!(encoded.len() <= usize::try_from(temper_engine_smith::CALL_NAME_BYTES).unwrap());
    let mut reader = skein_lib::Reader::new(&encoded);
    assert_eq!(smith_channel::CallName::decode(&smith_channel::CEILINGS, &mut reader).unwrap(), name);
    assert!(reader.is_empty());
}
#[test]
fn a_decided_answer_is_rendered_as_smiths_text() {
    let event = crate::tools::render(
        TO,
        &root::CallAnswer::NoteWritten { name: 4, revision: 2, lines: 3, added: 2, removed: 1, first: 2 },
        b"note",
        1024,
        temper_engine_smith::ForgeReadBounds::STOCK,
    );
    let root::Event::Worker(root::WorkerInput::RenderedHostCall { to, answer }) = event else {
        panic!("rendered answer")
    };
    assert_eq!(to, TO);
    assert_eq!(
        answer,
        core::SettledAnswer::Host {
            error: false,
            body: "Note 4 revision 2 written · +2 −1 · line 2 · 3 lines".as_bytes().into()
        }
    );
}
#[test]
fn a_result_that_does_not_decode_fails_its_attempt_invalid() {
    for (bytes, expected) in [
        (b"".as_slice(), jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid)),
        (b"not a result", jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid)),
        (&[0, 0, 0, 99], jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid)),
    ] {
        let event = crate::tools::result(Token::new(3), bytes, None);
        let root::Event::Worker(root::WorkerInput::ResultRead { payload, end }) = event else {
            panic!("read result {bytes:?}")
        };
        assert_eq!(payload, Token::new(3), "{bytes:?}");
        assert_eq!(end, expected, "{bytes:?}");
    }
}

#[test]
fn each_invalid_call_names_its_problem() {
    let oversized = skein_lib::bytes::zeroed(smith_domain_run::HostInput::CAPACITY + 1);
    for (input, expected) in [
        (b"[]".as_slice(), b"malformed host call".as_slice()),
        (b"{broken}", b"malformed host call"),
        (b"{}", b"host call member missing: target"),
        (br#"{"target":"five"}"#, b"host call member of wrong type: target"),
        (br#"{"target":0}"#, b"host call out of range: target"),
        (oversized.as_ref(), b"host call too large"),
    ] {
        let event = crate::tools::decode(TO, 3, 7, &call(7, b"release", input));
        let root::Event::Worker(root::WorkerInput::RenderedHostCall { answer, .. }) = event else {
            panic!("invalid input {input:?}")
        };
        assert_eq!(answer, core::SettledAnswer::Host { error: true, body: expected.into() }, "{input:?}");
    }
}
#[test]
fn red_wrong_shapes_are_errors_at_decode_naming_the_member() {
    for (input, expected) in [
        (
            r#"{"act":"end","key":"test a::t x","why":"fixed","at":"…"}"#.as_bytes(),
            b"red: member at does not belong to this act".as_slice(),
        ),
        (br#"{"act":"list","why":"fixed"}"#, b"red: member why does not belong to this act"),
        (br#"{"act":"list","strange":"x"}"#, b"red: member strange does not belong to this act"),
        (br#"{"act":"end","key":"k"}"#, b"host call member missing: why"),
        (
            br#"{"act":"record","key":"k","at":"bad","why":"w"}"#,
            b"host call out of range: red: at must be 40 or 64 lowercase hexadecimal digits",
        ),
    ] {
        let mut called = call(7, b"red", input);
        called.contract = temper_engine_smith::wire::VERSION;
        let event = crate::tools::decode(TO, 3, 7, &called);
        let root::Event::Worker(root::WorkerInput::RenderedHostCall { answer, .. }) = event else {
            panic!("refused at decode")
        };
        assert_eq!(answer, core::SettledAnswer::Host { error: true, body: expected.into() });
    }
}
#[test]
fn a_report_result_keeps_its_words() {
    let report = smith_domain_run::outcome::Declared::Report(smith_domain_run::outcome::Report {
        text: b"finished".as_slice().into(),
        fields: Box::new([]),
    });
    let bytes = smith_protocol_channel::encode_result(&report, &smith_charter::CEILINGS).unwrap();
    let event = crate::tools::result(Token::new(3), &bytes, None);
    let root::Event::Worker(root::WorkerInput::ResultRead { payload, end }) = event else { panic!("report read") };
    assert_eq!(payload, Token::new(3));
    assert_eq!(
        end,
        jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Report { words: b"finished".as_slice().into() },
            cancel_delegates: false,
        }
    );
}
#[test]
fn a_change_result_without_settled_resource_is_invalid() {
    let change =
        smith_domain_run::outcome::Declared::Change(smith_domain_run::outcome::Change { fields: Box::new([]) });
    let bytes = smith_protocol_channel::encode_result(&change, &smith_charter::CEILINGS).unwrap();
    let event = crate::tools::result(Token::new(3), &bytes, None);
    let root::Event::Worker(root::WorkerInput::ResultRead { end, .. }) = event else { panic!("change read") };
    assert_eq!(end, jig_domain_tasks::End::Failed(jig_domain_tasks::Class::Invalid));
}

#[test]
fn a_change_result_keeps_the_settled_resource() {
    let change = smith_domain_run::outcome::Declared::Change(smith_domain_run::outcome::Change {
        fields: Box::new([
            smith_domain_run::outcome::Field { name: b"title".as_slice().into(), value: b"Change".as_slice().into() },
            smith_domain_run::outcome::Field { name: b"body".as_slice().into(), value: b"Changed".as_slice().into() },
        ]),
    });
    let bytes = smith_protocol_channel::encode_result(&change, &smith_charter::CEILINGS).unwrap();
    let resource = temper_engine_smith::ChangeResource { connector: 1, kind: 2, resource: 42 };
    let event = crate::tools::result(Token::new(3), &bytes, Some(resource));
    let root::Event::Worker(root::WorkerInput::ResultRead { payload, end }) = event else { panic!("change read") };
    assert_eq!(payload, Token::new(3));
    assert_eq!(
        end,
        jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Change {
                title: jig_domain_tasks::ChangeTitle::new(b"Change".as_slice().into()),
                connector: 1,
                kind: 2,
                resource: 42,
                words: b"Changed".as_slice().into(),
            },
            cancel_delegates: false,
        }
    );
}
#[test]
fn each_settled_delivery_becomes_the_hubs_answer() {
    for (outcome, error, text) in [
        (core::DeliveryOutcome::Delivered, false, b"delivered: evidence".as_slice()),
        (core::DeliveryOutcome::Nothing, false, b"nothing to deliver: evidence"),
        (core::DeliveryOutcome::Stale, true, b"stale delivery: evidence"),
        (core::DeliveryOutcome::Refused, true, b"delivery refused: evidence"),
        (core::DeliveryOutcome::Invalid, true, b"invalid delivery: evidence"),
        (core::DeliveryOutcome::Failed, true, b"delivery failed: evidence"),
    ] {
        let answer =
            crate::tools::settled(core::SettledAnswer::Delivery { outcome, evidence: b"evidence".as_slice().into() });
        assert_eq!(answer, host::Relayed::Answer { error, text: text.into() }, "{outcome:?}");
    }
}

#[test]
fn a_flat_forge_read_error_names_its_missing_member() {
    for (tool, input, expected) in [
        (
            b"settings".as_slice(),
            br#"{"setting":{"kind":"landing-rule"}}"#.as_slice(),
            b"landing-rule needs number; read the inventory and name what it lists".as_slice(),
        ),
        (
            b"settings",
            br#"{"setting":{"kind":"project-ways"}}"#,
            b"project-ways needs project; read the inventory and name what it lists",
        ),
        (b"read_branch".as_slice(), b"{}".as_slice(), b"host call member missing: branch".as_slice()),
        (b"read_branch", br#"{"branch":1}"#, b"host call member of wrong type: branch"),
        (b"read_reviews", br#"{"number":1,"page":0}"#, b"host call out of range: page"),
        (
            b"change",
            br#"{"landing":"goal","changes":[{"words":"a"}]}"#,
            b"host call member belongs in each change, not beside changes: landing",
        ),
    ] {
        let event = crate::tools::decode(TO, 3, 7, &call(7, tool, input));
        let root::Event::Worker(root::WorkerInput::RenderedHostCall { answer, .. }) = event else {
            panic!("read error");
        };
        assert_eq!(answer, core::SettledAnswer::Host { error: true, body: expected.into() });
    }
}

#[test]
fn a_denied_release_retains_its_family_when_rendered_for_the_host() {
    use jig_domain_authority::{Answer, Finding};
    let denied = root::CallAnswer::ToolDenied {
        answer: Answer::Refuse,
        findings: Box::new([Finding::Tool, Finding::Tool, Finding::Tool]),
    };
    let event = crate::tools::render(TO, &denied, b"release", 1024, temper_engine_smith::ForgeReadBounds::STOCK);
    let root::Event::Worker(root::WorkerInput::RenderedHostCall { to, answer }) = event else {
        panic!("rendered refusal")
    };
    assert_eq!(to, TO);
    assert_eq!(
        answer,
        core::SettledAnswer::Host { error: true, body: b"Authority refused: tool family control".as_slice().into() }
    );
}

#[test]
fn an_abbreviated_commit_read_explains_the_full_identifier_and_review_head() {
    for (tool, input) in [
        (b"read_file".as_slice(), br#"{"head":"f471720","path":"README.md"}"#.as_slice()),
        (b"read_compare", br#"{"before":"f471720","after":"f471720"}"#),
    ] {
        let event = crate::tools::decode(TO, 3, 7, &call(7, tool, input));
        let root::Event::Worker(root::WorkerInput::RenderedHostCall {
            to,
            answer: core::SettledAnswer::Host { error, body },
        }) = event
        else {
            panic!("read refusal")
        };
        assert_eq!(to, TO);
        assert!(error);
        assert_eq!(body.as_ref(), b"expected a full commit identifier (40 or 64 lowercase hexadecimal digits); use the full head from the review brief");
    }
}
