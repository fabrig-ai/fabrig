//! Fixed journey inputs (temper/docs/design/client/journey-walks.md, 3.2).

use rand_chacha::{ChaCha20Rng, rand_core::SeedableRng};
use rsa::{
    RsaPrivateKey,
    pkcs8::{EncodePrivateKey, LineEnding},
};
use skein_lib::{Secret, Wall};
use std::cell::RefCell;
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
    start.deployment["engine"]["clients"]["provider"] =
        serde_json::json!({"key":"provider.key", "clients":[], "addresses":[1]});
    // The native host draws the client; the served mount carries its page facts.
    start.deployment["world_settings"]["page"] =
        "<!doctype html><html><head><title>temper</title></head><body><div id=\"app\"></div></body></html>".into();
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

thread_local! {
    // Amortize RSA generation across the fixed-seed worlds in a print or test run.
    static PROVIDER_PEM: RefCell<Option<(u64, Secret)>> = const { RefCell::new(None) };
}

/// Only walks opt into the box's provider. The key is a synthetic world input,
/// not a host credential.
pub(crate) fn prepare(disk: &mut skein_fake_machine::Machine, document: &documents::Configuration, seed: u64) {
    if document["engine"]["clients"]["provider"].is_null() {
        return;
    }
    PROVIDER_PEM.with_borrow_mut(|cached| {
        if cached.as_ref().is_none_or(|(held, _)| *held != seed) {
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let key = RsaPrivateKey::new(&mut rng, 3072).expect("seeded world RSA key");
            let pem = key.to_pkcs8_pem(LineEnding::LF).expect("world provider PKCS#8");
            *cached = Some((seed, Secret::new(pem.as_bytes().into())));
        }
        let (_, pem) = cached.as_ref().expect("seeded provider PEM");
        temper_playground::machine::lay_handover(disk, b"/setup/provider.key", pem.expose());
    });
}

/// As with the composition's other private files, startup receives the contents
/// through `Files`, not through the real filesystem.
pub(crate) fn files() -> temper_service::settings::Files {
    let files = documents::files();
    let parameters = temper_playground::programs::parameters();
    if parameters.deployment["engine"]["clients"]["provider"].is_null() {
        return files;
    }
    let mut disk = parameters.disk.borrow_mut();
    let root = disk.reopen_root(3);
    let file = disk.open(root, b"provider.key", skein_fake_machine::How::ReadNoFollow).expect("world provider key");
    let stat = disk.stat(file);
    assert_eq!(stat.mode & 0o777, 0o600, "world provider key is private");
    assert_eq!(stat.owner, 1000, "world provider key belongs to the service");
    let key = disk.read(file, 0, 4096).expect("bounded provider PEM");
    disk.close(file);
    disk.close(root);
    let mut entries = skein_lib::List::with_capacity(files.entries.len() + 1);
    for entry in files.entries.into_boxed() {
        entries.push(entry).expect("composition files counted");
    }
    entries.push((b"provider.key".as_slice().into(), Secret::new(key.into()))).expect("provider file counted");
    temper_service::settings::Files { entries }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_walk_opts_into_the_provider_and_served_mount() {
        let walk = start(None).expect("walk start");
        assert_eq!(
            walk.deployment["engine"]["clients"]["provider"],
            serde_json::json!({"key":"provider.key", "clients":[], "addresses":[1]})
        );
        assert!(walk.deployment["world_settings"]["page"].is_string());
        let story = stories::A.start(SEED, WALL);
        assert!(story.deployment["engine"]["clients"]["provider"].is_null());
        assert!(story.deployment["world_settings"]["page"].is_null());
    }

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
