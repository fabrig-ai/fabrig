//! Load the B0 twin for the box proof.
use crate::llm;

#[must_use]
pub fn from_document(bytes: &[u8]) -> llm::Recipe {
    let document = temper_fake::llm::Document::decode(bytes).expect("B0 LLM document");
    let mut recipe = llm::scripted(Box::new([]));
    recipe.domain = document.domain(17);
    recipe.domain_limits = temper_fake::llm::LIMITS;
    recipe
}
