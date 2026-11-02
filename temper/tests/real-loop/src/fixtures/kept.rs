//! Stored fixture forges (temper/docs/design/protocol/forge.md, 2.1).
use std::{net::SocketAddr, path::Path};
use temper_engine_domain_forge as forge;
use temper_protocol_records::{self as records, Item};

pub(crate) fn seed(path: &Path, address: SocketAddr, github: Option<crate::github::Fixture>) {
    let mut record = forgejo_record(address);
    let secrets = if let Some(fixture) = github {
        record.kind = forge::Kind::GitHub;
        record.place.web = Some(format!("http://{address}").into_bytes().into());
        record.bot.login = b"x-access-token".as_slice().into();
        if fixture.connected {
            temper_github_app::records(&temper_github_app::APP, 1, &[crate::github::INSTALLATION], address)
        } else {
            Box::new([])
        }
    } else {
        seed_forgejo(
            &path.join("state"),
            record,
            1,
            temper_fake_git_hosted::fixtures::TOKEN,
            crate::forge::HOOK_SECRET,
            None,
        );
        return;
    };
    seed_record(&path.join("state"), record, &secrets);
}

/// Commit a kept forge and its secrets in an initialized deployment store directory.
pub fn seed_record(path: &Path, record: forge::ForgeRecord, secrets: &[(u64, Box<[u8]>)]) {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(path).expect("fixture state directory");
    let number = record.number;
    let writes = [
        (forge::Key::Forge(number), forge::Stored::Forge(record)),
        (forge::Key::ForgeNumber, forge::Stored::ForgeNumber { last: number }),
    ]
    .into_iter()
    .map(|(key, row)| jig_protocol_store::Write::Put {
        key: temper_protocol_store::keys::encode(&key, 256).expect("fixture forge key"),
        value: temper_protocol_store::records::encode(&row),
    })
    .collect();
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
    let origin = records::Origin {
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
    seed_record(path, record, &secrets);
}

/// Forge 1 on an assigned plaintext peer, ready for fixture-specific identity and mode.
#[must_use]
pub fn forgejo_record(address: SocketAddr) -> forge::ForgeRecord {
    forge::ForgeRecord {
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
