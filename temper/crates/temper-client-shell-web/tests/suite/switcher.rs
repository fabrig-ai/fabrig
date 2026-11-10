//! The switcher's browser form (temper/docs/design/client/view.md, 3).
const TASK_PAGES: &str = include_str!("../../task-pages.css");
const SHEET: &str = include_str!("../../temper.css");

fn rule(selector: &str) -> &str {
    SHEET.split_once(&format!("{selector} {{")).expect("selector").1.split_once('}').expect("rule").0
}

#[test]
fn the_new_box_plus_is_drawn_without_accessible_text() {
    let plus = rule("#app > .temper-rail > .temper-rail-new > :is(a, [aria-haspopup])::before");
    assert!(plus.contains("content: '';"), "the decorative plus must not add text to either control's name");
    assert!(plus.contains("display: inline-block;"), "the empty decoration needs a box");
    assert_eq!(plus.matches("linear-gradient(currentColor, currentColor)").count(), 2, "the plus has two strokes");
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
    for selector in [
        "#app > .temper-session .temper-thread > [role=log]",
        ".temper-agent-page .temper-thread > [role=log]",
        "#app > .temper-session > .temper-doc",
    ] {
        assert!(rule(selector).contains("position: relative;"), "{selector}");
        assert!(rule(selector).contains("overflow-y: auto;"), "{selector}");
    }
    // The log is what the page keys scroll; nothing around it scrolls the column (client/pages/task.md, 2).
    for thread in ["#app > .temper-session .temper-thread:has(> [role=log])", ".temper-agent-page .temper-thread"] {
        assert!(rule(thread).contains("overflow: hidden;"), "{thread}");
    }
    assert!(!TASK_PAGES.contains("overflow: visible"));
    assert!(!TASK_PAGES.contains("> .temper-thread {"));
}

#[test]
fn repository_links_underline_only_on_hover_and_counts_never_shrink() {
    assert!(rule(".temper-projects-repo a").contains("color: inherit;"));
    assert!(rule(".temper-projects-repo a").contains("text-decoration: none;"));
    assert!(rule(".temper-projects-repo a:hover").contains("text-decoration: underline;"));
    assert!(rule("a.temper-repository-link").contains("text-decoration: none;"));
    assert!(rule("a.temper-repository-link:hover").contains("text-decoration: underline;"));
    assert!(rule("#app > .temper-rail .temper-projects-repo > .temper-projects-more").contains("flex: none;"));
}
