//! Observed provider calls refuse nesting through main and child sessions.
//! Contract: protocol/llm.md, section 4; domain/run.md, sections 5.3 and 13.

use skein_fake_llm_domain::api::{Line, Part};
use smith_domain::{run, session::llm};
use smith_protocol_llm_world::{Job, Settings, World, client_peer, codec, script};

#[test]
fn nesting_asked_for_is_answered_as_its_problem_on_observed_provider_requests() {
    let mut crossed_old_document_cap = false;
    for configuration in client_peer::configurations() {
        let error_flag = configuration.error_flag;
        let mut scripts = script::all().into_vec();
        let main = scripts.iter_mut().find(|script| script.cue.as_ref() == b"@delegate").expect("main script");
        main.turns.last_mut().expect("main finish").lines = Box::new([Line::Call {
            name: b"finish".as_slice().into(),
            arguments:
                br#"{"form":"change","fields":{"title":"Make the answer 43","body":"A sub-agent made the answer 43."}}"#
                    .as_slice()
                    .into(),
        }]);
        let mut turns = vec![smith_protocol_llm_world::fixture::calls(
            Box::new([Line::Call {
                name: b"sub_agent".as_slice().into(),
                arguments:
                    br#"{"brief":"@explore Never opened.","tools":["inspect"],"agents":false}"#.as_slice().into(),
            }]),
            4,
        )];
        turns.extend(std::mem::take(&mut main.turns).into_vec());
        main.turns = turns.into_boxed_slice();
        let mut bounds =
            smith_protocol_llm::fixture::small(skein_llm_world::limits(), smith_protocol_llm_world::codec::stated());
        bounds.client.http.request = 16384;
        bounds.client.request = 16384;
        // The byte peer decodes the same complete native documents the client renders.
        let mut settings = Settings { job: Job::Delegating, ..smith_protocol_llm_world::fixture::settings(704) };
        settings.limits.session.completion_bytes =
            codec::completion_bound(&bounds.client, settings.limits.decoded_call_bytes).expect("receiving bound");
        settings.limits.session.completion_blocks = bounds.client.output_items;
        settings.limits.session.failure_bytes = bounds.client.detail_bytes;
        let mut world = World::with_protocol(settings, None, configuration, bounds, scripts.into_boxed_slice());
        world.run(100_000);
        assert!(
            matches!(world.answer(), run::Answer::Accepted { .. }),
            "observed native nesting terminal: {:?}; request bytes: {:?}",
            world.answer(),
            world.protocol_calls().iter().map(|binding| binding.request.len()).collect::<Vec<_>>()
        );
        crossed_old_document_cap |= check_document_bound(&world, &bounds, error_flag);
        let note = b"a sub-agent does its work itself";
        let feedback = world
            .prompts()
            .iter()
            .flat_map(|query| &query.messages)
            .flat_map(|message| &message.parts)
            .filter(|part| {
                matches!(part, Part::ToolOutput { output, is_error, .. }
                if *is_error == error_flag && skein_lib::bytes::find(output, note).is_some())
            })
            .count();
        assert!(feedback >= 2, "main's agents field and child's unoffered call each receive the nesting explanation");
        assert_eq!(
            world.facts().iter().filter(|fact| matches!(fact, smith_domain::Fact::Run { fact, opening: _ } if matches!(fact.kind, run::facts::FactKind::Opened { .. }))).count(),
            4,
            "only main and the three legitimate children open"
        );
        assert!(
            world.turns().iter().flat_map(|turn| &turn.messages).flat_map(|message| &message.content).any(
                |block| matches!(
                    block,
                    llm::Block::ToolResult { result: llm::Returned::Invalid { problem: llm::Problem::Nesting }, .. }
                )
            ),
            "observed persisted main result retains the typed problem"
        );
        let child = world.prompts().iter().filter(|query| skein_lib::bytes::find(&query.system, b"@fix").is_some());
        assert!(child.clone().count() > 0);
        assert!(child.clone().all(|query| !query.tools.iter().any(|tool| tool.name.as_ref() == b"sub_agent")));
        assert!(
            child
                .flat_map(|query| &query.messages)
                .flat_map(|message| &message.parts)
                .any(|part| matches!(part, Part::ToolOutput { output, is_error, .. }
                if *is_error == error_flag && skein_lib::bytes::find(output, note).is_some()))
        );
    }
    assert!(crossed_old_document_cap, "a real native request witnesses the former peer-document refusal");
}

// Positive-first native document control: the larger application schema makes
// the real final request exceed the former 8 KiB peer-document cap.
fn check_document_bound(world: &World, bounds: &codec::Limits, error_flag: bool) -> bool {
    use skein_fake_llm_protocol::documents::{self, Provider};

    let tape = skein_llm_world::identity(
        &world.protocol_calls().iter().max_by_key(|binding| binding.request.len()).expect("observed requests").request,
    );
    let head = tape.windows(4).position(|part| part == b"\r\n\r\n").expect("HTTP head");
    let body = &tape[head + 4..];
    let dialect = if error_flag { Provider::Anthropic } else { Provider::OpenAi };
    let peer = skein_llm_world::fake::limits(&bounds.client);
    assert!(
        documents::request(dialect, body, &peer.documents).is_ok(),
        "the observed complete request fits its configured document bound"
    );
    let mut small = peer.documents;
    small.anthropic.request = 8192;
    small.openai.request = 8192;
    if body.len() <= 8192 {
        return false;
    }
    assert!(
        documents::request(dialect, body, &small).is_err(),
        "lowering only the document cap rejects the same captured request"
    );
    true
}
