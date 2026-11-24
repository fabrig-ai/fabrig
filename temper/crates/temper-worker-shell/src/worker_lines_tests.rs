use super::{ending_with_binary as worker_ending_words, note_line as worker_note_words};
use temper_worker_service::{Refusal, WorkerJobEnding, WorkerLoss, worker_alone as worker};

const BINARY: &str = "temper-worker";

#[test]
fn unswept_lines_escape_the_path_and_name_the_error() {
    let note = worker::Note::Unswept {
        path: b"trash/4/target\n\xff".as_slice().into(),
        error: skein_io::kernel::Error::Permission,
    };
    assert_eq!(
        worker_note_words("temper", note),
        "temper worker: trash entry not removed: trash/4/target\\n\\xff: Permission\n"
    );
}

/// domain/worker/hosted-runs.md, 1.3: sources, destinations and names stay on one line.
#[test]
fn git_fetch_words_name_each_source_and_destination() {
    use temper_worker_domain::{
        Git,
        checkout::git::{Commit, Into, Want},
    };
    let mut sha1 = [0; 32];
    sha1[..20].fill(0xab);
    for (repository, want, into, expected) in [
        (
            b"app".as_slice(),
            Want::Branch { branch: b"main".as_slice().into() },
            Into::Fetched,
            "temper worker: run 81 attempt 2 fetch app branch main into fetched\n",
        ),
        (
            b"app".as_slice(),
            Want::Commit { commit: Commit::new(sha1) },
            Into::Fetched,
            "temper worker: run 81 attempt 2 fetch app commit abababababababababababababababababababab into fetched\n",
        ),
        (
            b"app".as_slice(),
            Want::Default,
            Into::Landing,
            "temper worker: run 81 attempt 2 fetch app default into landing\n",
        ),
        (
            b"app\n\x01\xff".as_slice(),
            Want::Branch { branch: b"main\n\x00\xc3\xa9".as_slice().into() },
            Into::Fetched,
            "temper worker: run 81 attempt 2 fetch app\\n\\x01\\xff branch main\\n\\x00\\xc3\\xa9 into fetched\n",
        ),
    ] {
        let note = worker::Note::Git {
            run: 81,
            attempt: 2,
            job: None,
            repository: repository.into(),
            what: Git::Fetch { want, into },
        };
        assert_eq!(worker_note_words("temper", note), expected);
    }
}

/// protocol/git.md, section 3: SHA-1 padding is omitted, SHA-256 is not shortened.
#[test]
fn git_check_out_words_name_the_job_and_full_commit() {
    use temper_worker_domain::{Git, checkout::git::Commit};
    let mut sha1 = [0; 32];
    sha1[..20].fill(0xcd);
    for (raw, verb, expected) in [
        (
            sha1,
            b"prepare".as_slice(),
            "temper-worker worker: run 9551 attempt 3 job 1 prepare check out app\\n\\x01\\xff cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd\n",
        ),
        (
            [0xcd; 32],
            b"prepare\n\x00\xff".as_slice(),
            concat!(
                "temper-worker worker: run 9551 attempt 3 job 1 prepare\\n\\x00\\xff check out app\\n\\x01\\xff ",
                "cdcdcdcdcdcdcdcd",
                "cdcdcdcdcdcdcdcd",
                "cdcdcdcdcdcdcdcd",
                "cdcdcdcdcdcdcdcd",
                "\n"
            ),
        ),
    ] {
        let note = worker::Note::Git {
            run: 9551,
            attempt: 3,
            job: Some((1, verb.into())),
            repository: b"app\n\x01\xff".as_slice().into(),
            what: Git::CheckOut { commit: Commit::new(raw) },
        };
        assert_eq!(worker_note_words(BINARY, note), expected);
    }
}

#[test]
fn a_cancel_names_the_run_attempt_and_engine() {
    assert_eq!(
        worker_note_words(BINARY, worker::Note::Cancelling { run: 7, attempt: 2 }),
        "temper-worker worker: run 7 attempt 2 cancelled by the engine\n"
    );
}

