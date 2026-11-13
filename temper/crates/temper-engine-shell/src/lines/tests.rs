use super::*;

#[test]
fn accepted_worker_lines_name_each_agreed_wire_in_statement_order() {
    use jig_protocol_worker_wire::Agreed;
    let wires = skein_lib::List::from_boxed(Box::new([
        Agreed { name: "jig-worker-wire", version: 10 },
        Agreed { name: "smith-channel", version: 7 },
        Agreed { name: "smith-charter", version: 4 },
        Agreed { name: "smith-transcript", version: 7 },
        Agreed { name: "temper-worker-wire", version: 2 },
    ]));
    assert_eq!(note_line("custom", temper_engine_service::Note::WorkerOpened { worker: b"worker-a".as_slice().into(), wires }), "custom serve: worker worker-a opened: jig-worker-wire 10, smith-channel 7, smith-charter 4, smith-transcript 7, temper-worker-wire 2\n");
}

#[test]
fn opening_refusal_lines_name_the_worker_or_the_precredential_version() {
    use temper_engine_service::Note;
    assert_eq!(note_line("custom", Note::WorkerOpeningRefused {
        worker: Some(b"worker-a".as_slice().into()), reason: 257,
        text: b"wire smith-charter: the worker states none".as_slice().into(),
    }), "custom serve: worker worker-a refused at its opening: wires: wire smith-charter: the worker states none\n");
    assert_eq!(note_line("custom", Note::WorkerOpeningRefused {
        worker: None, reason: 1, text: b"speaks 10..10, offered 9..9".as_slice().into(),
    }), "custom serve: a worker refused at its opening: version: speaks 10..10, offered 9..9\n");
}

#[test]
fn github_start_refusals_name_their_paths() {
    use skein_forge_connection::{StartRead, Stopped};
    for (why, words) in [
        (Stopped::KeyRefused, "app_key: refused"),
        (Stopped::TokenRefused { installation: 7, status: 403 }, "/app/installations/7/access_tokens: status 403"),
        (Stopped::Refused { read: StartRead::Installations, status: 401 }, "/app/installations: status 401"),
        (Stopped::Refused { read: StartRead::Mint, status: 401 }, "/app/installations/{id}/access_tokens: status 401"),
        (Stopped::Refused { read: StartRead::Viewer, status: 401 }, "/graphql viewer: status 401"),
    ] {
        assert_eq!(stopped_line("temper", 1, why), format!("temper serve: forge 1: {words}\n"));
    }
}

#[test]
fn records_and_installation_notes_name_the_forge_and_reason() {
    use temper_engine_service::{NotConnected, Note};
    assert_eq!(
        note_line("temper", Note::ForgeNotConnected { number: 7, why: NotConnected::Unreadable }),
        "temper serve: forge 7: not connected: its records are unreadable\n"
    );
    assert_eq!(
        note_line(
            "temper",
            Note::ForgeNotConnected {
                number: 7,
                why: NotConnected::OtherForge {
                    kind: skein_forge_connection::settings::Kind::Forgejo,
                    api: Box::from("fake.test:443")
                }
            }
        ),
        "temper serve: forge 7: not connected: its records were made for forgejo at fake.test:443\n"
    );

    assert_eq!(
        note_line("temper", Note::ForgeUnkept { number: 7, installation: 93 }),
        "temper serve: forge 7: installation 93: not kept\n"
    );
}

#[test]
fn forge_sign_in_notes_name_each_unserved_reason() {
    use temper_engine_service::{Note, clients::forge_sign_in::Unserved};
    for (why, words) in [
        (Unserved::NotKept, "no such forge is kept"),
        (Unserved::NotForgejo, "it is not a Forgejo"),
        (
            Unserved::Origin { origin: b"https://git.example.com".as_slice().into() },
            "its browser origin https://git.example.com is not client forgejo's redirect_uri's",
        ),
        (Unserved::NoRoom, "no room for its meta within clients.bundle_bytes"),
    ] {
        assert_eq!(
            note_line("temper", Note::ForgeSignIn { number: 7, client: b"forgejo".as_slice().into(), why }),
            format!("temper serve: forge 7: no sign-in through temper: {words}\n")
        );
    }
}

#[test]
fn older_store_refusals_use_jigs_recovery_words_and_the_store_path() {
    use jig_protocol_store::{Family, Older, OpenRefusal};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for older in [Older::Unleveled, Older::Unrecorded, Older::Layout(Family::Application), Older::Directory] {
        let why = OpenRefusal::OlderBuild { why: older };
        for store in ["/data/deployment", "/elsewhere/store"] {
            let expected = format!("temper serve: {}\n", why.words().join(store));
            let (line, code) = ending_with_binary(
                "temper",
                Some(&End::Stopped(Stop::StoreOpen { why: why.clone() })),
                false,
                &directories,
                store.as_bytes(),
                &[],
            );
            assert_eq!(line, expected);
            assert!(line.contains("this start has changed nothing"));
            assert!(line.contains("together"));
            assert!(line.contains("every client signs in again"));
            assert_eq!(code, 1);
        }
    }
}

