mod tests {
    use jig_test_store_files::{KillFrom, component_limits, testing::fork_first_operation};

    #[test]
    fn a_failed_operation_child_names_its_cut() {
        let limits = component_limits(&jig_test_profile::limits());
        let (result, child) = fork_first_operation(&limits, KillFrom::Commit(1));
        if child {
            std::process::exit(23);
        }
        assert_eq!(result, Err("forked disk cut (Commit(1), 0) failed: Code(23)".to_owned()));
    }
}