#[test]
fn opening_refusals_name_the_reason_and_the_engines_text() {
    for (reason, text, expected) in [
        (
            257,
            "wire smith-charter: the worker's text is not its layout cd34000000000000",
            "temper-worker worker: the engine refused the opening: wires: wire smith-charter: the worker's text is not its layout cd34000000000000\n",
        ),
        (
            1,
            "speaks 10..10, offered 9..9",
            "temper-worker worker: the engine refused the opening: version: speaks 10..10, offered 9..9\n",
        ),
    ] {
        let note = worker::Note::OpeningRefused { reason, text: text.as_bytes().into() };
        assert_eq!(worker_note_words(BINARY, note), expected);
    }
}
fn directories() -> temper_shell::directories::Directories {
    let arguments = ["--config", "/c", "--data", "/d", "--cache", "/k"].map(|word| Box::from(word.as_bytes()));
    temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories").0
}
#[test]
fn worker_job_words_keep_the_binary_prefix_and_reason() {
    let ending = WorkerJobEnding::Failed {
        failure: jig_domain_host::JobFailure::Deadline,
        detail: b"details\n".as_slice().into(),
    };
    let worker = worker::Note::WorkerJob {
        run: 7,
        attempt: 2,
        kind: 3,
        verb: b"merge".as_slice().into(),
        ending,
        why: b"deadline".as_slice().into(),
    };
    assert_eq!(
        worker_note_words(BINARY, worker),
        "temper-worker worker: run 7 attempt 2 job 3 merge failed Deadline: deadline\n"
    );
}
/// temper/docs/design/domain/worker/checkouts.md, 1.3: one escaped operator line.
#[test]
fn unprepared_words_name_the_run_failure_and_escape_the_hosted_detail() {
    for (detail, expected) in [
        (
            b"host w1: check out: TimedOut (workspace 3, repository 0 fabrig, command checkout, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms)".as_slice(),
            "temper worker: run 5 attempt 1 failed Unprepared(Transient): host w1: check out: TimedOut (workspace 3, repository 0 fabrig, command checkout, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms)\n",
        ),
        (
            b"host w1: prepare refused: Busy\n\0\"\xff".as_slice(),
            "temper worker: run 5 attempt 1 failed Unprepared(Transient): host w1: prepare refused: Busy\\n\\x00\\\"\\xff\n",
        ),
    ] {
        let note = worker::Note::Unprepared {
            run: 5, attempt: 1,
            failure: jig_domain_host::Failure::Unprepared(jig_domain_host::Preparation::Transient), detail: detail.into(),
        };
        assert_eq!(worker_note_words("temper", note), expected);
    }
    let note = worker::Note::Unprepared {
        run: 6,
        attempt: 2,
        failure: jig_domain_host::Failure::Unprepared(jig_domain_host::Preparation::Permanent { resource: None }),
        detail: b"host w1: prepare aborted".as_slice().into(),
    };
    assert_eq!(
        worker_note_words(BINARY, note),
        "temper-worker worker: run 6 attempt 2 failed Unprepared(Permanent { resource: None }): host w1: prepare aborted\n"
    );
}

/// temper/docs/design/domain/worker/hosted-runs.md, 1.3: agent faults use serve's failure form.
#[test]
fn agent_stopped_words_include_only_the_no_progress_bound() {
    use jig_domain_host::AgentFailure;
    for (failure, no_progress_ms, expected) in [
        (
            AgentFailure::NoProgress,
            Some(1_205_000),
            "temper-worker worker: run 7 attempt 9 agent stopped Agent(NoProgress): no progress for 1205000 ms\n",
        ),
        (AgentFailure::WallTime, None, "temper-worker worker: run 7 attempt 9 agent stopped Agent(WallTime)\n"),
    ] {
        let note = worker::Note::AgentStopped { run: 7, attempt: 9, failure, no_progress_ms };
        assert_eq!(worker_note_words(BINARY, note), expected);
    }
}

