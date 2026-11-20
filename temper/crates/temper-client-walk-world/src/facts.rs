//! Fixed journey inputs (temper/docs/design/client/journey-walks.md, 3.2).

use skein_lib::Wall;
use temper_fake::{backends, credential, llm, oauth};
use temper_playground::{documents, stories, story::Start};

pub const SEED: u64 = 7;
pub const WALL: Wall = Wall::from_nanos(1_767_690_000_000_000_000);
pub const SERVICE: &str = "127.0.0.1:10001";
pub const FORGE: &str = "127.0.0.1:10002";
pub const ISSUER: &str = "127.0.0.1:10003";
pub const SIZE: skein_ui_screen::Size = skein_ui_screen::Size { columns: 80, rows: 24 };
pub const VERSION: &[u8] = b"0.0.0 (000000000000)";
pub const SCHEME: temper_client_tui::Scheme = temper_client_tui::Scheme::Dark;

/// Each terminal run draws a distinct, reproducible sequence of request keys.
#[must_use]
pub fn tui_seed(before: u64) -> u64 {
    SEED.wrapping_add(1).wrapping_add(before)
}

/// Own story A's start with a box's bounds, its listener and optional model scripts.
pub fn start(scripts: Option<&[u8]>) -> Result<Start, String> {
    let mut start = stories::A.start_at(SEED, WALL, FORGE.parse().expect("fixed forge address"));
    let stock = documents::Configuration::stock(start.deployment.document.clone());
    start.deployment.jig_over("tasks.spec_bytes", stock.jig_value("tasks.spec_bytes"));
    start.deployment.jig_over("people.amendment_bytes", stock.jig_value("people.amendment_bytes"));
    // The owned preparation's default person, on the composition's same forge.
    start.fakes.peer.setup_mut().account(b"owner", false, b"");
    start.deployment.document["application"]["clients"]["listener"]["socket"] = SERVICE.into();
    let document = llm::Document::decode_with_scripts(stories::A.llm, scripts).map_err(|why| format!("{why:?}"))?;
    let mut configuration = llm::configuration(&document, documents::KEY, documents::seeds(SEED).llm);
    // The box's issuer and first added model key use the same seed. Keep the
    // story's script seed independent, adding only this accepted synthetic bearer.
    let bearer = credential::credential(SEED, credential::added_key(0));
    configuration.added_keys = Box::new([skein_lib::Secret::new(bearer.as_slice().into())]);
    start.fakes.llm = llm::recording_peer(configuration).map_err(|why| format!("{why:?}"))?;
    let issuer = serde_json::json!({"listen":{"socket":ISSUER},"redirect_port":1455,"redirect_path":"auth/callback"});
    start.fakes.issuer = Some(
        oauth::Document::decode(&serde_json::to_vec(&issuer).expect("issuer JSON"))
            .map_err(|why| format!("{why:?}"))?,
    );
    let peers = backends::Peers {
        github: start.deployment.forge,
        forgejo: start.deployment.forge,
        oauth: ISSUER.parse().expect("issuer socket"),
        llm: document.listen(),
    };
    let providers = backends::providers(&peers).map_err(|why| format!("{why:?}"))?;
    start.deployment["world_settings"]["providers"] = serde_json::from_slice(&providers).expect("providers JSON");
    start.deployment["world_settings"]["issuer"] = ISSUER.into();
    Ok(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walks_lift_draft_goal_bounds_to_deployment_defaults_but_story_a_keeps_its_bounds() {
        let walk = start(None).expect("walk start");
        let stock = documents::Configuration::stock(walk.deployment.document.clone());
        let walk_engine = documents::settings::configuration(&walk.deployment);
        assert_eq!(u64::from(walk_engine.limits.core.tasks.spec_bytes), stock.jig_value("tasks.spec_bytes"));
        assert_eq!(
            u64::from(walk_engine.limits.core.people.amendment_bytes),
            stock.jig_value("people.amendment_bytes")
        );
        let story = stories::A.start(SEED, WALL);
        let story_engine = documents::settings::configuration(&story.deployment);
        assert_eq!(story_engine.limits.core.tasks.spec_bytes, 2048);
        assert_eq!(story_engine.limits.core.people.amendment_bytes, 1218);
    }
}
