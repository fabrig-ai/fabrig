//! Durable forge fixtures (temper/docs/plan/playground-reset-r6/notes/contracts.md, T1 and T2).
use crate::{Item, codec, store};
use skein_forge_connection::settings::Kind;
use std::{net::SocketAddr, path::Path};
use temper_engine_domain_forge::{self as forge, Api, ApiTransport, Bot, ForgeRecord, ForgeState, Mode};

/// A connected Forgejo with a plaintext API and the fixture bot's git identity.
#[must_use]
pub fn forgejo(number: u16, api: SocketAddr, git: &[u8]) -> ForgeRecord {
    ForgeRecord {
        number,
        kind: forge::Kind::Forgejo,
        mode: Mode::Connected,
        api: Api {
            host: api.ip().to_string().into_bytes().into(),
            port: api.port(),
            transport: ApiTransport::Plaintext,
        },
        git: git.into(),
        git_program: None,
        web: None,
        read_only: false,
        bot: Bot {
            login: b"temper".as_slice().into(),
            name: b"Temper".as_slice().into(),
            email: b"temper@localhost".as_slice().into(),
        },
        setup: None,
        first: None,
        playground: false,
        state: ForgeState::Kept,
    }
}

/// A connected GitHub App with the GitHub defaults and the fixture's TLS API.
#[must_use]
pub fn github(number: u16, api: SocketAddr, server_name: &[u8], trust: Option<&[u8]>) -> ForgeRecord {
    ForgeRecord {
        number,
        mode: Mode::Connected,
        api: Api {
            host: api.ip().to_string().into_bytes().into(),
            port: api.port(),
            transport: ApiTransport::Tls { server_name: server_name.into(), trust: trust.map(Box::from) },
        },
        ..ForgeRecord::github_com()
    }
}

/// A connected Forgejo's versioned credentials (temper/docs/design/protocol/credentials.md, 2.1).
#[must_use]
pub fn forgejo_records(
    forge: u16,
    token: &[u8],
    user: u64,
    webhook_secret: Option<&[u8]>,
    git_token: &[u8],
    origin: SocketAddr,
) -> Box<[(u64, Box<[u8]>)]> {
    let origin = codec::Origin { kind: Kind::Forgejo, api: origin.to_string().into_boxed_str() }.encode();
    let mut items = vec![
        (Item::ApiToken, token.to_vec()),
        (Item::User, user.to_le_bytes().to_vec()),
        (Item::GitToken, git_token.to_vec()),
        (Item::Origin, origin.into_vec()),
    ];
    if let Some(secret) = webhook_secret {
        items.push((Item::WebhookSecret, secret.to_vec()));
    }
    items
        .into_iter()
        .map(|(item, bytes)| {
            (
                codec::number(forge, item),
                codec::encode(item, &bytes).expect("fixture item within schema").into_boxed_slice(),
            )
        })
        .collect()
}

/// Commit the forge rows and next number, then durably keep their credentials.
/// The private data directory may be empty; returns after all files close.
pub fn seed_forges(data: &Path, forges: &[ForgeRecord], records: &[(u64, Box<[u8]>)]) -> Result<(), String> {
    let next = forges
        .iter()
        .map(|record| record.number)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("fixture forge numbers exhausted")?;
    let mut rows = Vec::with_capacity(forges.len() + 1);
    for record in forges {
        let key = temper_protocol_store::keys::encode(&forge::Key::Forge(record.number), u32::MAX)
            .expect("fixture forge key fits");
        let value = temper_protocol_store::records::encode(&forge::Stored::Forge(record.clone()));
        rows.push((key, value));
    }
    rows.push((
        temper_protocol_store::keys::encode(&forge::Key::ForgeNumbers, u32::MAX).expect("fixture counter key fits"),
        temper_protocol_store::records::encode(&forge::Stored::ForgeNumbers { next }),
    ));
    store::run(data, &rows, records, false).map(|_| ())
}
