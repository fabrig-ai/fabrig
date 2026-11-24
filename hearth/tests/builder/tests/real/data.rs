use hearth::Form;
use hearth_builder_world::fixtures::{self, Repository, data};
#[test]
fn hearth_render_lists_the_data_entries() {
    let repository = Repository::make("data", &fixtures::thin_with(&[data::DATA]));
    let rendered = fixtures::render(&repository, &[], Form::Text);
    assert_eq!(
        fixtures::lines_starting(&rendered.stdout, &["  iwd ", "  journal "]),
        "  iwd        /var/lib/iwd\n  journal    /var/log/journal\n"
    );
}
