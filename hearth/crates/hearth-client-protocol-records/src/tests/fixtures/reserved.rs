use crate::errors::{Kind, Reserved, Rule};
pub const CMDLINE_BYTES: u32 = 2049; // One past hearth/docs/design/domain/machine/boot.md, 3.2's command-line bound.
pub fn error(rule: Rule) -> crate::errors::Error {
    let mut error = super::duplicate();
    error.kind = Kind::Reserved(Reserved { rule, what: Box::from(b"rootwait".as_slice()) });
    error.at = None;
    error.also = None;
    error
}
