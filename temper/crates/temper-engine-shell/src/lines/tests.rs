use super::*;

#[test]
fn a_lowered_limit_names_its_setting_figure_and_held_work() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("directories");
    let end = End::Stopped(Stop::LimitRestore {
        name: b"forge.repositories",
        kind: temper_engine_service::startup::LimitKind::Temper,
        configured: 1,
        held: 2,
    });
    assert_eq!(ending_with_binary("temper-engine",Some(&end),false,&directories,b"/data/deployment",&[]),
        ("temper-engine serve: forge.repositories (setting temper-limits): its figure is 1, but the store holds 2; put limits-held in the data directory to start at the stock\n".into(),1));
}

#[test]
fn a_codec_lowering_names_its_setting_and_figures_without_row_values() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").unwrap();
    for (name, kind, setting) in [
        (b"forge.name_bytes".as_slice(), temper_engine_service::startup::LimitKind::Temper, "temper-limits"),
        (b"people.identity_bytes".as_slice(), temper_engine_service::startup::LimitKind::Jig, "limits"),
    ] {
        let end = End::Stopped(Stop::CodecRestore {
            bound: jig_protocol_store::codec::Lowered { name, configured: 200, held: 300 },
            kind,
            row: Some(jig_protocol_store::restore::RowSite {
                family: Some(jig_protocol_store::Family::Application),
                key: Box::new([2, 2]),
                key_bytes: 2,
                truncated: false,
            }),
        });
        let (words, code) =
            ending_with_binary("temper-engine", Some(&end), false, &directories, b"/data/deployment", &[]);
        assert_eq!(code, 1);
        assert_eq!(
            words,
            format!(
                "temper-engine serve: {} (setting {setting}): its figure is 200, but the store holds a record that needs 300, key [02, 02]; put limits-held in the data directory to start at the stock\n",
                String::from_utf8_lossy(name)
            )
        );
    }
}

#[test]
fn ran_lines_include_declared_counts_in_order_or_no_suffix() {
    use jig_protocol_store::{JobCount, Owner};
    for (binary, shown) in [("temper-box", "temper-box"), ("temper-engine", "temper-engine"), ("temper", "temper-box")]
    {
        for (counts, suffix) in [
            (vec![], ""),
            (
                vec![
                    JobCount { name: "dropped".into(), count: 5 },
                    JobCount { name: "BranchHead".into(), count: 1 },
                    JobCount { name: "PullState".into(), count: 2 },
                    JobCount { name: "Ci".into(), count: 0 },
                ],
                "; dropped 5, BranchHead 1, PullState 2, Ci 0",
            ),
        ] {
            assert_eq!(
                note_line(
                    binary,
                    temper_engine_service::Note::Ran {
                        of: Owner::Application,
                        job: "deleted-repository-forge-rows".into(),
                        rows: 812,
                        commits: 2,
                        counts: counts.into_boxed_slice(),
                    }
                ),
                format!("{shown} serve: ran temper deleted-repository-forge-rows: 812 rows in 2 commits{suffix}\n")
            );
        }
    }
}

#[test]
fn accepted_worker_lines_name_each_agreed_wire_in_statement_order() {
    use jig_protocol_worker_wire::Agreed;
    assert_eq!(
        note_line(
            "custom",
            temper_engine_service::Note::WorkerFigures {
                limits: 440,
                endpoints: 2,
                revision: temper_engine_service::FiguresRevision::Agents(7),
            }
        ),
        "custom serve: figures for workers: 440 limits, 2 endpoints, agents setting revision 7\n"
    );
    assert_eq!(
        note_line(
            "custom",
            temper_engine_service::Note::WorkerFigures {
                limits: 440,
                endpoints: 2,
                revision: temper_engine_service::FiguresRevision::Limits(8),
            }
        ),
        "custom serve: figures for workers: 440 limits, 2 endpoints, limit settings revision 8\n"
    );
    assert_eq!(
        note_line("custom", temper_engine_service::Note::WorkerFiguresTaken { worker: b"worker-a".as_slice().into() }),
        "custom serve: worker worker-a took its figures\n"
    );

    let wires = skein_lib::List::from_boxed(Box::new([
        Agreed { name: "jig-worker-wire", version: 10 },
        Agreed { name: "smith-pieces", version: 1 },
        Agreed { name: "smith-charter", version: 4 },
        Agreed { name: "smith-transcript", version: 7 },
        Agreed { name: "temper-worker-wire", version: 2 },
    ]));
    assert_eq!(
        note_line("custom", temper_engine_service::Note::WorkerOpened { worker: b"worker-a".as_slice().into(), wires }),
        "custom serve: worker worker-a opened: jig-worker-wire 10, smith-pieces 1, smith-charter 4, smith-transcript 7, temper-worker-wire 2\n"
    );
}