#[test]
fn other_endings_keep_their_exit_behavior() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    let mut problems = jig_protocol_config::Problems::empty();
    problems.push(jig_protocol_config::Path::root().member(b"deployment"), jig_protocol_config::What::Refers);
    for (end, output_failed, expected, code) in [
        (
            Some(End::Stopped(Stop::RestorePage {
                range: temper_engine_service::RestoreRangeKind::Census,
                failure: None,
            })),
            false,
            "temper serve: restore census: store load failed\n",
            1,
        ),
        (Some(End::Stopped(Stop::Start(problems))), false, "temper serve: deployment: unknown reference\n", 1),
        (
            Some(End::Stopped(Stop::Room(temper_engine_domain::room::Short {
                charter: 3,
                room: 2_047,
                bound: 51_717,
            }))),
            false,
            "temper serve: restart refused: charter 3 leaves its brief 2,047 bytes of its host part's 51,717; it needs at least 2,048\n",
            1,
        ),
        (Some(End::Stopped(Stop::Binding)), false, "temper serve: deployment namespace binding refused\n", 1),
        (Some(End::Stopped(Stop::CommitFailed { number: 7 })), false, "temper serve: commit 7 failed\n", 1),
        (
            Some(End::Stopped(Stop::Journal {
                why: temper_engine_service::JournalStop::Refused { commit: 7, sent: 9, durable: 8 },
            })),
            false,
            "temper serve: the engine's journal stopped: the store answered commit 7, with 9 sent and 8 durable\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::Journal { why: temper_engine_service::JournalStop::PastEdge { writes: 32 } })),
            false,
            "temper serve: the engine's journal stopped: a decision past the store's 32 writes\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::Journal { why: temper_engine_service::JournalStop::Counter })),
            false,
            "temper serve: the engine's journal stopped: the commit counter at its end\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::FrameTooLarge { number: 7, task: 11, bytes: 79, limit: 78 })),
            false,
            "temper serve: commit 7 refused: task 11's transcript frame, 79 bytes, past the store's page of 78 bytes\n",
            1,
        ),
        (Some(End::Stopped(Stop::Hooks)), false, "temper serve: webhooks listener failed\n", 1),
        (
            Some(End::Stopped(Stop::Listener {
                listener: temper_engine_service::Listening::Clients,
                address: Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
                error: Some(skein_io::Error::Other),
            })),
            false,
            "temper serve: client listener 127.0.0.1:7100 failed: other\n",
            1,
        ),
        (
            Some(End::Stopped(Stop::Listener {
                listener: temper_engine_service::Listening::Workers,
                address: Some(skein_io::Addr::InProcess(
                    skein_io::Name::new(b"workers").expect("configured worker listener"),
                )),
                error: None,
            })),
            false,
            "temper serve: worker listener workers failed: closed\n",
            1,
        ),
        (Some(End::Drained), false, "", 0),
        (Some(End::Drained), true, "", 1),
        (Some(End::Aborted), false, "", 1),
        (None, false, "", 1),
    ] {
        assert_eq!(
            ending_with_binary("temper", end.as_ref(), output_failed, &directories, b"/data/store", &[]),
            (expected.into(), code),
            "{end:?}, output failed {output_failed}"
        );
    }
}

#[test]
fn listening_notes_print_each_listener_and_socket_address_exactly() {
    use temper_engine_service::{Listening, Note};
    for (listener, name) in
        [(Listening::Clients, "clients"), (Listening::Workers, "workers"), (Listening::Webhooks, "webhooks")]
    {
        for (addr, printed) in [
            (([127, 0, 0, 1], 41733).into(), "127.0.0.1:41733"),
            (([0, 0, 0, 0, 0, 0, 0, 1], 41733).into(), "[::1]:41733"),
        ] {
            assert_eq!(
                note_line("temper", Note::Listening { listener, addr }),
                format!("listening {name} {printed}\n")
            );
        }
    }
}

#[test]
fn startup_warnings_print_the_configuration_path_and_figures() {
    let line = b"engine.charters[1].reserve_time_ms: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout";
    assert_eq!(
        note_line("temper", temper_engine_service::Note::StartupWarning { line: line.as_slice().into() }),
        "temper serve: warning: engine.charters[1].reserve_time_ms: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n"
    );
}

