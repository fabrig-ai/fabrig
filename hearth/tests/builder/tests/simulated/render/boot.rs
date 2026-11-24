use hearth_builder_world::fixtures::{self, lines_starting};

#[test]
fn the_boot_part_lists_the_parameters_in_order() {
    let rendered = super::render(&fixtures::command_line_overrides(false));
    assert_eq!(
        lines_starting(&rendered.stdout, &["boot ", "  params", "  mutable"]),
        "boot         part 68ef6aa14d25\n  params     quiet audit=0 console=ttyS0,115200n8\n  mutable    false\n"
    );
}

#[test]
fn an_order_past_999_prints_its_fault_at_the_field_and_exits_2() {
    let rendered = super::render(&fixtures::command_line_overrides(true));
    assert_eq!(rendered.stdout, b"tasks/params/task.json:1:41: x1-prod: params: past its max 999\n");
    assert_eq!(rendered.code, Some(2));
}

#[test]
fn placement_mutable_true_prints_mutable_true() {
    let placement = fixtures::File {
        path: "hosts/x1/x1-prod.json",
        body: br#"{"placement": {"kind": "metal", "firmware": "efi", "hardware": "7th-gen", "mutable": true}}"#,
        mode: 0o644,
    };
    let rendered = super::render(&[placement]);
    assert_eq!(lines_starting(&rendered.stdout, &["  mutable"]), "  mutable    true\n");
}
