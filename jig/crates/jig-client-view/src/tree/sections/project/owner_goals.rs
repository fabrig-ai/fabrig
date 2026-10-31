//! Project goal form over the creating bounds in me; pages.md, 6.2.
use crate::JigBindings;
use crate::words::Words;
use skein_ui::Builder;

use crate::Borrowed;
use jig_client::contract::Project;

pub(super) fn form(b: &mut Builder, client: &Borrowed<'_>, doc: &Project, words: &Words, bindings: &mut JigBindings) {
    super::goals::form(b, client, doc.number, words, bindings);
}