#[test]
fn hello_refusal_notes_print_the_channel_and_words_exactly() {
    assert_eq!(
        note_line(
            "temper",
            temper_engine_service::Note::HelloRefused {
                channel: 7,
                text: b"charter 2: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout".as_slice().into(),
            }
        ),
        "temper serve: hello refused channel=7: charter 2: needs checks within 60000 ms; the agent's profile.policy.check_timeout is 90000 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n",
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table pins every engine loss and refusal for both a lost channel and an opening"
)]
fn engine_worker_loss_lines_name_each_cause_and_refusal_exactly() {
    use temper_engine_service::{EngineWorkerLoss as Loss, Note};
    for (why, lost, opening) in [
        (
            Loss::OutputRoom { engine: 1_048_576, worker: 18_432_634 },
            "temper serve: worker lost worker=worker-a channel=7: engine io.output 1048576 bytes differs from worker io.output 18432634 bytes; change io.output to agree\n",
            "temper serve: worker opening ended worker=-: engine io.output 1048576 bytes differs from worker io.output 18432634 bytes; change io.output to agree\n",
        ),
        (
            Loss::Silence,
            "temper serve: worker lost worker=worker-a channel=7: silence\n",
            "temper serve: worker opening ended worker=-: silence\n",
        ),
        (
            Loss::Stall,
            "temper serve: worker lost worker=worker-a channel=7: stall\n",
            "temper serve: worker opening ended worker=-: stall\n",
        ),
        (
            Loss::Full,
            "temper serve: worker lost worker=worker-a channel=7: full\n",
            "temper serve: worker opening ended worker=-: full\n",
        ),
        (
            Loss::Deadline,
            "temper serve: worker lost worker=worker-a channel=7: deadline\n",
            "temper serve: worker opening ended worker=-: deadline\n",
        ),
        (
            Loss::Stream,
            "temper serve: worker lost worker=worker-a channel=7: stream\n",
            "temper serve: worker opening ended worker=-: stream\n",
        ),
        (
            Loss::Framing,
            "temper serve: worker lost worker=worker-a channel=7: framing\n",
            "temper serve: worker opening ended worker=-: framing\n",
        ),
        (
            Loss::NoRoom,
            "temper serve: worker lost worker=worker-a channel=7: no-room\n",
            "temper serve: worker opening ended worker=-: no-room\n",
        ),
        (
            Loss::Stopping,
            "temper serve: worker lost worker=worker-a channel=7: stopping\n",
            "temper serve: worker opening ended worker=-: stopping\n",
        ),
        (
            Loss::Refused { reason: 2 },
            "temper serve: worker lost worker=worker-a channel=7: refused terms\n",
            "temper serve: worker opening ended worker=-: refused terms\n",
        ),
        (
            Loss::Refused { reason: 3 },
            "temper serve: worker lost worker=worker-a channel=7: refused malformed\n",
            "temper serve: worker opening ended worker=-: refused malformed\n",
        ),
        (
            Loss::Refused { reason: 256 },
            "temper serve: worker lost worker=worker-a channel=7: refused unauthorized\n",
            "temper serve: worker opening ended worker=-: refused unauthorized\n",
        ),
        (
            Loss::Refused { reason: 257 },
            "temper serve: worker lost worker=worker-a channel=7: refused wires\n",
            "temper serve: worker opening ended worker=-: refused wires\n",
        ),
        (
            Loss::Refused { reason: 258 },
            "temper serve: worker lost worker=worker-a channel=7: refused busy\n",
            "temper serve: worker opening ended worker=-: refused busy\n",
        ),
        (
            Loss::Refused { reason: 259 },
            "temper serve: worker lost worker=worker-a channel=7: refused replaced\n",
            "temper serve: worker opening ended worker=-: refused replaced\n",
        ),
        (
            Loss::Refused { reason: 260 },
            "temper serve: worker lost worker=worker-a channel=7: refused not-admitted\n",
            "temper serve: worker opening ended worker=-: refused not-admitted\n",
        ),
        (
            Loss::Refused { reason: 261 },
            "temper serve: worker lost worker=worker-a channel=7: refused stopping\n",
            "temper serve: worker opening ended worker=-: refused stopping\n",
        ),
        (
            Loss::Refused { reason: 42 },
            "temper serve: worker lost worker=worker-a channel=7: refused code-42\n",
            "temper serve: worker opening ended worker=-: refused code-42\n",
        ),
        (
            Loss::RefusedPeer { reason: 2 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer terms\n",
            "temper serve: worker opening ended worker=-: refused-by-peer terms\n",
        ),
        (
            Loss::RefusedPeer { reason: 3 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer malformed\n",
            "temper serve: worker opening ended worker=-: refused-by-peer malformed\n",
        ),
        (
            Loss::RefusedPeer { reason: 256 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer unauthorized\n",
            "temper serve: worker opening ended worker=-: refused-by-peer unauthorized\n",
        ),
        (
            Loss::RefusedPeer { reason: 257 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer wires\n",
            "temper serve: worker opening ended worker=-: refused-by-peer wires\n",
        ),
        (
            Loss::RefusedPeer { reason: 258 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer busy\n",
            "temper serve: worker opening ended worker=-: refused-by-peer busy\n",
        ),
        (
            Loss::RefusedPeer { reason: 259 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer replaced\n",
            "temper serve: worker opening ended worker=-: refused-by-peer replaced\n",
        ),
        (
            Loss::RefusedPeer { reason: 260 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer not-admitted\n",
            "temper serve: worker opening ended worker=-: refused-by-peer not-admitted\n",
        ),
        (
            Loss::RefusedPeer { reason: 261 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer stopping\n",
            "temper serve: worker opening ended worker=-: refused-by-peer stopping\n",
        ),
        (
            Loss::RefusedPeer { reason: 42 },
            "temper serve: worker lost worker=worker-a channel=7: refused-by-peer code-42\n",
            "temper serve: worker opening ended worker=-: refused-by-peer code-42\n",
        ),
    ] {
        assert_eq!(
            note_line(
                "temper",
                Note::WorkerLost {
                    worker: Some(b"worker-a".as_slice().into()),
                    channel: skein_lib::Token::new(7),
                    why,
                }
            ),
            lost,
            "{why:?}"
        );
        assert_eq!(note_line("temper", Note::WorkerOpeningEnded { worker: None, why }), opening, "{why:?}");
    }
    assert_eq!(
        note_line(
            "temper",
            Note::WorkerOpeningEnded { worker: Some(b"worker-a".as_slice().into()), why: Loss::Silence }
        ),
        "temper serve: worker opening ended worker=worker-a: silence\n"
    );
}

#[test]
fn worker_unsent_lines_name_the_record_bytes_and_bound() {
    use jig_protocol_engine_workers::{Bound, Unsent};
    use jig_protocol_worker_wire::Kind;
    for (kind, bytes, bound, expected) in [
        (
            Kind::Assignment,
            1_300_000,
            Bound::Room(1_048_576),
            "temper serve: worker unsent worker=worker-a channel=7: assignment of 1300000 bytes past the room of 1048576 bytes\n",
        ),
        (
            Kind::Message,
            4_200,
            Bound::Channel(skein_channel::Unsent::TooLarge),
            "temper serve: worker unsent worker=worker-a channel=7: message of 4200 bytes refused: too-large\n",
        ),
    ] {
        assert_eq!(
            note_line(
                "temper",
                temper_engine_service::Note::WorkerUnsent {
                    worker: b"worker-a".as_slice().into(),
                    channel: skein_lib::Token::new(7),
                    unsent: Unsent { kind, bytes, bound },
                }
            ),
            expected
        );
    }
}

#[test]
fn worker_listener_failures_print_the_exit_lines_words_at_once() {
    use temper_engine_service::Note;
    for (address, error, expected) in [
        (
            Some(skein_io::Addr::Ip(([127, 0, 0, 1], 7100).into())),
            Some(skein_io::Error::Unreachable),
            "temper serve: worker listener 127.0.0.1:7100 failed: unreachable\n",
        ),
        (
            Some(skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap())),
            None,
            "temper serve: worker listener workers failed: closed\n",
        ),
    ] {
        assert_eq!(note_line("temper", Note::WorkerListenerFailed { address, error }), expected);
    }
}

#[test]
fn setup_stops_print_the_forge_plan_and_typed_step_without_a_subject() {
    use temper_engine_service::{Note, SetupPlan, SetupStage, SetupStep, SetupStopped, SetupWhy};
    for (why, line) in [
        (SetupStopped::NotPrepared, "prepare it with `temper forge prepare`"),
        (SetupStopped::Missing, "setup host missing"),
        (SetupStopped::Busy, "setup host busy"),
        (
            SetupStopped::Stopped {
                plan: SetupPlan::Reconcile,
                step: SetupStep { stage: SetupStage::Repository, why: SetupWhy::Refused(403) },
            },
            "reconciling stopped at Repository: refused 403",
        ),
    ] {
        assert_eq!(
            note_line("temper", Note::ForgeSetupStopped { number: 3, why }),
            format!("temper serve: forge 3: Not connected: {line}\n")
        );
    }
}

#[test]
fn the_shell_removes_only_the_named_handover_and_reports_unlink_failure() {
    use std::os::unix::ffi::OsStrExt;
    let directory =
        std::env::temp_dir().join(format!("temper-handover-{}", skein_shell::seed().expect("scratch name")));
    std::fs::create_dir(&directory).expect("handover fixture");
    let path = directory.join("admin_token");
    let removal = temper_engine_service::HandoverRemoval {
        forge: 1,
        run: skein_lib::Token::new(7),
        path: path.as_os_str().as_bytes().into(),
    };
    assert!(remove_handover(&removal).is_ok(), "an absent handover counts as removed");
    std::fs::create_dir(&path).expect("handover fixture");
    std::fs::write(path.join("held"), b"").expect("handover fixture");
    let errno = remove_handover(&removal).expect_err("an unlink failure is never acknowledged as removed");
    let line = note_line(
        "temper",
        temper_engine_service::Note::ForgeSetupStopped {
            number: removal.forge,
            why: temper_engine_service::SetupStopped::Removal { path: removal.path.clone(), errno },
        },
    );
    assert!(line.contains("removing the handover:") && line.contains(path.to_str().expect("handover fixture")));
    std::fs::remove_dir_all(&path).expect("handover fixture");
    std::fs::write(&path, b"").expect("handover fixture");
    assert!(remove_handover(&removal).is_ok());
    assert!(!path.exists());
    std::fs::remove_dir(&directory).expect("handover fixture");
}
#[test]
fn the_shell_trims_only_the_handovers_final_newline() {
    use std::os::unix::{ffi::OsStrExt, fs::PermissionsExt};
    let directory =
        std::env::temp_dir().join(format!("temper-handover-read-{}", skein_shell::seed().expect("scratch name")));
    std::fs::create_dir(&directory).expect("handover fixture");
    let path = directory.join("admin_token");
    let read = temper_engine_service::HandoverRead {
        forge: 1,
        run: skein_lib::Token::new(7),
        path: path.as_os_str().as_bytes().into(),
        max: 11,
    };
    for (bytes, expected) in [
        (b"test-admin\n".as_slice(), b"test-admin".as_slice()),
        (b"test-admin".as_slice(), b"test-admin".as_slice()),
        (b"token\r\n".as_slice(), b"token\r".as_slice()),
        (b"one\ntwo".as_slice(), b"one\ntwo".as_slice()),
        (b"".as_slice(), b"".as_slice()),
        (b"12345678901".as_slice(), b"12345678901".as_slice()),
    ] {
        std::fs::write(&path, bytes).expect("handover fixture");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).expect("private fixture");
        let answer = read_handover(&read).expect("within read bound").expect("present fixture");
        assert!(answer.expose() == expected, "only one final LF is removed");
    }
    std::fs::write(&path, b"123456789012").expect("one byte beyond the bound");
    assert!(matches!(read_handover(&read), Err(temper_engine_service::HandoverUnread::TooLarge)));
    std::fs::remove_dir_all(directory).expect("handover fixture");
}

#[test]
fn a_drained_end_prints_each_unsettled_write() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    let unfinished = [
        temper_engine_service::Unsettled { entry: 17, task: 3 },
        temper_engine_service::Unsettled { entry: 22, task: 9 },
    ];
    assert_eq!(ending_with_binary("temper", Some(&End::Drained), false, &directories, b"/data/store", &unfinished), (
        "temper serve: stopped before task 3's forge write 17 settled; the next start settles it\ntemper serve: stopped before task 9's forge write 22 settled; the next start settles it\n".into(), 0
    ));
}

#[test]
fn forge_fit_endings_name_phase_family_set_and_measured_limit() {
    use temper_engine_domain_forge::{RecordKind, RestoreFit, RestoreSet};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (family, set, held, additional, limit, detail) in [
        (
            Some(RecordKind::Binding),
            RestoreSet::Rows(RecordKind::Binding),
            4,
            1,
            4,
            "forge Binding Rows(Binding), held 4, additional 1, limit 4",
        ),
        (None, RestoreSet::LandingFlights, 9, 0, 8, "forge LandingFlights, held 9, additional 0, limit 8"),
    ] {
        let end = End::Stopped(Stop::ForgeFit {
            phase: Some(jig_domain_core::start::Phase::RestoreConnector { connector: 7 }),
            failure: RestoreFit { family, set, held, additional, limit },
        });
        let (line, code) = ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]);
        assert_eq!(line, format!("temper serve: restart refused during connector 7 restore: {detail}\n"));
        assert_eq!(code, 1);
    }
}

#[test]
fn start_refusals_name_every_phase_and_the_refused_project() {
    use jig_domain_core::start::{Failure, Phase};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (phase, words) in [
        (Some(Phase::LoadCore), "core restore"),
        (Some(Phase::LoadInbox), "inbox census"),
        (Some(Phase::RestoreConnector { connector: 7 }), "connector 7 restore"),
        (Some(Phase::Link), "link"),
        (Some(Phase::Sweep), "sweep"),
        (Some(Phase::Switch), "store switch"),
        (Some(Phase::Wires), "wires"),
        (Some(Phase::AdoptRuns), "run adoption"),
        (Some(Phase::ReadAfresh { connector: 7 }), "connector 7 refresh"),
        (Some(Phase::SettleOutbox { connector: 7 }), "connector 7 outbox settlement"),
        (Some(Phase::Open), "opening decisions"),
        (None, "unknown phase"),
    ] {
        let end = End::Stopped(Stop::StartRefused { phase, failure: Failure::ProjectPolicy { project: 3 } });
        assert_eq!(
            ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]),
            (format!("temper serve: restart refused during {words}: project 3: policy refused\n"), 1)
        );
    }
}

#[test]
fn start_failures_name_the_set_key_setting_and_measured_limit() {
    use jig_domain_core::{
        HeldLimit, Key, settings,
        start::{Failure, LiveSet},
    };
    let key = Key::Tasks(jig_domain_tasks::Key::Live(17));
    for (failure, words) in [
        (
            Failure::LiveSet {
                set: LiveSet::TasksBound(jig_domain_tasks::RestoreBound::Tasks),
                key: key.clone(),
                held: 5,
                limit: 4,
            },
            "TasksBound(Tasks), key Tasks(Live(17)), held 5, limit 4",
        ),
        (Failure::HeldPastLimit { which: HeldLimit::Revisions, held: 5, limit: 4 }, "Revisions, held 5, limit 4"),
        (Failure::HeldPastLimit { which: HeldLimit::Versions, held: 5, limit: 4 }, "Versions, held 5, limit 4"),
        (
            Failure::Settings {
                failure: settings::Failure {
                    setting: settings::Setting::Policy(1),
                    fault: settings::Fault::Shape,
                    detail: None,
                },
                changed: true,
            },
            "changed setting Policy(1): Shape",
        ),
        (
            Failure::Settings {
                failure: settings::Failure {
                    setting: settings::Setting::Rules,
                    fault: settings::Fault::Reference,
                    detail: None,
                },
                changed: false,
            },
            "default setting Rules: Reference",
        ),
        (Failure::Accounting { key: key.clone() }, "accounting refused, key Tasks(Live(17))"),
        (
            Failure::Link {
                record: Key::Tasks(jig_domain_tasks::Key::Live(199)),
                linked: jig_domain_core::Linked::Record(Key::People(jig_domain_people::Key::Person(102))),
            },
            "Tasks(Live(199)) links missing People(Person(102))",
        ),
        (
            Failure::Link { record: key.clone(), linked: jig_domain_core::Linked::Charter { charter: 3, revision: 2 } },
            "Tasks(Live(17)) links missing charter 3 at revision 2",
        ),
        (
            Failure::Link { record: key.clone(), linked: jig_domain_core::Linked::Project(9) },
            "Tasks(Live(17)) links missing project 9",
        ),
        (
            Failure::Link { record: key.clone(), linked: jig_domain_core::Linked::Connector(9) },
            "Tasks(Live(17)) links missing connector 9",
        ),
        (
            Failure::Link {
                record: key.clone(),
                linked: jig_domain_core::Linked::SettingsProposal {
                    proposer: jig_domain_people::Proposer::Party(1),
                    proposal: 4_294_967_296,
                },
            },
            "Tasks(Live(17)) links missing settings proposal Party(1)/4294967296",
        ),
        (
            Failure::Link {
                record: Key::Tasks(jig_domain_tasks::Key::Writer(1)),
                linked: jig_domain_core::Linked::Holder(jig_domain_tasks::Name { connector: 1, path: Box::new([]) }),
            },
            "Tasks(Writer(1)) links missing hold owner for Name { connector: 1, path: [] }",
        ),
        (Failure::LlmChoice { key }, "LLM choice refused, key Tasks(Live(17))"),
    ] {
        assert_eq!(start_failure(&failure), words);
    }
}

#[test]
fn unreadable_temper_rows_end_with_the_named_recovery_words_and_exit_one() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (tag, name) in [(21, "Flight"), (9, "Landed"), (20, "Setting")] {
        let end = End::Stopped(Stop::StoreRow { row: temper_engine_service::UnreadableRow { key: Box::from([tag]) } });
        for deployment in ["/data/deployment", "/elsewhere/store"] {
            for binary in ["temper", "temper-engine"] {
                assert_eq!(
                    ending_with_binary(binary, Some(&end), false, &directories, deployment.as_bytes(), &[]),
                    (
                        format!(
                            "{binary} serve: The store in {deployment} holds a {name} record, family {tag}, that this build cannot read, and the start stopped there. Start the build that wrote it again, or, to start afresh, stop the service, move `active`, `generations/` and `transcripts/` out of {deployment} together, and start again: nothing is deleted, and the next start makes a new, empty store.\n"
                        ),
                        1
                    )
                );
            }
        }
    }
}

