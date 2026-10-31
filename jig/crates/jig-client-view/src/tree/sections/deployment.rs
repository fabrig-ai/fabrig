//! Whole host, account and capacity facts, with connector slots (pages.md, 7).
use crate::JigBindings;
use crate::words::Words;
use crate::words::phrases::jig::JigPhrase;
use skein_ui::Builder;

use crate::Borrowed;
/// Show the admitted deployment document and its connector-owned parts.
pub(crate) fn section(b: &mut Builder, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) -> bool {
    let deployment_page = client.deployment_page();
    if !client.shows(crate::state::page::Section::Deployment) {
        return false;
    }
    crate::tree::pages::work::section(b, words, JigPhrase::Deployment, crate::regions::JigRegion::Deployment);
    super::status(b, client, words, crate::state::page::Section::Deployment);
    if let Some(doc) = &deployment_page.owner {
        crate::tree::fragments::deployment::render(b, client, doc, words, bindings);
        b.close();
        return true;
    }
    words.status(b, JigPhrase::LoadingObject, &[]);
    b.close();
    true
}