#[test]
fn stop_bound_words_count_the_abandoned_turns_and_answers() {
    assert_eq!(
        worker_note_words("temper", worker::Note::StopBoundPassed { abandoned: 4 }),
        "temper worker: stop bound passed; abandoned 4 turns and answers\n"
    );
}
#[test]
fn worker_linked_words_name_the_worker_and_address() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let worker = worker::Note::Linked { worker: b"worker-a".as_slice().into(), address };
    assert_eq!(worker_note_words(BINARY, worker), "linked worker-a 127.0.0.1:7100\n");
}
#[test]
fn worker_lost_words_name_the_worker_address_and_reason() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let why = WorkerLoss::Silence;
    let worker = worker::Note::Lost { worker: b"worker-a".as_slice().into(), address, why };
    assert_eq!(worker_note_words(BINARY, worker), "lost worker-a 127.0.0.1:7100 silence\n");
}
#[test]
fn worker_refused_words_name_the_peer_refusal() {
    let address: Box<core::net::SocketAddr> = Box::new(([127, 0, 0, 1], 7100).into());
    let reason = Refusal::Peer(256);
    let worker = worker::Note::Refused { worker: b"worker-a".as_slice().into(), address, reason };
    assert_eq!(worker_note_words(BINARY, worker), "refused worker-a 127.0.0.1:7100 unauthorized\n");
}
#[test]
fn worker_start_problem_words_and_exit_keep_their_behavior() {
    let mut problems = jig_protocol_config::Problems::empty();
    problems.push(jig_protocol_config::Path::root().member(b"git"), jig_protocol_config::What::Form);
    problems.push(jig_protocol_config::Path::root().member(b"limits"), jig_protocol_config::What::Bound);
    let worker = worker::End::Stopped(worker::Stop::Start(problems));
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker), false, &directories, b"");
    assert_eq!(
        actual,
        ("temper-worker serve: git: invalid form\ntemper-worker serve: limits: past its bound\n".into(), 1)
    );
}
#[test]
fn worker_drained_words_and_exit_keep_their_behavior() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Drained), false, &directories, b"");
    assert_eq!(actual, (String::new(), 0));
}
#[test]
fn worker_output_failure_words_and_exit_keep_their_behavior() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Drained), true, &directories, b"");
    assert_eq!(actual, (String::new(), 1));
}
#[test]
fn worker_aborted_words_and_exit_keep_their_behavior() {
    let directories = directories();
    let actual = worker_ending_words(BINARY, Some(&worker::End::Aborted), false, &directories, b"");
    assert_eq!(actual, (String::new(), 1));
}

#[test]
fn worker_figures_notes_print_each_part_on_one_bounded_line() {
    use jig_protocol_worker_wire::Limit;
    use temper_worker_service::worker_alone::Note;
    let figure = |name: &str, value| Limit { name: name.as_bytes().into(), value };
    assert_eq!(
        worker_note_words(
            BINARY,
            Note::FiguresTaken {
                figures: Box::new([
                    figure("checkout.remote_timeout_ms", 600_000),
                    figure("checkout.local_timeout_ms", 20_000)
                ]),
            },
        ),
        "the engine's figures changed; taken live: checkout.remote_timeout_ms 600000, checkout.local_timeout_ms 20000\n"
    );
    assert_eq!(
        worker_note_words(
            BINARY,
            Note::FiguresWaiting {
                figures: Box::new([
                    figure("hub.turn_bytes", 2_097_152),
                    figure("places", 5),
                    figure("agents.profile.declared.tool_payload", 63_000),
                    figure("link.ping_ms", 20_000),
                    figure("link.silence_ms", 60_000),
                    figure("io.output", 1024),
                    figure("checkout.repositories", 2),
                    figure("checkout.workspaces", 5),
                    figure("git_component.operations", 5),
                    figure("hub.inbox", 2),
                ]),
            },
        ),
        concat!(
            "the engine's figures changed; at this worker's next start: hub.turn_bytes 2097152, places 5, ",
            "agents.profile.declared.tool_payload 63000, link.ping_ms 20000, link.silence_ms 60000, ",
            "io.output 1024, checkout.repositories 2, checkout.workspaces 5, and 2 more\n",
        )
    );
    assert_eq!(
        worker_note_words(
            BINARY,
            Note::FiguresRefused { problem: b"the engine's figures: git_component.stdout_bytes 1024: 1024 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 70000 bytes".as_slice().into() },
        ),
        concat!(
            "the engine's figures changed; not taken live: the engine's figures: git_component.stdout_bytes 1024: ",
            "1024 bytes, short of checkout.conflicts * (checkout.path_bytes + 1), 70000 bytes; ",
            "they apply at this worker's next start\n",
        )
    );
}