#[test]
fn restore_pages_name_only_the_store_supplied_cause_family_and_key() {
    use jig_protocol_store::{
        Family,
        restore::{Cause, DecodeFailure, Failure, RangeFailure, RowSite},
    };
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (failure, words) in [
        (None, "store load failed"),
        (Some(Failure { cause: Cause::Range(RangeFailure::Cursor), row: None }), "Range(Cursor)"),
        (
            Some(Failure {
                cause: Cause::Decode(DecodeFailure::Value),
                row: Some(RowSite {
                    family: Some(Family::Tasks),
                    key: Box::from([0x01, 0xab]),
                    key_bytes: 2,
                    truncated: false,
                }),
            }),
            "Decode(Value), family Tasks, key [01, ab]",
        ),
        (
            Some(Failure {
                cause: Cause::Decode(DecodeFailure::Key),
                row: Some(RowSite { family: None, key: Box::from([0xff]), key_bytes: 3, truncated: true }),
            }),
            "Decode(Key), key [ff] (truncated, 3 bytes total)",
        ),
    ] {
        let end = End::Stopped(Stop::RestorePage { range: temper_engine_service::RestoreRangeKind::Tasks, failure });
        assert_eq!(
            ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]),
            (format!("temper serve: restore tasks: {words}\n"), 1)
        );
    }
}

