//! Confirmation placement (temper/docs/design/client/README.md, 3.6).
const SHEET: &str = include_str!("../../temper.css");

#[test]
fn confirmations_keep_auto_margins_in_every_page() {
    let rule = SHEET.split_once("#app dialog.temper-confirm {").expect("confirmation selector").1;
    let declarations = rule.split_once('}').expect("confirmation declarations").0;
    assert!(declarations.contains("margin: auto;"), "confirmations must override page content margins");
}
