use hearth_builder_world::fixtures::{ids, lines_starting};

#[test]
fn hearth_render_seeds_the_owners_of_what_persists() {
    let rendered = super::render(&[ids::SECRETS_FIXED]);
    assert_eq!(lines_starting(&rendered.stdout, &["  seed "]), "  seed       group opendkim 120\n");
}

#[test]
fn an_owner_with_no_fixed_id_prints_its_fault_at_the_owner_and_exits_2() {
    let rendered = super::render(&[ids::SECRETS_NO_ID]);
    assert_eq!(rendered.stdout, b"tasks/motd/task.json:1:71: x1-prod: motd: postfix owns mail/tls/mx1.key and has no fixed id: no user postfix is declared\n");
    assert_eq!(rendered.code, Some(2));
}