#[test]
fn opening_refusal_lines_name_the_worker_or_the_precredential_version() {
    use temper_engine_service::Note;
    assert_eq!(
        note_line(
            "custom",
            Note::WorkerOpeningRefused {
                worker: Some(b"worker-a".as_slice().into()),
                reason: 257,
                text: b"wire smith-charter: the worker states none".as_slice().into(),
            }
        ),
        "custom serve: worker worker-a refused at its opening: wires: wire smith-charter: the worker states none\n"
    );
    assert_eq!(
        note_line(
            "custom",
            Note::WorkerOpeningRefused {
                worker: None,
                reason: 1,
                text: b"speaks 10..10, offered 9..9".as_slice().into(),
            }
        ),
        "custom serve: a worker refused at its opening: version: speaks 10..10, offered 9..9\n"
    );
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

    for (note, words) in [
        (
            Note::GitHubLogOriginUnresolved { host: b"productionresultssa7.blob.core.windows.net".as_slice().into() },
            "GitHub log origin https://productionresultssa7.blob.core.windows.net:443: unresolved; skipped",
        ),
        (Note::GitHubLogOriginsUnresolved, "GitHub log origins: none resolved; no job log will be fetched"),
    ] {
        assert_eq!(note_line("custom", note), format!("custom serve: {words}\n"));
    }
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
    for older in [
        Older::Unleveled,
        Older::Floor,
        Older::Unregistered,
        Older::Behind,
        Older::Layout(Family::Application),
        Older::Directory,
    ] {
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
            match older {
                Older::Floor | Older::Behind | Older::Unregistered => {
                    assert!(line.contains(&format!("build `{}`", jig_protocol_store::FLOOR_BRIDGE)));
                }
                Older::Unleveled | Older::Layout(_) | Older::Directory => {
                    assert!(line.contains("together"));
                    assert!(line.contains("every client signs in again"));
                }
            }
            assert_eq!(code, 1);
        }
    }
}

