//! Stories compiled as data (temper/docs/design/playground.md, 5.1–5.4).
use crate::{Fakes, Links, Start, documents};
use skein_lib::Wall;
use temper_fake::{credential, forge, llm};

/// A deployment, its scripted neighbours and the visitor's description.
pub struct Story {
    pub name: &'static str,
    pub description: &'static str,
    pub deployment: &'static [u8],
    pub llm: &'static [u8],
    pub forge: &'static [u8],
}

/// The box's three suggested requests, paced for the browser.
pub const A: Story = Story {
    name: "a",
    description: include_str!("../stories/a/description.txt"),
    deployment: include_bytes!("../stories/a/deployment.json"),
    llm: include_bytes!("../stories/a/llm.json"),
    forge: include_bytes!("../stories/a/forge.json"),
};

/// Stories offered by the playground's frame.
#[must_use]
pub fn all() -> &'static [Story] {
    &[A]
}

impl Story {
    /// Decode the compiled documents and build the same peers as `temper fake`.
    #[must_use]
    pub fn start(&self, seed: u64, wall: Wall) -> Start {
        let seeds = documents::seeds(seed);
        let llm = llm::Document::decode(self.llm).expect("compiled story's LLM document");
        let forge = forge::Document::decode(self.forge).expect("compiled story's forge document");
        let llm = llm::peer(llm::configuration(&llm, documents::KEY, seeds.llm)).expect("story's LLM peer");
        let token = credential::credential(seeds.forge, credential::ADMIN_TOKEN);
        let (peer, shared) = forge::peer(&forge, &token, seeds.forge).expect("story's forge peer");
        let (_, table) = forge.seed(seeds.forge);
        let forge = temper_fake_git::Typed::new(shared, &table);
        Start { seed, wall, deployment: self.deployment.into(), fakes: Fakes { llm, forge, peer }, links: Links::Print }
    }
}

#[cfg(test)]
#[path = "tests/stories.rs"]
mod tests;
