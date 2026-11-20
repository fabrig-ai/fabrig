//! Named local refusals and the receiving diagnostic bound (protocol/llm.md, 6).

use super::{direct_limits, direct_prompt, host_tool, native_text, native_text_prompt};
use crate::{Cause, Error, Refusal};
use skein_lib::{Token, bytes};
use smith_domain::{Below, Event, llm, tools};

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "the test reads the design's table and compares finite names with ordinary Rust containers"
)]
fn refusal_names_match_the_design_and_are_disjoint_from_skein() {
    use skein_llm::{Invalid, Unsupported};
    let causes = [
        Refusal::ChoiceNotServed,
        Refusal::ToolName,
        Refusal::HostSchema,
        Refusal::NotObject,
        Refusal::DeliverContract,
        Refusal::FailedWrite,
        Refusal::Feedback,
        Refusal::ResultReplay,
        Refusal::ReadLine,
        Refusal::Guard,
        Refusal::Observation,
        Refusal::FixedSchema,
        Refusal::CompletionResult,
        Refusal::UnknownEndpoint,
        Refusal::Owner,
        Refusal::Grant,
    ];
    let skein = [
        Invalid::Model.name(),
        Invalid::NoMessages.name(),
        Invalid::EmptyMessage.name(),
        Invalid::Output.name(),
        Invalid::ToolSchema.name(),
        Invalid::Identifier.name(),
        Invalid::Role.name(),
        Invalid::Outcome.name(),
        Invalid::Choice.name(),
        Invalid::Text.name(),
        Invalid::Arguments.name(),
        Invalid::Replay.name(),
        Invalid::Reasoning.name(),
        Invalid::Credential.name(),
        Invalid::Endpoint.name(),
        Invalid::Header.name(),
        Invalid::Head.name(),
        Invalid::Limits.name(),
        Invalid::Document.name(),
        Unsupported::ReplayProvider.name(),
        Unsupported::ReplayBlock.name(),
        Unsupported::ReplayForm.name(),
        Unsupported::OutputCap.name(),
        Unsupported::Effort.name(),
    ];
    let design = include_str!("../../../../docs/design/protocol/llm.md");
    let table = design
        .split("| Name | Refused when |")
        .nth(1)
        .expect("refusal table")
        .split("Where it meets")
        .next()
        .expect("table end");
    let names: Vec<&[u8]> = table
        .lines()
        .filter_map(|line| line.trim().strip_prefix("| `"))
        .map(|line| line.split('`').next().expect("table name").as_bytes())
        .collect();
    assert_eq!(causes.len(), names.len());
    for (index, cause) in causes.into_iter().enumerate() {
        let name = cause.name();
        assert_eq!(Some(&name), names.get(index), "design table row {index}");
        assert_ne!(name, b"");
        assert!(name.len() <= skein_lib::usize_of(skein_llm::CAUSE_BYTES));
        assert!(name.iter().all(|byte| byte.is_ascii_lowercase() || *byte == b'-'));
        assert!(!skein.contains(&name));
        assert!(!names.get(..index).expect("preceding rows").contains(&name));
    }
}

fn assert_refusal(error: Error, name: &[u8]) {
    let expected = bytes::concat(&[b"refused before sending: ", name]);
    for maximum in [0, 5, 24, 128] {
        let event = crate::refusal(Token::new(9), error, maximum);
        let Event::Below(Below::Failed { owner, failure, evidence, report, detail }) = event else {
            unreachable!("refusal is a failed terminal");
        };
        assert_eq!(owner, Token::new(9));
        assert_eq!(failure, llm::Failure::Invalid);
        assert_eq!(evidence, llm::Evidence::Unsent);
        assert_eq!(report, llm::Report { status: None, code: name.into(), request: Box::new([]), field: Box::new([]) });
        assert_eq!(detail.as_ref(), expected.get(..expected.len().min(skein_lib::usize_of(maximum))).expect("cap"));
    }
}

#[test]
fn preparation_causes_keep_their_class_code_and_clipped_detail() {
    for (error, name) in [
        (
            Error::from_client(skein_llm::Error::Invalid { cause: skein_llm::Invalid::Identifier }),
            b"identifier".as_slice(),
        ),
        (Error::from_client(skein_llm::Error::Unsupported { cause: skein_llm::Unsupported::Effort }), b"effort"),
        (Error::Unsupported { cause: Cause::Smith(Refusal::ResultReplay) }, b"result-replay"),
    ] {
        assert_refusal(error, name);
    }
}

fn refused(input: llm::Prompt, expected: Refusal) {
    let limits = direct_limits();
    let receiving = crate::fixture::receiving(&limits.client, 4096).expect("receiving");
    let error = crate::prepare_component(Token::new(9), input, crate::fixture::contract(), None, receiving, &limits)
        .expect_err("prompt cannot be built");
    assert_eq!(error, Error::Invalid { cause: Cause::Smith(expected) });
    assert_refusal(error, expected.name());
}