#[test]
fn a_full_map_names_its_usage_budget_and_projected_bytes() {
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    let end = End::Stopped(Stop::CommitFailed {
        number: 685_204,
        why: CommitStop::Store(jig_protocol_store::CommitFailure::Refused(skein_kv::Refusal::Full(
            skein_kv::Exhausted::Budget { used: 268_433_350, budget: 268_435_456, projected: 268_435_862 },
        ))),
    });
    assert_eq!(
        ending_with_binary("temper-engine", Some(&end), false, &directories, b"/data/store", &[]),
        (
            "temper-engine serve: commit 685204 failed: the store's map is full (268,433,350 of 268,435,456 bytes; this commit needs 268,435,862)\n".into(),
            1,
        )
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table pins every protocol/store-versions.md section 1 cause and CHECK word"
)]
fn every_commit_cause_has_bounded_content_free_words_and_a_whole_stop_line() {
    use CommitStop::{Route, Store};
    use jig_protocol_store::{CommitCheck, CommitFailure};
    use skein_kv::{Exhausted, Large, Refusal};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    for (why, expected) in [
        (
            Store(CommitFailure::Refused(Refusal::Full(Exhausted::Budget {
                used: u64::MAX,
                budget: u64::MAX,
                projected: u64::MAX,
            }))),
            "the store's map is full (18,446,744,073,709,551,615 of 18,446,744,073,709,551,615 bytes; this commit needs 18,446,744,073,709,551,615)",
        ),
        (
            Store(CommitFailure::Refused(Refusal::Full(Exhausted::Entries { capacity: u32::MAX }))),
            "the store's map is full (4,294,967,295 entries)",
        ),
        (
            Store(CommitFailure::Refused(Refusal::Full(Exhausted::Numbers))),
            "the store's commit numbers are at their end",
        ),
        (
            Store(CommitFailure::Refused(Refusal::TooLarge(Large::Key))),
            "the store refused it: a key past the key limit",
        ),
        (
            Store(CommitFailure::Refused(Refusal::TooLarge(Large::Value))),
            "the store refused it: a value past the value limit",
        ),
        (
            Store(CommitFailure::Refused(Refusal::TooLarge(Large::Ops))),
            "the store refused it: more operations than the commit limit",
        ),
        (
            Store(CommitFailure::Refused(Refusal::TooLarge(Large::Frame))),
            "the store refused it: a frame past the commit limit",
        ),
        (
            Store(CommitFailure::Refused(Refusal::TooLarge(Large::Page))),
            "the store refused it: a page past the page limit",
        ),
        (
            Store(CommitFailure::Refused(Refusal::TooLarge(Large::Range))),
            "the store refused it: a range that ends before it starts",
        ),
        (Store(CommitFailure::Refused(Refusal::Busy)), "the store refused it: its queue is full"),
        (Store(CommitFailure::Refused(Refusal::Unavailable)), "the store refused it: the store is not open"),
        (Store(CommitFailure::Failed), "the store's log failed a write or a sync, so its outcome is uncertain"),
        (
            Store(CommitFailure::Owner { owner: u64::MAX }),
            "the store committed root commit 18446744073709551615 as its commit 18446744073709551615",
        ),
        (Store(CommitFailure::Transcript), "a file step of its transcripts failed"),
        (
            Store(CommitFailure::Check(CommitCheck::Number)),
            "the store's own check refused it: its number is not the store's next",
        ),
        (Store(CommitFailure::Check(CommitCheck::Closed)), "the store's own check refused it: the store is not open"),
        (
            Store(CommitFailure::Check(CommitCheck::Queue)),
            "the store's own check refused it: the queue behind transcripts' syncs is full",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Writes)),
            "the store's own check refused it: more writes than the root's bound",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Switching)),
            "the store's own check refused it: a patch switch is pending",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Encoding)),
            "the store's own check refused it: a key, record or frame past its codec's limits",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Copy)),
            "the store's own check refused it: a copy's first key is not in the map",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Reads)),
            "the store's own check refused it: a read of the map it needs failed",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Capacity)),
            "the store's own check refused it: more operations than the commit's bounds hold",
        ),
        (
            Store(CommitFailure::Check(CommitCheck::Bytes)),
            "the store's own check refused it: its operations past the store's commit bytes",
        ),
        (Store(CommitFailure::Stopped), "the store had stopped at another commit's failure"),
        (Route(RouteStop::Open), "the engine's store routing stopped: the store refused to open while the engine ran"),
        (Route(RouteStop::Secrets), "the engine's store routing stopped: the store's secrets were not ready"),
        (Route(RouteStop::Cursor), "the engine's store routing stopped: a load's cursor did not translate or advance"),
        (Route(RouteStop::Page), "the engine's store routing stopped: a load's page could not be measured or cut"),
    ] {
        let words = commit_why(u64::MAX, why);
        assert_eq!(words, expected, "{why:?}");
        assert!(words.len() <= 160, "{why:?}: {} bytes", words.len());
        let end = End::Stopped(Stop::CommitFailed { number: u64::MAX, why });
        assert_eq!(
            ending_with_binary("temper-engine", Some(&end), false, &directories, b"/data/store", &[]),
            (format!("temper-engine serve: commit 18446744073709551615 failed: {expected}\n"), 1),
            "{why:?}",
        );
    }
}