#[test]
fn refusal_lines_name_the_task_attempt_tool_layer_and_reason() {
    use temper_engine_domain_forge::LinkNote;
    use temper_engine_service::Note;
    for (note, expected) in [
        (
            Note::CallAdmissionRefused {
                task: 7,
                attempt: 2,
                tool: b"read_file".as_slice().into(),
                layer: "core-call-room",
                reason: "call record room full",
            },
            "temper serve: call admission refused task=7 attempt=2 tool=read_file layer=core-call-room reason=call record room full\n",
        ),
        (
            Note::ForgeLink { forge: 7, note: LinkNote::Gone },
            "forge 7's person: their account is gone, the link kept\n",
        ),
        (Note::ForgeLink { forge: 7, note: LinkNote::Writer }, "forge 7's person: refused, temper's writer\n"),
        (Note::ForgeLink { forge: 7, note: LinkNote::Replaced }, "forge 7's person: replaced\n"),
        (Note::ForgeLink { forge: 7, note: LinkNote::Cleared }, "forge 7's person: cleared\n"),
    ] {
        assert_eq!(note_line("temper", note), expected);
    }
    let line = note_line(
        "temper",
        Note::ReadRefused {
            task: 7,
            attempt: 2,
            tool: b"read_file".as_slice().into(),
            layer: "authority",
            reason: "connector read denied",
            job: None,
            findings: Box::new([]),
            error: None,
        },
    );
    assert!(line.contains("task=7 attempt=2 tool=read_file layer=authority reason=connector read denied"));
    assert!(line.contains("findings=[]"));
}

