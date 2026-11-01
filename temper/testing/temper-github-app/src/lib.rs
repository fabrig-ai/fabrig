//! A shared fixture for GitHub stories (domain/testing.md, 2.4 and 2.6).
mod forges;
mod store;
pub use forges::{forgejo, forgejo_records, github, seed_forges};
use std::{net::SocketAddr, path::Path};
use temper_protocol_records::{self as codec, Item};

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
    let origin = codec::Origin {
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
    store::run(data, &[], records, false).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use temper_engine_domain_forge as forge;

    #[test]
    fn the_seeded_forges_are_what_temper_restores() {
        let api = "127.0.0.1:4321".parse().unwrap();
        let forges =
            [github(17, api, b"forge.localhost", Some(b"trust.pem")), forgejo(3, api, b"http://localhost/git")];
        let records = forgejo_records(3, b"fixture-api", 42, Some(b"fixture-hook"), b"fixture-git", api);
        let jig = jig_protocol_config::profile(jig_protocol_config::Profile::Small);
        let own = temper_service::limits::profile(temper_service::limits::Profile::Small);
        let limits = temper_service::limits::engine(&jig, &own).unwrap();
        let bounds = temper_protocol_store::bounds(&limits.forge, limits.tasks.spec_bytes).unwrap();
        for existing in [false, true] {
            let scratch = skein_scratch::Scratch::new("temper-forges");
            if existing {
                seed(scratch.path(), &[]).unwrap();
            }
            seed_forges(scratch.path(), &forges, &records).unwrap();
            let loaded = store::run(scratch.path(), &[], &[], true).unwrap();
            assert_eq!(loaded.application.len(), forges.len() + 1);
            let decoded: Vec<_> = loaded
                .application
                .iter()
                .map(|(key, value)| temper_protocol_store::records::row(key, value, &bounds).unwrap())
                .collect();
            for record in &forges {
                assert!(decoded.contains(&forge::Stored::Forge(record.clone())));
            }
            assert!(decoded.contains(&forge::Stored::ForgeNumbers { next: 18 }));
            let mut expected = records.to_vec();
            expected.sort_by_key(|(number, _)| *number);
            assert_eq!(loaded.secrets, expected);
        }
    }

    #[test]
    fn forgejo_records_build_the_access_temper_loads() {
        let api = "[::1]:4321".parse().unwrap();
        let kind = skein_forge_connection::settings::Kind::Forgejo;
        let bounds =
            skein_forge_setup::item::Bounds { token_bytes: 512, name_bytes: 256, key_bytes: 4096, installations: 8 };
        for webhook in [None, Some(b"fixture-hook".as_slice())] {
            let records = forgejo_records(3, b"fixture-api", 42, webhook, b"fixture-git", api);
            assert_eq!(records.len(), if webhook.is_some() { 5 } else { 4 });
            let mut rows = codec::Rows::new();
            for (number, bytes) in &records {
                rows.push(*number, skein_lib::Secret::new(bytes.clone()), 4096);
            }
            let rows = rows.forge(3).unwrap();
            assert_eq!(rows.get(Item::GitToken).unwrap(), b"fixture-git");
            let start =
                codec::access(kind, &codec::Origin { kind, api: api.to_string().into() }, rows, &bounds, true).unwrap();
            assert_eq!(start.access.user, 42);
            assert_eq!(start.access.webhook_secret.as_ref().map(skein_lib::Secret::expose), webhook);
            let skein_forge::credential::Credential::Token(token) = start.access.credential else {
                panic!("token access expected")
            };
            assert_eq!(token.expose(), b"fixture-api");
        }
    }

    #[test]
    fn the_seeded_records_are_what_temper_loads_at_start() {
        let scratch = skein_scratch::Scratch::new("temper-github-app");
        let origin = "127.0.0.1:4321".parse().unwrap();
        let records = records(&APP, 17, &[3, 1, 3], origin);
        seed(scratch.path(), &records).unwrap();
        let loaded = store::run(scratch.path(), &[], &[], true).unwrap().secrets;
        assert_eq!(loaded.len(), 6);
        let mut rows = codec::Rows::new();
        for (number, bytes) in &loaded {
            assert!(records.iter().any(|row| row == &(*number, bytes.clone())));
            rows.push(*number, skein_lib::Secret::new(bytes.clone()), 4096);
        }
        let rows = rows.forge(17).unwrap();
        assert_eq!(rows.get(Item::AppKey).unwrap(), skein_fake_forge_github::keys::KEYS[0]);
        let start = codec::access(
            skein_forge_connection::settings::Kind::GitHub,
            &codec::Origin { kind: skein_forge_connection::settings::Kind::GitHub, api: origin.to_string().into() },
            rows,
            &skein_forge_setup::item::Bounds { token_bytes: 512, name_bytes: 256, key_bytes: 4096, installations: 8 },
            false,
        )
        .unwrap();
        assert_eq!(start.access.user, APP.bot);
        assert_eq!(start.slug.as_deref(), Some(APP.slug.as_bytes()));
        assert_eq!(&*start.accepted, &[1, 3]);
        match start.access.credential {
            skein_forge::credential::Credential::App { id, key } => {
                assert_eq!(id, APP.id);
                assert_eq!(key.expose(), skein_fake_forge_github::keys::KEYS[0]);
            }
            skein_forge::credential::Credential::Token(_) => panic!("App access expected"),
        }
    }
}
