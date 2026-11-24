//! Stored fixture forges (temper/docs/design/protocol/forge/engine.md, 1).
use std::{net::SocketAddr, path::Path};
use temper_engine_domain_forge as forge;
use temper_protocol_records::{self as records, Item};
use temper_protocol_records_forge as forge_records;

/// Seed the bootstrap's fixture rows in one store life, in forge/settings/agent order.
pub(crate) fn seed(
    path: &Path,
    address: SocketAddr,
    github: Option<crate::github::Fixture>,
    mut agent_writes: Vec<jig_protocol_store::Write>,
) {
    let mut record = forgejo_record(address);
    let secrets = if let Some(fixture) = github {
        record.kind = forge::Kind::GitHub;
        record.place.web = Some(format!("http://{address}").into_bytes().into());
        record.bot.login = b"x-access-token".as_slice().into();
        if fixture.connected {
            temper_github_app::records(&temper_github_app::APP, 1, &[crate::github::INSTALLATION], address).into_vec()
        } else {
            Vec::new()
        }
    } else {
        forgejo_secrets(&record, 1, temper_worker_checkout_world::hosted::TOKEN, crate::forge::HOOK_SECRET, None)
    };
    let mut writes = Vec::new();
    let flags = path.join("world-settings.json");
    if flags.exists() {
        let flags: serde_json::Value =
            serde_json::from_slice(&std::fs::read(flags).expect("fixture flags")).expect("fixture flags JSON");
        if !flags["kinds"].is_null() {
            writes = settings_writes(&serde_json::to_vec(&flags["kinds"]).expect("fixture application settings"));
        }
    }
    writes.append(&mut agent_writes);
    seed_record_with(&path.join("state"), record, &secrets, writes);
}

/// Commit a kept forge and its secrets in an initialized deployment store directory.
pub fn seed_record(path: &Path, record: forge::ForgeRecord, secrets: &[(u64, Box<[u8]>)]) {
    seed_record_with(path, record, secrets, Vec::new());
}
fn seed_record_with(
    path: &Path,
    record: forge::ForgeRecord,
    secrets: &[(u64, Box<[u8]>)],
    mut additional: Vec<jig_protocol_store::Write>,
) {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(path).expect("fixture state directory");
    let number = record.number;
    let mut writes: Vec<_> = [
        (forge::Key::Forge(number), forge::Stored::Forge(record)),
        (forge::Key::ForgeNumber, forge::Stored::ForgeNumber { last: number }),
    ]
    .into_iter()
    .map(|(key, row)| jig_protocol_store::Write::Put {
        key: temper_protocol_store::keys::encode(&key, 256).expect("fixture forge key"),
        value: temper_protocol_store::records::encode(&row),
    })
    .collect();
    writes.append(&mut additional);
    temper_github_app::seed_with(path, secrets, writes).expect("durable forge fixture");
}

/// Keep a Forgejo fixture's bot access, and its administrator for managed or owned reconciliation.
pub fn seed_forgejo(
    path: &Path,
    record: forge::ForgeRecord,
    user: u64,
    token: &[u8],
    hook: &[u8],
    admin: Option<&[u8]>,
) {
    let secrets = forgejo_secrets(&record, user, token, hook, admin);
    seed_record(path, record, &secrets);
}
fn forgejo_secrets(
    record: &forge::ForgeRecord,
    user: u64,
    token: &[u8],
    hook: &[u8],
    admin: Option<&[u8]>,
) -> Vec<(u64, Box<[u8]>)> {
    let origin = forge_records::Origin {
        kind: skein_forge_connection::settings::Kind::Forgejo,
        api: SocketAddr::new(
            std::str::from_utf8(&record.place.api.host).expect("fixture host").parse().expect("fixture IP"),
            record.place.api.port,
        )
        .to_string()
        .into(),
    }
    .encode();
    let mut secrets: Vec<_> = [
        (Item::Origin, origin.as_ref()),
        (Item::User, user.to_le_bytes().as_slice()),
        (Item::ApiToken, token),
        (Item::GitToken, token),
        (Item::WebhookSecret, hook),
    ]
    .into_iter()
    .map(|(item, bytes)| {
        (
            records::number(record.number, item),
            records::encode(item, bytes).expect("fixture secret encoding").into_boxed_slice(),
        )
    })
    .collect();
    if let Some(admin) = admin {
        secrets.push((
            records::number(record.number, Item::AdminToken),
            records::encode(Item::AdminToken, admin).expect("fixture admin encoding").into_boxed_slice(),
        ));
    }
    secrets
}

/// Forge 1 on an assigned plaintext peer, ready for fixture-specific identity and mode.
#[must_use]
pub fn forgejo_record(address: SocketAddr) -> forge::ForgeRecord {
    forge::ForgeRecord {
        person: None,
        git_program: None,
        number: 1,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Connected,
        state: forge::ForgeState::Kept,
        place: forge::Place {
            api: forge::Address { host: address.ip().to_string().into_bytes().into(), port: address.port(), tls: None },
            git: format!("http://{address}").into_bytes().into(),
            web: None,
        },
        read_only: false,
        bot: forge::Bot {
            login: b"worker".as_slice().into(),
            name: b"Temper".as_slice().into(),
            email: b"temper@example.test".as_slice().into(),
        },
        asked: false,
        first: None,
    }
}

/// Seed typed application values beside their core metadata after the fixture's first start.
/// This is bootstrap data, before any work or owner changes (domain/landing-settings.md, 1.1).
pub fn seed_settings_file(path: &Path, bytes: &[u8]) {
    seed_settings_file_with(path, bytes, |path, writes| {
        temper_github_app::seed_with(path, &[], writes).expect("durable settings fixture");
    });
}

/// Compose the same setting metadata and application rows through a supplied store sink.
pub fn seed_settings_file_with(path: &Path, bytes: &[u8], seed: impl FnOnce(&Path, Vec<jig_protocol_store::Write>)) {
    seed(path, settings_writes(bytes));
}

fn settings_writes(bytes: &[u8]) -> Vec<jig_protocol_store::Write> {
    use jig_domain_core::settings as s;
    let mut writes = Vec::new();
    for stock in temper_playground::documents::settings::kind_defaults(bytes) {
        let setting = s::Setting::Application { kind: stock.kind, scope: stock.scope, number: stock.number };
        let metadata = s::Record::Application {
            setting,
            state: s::application::State {
                follows: s::application::Follows::Changed { revision: 1 },
                size: stock.value.size,
                charters: Box::new([]),
            },
        };
        writes.push(jig_protocol_store::Write::Save(jig_domain_core::Record::Core(
            jig_domain_core::CoreRecord::Settings(metadata),
        )));
        let key = forge::settings::SettingKey {
            kind: stock.kind,
            scope: stock.scope,
            number: stock.number,
            slot: forge::settings::SettingSlot::Change,
        };
        let row =
            forge::Stored::Setting(forge::settings::SettingRow { key, edit: forge::settings::Edit::Set(stock.value) });
        writes.push(jig_protocol_store::Write::Put {
            key: temper_protocol_store::keys::encode(&forge::Key::Setting(key), 256).expect("fixture landing key"),
            value: temper_protocol_store::records::encode(&row),
        });
    }
    writes
}