#[test]
fn read_job_refusal_line_names_the_asked_attempt() {
    let line = note_line(
        "temper",
        temper_engine_service::Note::ReadRefused {
            task: 7,
            attempt: 4,
            tool: Box::from(*b"read_job"),
            layer: "forge-client",
            reason: "forge read failed",
            findings: Box::new([]),
            error: Some(skein_forge::api::Error::MissingJob),
            job: Some(skein_forge::api::JobAttempt { head: [0xab; 32], run: 2, job: 0, attempt: 1 }),
        },
    );
    assert_eq!(
        line,
        format!(
            "temper serve: read refused task=7 attempt=4 tool=read_job layer=forge-client reason=forge read failed head={} run=2 job=0 job_attempt=1 error=Some(MissingJob) findings=[]\n",
            "ab".repeat(32)
        )
    );
}

#[test]
fn brief_read_refusal_line_names_the_task_section_stage_and_error() {
    use skein_forge::api::{Bound, Error, Excess};
    for (stage, reason, heads, error, expected) in [
        (
            "item",
            "the forge's answer did not match its read",
            None,
            Some(Error::InvalidAnswer),
            "temper serve: brief read refused task=7 section=pull stage=item reason=the forge's answer did not match its read error=InvalidAnswer\n",
        ),
        (
            "pull",
            "the change's head moved",
            Some(([1; 32], [99; 32])),
            None,
            "temper serve: brief read refused task=7 section=pull stage=pull reason=the change's head moved pinned=0101010101010101010101010101010101010101010101010101010101010101 head=6363636363636363636363636363636363636363636363636363636363636363\n",
        ),
        (
            "files",
            "the answer is past",
            None,
            Some(Error::TooLarge(Excess::exact(Bound::Rows, 8, 22))),
            "temper serve: brief read refused task=7 section=pull stage=files reason=the answer is past forge.client.rows (8): 22 rows error=TooLarge: past forge.client.rows (8): 22 rows\n",
        ),
    ] {
        assert_eq!(
            note_line(
                "temper",
                temper_engine_service::Note::BriefReadRefused {
                    task: 7,
                    section: b"pull",
                    stage,
                    reason,
                    heads,
                    error
                }
            ),
            expected
        );
    }
}

