//! Stored fixture forges (temper/docs/design/draft/playground-reset.md, 3).
use std::{net::SocketAddr, path::Path};
use temper_engine_domain_forge as forge;
use temper_protocol_records::{self as records, Item};

pub(crate) fn seed(path: &Path, address: SocketAddr, github: Option<crate::github::Fixture>) {
    let is_github = github.is_some();
    let record = forge::ForgeRecord {
        number: 1,
        kind: if is_github { forge::Kind::GitHub } else { forge::Kind::Forgejo },
        mode: forge::Mode::Connected,
        state: forge::ForgeState::Kept,
        place: forge::Place {
            api: forge::Address { host: address.ip().to_string().into_bytes().into(), port: address.port(), tls: None },
            git: format!("http://{address}").into_bytes().into(),
            web: is_github.then(|| format!("http://{address}").into_bytes().into()),
        },
        read_only: false,
        bot: forge::Bot {
            login: if is_github { b"x-access-token".as_slice() } else { b"worker".as_slice() }.into(),
            name: b"Temper".as_slice().into(),
            email: b"temper@example.test".as_slice().into(),
        },
        asked: false,
        first: None,
    };
    let writes = [
        (forge::Key::Forge(1), forge::Stored::Forge(record)),
        (forge::Key::ForgeNumber, forge::Stored::ForgeNumber { last: 1 }),
    ]
    .into_iter()
    .map(|(key, row)| jig_protocol_store::Write::Put {
        key: temper_protocol_store::keys::encode(&key, 256).expect("fixture forge key"),
        value: temper_protocol_store::records::encode(&row),
    })
    .collect();
    let secrets = match github {
        Some(fixture) if fixture.connected => {
            temper_github_app::records(&temper_github_app::APP, 1, &[crate::github::INSTALLATION], address)
        }
        Some(_) => Box::new([]),
        None => {
            let origin = records::Origin {
                kind: skein_forge_connection::settings::Kind::Forgejo,
                api: address.to_string().into(),
            }
            .encode();
            [
                (Item::Origin, origin.as_ref()),
                (Item::User, 1_u64.to_le_bytes().as_slice()),
                (Item::ApiToken, temper_fake_git_hosted::fixtures::TOKEN),
                (Item::GitToken, temper_fake_git_hosted::fixtures::TOKEN),
                (Item::WebhookSecret, crate::forge::HOOK_SECRET),
            ]
            .into_iter()
            .map(|(item, bytes)| {
                (
                    records::number(1, item),
                    records::encode(item, bytes).expect("fixture secret encoding").into_boxed_slice(),
                )
            })
            .collect()
        }
    };
    temper_github_app::seed_with(&path.join("state"), &secrets, writes).expect("durable forge fixture");
}