#[test]
fn worker_link_notes_print_each_address_and_refusal_exactly() {
    use temper_worker_service::{Refusal, worker_alone::Note};
    for (address, printed) in
        [(([127, 0, 0, 1], 7100).into(), "127.0.0.1:7100"), (([0, 0, 0, 0, 0, 0, 0, 1], 7100).into(), "[::1]:7100")]
    {
        assert_eq!(
            worker_note_words(
                "temper",
                Note::Linked { worker: b"worker-a".as_slice().into(), address: Box::new(address) }
            ),
            format!("linked worker-a {printed}\n")
        );
        for (reason, expected) in [
            (Refusal::Peer(256), "unauthorized"),
            (Refusal::Peer(257), "wires"),
            (Refusal::Peer(258), "busy"),
            (Refusal::Peer(259), "replaced"),
            (Refusal::Peer(260), "not-admitted"),
            (Refusal::Peer(261), "stopping"),
            (Refusal::Terms, "terms"),
            (Refusal::Malformed, "malformed"),
            (Refusal::Peer(42), "code-42"),
        ] {
            assert_eq!(
                worker_note_words(
                    "temper",
                    Note::Refused { worker: b"worker-a".as_slice().into(), address: Box::new(address), reason }
                ),
                format!("refused worker-a {printed} {expected}\n")
            );
        }
    }
}

