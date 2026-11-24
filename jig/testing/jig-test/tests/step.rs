//! Offline checks of startup, sign-in and shutdown diagnostics.
use jig_test::testing::{self, Given, GivenFile};
use jig_test_kit::diagnostics::{STDERR_TAIL, premature_exit};
use skein_io::kernel::Exit;
use skein_shell::{Roots, Trust, Unread};
use std::path::Path;

fn fixture_roots(path: &Path) -> Roots {
    Roots {
        path: path.into(),
        certificates: Box::new([Box::from(skein_tls_kit::pki::ROOT)]),
        malformed: 0,
        other_blocks: 0,
    }
}

#[test]
fn a_secret_file_readable_by_others_refuses_the_start() {
    let directory = Path::new("/fixture/private-secret");
    let secret = directory.join("worker.secret");
    let private = b"private-worker-value";
    let configuration = directory.join("config.json");
    let agent = directory.join("agent.json");
    let agent_bytes = testing::agent_document_bytes(directory, false);
    let root = directory.join("root.der");
    let bytes = format!(
        r#"{{"workers":[{{
        "engine":{{"socket":"127.0.0.1:7200"}},"name":"worker-a","secret":"{}","slots":1,
        "programs":{{"check_interval_ms":60000}},
        "graces":{{"grace_ms":10000,"agent_stop_ms":5000,"delivery_ms":1000,"save_ms":1000}},
        "agent":{{"program":"jig-test","configuration":"{}","arguments":["agent"]}},
        "limits":{{"profile":"standard"}},"memory_bytes":1152921504606846976
    }}],"application":{{"workers":[{{"root":"agent-root"}}]}}}}"#,
        secret.display(),
        agent.display()
    );
    let given = Given {
        files: &[
            GivenFile { path: &configuration, contents: Ok(bytes.as_bytes()) },
            GivenFile { path: &agent, contents: Ok(&agent_bytes) },
            GivenFile { path: &secret, contents: Err(Unread::Unsafe(skein_io::file::Unsafe::Mode(0o44))) },
        ],
        read_roots: &mut |source, _| {
            assert!(matches!(source, Trust::Der(path) if path == &root), "agent trust: {source:?}");
            Ok(fixture_roots(&root))
        },
    };
    let mut owned = Vec::new();
    let error = testing::prepare_with(&configuration, &mut owned, given).err().expect("start refused");
    assert!(error.contains("Unsafe"), "private read refusal: {error}");
    assert!(error.contains(secret.to_str().expect("fixture path")), "names the refused secret: {error}");
    assert!(!error.contains(std::str::from_utf8(private).expect("fixture secret")), "diagnostic contains no secret");
    assert!(owned.is_empty(), "refusal happens before opening runtime descriptors");
}

#[test]
fn the_live_configurations_are_ones_jig_test_reads() {
    use jig_test_kit::live_sign_in::{Peers, configuration_documents};

    let directory = Path::new("/fixture/live-sign-in");
    let configuration = directory.join("config.json");
    let agent = directory.join("agent.json");
    let root = directory.join("root.der");
    for (peers, address) in
        [(Peers::Real, "192.0.2.1:443"), (Peers::Fake, "127.0.0.1:34443"), (Peers::Refusing, "127.0.0.1:34443")]
    {
        let documents = configuration_documents(directory, address, peers, Path::new("jig-test"));
        let given = Given {
            files: &[
                GivenFile { path: &configuration, contents: Ok(&documents.deployment) },
                GivenFile { path: &agent, contents: Ok(&documents.agent) },
            ],
            read_roots: &mut |source, _| match (peers, source) {
                (Peers::Real, Trust::Machine { ssl_cert_file: None }) => Ok(fixture_roots(Path::new("machine.pem"))),
                (Peers::Fake | Peers::Refusing, Trust::Der(path)) if path == &root => Ok(fixture_roots(path)),
                _ => panic!("unexpected agent trust: {source:?}"),
            },
        };
        assert_eq!(testing::check_with(&configuration, given), Ok(()));
    }
}

#[test]
fn a_premature_exit_names_its_status_milestone_and_safe_stderr_tail() {
    assert_eq!(
        premature_exit(
            Exit::Code(1),
            "sign-in asked",
            b"jig-test: stopped: Listener: an io event no component owns\n\xff"
        ),
        "binary exited before SIGTERM: status=Code(1); last milestone=sign-in asked; stderr tail (up to 4096 bytes):\njig-test: stopped: Listener: an io event no component owns\n\u{fffd}"
    );
    assert_eq!(
        premature_exit(
            Exit::Signal(9),
            "client live",
            b"link=\"https://issuer.example/authorize?secret=private\"\nlast error"
        ),
        "binary exited before SIGTERM: status=Signal(9); last milestone=client live; stderr tail (up to 4096 bytes):\nlink=\"[address omitted]\"\nlast error"
    );
    let mut stderr = vec![b'x'; STDERR_TAIL];
    stderr.extend_from_slice(b"last error");
    let finding = premature_exit(Exit::Code(2), "binary started", &stderr);
    assert_eq!(
        finding.split_once('\n').expect("stderr follows heading").1,
        format!("{}last error", "x".repeat(STDERR_TAIL - 10))
    );
    let mut stderr = b"https://issuer.example/authorize?state=".to_vec();
    stderr.extend(vec![b's'; STDERR_TAIL]);
    stderr.extend_from_slice(b"\nlast error");
    assert_eq!(
        premature_exit(Exit::Code(1), "AccountSignInStarted seen", &stderr),
        "binary exited before SIGTERM: status=Code(1); last milestone=AccountSignInStarted seen; stderr tail (up to 4096 bytes):\n[address omitted]\nlast error"
    );
}

mod live_sign_in {
    use jig_client::contract as c;
    use jig_test_kit::live_sign_in::{failed, run_line};

    mod tests {
        use super::*;

        /// The diagnostic checked by the refused-plan process story is formatted without a process.
        #[test]
        fn a_refused_plan_is_reported_with_the_run_failure() {
            let mut task = jig_client_protocol_link::translate::documents::task(
                jig_client_test_records::task_with_hold_record(),
                &jig_client_protocol_kit::fixtures::link_limits(),
            )
            .expect("owner task fixture");
            let c::ChipPhaseFacts { phase, details } = task.chip.details.as_mut();
            *phase = c::ChipPhase::Active(c::RunStage::BackingOff { until: skein_lib::Wall::from_nanos(17) });
            details.attempt = Some(1);
            details.retry =
                Some(c::RetryNotice::OwnerFailed { class: c::HoldFailure::Run, count: 1, allowance: 3, reason: None });
            details.hold = None;
            details.ending = None;
            details.observation = None;
            task.run_wait = None;
            assert!(failed(&task));
            assert_eq!(
                run_line(&task),
                "phase=Active(BackingOff { until: Wall(17) }); attempt=Some(1); retry=Some(OwnerFailed { class: Run, count: 1, allowance: 3, reason: None }); work=None; hold=None; ending=none; wait=None"
            );
        }
    }
}
