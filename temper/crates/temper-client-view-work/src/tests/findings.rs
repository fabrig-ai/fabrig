//! Severity and chips (temper/docs/design/client/pages/home.md, 1, Findings, drawn).
use super::fixtures as f;
use crate::{look, pages::findings};
use skein_ui::{Builder, Role, Run};
use temper_client_protocol_wire::v1 as w;

#[test]
fn browser_severity_is_named_and_hinted_with_three_empty_bars() {
    for (severity, style, name, label) in [
        (w::Severity::Low, look::SEVERITY_LOW, b"temper-severity-low".as_slice(), b"low severity".as_slice()),
        (w::Severity::Medium, look::SEVERITY_MEDIUM, b"temper-severity-medium", b"medium severity"),
        (w::Severity::High, look::SEVERITY_HIGH, b"temper-severity-high", b"high severity"),
    ] {
        let mut b = Builder::new(&f::tree_limits());
        findings::severity(&mut b, &f::browser_words(), severity);
        let tree = b.finish();
        assert_eq!(tree.nodes().len(), 4);
        let glyph = tree.nodes().first().expect("severity glyph");
        assert_eq!(glyph.role, Role::Text);
        assert_eq!(glyph.styles.as_slice(), &[look::SEVERITY, style]);
        assert_eq!(look::NAMES.style(look::SEVERITY), Some(b"temper-severity".as_slice()));
        assert_eq!(look::NAMES.style(style), Some(name));
        assert_eq!(glyph.name.as_deref(), Some(label));
        assert_eq!(glyph.hint.as_deref(), Some(label));
        assert!(glyph.text.is_empty());
        assert_eq!(glyph.size, 4);
        for bar in tree.nodes().iter().skip(1) {
            assert_eq!(bar.role, Role::Text);
            assert_eq!(bar.size, 1);
            assert!(bar.text.is_empty());
            assert!(bar.name.is_none());
            assert!(bar.styles.is_empty());
        }
    }
}

#[test]
fn terminal_severity_leaves_only_the_paragraphs_words() {
    for severity in [w::Severity::Low, w::Severity::Medium, w::Severity::High] {
        let mut b = Builder::new(&f::tree_limits());
        b.open(Role::Paragraph);
        findings::severity(&mut b, &f::words(), severity);
        b.text(Run::Plain, b"x");
        b.close();
        let tree = b.finish();
        assert_eq!(tree.nodes().len(), 2);
        let mut print = skein_ui_text::Print::new(&f::tree_limits()).expect("print bounds");
        skein_ui_text::render(&tree, b"temper", None, &mut print);
        assert_eq!(print.text.as_slice(), b"== temper == \nx\n");
    }
}

#[test]
fn browser_chip_takes_its_severitys_ground_with_glyph_before_label() {
    for (severity, style, name, glyph_style, label) in [
        (w::Severity::Low, look::CHIP_LOW, b"temper-chip-low".as_slice(), look::SEVERITY_LOW, b"5 low".as_slice()),
        (w::Severity::Medium, look::CHIP_MEDIUM, b"temper-chip-medium", look::SEVERITY_MEDIUM, b"4 medium"),
        (w::Severity::High, look::CHIP_HIGH, b"temper-chip-high", look::SEVERITY_HIGH, b"2 high"),
    ] {
        let mut b = Builder::new(&f::tree_limits());
        findings::chip(&mut b, &f::browser_words(), severity, label);
        let tree = b.finish();
        assert_eq!(tree.nodes().len(), 6);
        let mut nodes = tree.nodes().iter();
        let chip = nodes.next().expect("chip");
        assert_eq!(chip.role, Role::Text);
        assert_eq!(chip.size, 6);
        assert_eq!(chip.styles.as_slice(), &[look::CHIP, style]);
        assert_eq!(look::NAMES.style(style), Some(name));
        let glyph = nodes.next().expect("glyph first");
        assert_eq!(glyph.styles.as_slice(), &[look::SEVERITY, glyph_style]);
        assert_eq!(glyph.size, 4);
        let text = nodes.nth(3).expect("label after the glyph's three bars");
        assert_eq!(text.role, Role::Text);
        assert_eq!(text.size, 1);
        assert_eq!(f::nodes_text(core::slice::from_ref(text)).as_ref(), label);
        assert!(nodes.next().is_none());
    }
}

#[test]
fn terminal_chip_is_its_label_alone() {
    let mut b = Builder::new(&f::tree_limits());
    b.open(Role::Paragraph);
    findings::chip(&mut b, &f::words(), w::Severity::Medium, b"4 medium");
    b.close();
    let tree = b.finish();
    assert_eq!(tree.nodes().len(), 2);
    let label = tree.nodes().last().expect("label");
    assert_eq!(label.role, Role::Text);
    assert!(label.styles.is_empty());
    let mut print = skein_ui_text::Print::new(&f::tree_limits()).expect("print bounds");
    skein_ui_text::render(&tree, b"temper", None, &mut print);
    assert_eq!(print.text.as_slice(), b"== temper == \n4 medium\n");
}