#[test]
fn worker_loss_lines_name_each_cause_and_refusal_exactly() {
    use temper_worker_service::{WorkerClosed as Closed, WorkerLoss as Loss, worker_alone::Note};
    for (why, expected) in [
        (Loss::Closed(Closed::Stream), "lost worker-a [::1]:7100 stream\n"),
        (Loss::Closed(Closed::Truncated), "lost worker-a [::1]:7100 truncated\n"),
        (Loss::Closed(Closed::Framing), "lost worker-a [::1]:7100 framing\n"),
        (Loss::Closed(Closed::Owner), "lost worker-a [::1]:7100 owner\n"),
        (Loss::Silence, "lost worker-a [::1]:7100 silence\n"),
        (Loss::Stall, "lost worker-a [::1]:7100 stall\n"),
        (Loss::Handshake, "lost worker-a [::1]:7100 handshake\n"),
        (Loss::Full, "lost worker-a [::1]:7100 full\n"),
        (Loss::Busy, "lost worker-a [::1]:7100 busy\n"),
        (Loss::Fault, "lost worker-a [::1]:7100 fault\n"),
        (Loss::Unreachable, "lost worker-a [::1]:7100 unreachable\n"),
        (Loss::Closed(Closed::RefusedHere(2)), "lost worker-a [::1]:7100 refused terms\n"),
        (Loss::Closed(Closed::RefusedHere(3)), "lost worker-a [::1]:7100 refused malformed\n"),
        (Loss::Closed(Closed::RefusedHere(256)), "lost worker-a [::1]:7100 refused unauthorized\n"),
        (Loss::Closed(Closed::RefusedHere(257)), "lost worker-a [::1]:7100 refused wires\n"),
        (Loss::Closed(Closed::RefusedHere(258)), "lost worker-a [::1]:7100 refused busy\n"),
        (Loss::Closed(Closed::RefusedHere(259)), "lost worker-a [::1]:7100 refused replaced\n"),
        (Loss::Closed(Closed::RefusedHere(260)), "lost worker-a [::1]:7100 refused not-admitted\n"),
        (Loss::Closed(Closed::RefusedHere(261)), "lost worker-a [::1]:7100 refused stopping\n"),
        (Loss::Closed(Closed::RefusedHere(42)), "lost worker-a [::1]:7100 refused code-42\n"),
        (Loss::Closed(Closed::RefusedPeer(2)), "lost worker-a [::1]:7100 refused-by-peer terms\n"),
        (Loss::Closed(Closed::RefusedPeer(3)), "lost worker-a [::1]:7100 refused-by-peer malformed\n"),
        (Loss::Closed(Closed::RefusedPeer(256)), "lost worker-a [::1]:7100 refused-by-peer unauthorized\n"),
        (Loss::Closed(Closed::RefusedPeer(257)), "lost worker-a [::1]:7100 refused-by-peer wires\n"),
        (Loss::Closed(Closed::RefusedPeer(258)), "lost worker-a [::1]:7100 refused-by-peer busy\n"),
        (Loss::Closed(Closed::RefusedPeer(259)), "lost worker-a [::1]:7100 refused-by-peer replaced\n"),
        (Loss::Closed(Closed::RefusedPeer(260)), "lost worker-a [::1]:7100 refused-by-peer not-admitted\n"),
        (Loss::Closed(Closed::RefusedPeer(261)), "lost worker-a [::1]:7100 refused-by-peer stopping\n"),
        (Loss::Closed(Closed::RefusedPeer(42)), "lost worker-a [::1]:7100 refused-by-peer code-42\n"),
    ] {
        assert_eq!(
            worker_note_words(
                "temper",
                Note::Lost {
                    worker: b"worker-a".as_slice().into(),
                    address: Box::new(([0, 0, 0, 0, 0, 0, 0, 1], 7100).into()),
                    why,
                }
            ),
            expected,
            "{why:?}"
        );
    }
}

#[test]
fn job_stderr_forms_escape_and_bound_their_why() {
    use jig_domain_host::{JobFailure, JobInvalid, JobRefusal, Preparation};
    const TAIL: usize = 4096;
    for (refused, prefix) in [
        (false, "temper worker: run 7 attempt 2 job 3 merge failed Unprepared(Transient): "),
        (true, "temper worker: run 7 attempt 2 job 3 merge refused Invalid(Start): "),
    ] {
        for (why, expected) in [
            (b"fetch base: Broken\n\xff".to_vec(), "fetch base: Broken\\n\\xff".to_owned()),
            (vec![b'x'; TAIL], "x".repeat(TAIL)),
            ([b"discard".as_slice(), vec![b'x'; TAIL].as_slice()].concat(), "x".repeat(TAIL)),
        ] {
            let ending = if refused {
                WorkerJobEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why: why.clone().into() }))
            } else {
                WorkerJobEnding::Failed {
                    failure: JobFailure::Unprepared(Preparation::Transient),
                    detail: why.clone().into(),
                }
            };
            let note = worker::Note::WorkerJob {
                run: 7,
                attempt: 2,
                kind: 3,
                verb: b"merge".as_slice().into(),
                ending,
                why: why.into(),
            };
            assert_eq!(worker_note_words("temper", note), format!("{prefix}{expected}\n"));
        }
    }
}

fn pause_line(state: jig_domain_link::PauseState) -> String {
    worker_note_words(BINARY, worker::Note::Paused { state, drain: skein_lib::Duration::from_secs(180) })
}

#[test]
fn pause_words_say_no_new_run_or_job_as_the_engine_asked() {
    assert_eq!(
        pause_line(jig_domain_link::PauseState::Paused),
        "temper-worker worker: paused: no new run or job, as the engine asked\n"
    );
}

