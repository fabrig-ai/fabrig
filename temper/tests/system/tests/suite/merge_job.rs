//! A change's merge job through the engine and worker (domain/testing.md, 2.1).
use std::fmt::Write;
use temper_system_world::composed::{World, merge_job::Row};
use temper_worker_checkout_world::translate::commit;

#[test]
fn a_changes_merge_job_runs_on_its_worker() {
    for (row, delivers, result) in [
        (Row::Settled, true, br#"{"settled":["FILE"]}"#.as_slice()),
        (Row::Left, false, br#"{"settled":[],"left":["FILE"]}"#.as_slice()),
    ] {
        let report = World::merge_job(927, row).run_merge_job();
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
        assert!(report.marked, "the command sees the conflict");
        assert_eq!(report.merge_head.as_deref(), Some(report.base.to_le_bytes().as_slice()));
        let jig_domain_tasks::End::Finished { result: job_result, .. } = &report.job else { panic!("{report:?}") };
        assert_eq!(job_result, &jig_domain_tasks::TaskResult::Job { bytes: result.into() });
        assert_eq!(report.failures, 0);
        if delivers {
            assert_ne!(report.branch, report.head);
            assert_eq!(report.parents, [report.head, report.base]);
            assert_eq!(report.tree.get(b"FILE".as_slice()).map(Vec::as_slice), Some(b"settled\n".as_slice()));
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
