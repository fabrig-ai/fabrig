//! The command reserves room for its longest Copy label (temper/docs/design/client/pages/README.md).
#[test]
fn the_welcome_command_reserves_the_copy_inset_label_and_gap_at_every_width() {
    let css = include_str!("../../temper.css");
    let mut padding = Vec::new();
    for rule in css.split('}') {
        let Some((selector, declarations)) = rule.rsplit_once('{') else { continue };
        if !selector.split(".temper-welcome-cmd").skip(1).any(|tail| !tail.starts_with('-')) {
            continue;
        }
        for declaration in declarations.split(';') {
            let Some((property, value)) = declaration.trim().split_once(':') else { continue };
            if property.starts_with("padding") {
                padding.push((property.trim(), value.trim()));
            }
        }
    }
    assert_eq!(padding, [("padding", "12px 116px 12px 14px")], "only the command’s rule sets its padding");
    let right = padding[0].1.split_whitespace().nth(1).unwrap().trim_end_matches("px").parse::<f64>().unwrap();
    let inset = 12.0;
    let longest_label = 13.0 * 7.2;
    let gap = 10.0;
    assert!(right >= inset + longest_label + gap, "the longest label must leave a gap beside selectable code");
    let (_, button) = css.split_once("#app:has(> .temper-welcome) .temper-welcome-copy {").expect("copy button rule");
    let button = button.split('}').next().unwrap();
    assert!(button.contains("right: 12px;"));
    assert!(button.contains("font: 600 12px/1.58 var(--gf-mono);"));
}
