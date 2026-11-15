use crate::tests::fixtures;

#[test]
fn a_render_made_file_is_kept_under_its_destination() {
    use crate::builder::vocabulary::Event;
    let (_, _, event) = fixtures::produced_keep_at(b"/etc/subuid");
    let Event::Produced { path, .. } = event else { panic!("produced") };
    assert_eq!(path.as_ref(), b"/etc/subuid");
}
