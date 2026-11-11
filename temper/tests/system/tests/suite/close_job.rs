//! A close job's delivery (temper/docs/design/domain/worker/jobs.md).
use temper_system_world::composed::{World, close_job::Row};

#[test]
fn a_close_changes_job_runs_on_its_worker() {
    for row in [Row::Settled, Row::Unchanged] {
        let report = World::close_job(928, row).run_close_job();
        assert_eq!(report.hello, [temper_service::limits::merge_bound(), temper_service::limits::close_bound()]);
        assert_eq!(report.program, [report.directory.as_slice(), b"/repo/.temper/close"].concat());
        assert_eq!(
            report.args,
            [report.program.clone(), b"close".to_vec(), [report.directory.as_slice(), b"/request.json"].concat()]
        );
        assert_eq!(report.request, br#"{"version":1,"goal":1,"changes":[{"task":9,"landed":true}]}"#);
        let result = if row == Row::Settled {
            br#"{"settled":["status.md"]}"#.as_slice()
        } else {
            br#"{"settled":[]}"#.as_slice()
        };
        assert_eq!(report.job_result, jig_domain_tasks::TaskResult::Job { bytes: result.into() });
        if row == Row::Settled {
            assert_ne!(report.head, Some(report.tip));
            assert_eq!(report.parents, [report.tip]);
            assert_eq!(report.message, b"Settle goal 1's status");
            assert_eq!(report.tree.get(b"status.md".as_slice()).map(Vec::as_slice), Some(b"settled\n".as_slice()));
            assert_eq!(report.pushes, 1);
            assert!(report.opened, "{report:?}");
            assert!(!report.no_change);
        } else {
            assert!(report.head.is_none());
            assert_eq!(report.pushes, 0);
            assert!(!report.opened);
            assert!(report.no_change, "{report:?}");
        }
    }
}
