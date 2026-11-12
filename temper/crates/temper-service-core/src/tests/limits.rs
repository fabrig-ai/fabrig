    /// temper/docs/design/domain/worker/jobs.md.
    #[test]
    fn the_engine_declares_the_merge_kind_where_its_tasks_child_holds_it() {
        let mut jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
        let bound = crate::limits::merge_bound();
        assert_eq!(crate::limits::merge_jobs(&jig).as_ref(), [bound]);
        jig.core.tasks.result_bytes = 32;
        assert!(crate::limits::merge_jobs(&jig).is_empty());
        jig.core.tasks.result_bytes = bound.result;
        jig.core.tasks.spec_bytes = bound.spec - 1;
        assert!(crate::limits::merge_jobs(&jig).is_empty());
        jig.core.tasks.spec_bytes = bound.spec;
        assert_eq!(crate::limits::merge_jobs(&jig).as_ref(), [bound]);
    }