#[test]
fn stale_files_refusal_names_the_error_without_heads() {
    assert_eq!(
        note_line(
            "temper",
            temper_engine_service::Note::BriefReadRefused {
                task: 7,
                section: b"pull",
                stage: "files",
                reason: "the change's head moved",
                heads: None,
                error: Some(skein_forge::api::Error::Stale),
            },
        ),
        "temper serve: brief read refused task=7 section=pull stage=files reason=the change's head moved error=Stale\n"
    );
}

#[test]
fn brief_failure_line_names_how_it_counts_and_why() {
    assert_eq!(
        note_line(
            "temper",
            temper_engine_service::Note::BriefFailed {
                task: 7,
                counts: "permanent",
                reason: b"pull: the forge's answer did not match its read".as_slice().into(),
            }
        ),
        "temper serve: brief failed task=7 counts=permanent reason=pull: the forge's answer did not match its read\n"
    );
}

#[test]
fn refused_job_log_origin_note_names_only_the_origin() {
    let line = note_line(
        "temper",
        temper_engine_service::Note::ForgeOrigin {
            number: 7,
            origin: skein_forge::github::redirect::Origin {
                tls: true,
                host: b"unrelated.example.test".as_slice().into(),
                port: 443,
            },
        },
    );
    assert_eq!(line, "temper serve: forge 7: refused job-log origin https://unrelated.example.test:443\n");
}

#[cfg(test)]
fn failed_run_note(detail: &[u8]) -> temper_engine_service::Note {
    temper_engine_service::Note::RunFailed(temper_engine_domain::RunDiagnostic {
        task: 3,
        run: 7,
        attempt: 2,
        failure: jig_domain_host::Failure::Run(jig_domain_host::RunFailure::Model),
        detail: detail.into(),
    })
}

#[test]
fn failed_run_stderr_escapes_control_bytes_on_one_line() {
    assert_eq!(
        note_line("temper", failed_run_note(b"exit 1: bad\nstream\r\t\0\x1b\x7f\xff")),
        "temper serve: task 3 run 7 attempt 2 failed Run(Model): exit 1: bad\\nstream\\r\\t\\x00\\x1b\\x7f\\xff\n"
    );
}

#[test]
fn failed_run_stderr_keeps_a_bounded_tail() {
    let mut detail = vec![b'x'; 4097];
    detail[0] = b'y';
    let line = note_line("temper", failed_run_note(&detail));
    let prefix = "temper serve: task 3 run 7 attempt 2 failed Run(Model): ";
    assert_eq!(line, format!("{prefix}{}\n", "x".repeat(4096)));
}

#[cfg(test)]
fn refused_run_note(reason: jig_domain_host::Refusal) -> temper_engine_service::Note {
    temper_engine_service::Note::RunRefused(temper_engine_service::RunRefusalDiagnostic {
        task: 3,
        run: 7,
        attempt: 2,
        reason,
    })
}

#[test]
fn refused_run_stderr_names_reason_and_bounds_start_words() {
    use jig_domain_host::{Invalid, Refusal, StartCause};
    let prefix = "temper serve: task 3 run 7 attempt 2 refused ";
    for (reason, class) in [
        (Refusal::Busy, "Busy"),
        (Refusal::Invalid(Invalid::Charter), "Invalid(Charter)"),
        (Refusal::Invalid(Invalid::Transcript), "Invalid(Transcript)"),
        (Refusal::Invalid(Invalid::Grants), "Invalid(Grants)"),
        (Refusal::Invalid(Invalid::Messages), "Invalid(Messages)"),
    ] {
        assert_eq!(note_line("temper", refused_run_note(reason)), format!("{prefix}{class}: \n"));
    }
    for (cause, class) in [(StartCause::Other, "Other"), (StartCause::Grants, "Grants")] {
        let reason =
            Refusal::Invalid(Invalid::Start { cause, why: b"charter\n\r\t\0\x1b\x7f\xff\\\"".as_slice().into() });
        assert_eq!(
            note_line("temper", refused_run_note(reason)),
            format!("{prefix}Invalid(Start({class})): charter\\n\\r\\t\\x00\\x1b\\x7f\\xff\\\\\\\"\n")
        );
    }
    let mut why = vec![b'x'; 4097];
    why[0] = b'y';
    assert_eq!(
        note_line(
            "temper",
            refused_run_note(Refusal::Invalid(Invalid::Start { cause: StartCause::Other, why: why.into() }))
        ),
        format!("{prefix}Invalid(Start(Other)): {}\n", "x".repeat(4096))
    );
}

