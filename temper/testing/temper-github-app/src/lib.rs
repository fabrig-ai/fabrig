//! A shared fixture for GitHub stories (testing/real-loop.md and testing/process-tier.md).
mod store;
use std::{net::SocketAddr, path::Path};
use temper_protocol_records::{self as codec, Item};
use temper_protocol_records_forge as forge_records;

/// A fake GitHub App: the layer's App and the records temper keeps for it.
#[derive(Clone, Copy, Debug)]
pub struct App {
    pub id: u64,
    pub slug: &'static str,
    pub bot: u64,
    pub key: usize,
}
/// The fixture App on the layer's first fixed, public test key.
pub const APP: App = App { id: 4242, slug: "temper-test", bot: 4243, key: 0 };

/// The App's six versioned records (protocol/credentials.md, 2.1).
/// `origin` names the forge API, as temper's start checks it. Accepted ids
/// are sorted and deduplicated to match the record's canonical encoding.
#[must_use]
pub fn records(app: &App, forge: u16, accepted: &[u64], origin: SocketAddr) -> Box<[(u64, Box<[u8]>)]> {
    let mut accepted = accepted.to_vec();
    accepted.sort_unstable();
    accepted.dedup();
    let accepted: Vec<u8> = accepted.iter().flat_map(|id| id.to_le_bytes()).collect();
    let origin = forge_records::Origin {
        kind: skein_forge_connection::settings::Kind::GitHub,
        api: origin.to_string().into_boxed_str(),
    }
    .encode();
    let key = skein_fake_forge_github::keys::KEYS.get(app.key).expect("fixture key index");
    [
        (Item::User, app.bot.to_le_bytes().to_vec()),
        (Item::AppId, app.id.to_le_bytes().to_vec()),
        (Item::AppSlug, app.slug.as_bytes().to_vec()),
        (Item::AppKey, key.to_vec()),
        (Item::Accepted, accepted),
        (Item::Origin, origin.into_vec()),
    ]
    .into_iter()
    .map(|(item, bytes)| {
        (
            codec::number(forge, item),
            codec::encode(item, &bytes).expect("fixture item within schema").into_boxed_slice(),
        )
    })
    .collect()
}

/// Keep application records through jig's public store API on a private
/// shell loop. Returns after every keep is durable and all files close.
pub fn seed(data: &Path, records: &[(u64, Box<[u8]>)]) -> Result<(), String> {
    store::run(data, records, false).map(|_| ())
}

/// Commit the fixture's application rows alongside its deployment header before keeping secrets.
pub fn seed_with(
    data: &Path,
    records: &[(u64, Box<[u8]>)],
    writes: Vec<jig_protocol_store::Write>,
) -> Result<(), String> {
    store::run_with(data, records, false, writes).map(|_| ())
}

/// Commit core fixture rows with the deployment's bounds, without keeping secrets.
pub fn seed_configured(
    data: &Path,
    writes: Vec<jig_protocol_store::Write>,
    limits: &jig_protocol_config::Limits,
    own: &temper_engine_service_vocabulary::limits::Limits,
) -> Result<(), String> {
    store::run_configured(data, &[], false, writes, limits, own).map(|_| ())
}
