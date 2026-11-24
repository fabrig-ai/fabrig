use hearth_builder_world::{
    fixtures::{self, data},
    protocol,
};

#[test]
fn hearth_diff_names_a_data_entry_moved_to_another_class() {
    let files = fixtures::thin_with(&[data::DATA]);
    let later = fixtures::edited(&files, &[data::DATA_MOVED], &[]);
    let run = protocol::run(&fixtures::diffing(&files, &later), fixtures::SIMULATION_SEED);
    assert_eq!(
        fixtures::lines_starting(&run.stdout, &["  data "]),
        "  data     ~ iwd                                    data /var/lib/iwd\n"
    );
    assert_eq!(run.code, Some(1));
}
