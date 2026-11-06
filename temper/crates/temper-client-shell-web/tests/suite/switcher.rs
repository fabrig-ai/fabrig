//! The switcher's browser form (temper/docs/design/client/README.md, 5.3).
const SHEET: &str = include_str!("../../temper.css");

fn rule(selector: &str) -> &str {
    SHEET.split_once(&format!("{selector} {{")).expect("selector").1.split_once('}').expect("rule").0
}

#[test]
fn escape_alone_replaces_the_visible_cancel() {
    assert!(rule(".temper-project-rename-form > [aria-keyshortcuts=Escape]").contains("display: none;"));
}
#[test]
fn the_rename_field_fills_its_row_and_keeps_its_ring() {
    let field = rule(".temper-project-rename-form > input");
    assert!(field.contains("flex: 1;"));
    assert!(field.contains("min-height: 34px;"));
    assert!(!field.contains("outline"));
    assert!(!SHEET.contains(".temper-project-rename-form input {"));
    assert!(!SHEET.contains(".temper-project-rename-form button {"));
    assert!(!SHEET.contains(".temper-projects > .temper-project-rename-form {"));
    let switcher = SHEET.split_once("/* ==== ui: switcher ==== */").expect("switcher block").1;
    let (switcher, after) = switcher.split_once("/* ==== ui: home ==== */").expect("home block");
    assert!(switcher.contains(".temper-project-rename-form > input"));
    assert!(!after.contains(".temper-project-rename-form"));
}
#[test]
fn the_home_sheet_rings_controls_without_ringing_focus_targets() {
    assert!(SHEET.contains(".temper-welcome-sheet :focus-visible:not([tabindex=\"-1\"])"));
}
#[test]
fn scrolling_panes_contain_their_hidden_absolute_parts() {
    for selector in ["#app > .temper-session .temper-thread > [role=log]", "#app > .temper-session > .temper-doc"] {
        assert!(rule(selector).contains("position: relative;"), "{selector}");
    }
}
