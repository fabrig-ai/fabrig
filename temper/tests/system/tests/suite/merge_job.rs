//! A change's merge job through the engine and worker (testing/README.md, 2.1).
use std::fmt::Write;
use temper_system_world::composed::{World, merge_job::Row};
use temper_worker_checkout_world::translate::commit;

#[test]
fn a_changes_merge_job_runs_on_its_worker() {
    for (row, delivers, result) in [
        (Row::PastTheBound, true, b"".as_slice()),
        (Row::Clean, true, br#"{"settled":[]}"#.as_slice()),
        (Row::OtherBranch, true, br#"{"settled":["FILE"]}"#.as_slice()),
        (Row::Settled, true, br#"{"settled":["FILE"]}"#.as_slice()),
        (Row::Left, false, br#"{"settled":[],"left":["FILE"]}"#.as_slice()),
    ] {
        let report = World::merge_job(927, row).run_merge_job();
        let files: Vec<_> = if row == Row::Clean {
            Vec::new()
        } else {
            let count = if row == Row::PastTheBound { report.conflict_bound + 1 } else { 1 };
            (0..count).map(|index| if index == 0 { "FILE".to_owned() } else { format!("FILE{index}") }).collect()
        };
        let past_result = format!("{{\"settled\":{}}}", serde_json::to_string(&files).expect("settled JSON"));
        let result = if row == Row::PastTheBound { past_result.as_bytes() } else { result };
        assert_eq!(report.conflicts, files.iter().map(|path| path.as_bytes().to_vec()).collect::<Vec<_>>(), "{row:?}");
        if delivers {
            let staged: Vec<_> = files.iter().flat_map(|path| [path.as_bytes(), b"\0settled\n"].concat()).collect();
            assert_eq!(report.staged, staged, "{row:?}: every settled file staged");
        }
        assert!(result.len() <= 512);
        assert!(report.command_found, "{row:?}: command_found: {report:?}");
        assert_eq!(
            report.hello,
            [jig_domain_link::JobBound {
                kind: jig_domain_link::JobKind(3),
                spec: 4096,
                result: 512,
                deadline: skein_lib::Duration::from_secs(600),
            }],
            "{row:?}"
        );
        assert_eq!(report.declared, report.hello);
        assert_eq!(report.program, [report.directory.as_slice(), b"/repo/.temper/merge"].concat());
        assert_eq!(
            report.args,
            [report.program.clone(), b"merge".to_vec(), [report.directory.as_slice(), b"/request.json"].concat()]
        );
        let request = format!("{{\"version\":1,\"head\":\"{}\",\"base\":\"{}\"}}", hex(report.head), hex(report.base));
        assert_eq!(report.request, request.as_bytes());
        assert_eq!(report.marked, row != Row::Clean, "{row:?}: conflict markers");
        assert_eq!(report.merge_head.as_deref(), Some(report.base.to_le_bytes().as_slice()));
        assert_eq!(report.failures, 0, "{row:?}: {report:?}");
        let jig_domain_tasks::End::Finished { result: job_result, .. } = &report.job else { panic!("{report:?}") };
        assert_eq!(job_result, &jig_domain_tasks::TaskResult::Job { bytes: result.into() });
        if row == Row::OtherBranch {
            assert!(report.cached_repository.is_some(), "producer left its clone cached");
            assert_eq!(report.base_present_before, Some(false), "base absent before preparation");
            assert_eq!(report.base_fetch_repository, report.cached_repository, "job reused the producer's clone");
            assert_eq!(report.base_present_at_fetch, Some(false), "branch fetch did not import the base");
        }
        if delivers {
            assert_ne!(report.branch, report.head);
            assert_eq!(report.parents, [report.head, report.base]);
            for path in &files {
                assert_eq!(report.tree.get(path.as_bytes()).map(Vec::as_slice), Some(b"settled\n".as_slice()));
            }
            if row == Row::Clean {
                assert_eq!(report.catch_up, Some(commit(report.base).raw()));
                assert!(!report.base_tree.contains_key(b"FILE".as_slice()), "the goal's file is its own");
                assert!(!report.head_tree.contains_key(b"MAIN".as_slice()), "main advanced outside the goal's tree");
                assert_eq!(report.tree.get(b"FILE".as_slice()), report.head_tree.get(b"FILE".as_slice()));
                assert_eq!(report.tree.get(b"FILE".as_slice()).map(Vec::as_slice), Some(b"goal\n".as_slice()));
                assert_eq!(report.tree.get(b"MAIN".as_slice()).map(Vec::as_slice), Some(b"main work\n".as_slice()));
            }
            assert_eq!(report.continued, Some(commit(report.branch).raw()));
            assert!(report.resolution.is_none());
        } else {
            assert_eq!(report.branch, report.head);
            assert_eq!(report.resolution_base, Some(commit(report.base).raw()));
            assert!(report.resolution.as_ref().expect("resolution made").ends_with(result));
        }
    }
}

fn hex(commit: u64) -> String {
    let mut text = String::new();
    for byte in skein_fake_forge_domain::object_bytes(commit) {
        write!(&mut text, "{byte:02x}").expect("commit hex");
    }
    text
}