#[test]
fn drain_words_name_the_stock_bound_from_the_engine_ask() {
    assert_eq!(
        pause_line(jig_domain_link::PauseState::Draining),
        "temper-worker worker: draining: each run stops at its turn's end, the rest cut in 180 s, as the engine asked\n"
    );
}

#[test]
fn drained_words_say_a_stop_now_loses_nothing() {
    assert_eq!(
        pause_line(jig_domain_link::PauseState::Drained),
        "temper-worker worker: drained: no run or job left; a stop now loses nothing\n"
    );
}

#[test]
fn resume_words_say_taking_work_again_as_the_engine_asked() {
    assert_eq!(
        pause_line(jig_domain_link::PauseState::Taking),
        "temper-worker worker: taking work again, as the engine asked\n"
    );
}

#[test]
fn restart_words_name_how_long_runs_are_kept_after_loss() {
    assert_eq!(
        worker_note_words("custom", worker::Note::Restarting { keep: skein_lib::Duration::from_millis(1_200_000) }),
        "custom worker: the engine said it restarts: runs kept up to 1200000 ms after the link drops\n"
    );
}

#[test]
fn an_agents_signal_line_leaves_its_stop_to_the_worker() {
    for (from, sender) in [(Some(42), "pid 42"), (None, "an unknown sender")] {
        assert_eq!(
            worker_note_words(BINARY, worker::Note::AgentSignalled { run: 7, attempt: 2, signal: "SIGTERM", from }),
            format!(
                "{BINARY} worker: run 7 attempt 2: its agent heard SIGTERM from {sender}; its stop is the worker's\n"
            )
        );
    }
}

#[test]
fn git_counts_words_name_the_repository() {
    let note = worker::Note::Git {
        run: 81,
        attempt: 2,
        job: None,
        repository: b"app\n".as_slice().into(),
        what: temper_worker_domain::Git::Counts,
    };
    assert_eq!(worker_note_words("temper", note), "temper worker: run 81 attempt 2 counts app\\n\n");
}

#[test]
fn startup_figures_line_names_engine_places_limits_and_endpoints() {
    use jig_protocol_worker_wire::{AgentEndpoint, Agents, EndpointLimits, Figures, Limit, Oversized, Transport};
    use skein_lib::{Duration, List};

    const PORT: u16 = 7100;
    const PLACES: u32 = 2;
    const LIMITS: [(&[u8], u64); 3] = [(b"hub.slots", 2), (b"hub.inbox", 8), (b"hub.turns", 4)];
    let engine = jig_protocol_config::Address::Addr(skein_io::Addr::Ip(([127, 0, 0, 1], PORT).into()));
    let mut limits = List::with_capacity(3);
    for (name, value) in LIMITS {
        limits.push(Limit { name: name.into(), value }).expect("one named limit");
    }
    let mut endpoints = List::with_capacity(1);
    endpoints
        .push(AgentEndpoint {
            name: b"model".as_slice().into(),
            number: 1,
            dialect: 3,
            provider: b"fixture".as_slice().into(),
            program: None,
            host: b"127.0.0.1".as_slice().into(),
            port: PORT,
            transport: Transport::Plaintext,
            headers: List::with_capacity(0),
            connect: Duration::from_secs(1),
            handshake: Duration::from_secs(1),
            limits: EndpointLimits {
                window: 1024,
                output: 512,
                reasoning_item: 2048,
                head: Duration::from_secs(1),
                idle: Duration::from_secs(1),
                oversized: Oversized::Fail,
            },
        })
        .expect("one endpoint");
    let figures = Figures { places: PLACES, limits, agents: Agents { profile: List::with_capacity(0), endpoints } };
    assert_eq!(
        crate::worker::figures_line(&engine, &figures),
        "figures taken from 127.0.0.1:7100: 2 places, 3 limits, 1 endpoints\n"
    );
}