#[test]
fn prompt_refusals_name_the_rule_that_prevents_building() {
    let mut input = direct_prompt();
    input.choice = llm::ToolChoice::Only(Box::new([llm::Served::Wait]));
    refused(input, Refusal::ChoiceNotServed);

    let mut input = direct_prompt();
    input.served = Box::new([host_tool(b"{}"), host_tool(b"{}")]);
    refused(input, Refusal::ToolName);

    let mut input = direct_prompt();
    input.served = Box::new([host_tool(br#"{"type":"string"}"#)]);
    refused(input, Refusal::HostSchema);

    let mut input = direct_prompt();
    input.served = Box::new([host_tool(b"[]")]);
    refused(input, Refusal::NotObject);

    let mut input = direct_prompt();
    input.served = Box::new([llm::Served::Deliver]);
    refused(input, Refusal::DeliverContract);

    let mut input = direct_prompt();
    input.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"call".as_slice().into(),
            result: llm::Returned::NotRun { why: llm::NotRun::AfterFailedWrite { call: 1 } },
        }]),
    }]);
    refused(input, Refusal::FailedWrite);

    let mut input = direct_prompt();
    input.messages = Box::new([llm::Message {
        role: llm::Role::User,
        content: Box::new([llm::Block::ToolResult {
            id: b"call".as_slice().into(),
            result: llm::Returned::Served {
                returned: smith_domain::run::Returned::Waited { text: Box::new([]) },
                error: true,
            },
        }]),
    }]);
    refused(input, Refusal::Feedback);
}

#[test]
fn guard_render_and_completion_refusals_name_their_causes() {
    let limits = direct_limits();
    let endpoints = crate::Endpoints::new(
        Box::new([crate::ConfiguredEndpoint {
            name: llm::Endpoint(0),
            connect: skein_lib::Duration::from_secs(1),
            handshake: skein_lib::Duration::from_secs(1),
            destination: skein_llm_connection::Endpoint {
                limits: limits.client,
                credential: skein_llm::client::CredentialLimits { value: 128 },
                address: core::net::SocketAddr::from(([127, 0, 0, 1], 443)),
                transport: skein_llm_connection::Transport::Plaintext,
                llm: skein_llm::Endpoint::openai(),
            },
            oversized_reasoning: crate::OversizedReasoning::Fail,
        }]),
        1,
    )
    .expect("one endpoint");
    let options = endpoints.resolve(llm::Endpoint(0)).expect("endpoint");
    let mut native = native_text_prompt(Box::new([native_text(skein_llm::Role::User, b"go")]));
    let error = crate::guard::measure(&mut native, None, options, 2).expect_err("guard past messages");
    assert_eq!(error, Error::Invalid { cause: Cause::Smith(Refusal::Guard) });
    assert_refusal(error, b"guard");

    let read = tools::Outcome::Read {
        content: Box::new([]),
        first: 0,
        lines: 0,
        total: 0,
        cut: tools::Cut::Whole,
        next: tools::Next::End,
        window: tools::BytesClamp::Within,
    };
    let error = crate::render_outcome(&read, &tools::Namespace::One, limits.rendered_result).expect_err("line zero");
    assert_eq!(error, Error::Invalid { cause: Cause::Smith(Refusal::ReadLine) });
    assert_refusal(error, b"read-line");

    let receiving = crate::fixture::receiving(&limits.client, 4096).expect("receiving");
    let (_, context) =
        crate::prepare_component(Token::new(9), direct_prompt(), crate::fixture::contract(), None, receiving, &limits)
            .expect("context");
    let error = crate::completion(
        context,
        skein_llm::Completion {
            content: Box::new([skein_llm::Block::ToolResult {
                id: b"call".as_slice().into(),
                text: Box::new([]),
                is_error: false,
            }]),
            stop: skein_llm::Stop::EndTurn,
            usage: skein_llm::Usage::NONE,
        },
    )
    .expect_err("completion cannot hold a result");
    assert_eq!(error, Error::Invalid { cause: Cause::Smith(Refusal::CompletionResult) });
    assert_refusal(error, b"completion-result");
}

#[test]
fn smith_checks_of_skein_rules_keep_skeins_cause_names() {
    let limits = direct_limits();
    let mut client = limits.client;
    client.declared_output_tokens = 0;
    let error = crate::prompt_allowance(skein_llm::Provider::Anthropic, &client, 0).expect_err("zero output");
    assert_eq!(error, Error::Invalid { cause: Cause::Invalid(skein_llm::Invalid::Output) });
    assert_refusal(error, b"output");

    let change = smith_domain::run::outcome::ChangeSpec {
        checks_must_pass: false,
        fields: Box::new([smith_domain::run::outcome::FieldRule { name: Box::new([0xff]), max: 16 }]),
    };
    let error = crate::deliver_schema(&change, limits.client.request).expect_err("JSON cannot spell field name");
    assert_eq!(error, Error::Invalid { cause: Cause::Invalid(skein_llm::Invalid::Text) });
    assert_refusal(error, b"text");
}

#[test]
fn local_limits_keep_their_class_bound_and_detail() {
    for (error, expected, text) in [
        (Error::Limit, llm::Failure::ApplicationLimit, b"skein Client preparation exceeds receiving limits".as_slice()),
        (
            Error::PathLimit { which: llm::Cap::Request, bound: 71 },
            llm::Failure::Limit { which: llm::Limit::Path(llm::Cap::Request), bound: 71 },
            b"native Client preparation exceeds its named path bound",
        ),
    ] {
        let Event::Below(Below::Failed { failure, evidence, detail, report, .. }) =
            crate::refusal(Token::new(9), error, 128)
        else {
            unreachable!("refusal terminal");
        };
        assert_eq!(failure, expected);
        assert_eq!(evidence, llm::Evidence::Unsent);
        assert_eq!(detail.as_ref(), text);
        assert_eq!(report, llm::Report::none());
    }
}
