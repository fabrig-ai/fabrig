//! Stories compiled as data (temper/docs/design/playground.md, 5.1–5.4).
use crate::{Fakes, Links, Start, documents};
use core::net::SocketAddr;
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
    // The closer procedure and close job give the story seventeen authority executor places.
    // Its task executor places hold the same eighteen-kind stock delegation.
    // Two added 8-byte executor places make its largest stock chat payload 1,117 bytes.
    deployment: include_bytes!("../stories/a/deployment.json"),
    llm: include_bytes!("../stories/a/llm.json"),
    forge: include_bytes!("../stories/a/forge.json"),
};

/// Stories offered by the playground's frame.
#[must_use]
pub fn all() -> &'static [Story] {
    &[A]
}

/// The story a start message names, by its `name`.
#[must_use]
pub fn named(name: &[u8]) -> Option<&'static Story> {
    all().iter().find(|story| story.name.as_bytes() == name)
}

/// The synthetic admin credential accepted by this story's forge peer.
#[must_use]
pub fn admin_token(seed: u64) -> [u8; 40] {
    credential::credential(documents::seeds(seed).forge, credential::ADMIN_TOKEN)
}

impl Story {
    #[must_use]
    pub fn configuration(&self) -> crate::documents::Configuration {
        crate::documents::configuration::story_a(serde_json::from_slice(self.deployment).expect("story document"))
    }

    /// Decode the compiled documents and build the same peers as `temper fake`.
    #[must_use]
    pub fn start(&self, seed: u64, wall: Wall) -> Start {
        self.start_at(seed, wall, documents::FORGE)
    }

    /// Start the composition at the caller's forge address (temper/docs/design/playground.md, 4).
    #[must_use]
    pub fn start_at(&self, seed: u64, wall: Wall, address: SocketAddr) -> Start {
        let seeds = documents::seeds(seed);
        let llm = llm::Document::decode(self.llm).expect("compiled story's LLM document");
        let forge = forge::Document::decode(self.forge).expect("compiled story's forge document");
        let llm = llm::peer(llm::configuration(&llm, documents::KEY, seeds.llm)).expect("story's LLM peer");
        let token = admin_token(seed);
        let (mut peer, shared) = forge::peer(&forge, &token, seeds.forge).expect("story's forge peer");
        let (_, mut table) = forge.seed(seeds.forge);
        table.base = format!("http://{address}").into_bytes().into();
        let transport = temper_fake_git::Typed::new(shared.clone(), &table);
        if address != forge.listen() {
            peer = skein_fake_peers::forge::Peer::new(
                address,
                temper_fake::limits::peer(),
                table,
                forge::git_limits(),
                shared,
                seeds.forge,
            )
            .expect("story's forge peer at the caller's address")
            .without_records();
            forge::setup(peer.setup_mut(), &token);
        }
        let mut deployment = self.configuration();
        deployment.forge = address;
        Start {
            seed,
            wall,
            deployment,
            fakes: Fakes { llm, forge: transport, peer, issuer: None },
            links: Links::Print,
        }
    }
}

#[cfg(test)]
#[path = "tests/stories.rs"]
mod tests;
