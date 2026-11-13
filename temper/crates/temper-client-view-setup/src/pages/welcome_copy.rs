//! The welcome's command block and its Copy (temper/docs/design/client/pages/README.md).
use crate::{accounts::Bindings, words::Words};
use skein_ui::Builder;
use temper_client_domain::Domain;

pub(super) fn block(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, words: &[u8]) {
    crate::flow::command(b, root, w, bindings, words, false);
}