#[test]
fn byte_and_entry_figures_group_in_threes() {
    for (figure, expected) in [(0, "0"), (999, "999"), (1000, "1,000"), (u64::MAX, "18,446,744,073,709,551,615")] {
        assert_eq!(grouped(figure), expected);
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
        (
            Some(End::Stopped(Stop::CommitFailed {
                number: 7,
                why: CommitStop::Store(jig_protocol_store::CommitFailure::Stopped),
            })),
            false,
            "temper serve: commit 7 failed: the store had stopped at another commit's failure\n",
            1,
        ),
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
    let read = temper_engine_service::HandoverRead {
        forge: 1,
        run: skein_lib::Token::new(7),
        path: b"/fixture/admin_token".as_slice().into(),
        max: 11,
    };
    let read_bytes = |bytes: &[u8]| {
        read_handover_with(&read, |_, max| {
            if bytes.len() > usize::try_from(max).expect("read bound fits usize") {
                Err(skein_shell::Unread::TooLarge { max })
            } else {
                Ok(skein_lib::Secret::new(bytes.into()))
            }
        })
    };
    for (bytes, expected) in [
        (b"test-admin\n".as_slice(), b"test-admin".as_slice()),
        (b"test-admin".as_slice(), b"test-admin".as_slice()),
        (b"token\r\n".as_slice(), b"token\r".as_slice()),
        (b"one\ntwo".as_slice(), b"one\ntwo".as_slice()),
        (b"".as_slice(), b"".as_slice()),
        (b"12345678901".as_slice(), b"12345678901".as_slice()),
    ] {
        let answer = read_bytes(bytes).expect("within read bound").expect("present fixture");
        assert!(answer.expose() == expected, "only one final LF is removed");
    }
    assert!(matches!(read_bytes(b"123456789012"), Err(temper_engine_service::HandoverUnread::TooLarge)));
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
fn a_landings_read_run_names_its_start_end_and_hold() {
    use temper_engine_domain_forge::{LandingReadNote, LandingReadNoteKind as Kind};
    use temper_engine_service::Note;
    const READING: &str = "commits after 0404040404040404040404040404040404040404 up to 0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b, page 2, for change 51";
    const WHY: &str = "past forge.client.rows (8): at least 9 rows";
    let elapsed = skein_lib::Duration::from_secs(37 * 60 + 12);
    for (kind, expected) in [
        (
            Kind::Started { why: WHY.as_bytes().into() },
            format!("temper-box serve: landing 7 round 2 waits for a fresh read: {READING}: {WHY}\n"),
        ),
        (
            Kind::Read { elapsed },
            format!("temper-box serve: landing 7 round 2 has its fresh read after 37m 12s: {READING}\n"),
        ),
        (
            Kind::Held { elapsed, why: WHY.as_bytes().into() },
            format!(
                "temper-box serve: warning: landing 7 round 2 held after 37m 12s waiting for a fresh read: {READING}: {WHY}\n"
            ),
        ),
    ] {
        let note = LandingReadNote { task: 7, round: 2, reading: READING.as_bytes().into(), kind };
        assert_eq!(note_line("temper-box", Note::LandingRead(note)), expected);
    }
}

#[test]
fn unmade_landing_and_dropped_hold_warnings_name_the_repaired_rows() {
    use temper_engine_service::Note;
    assert_eq!(
        note_line(
            "temper-box",
            Note::LandingUnmade {
                task: 10030,
                repository: skein_forge::api::Repository { forge: 1, repository: 7 },
                branch: b"main".as_slice().into(),
                changes: Box::new([8975, 9104, 9885, 10023]),
            }
        ),
        "temper-box serve: warning: landing 10030 into main of repository 1/7 has no task: ended; its changes 8975, 9104, 9885, 10023 wait again\n"
    );
    assert_eq!(
        note_line(
            "temper-box",
            Note::HoldDropped {
                name: temper_engine_domain_forge::Name {
                    forge: 1,
                    repository: 7,
                    what: temper_engine_domain_forge::What::Branch(Box::new([
                        b"temper610030".as_slice().into(),
                        b"l10030".as_slice().into(),
                    ])),
                },
                task: 10030,
            }
        ),
        "temper-box serve: warning: hold on temper610030/l10030 of repository 1/7 by task 10030, which has no task: dropped\n"
    );
}

#[test]
fn forge_link_refusals_name_the_phase_kind_key_and_missing_target() {
    use temper_engine_domain_forge::{Key, MissingLink, Need, RecordKind};
    let arguments =
        [b"--config".as_slice(), b"/configuration", b"--data", b"/data", b"--cache", b"/cache"].map(Box::from);
    let (directories, _) = temper_shell::directories::resolve(&arguments, &[], b"/").expect("resolved directories");
    let end = End::Stopped(Stop::ForgeLink {
        phase: Some(jig_domain_core::start::Phase::Link),
        link: MissingLink {
            kind: RecordKind::Names,
            key: Key::Names(17),
            need: Need::Repository(skein_forge::api::Repository { forge: 1, repository: 999 }),
        },
    });
    for binary in ["temper", "temper-engine"] {
        assert_eq!(
            ending_with_binary(binary, Some(&end), false, &directories, b"/data/deployment", &[]),
            (
                format!(
                    "{binary} serve: restart refused during link: forge Names Names(17) links missing Repository(Repository {{ forge: 1, repository: 999 }})\n"
                ),
                1
            )
        );
    }
}

#[test]
fn start_numbers_name_every_source_kind_and_both_ranges() {
    use jig_domain_core::{
        settings::Setting,
        start::{Failure, Numbered},
    };
    assert_eq!(
        start_failure(&Failure::Numbers {
            settings: Box::new([
                Numbered { setting: Setting::Template(9), changed: true },
                Numbered { setting: Setting::Charter(17), changed: true },
                Numbered { setting: Setting::Policy(40), changed: false },
                Numbered { setting: Setting::Model(70), changed: true },
            ])
        }),
        "settings out of their numbers (the stock's 1 to 32, a deployment's own 33 to 64): changed template 9, changed charter 17, default policy 40, changed model 70"
    );
}

#[test]
fn start_taken_names_settings_and_the_move_before_upgrade() {
    use jig_domain_core::{settings::Setting, start::Failure};
    assert_eq!(
        start_failure(&Failure::Taken { settings: Box::new([Setting::Charter(17), Setting::Template(16)]) }),
        "the stock's new settings take numbers a deployment's own hold (the stock's 1 to 32, a deployment's own 33 to 64): charter 17, template 16; move each to 33 to 64 on the build that runs now, then start this one"
    );
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
            Failure::NeedsPastLimit { held: 4097, limit: 4096 },
            "stored settings' agent needs hold 4097 bytes, past the 4096 byte bound; raise fleet.needs_bytes",
        ),
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
    for (tag, name) in [(21, "Flight"), (9, "Landed"), (20, "Setting"), (47, "ChangeCold"), (48, "ChangeHot")] {
        let end = End::Stopped(Stop::StoreRow {
            row: temper_engine_service::UnreadableRow { key: Box::from([tag]), lowered: None },
        });
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
fn a_landing_refusal_names_its_train_reason_and_stack() {
    for (why, words) in [(Some(jig_domain_tasks::Refusal::Live), "Live"), (None, "unknown")] {
        let line = note_line(
            "temper-box",
            temper_engine_service::Note::LandingRefused {
                task: 10030,
                repository: skein_forge::api::Repository { forge: 1, repository: 7 },
                branch: b"main".as_slice().into(),
                why,
                changes: Box::new([8975, 9104]),
            },
        );
        assert_eq!(
            line,
            format!(
                "temper-box serve: warning: landing 10030 into main of repository 1/7 not formed: jig refused its task: {words}; its changes 8975, 9104 wait\n"
            )
        );
    }
}

#[test]
fn restore_history_failures_name_the_task_check_and_place() {
    use jig_protocol_store::{
        Family,
        restore::{Cause, Failure, HistoryFailure, RowSite},
    };
    let failure = Failure {
        cause: Cause::History { task: 4097, failure: HistoryFailure::PlaceMissing },
        row: Some(RowSite {
            family: Some(Family::Component),
            key: Box::from([0, 10, 0, 0, 0, 0, 0, 0, 16, 1]),
            key_bytes: 10,
            truncated: false,
        }),
    };
    assert_eq!(
        page_failure(Some(&failure)),
        "task 4097 history fetch: history place missing, family Component, key [00, 0a, 00, 00, 00, 00, 00, 00, 10, 01]"
    );
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
fn failed_run_note(detail: &[u8], failure: jig_domain_host::Failure) -> temper_engine_service::Note {
    temper_engine_service::Note::RunFailed(temper_engine_domain::RunDiagnostic {
        task: 3,
        run: 7,
        attempt: 2,
        failure,
        detail: detail.into(),
    })
}

#[test]
fn failed_run_stderr_escapes_control_bytes_on_one_line() {
    assert_eq!(
        note_line(
            "temper",
            failed_run_note(
                b"exit 1: bad\nstream\r\t\0\x1b\x7f\xff",
                jig_domain_host::Failure::Run(jig_domain_host::RunFailure::Model),
            ),
        ),
        "temper serve: task 3 run 7 attempt 2 failed Run(Model): exit 1: bad\\nstream\\r\\t\\x00\\x1b\\x7f\\xff\n"
    );
}

const PREPARATION_DETAIL: &str = "host w1: check out: TimedOut (workspace 3, repository 0 quo\"ted, command clean, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms): error: x";

#[test]
fn failed_prepare_stderr_preserves_fields_and_escapes_every_byte() {
    let detail = [PREPARATION_DETAIL.as_bytes(), b"\n\0\xff"].concat();
    let note = failed_run_note(&detail, jig_domain_host::Failure::Unprepared(jig_domain_host::Preparation::Transient));
    assert_eq!(
        note_line("temper", note),
        "temper serve: task 3 run 7 attempt 2 failed Unprepared(Transient): host w1: check out: TimedOut (workspace 3, repository 0 quo\\\"ted, command clean, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms): error: x\\n\\x00\\xff\n"
    );
}

#[test]
fn failed_run_stderr_keeps_a_bounded_head() {
    // temper/docs/design/domain/worker/checkouts.md, 1.4: the bound counts input bytes.
    const DETAIL_BYTES: usize = 4096;
    const OVERSIZED_BYTES: usize = 5000;
    let prefix = "temper serve: task 3 run 7 attempt 2 failed Run(Model): ";
    let expected_detail = format!(
        "host w1: check out: TimedOut (workspace 3, repository 0 quo\\\"ted, command clean, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms): error: x{}",
        "x".repeat(DETAIL_BYTES - PREPARATION_DETAIL.len()),
    );
    for length in [DETAIL_BYTES, OVERSIZED_BYTES] {
        let mut detail = PREPARATION_DETAIL.as_bytes().to_vec();
        detail.resize(DETAIL_BYTES, b'x');
        detail.resize(length, b'z');
        let note = failed_run_note(&detail, jig_domain_host::Failure::Run(jig_domain_host::RunFailure::Model));
        assert_eq!(note_line("temper", note), format!("{prefix}{expected_detail}\n"));
    }
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
fn a_start_wait_note_is_one_bounded_stderr_line() {
    use temper_engine_service::Note;
    let line = b"starting: reading the forge afresh for 30 s: 2 fresh reads (work/hello branch main, work/hello pull 7), 4 goal branches (task 40, task 10, task 30)";
    assert_eq!(
        note_line("temper-engine", Note::StartWaiting { line: line.as_slice().into() }),
        "temper-engine serve: starting: reading the forge afresh for 30 s: 2 fresh reads (work/hello branch main, work/hello pull 7), 4 goal branches (task 40, task 10, task 30)\n"
    );
    for text in ["starting: a name\nwith\rcontrols\0and é", &"界".repeat(1024)] {
        let line = note_line("temper", Note::StartWaiting { line: text.as_bytes().into() });
        assert!(line.len() <= 1024);
        assert_eq!(line.lines().count(), 1);
        assert!(line.ends_with('\n'));
        assert!(!line.contains('\r'));
        assert!(!line.contains('\0'));
    }
    assert_eq!(
        note_line("custom", Note::StartWaiting { line: b"starting: link for 30 s: task A\ntask B".as_slice().into() }),
        "custom serve: starting: link for 30 s: task A\\ntask B\n"
    );
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

#[test]
fn a_version_refusal_names_the_wire_and_both_versions() {
    let reason = jig_domain_host::Refusal::Version(jig_domain_host::Mismatch {
        wire: b"smith-charter".as_slice().into(),
        version: 5,
        lowest: 4,
        highest: 4,
    });
    assert_eq!(
        note_line("temper", refused_run_note(reason)),
        "temper serve: task 3 run 7 attempt 2 refused: version mismatch: smith-charter 5, the worker reads 4..4\n"
    );
}

#[test]
fn unsent_pause_lines_name_the_mode_worker_and_wire_version() {
    use jig_domain_link::PauseMode;
    use temper_engine_service::Note;
    for (mode, text) in [(PauseMode::Resume, "resume"), (PauseMode::Pause, "pause"), (PauseMode::Drain, "drain")] {
        assert_eq!(
            note_line("temper", Note::WorkerUnpaused { worker: b"w2".as_slice().into(), version: 18, mode }),
            format!(
                "temper serve: worker w2 not told to {text}: jig-worker-wire 18; the engine's placement follows the ask\n"
            )
        );
    }
}

#[test]
fn restart_lines_name_the_wait_or_the_workers_older_wire() {
    use temper_engine_service::Note;
    for (wait, version, expected) in [
        (
            Some(skein_lib::Duration::from_millis(1_200_000)),
            15,
            "custom serve: worker worker-a told the engine restarts: wait 1200000 ms\n",
        ),
        (
            None,
            14,
            "custom serve: worker worker-a not told the engine restarts: jig-worker-wire 14; it keeps its grace\n",
        ),
    ] {
        assert_eq!(
            note_line("custom", Note::WorkerTold { worker: b"worker-a".as_slice().into(), version, wait }),
            expected
        );
    }
}

#[test]
fn a_shutdown_run_is_logged_suspended_but_other_cancellations_stay_failed() {
    use jig_domain_host::{Failure, Reason};
    let note = temper_engine_service::Note::RunFailed(temper_engine_domain::RunDiagnostic {
        task: 812,
        run: 812,
        attempt: 3,
        failure: Failure::Cancelled(Reason::Shutdown),
        detail: b"cut by stop".as_slice().into(),
    });
    assert_eq!(note_line("temper", note), "temper serve: task 812 run 812 attempt 3 suspended: its worker stopped\n");
    assert_eq!(
        note_line("temper", failed_run_note(b"person stopped it", Failure::Cancelled(Reason::Engine))),
        "temper serve: task 3 run 7 attempt 2 failed Cancelled(Engine): person stopped it\n"
    );
}

#[test]
fn full_table_notes_are_single_bounded_serve_lines() {
    for words in [
        "the forge's outbox full for 30 s: 2 of 2, held by entry 7 (task 1), entry 9 (task 2); 1 waiting for room",
        "the forge's read admission full for 60 s: 2 of 2, held by pull 12 of work/hello for task 42, branch main of work/hello",
        "the forge's outbox has room again",
        "the forge's read admission has room again",
    ] {
        let line =
            note_line("temper-engine", temper_engine_service::Note::ForgeTable { words: words.as_bytes().into() });
        assert_eq!(line, format!("temper-engine serve: {words}\n"));
        assert_eq!(line.lines().count(), 1);
        assert!(line.len() <= 1024);
    }
    let words = format!("the forge's outbox full for 2 min: {}\n{}", "é".repeat(1000), "z".repeat(100));
    let line = note_line(
        "temper-box",
        temper_engine_service::Note::ForgeTable { words: words.into_bytes().into_boxed_slice() },
    );
    assert!(line.len() <= 1024);
    assert_eq!(line.lines().count(), 1);
}
