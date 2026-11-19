//! Fixed journey inputs (temper/docs/design/client/journey-walks.md, 3.2).

use skein_lib::Wall;
use temper_fake::llm;
use temper_playground::{documents, stories, story::Start};

pub const SEED: u64 = 7;
pub const WALL: Wall = Wall::from_nanos(1_767_690_000_000_000_000);
pub const SERVICE: &str = "127.0.0.1:10001";

/// Own story A's start, changing its listener and, when supplied, its model scripts.
pub fn start(scripts: Option<&[u8]>) -> Result<Start, String> {
    let mut start = stories::A.start(SEED, WALL);
    start.deployment.document["application"]["clients"]["listener"]["socket"] = SERVICE.into();
    if let Some(scripts) = scripts {
        let document =
            llm::Document::decode_with_scripts(stories::A.llm, Some(scripts)).map_err(|why| format!("{why:?}"))?;
        start.fakes.llm = llm::peer(llm::configuration(&document, documents::KEY, documents::seeds(SEED).llm))
            .map_err(|why| format!("{why:?}"))?;
    }
    Ok(start)
}
