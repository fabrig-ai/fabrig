//! A request deadline and the host watchdog meet the same silent provider.
//! Contract: temper/docs/design/draft/failure-worlds.md, section 7 (P71).
use skein_fake_llm_domain::api::{Line, Piece, Script};
use skein_lib::Duration;
use smith_domain::{llm, run};
use smith_host_domain::{self as host, parent};
use smith_protocol_llm_world::{Job, World as Protocol, codec, fixture};

fn held_request(configuration: fixture::Configuration) -> Protocol {
    let mut settings = fixture::settings(71);
    settings.job = Job::Reporting;
    let mut limits = fixture::limits();
    limits.client.request = 16_384;
    limits.client.http.request = 16_384;
    settings.limits.session.completion_bytes =
        codec::completion_bound(&limits.client, settings.limits.decoded_call_bytes).expect("provider receiving bounds");
    settings.limits.session.completion_blocks = limits.client.output_items;
    settings.limits.session.failure_bytes = limits.client.detail_bytes;
    Protocol::with_protocol(
        settings,
        None,
        configuration,
        limits,
        Box::new([Script {
            cue: b"@report".as_slice().into(),
            turns: Box::new([fixture::says(
                Box::new([Line::Pieces {
                    pieces: Box::new([Piece {
                        text: b"never released".as_slice().into(),
                        hold: Some(b"silent".as_slice().into()),
                    }]),
                }]),
                1,
            )]),
        }]),
    )
}

fn watchdog() -> smith_host_world::World {
    let profile = smith_agent_process_world::profile();
    let mut configuration = smith_agent_process_world::configuration();
    for endpoint in &mut configuration.domain.limits {
        // The protocol world's endpoint, not the process fixture's tiny model.
        endpoint.head = Duration::from_secs(60);
        endpoint.idle = Duration::from_secs(30);
    }
    let limits = smith_agent_profile::host_limits(&profile, &configuration.limits, &configuration.domain.limits)
        .expect("derived host watchdog");
    let mut world = smith_host_world::World::new(71, limits);
    world.live();
    world
}

fn forward_longs(protocol: &Protocol, host: &mut smith_host_world::World, forwarded: &mut usize) {
    for (at, operation, until) in protocol.long_notices().iter().skip(*forwarded) {
        host.at_time(*at);
        let message = match operation {
            Some(operation) => host::Up::Long {
                operation: match operation {
                    smith_domain::Operation::Completion => host::Operation::Completion,
                    smith_domain::Operation::Command => host::Operation::Command,
                    smith_domain::Operation::Checks => host::Operation::Checks,
                    smith_domain::Operation::Grant => host::Operation::Grant,
                },
                span: until.expect("Long deadline").saturating_since(*at),
            },
            None => host::Up::LongDone,
        };
        host.up(message);
        *forwarded += 1;
    }
    host.at_time(protocol.now());
}

#[test]
fn a_silent_run_is_cut_at_its_derived_bound() {
    for configuration in fixture::configurations() {
        let mut protocol = held_request(configuration);
        let mut host = watchdog();
        let mut forwarded = 0;
        let mut settled = false;
        for _ in 0..100_000 {
            settled = protocol.drive(1);
            forward_longs(&protocol, &mut host, &mut forwarded);
            if settled || host.seen.fault.is_some() {
                break;
            }
        }
        assert_eq!(host.seen.fault, None, "request deadline, not the watchdog, ends the silence");
        assert!(settled, "the never-released request reaches a bounded terminal");
        let calls = protocol.protocol_calls();
        assert!(calls.len() > 1, "the session retries an idle timeout");
        for call in calls {
            assert_eq!(call.failure, Some(llm::Failure::TimedOut { phase: llm::CompletionPhase::Idle }));
            assert_eq!(
                call.evidence,
                Some(llm::Evidence::Response { status: Some(200) }),
                "the head arrived before the hold"
            );
            let end = call.terminal.expect("each request has its own terminal").0;
            assert_eq!(end.saturating_since(call.received_at.expect("request reached peer")), Duration::from_secs(30));
        }
        for (previous, next) in calls.iter().zip(calls.iter().skip(1)) {
            assert!(next.started.0 > previous.terminal.expect("previous request ended").0, "nonzero retry backoff");
        }
        assert!(
            matches!(protocol.answer(), run::Answer::Failed {
            failure: run::RunFailure::Model(run::Fault::Completion {
                failure: run::CompletionFailure::TimedOut { phase: run::CompletionPhase::Idle }, attempts, ..
            }), ..
        } if usize::try_from(*attempts).expect("attempt count") == calls.len()),
            "{:?}",
            protocol.answer()
        );
        assert!(protocol.turns().is_empty(), "the held text never enters history");
        assert!(forwarded > 0, "the watchdog heard the run's real long-operation notices");
        host.event(host::Input::Parent(parent::Event::Drop { agent: host.agent() }));
        host.cleanup(Some(host::Exit::Code(0)));
        host.settled();
    }
}