#[test]
fn job_stderr_forms_escape_and_bound_their_why() {
    use jig_domain_host::{JobFailure, JobInvalid, JobRefusal, Preparation};
    use temper_engine_service::{JobDiagnostic, JobDiagnosticEnding};
    // The engine job forms retain the last 4,096 input bytes.
    const TAIL: usize = 4096;
    for (refused, prefix) in [
        (false, "temper serve: task 3 run 7 attempt 2 failed Unprepared(Transient): job 3: "),
        (true, "temper serve: task 3 run 7 attempt 2 refused Invalid(Start): job 3: "),
    ] {
        for (why, expected) in [
            (b"fetch base: Broken\n\xff".to_vec(), "fetch base: Broken\\n\\xff".to_owned()),
            (vec![b'x'; TAIL], "x".repeat(TAIL)),
            ([b"discard".as_slice(), vec![b'x'; TAIL].as_slice()].concat(), "x".repeat(TAIL)),
        ] {
            let ending = if refused {
                JobDiagnosticEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why: why.clone().into() }))
            } else {
                JobDiagnosticEnding::Failed {
                    failure: JobFailure::Unprepared(Preparation::Transient),
                    detail: why.clone().into(),
                }
            };
            let note = temper_engine_service::Note::JobAnswered(JobDiagnostic {
                task: 3,
                run: 7,
                attempt: 2,
                kind: 3,
                ending,
            });
            assert_eq!(note_line("temper", note), format!("{prefix}{expected}\n"));
        }
    }
    for (failure, class) in [
        (JobFailure::Deadline, "Deadline"),
        (JobFailure::Undelivered(jig_domain_host::DeliveryOutcome::Failed), "Undelivered(Failed)"),
    ] {
        let note = temper_engine_service::Note::JobAnswered(JobDiagnostic {
            task: 3,
            run: 7,
            attempt: 2,
            kind: 3,
            ending: JobDiagnosticEnding::Failed { failure, detail: Box::new([]) },
        });
        assert_eq!(
            note_line("temper", note),
            format!("temper serve: task 3 run 7 attempt 2 failed {class}: job 3: \n")
        );
    }
}
#[test]
fn core_wait_notes_name_the_reason_and_recovery() {
    use temper_engine_service::{CoreWait, Note};
    for (binary, note, expected) in [
        (
            "temper",
            Note::CoreWaiting { why: CoreWait::Starting { step: None } },
            "temper serve: the core is not answering: starting: loading the core's records\n",
        ),
        ("temper", Note::CoreAnswers, "temper serve: the core answers again\n"),
        ("temper-engine", Note::CoreAnswers, "temper-engine serve: the core answers again\n"),
    ] {
        assert_eq!(note_line(binary, note), expected);
        assert!(expected.len() <= 240);
    }
}
#[test]
fn retired_settings_name_the_ctl_setting_at_start() {
    use temper_engine_domain_forge::settings::{SettingKey, SettingSlot};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    for (kind, scope, number, words) in [(6, 2, 1, "project-questions 2"), (3, 0, 3, "forge-charter 3")] {
        let end = End::Stopped(Stop::Setting(SettingKey { kind, scope, number, slot: SettingSlot::Change }));
        assert_eq!(
            ending_with_binary("temper", Some(&end), false, &directories, b"/data/deployment", &[]),
            (format!("temper serve: restart refused: {words}: retired stored setting\n"), 1)
        );
    }
}

#[test]
fn cutoff_comparison_refusal_names_no_error_or_heads() {
    assert_eq!(
        note_line(
            "temper",
            temper_engine_service::Note::BriefReadRefused {
                task: 7,
                section: b"pull",
                stage: "compare",
                reason: "not read by the section's cutoff",
                heads: None,
                error: None,
            },
        ),
        "temper serve: brief read refused task=7 section=pull stage=compare reason=not read by the section's cutoff\n"
    );
}

#[test]
fn client_stream_failure_line_names_the_sign_in_and_bound() {
    let note = temper_engine_service::Note::ClientStreamFailed {
        sign_in: 41,
        why: temper_engine_service::StreamFailure::QueueBound { queued_bytes: 65_536 },
    };
    assert_eq!(
        super::note_line("temper", note),
        "temper serve: client stream closed: sign_in=41 reason=QueueBound { queued_bytes: 65536 }\n"
    );
}

#[test]
fn head_refusal_line_names_the_task_only_when_concerned() {
    for (task, expected) in [
        (Some(7), "temper-engine serve: branch head refused task=7: WORDS\n"),
        (None, "temper-engine serve: branch head refused: WORDS\n"),
    ] {
        assert_eq!(
            note_line(
                "temper-engine",
                temper_engine_service::Note::HeadRefused { task, words: b"WORDS".as_slice().into() }
            ),
            expected
        );
    }
}
