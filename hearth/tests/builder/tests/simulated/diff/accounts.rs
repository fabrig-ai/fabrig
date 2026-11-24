use hearth_builder_world::{
    fixtures::{self, accounts},
    protocol,
};

#[test]
fn hearth_diff_names_each_account_group_and_member_moved() {
    let files = fixtures::thin_with(&[accounts::ACCOUNTS]);
    let later = fixtures::edited(&files, &[accounts::ACCOUNTS_MOVED], &[]);
    let run = protocol::run(&fixtures::diffing(&files, &later), fixtures::SIMULATION_SEED);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  config "]),
        concat!(
            "  config   - groups ops                             1500\n",
            "  config   ~ users alice                            1001 alice /home/alice /bin/bash\n",
            "  config   - members alice ops\n",
            "  config   + members alice video\n"
        )
    );
    assert_eq!(run.code, Some(1));
}
