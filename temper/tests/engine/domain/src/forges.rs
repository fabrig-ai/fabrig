//! Forge records for world fixtures (temper/docs/design/draft/playground-reset.md, 3).
use temper_engine_domain_forge as forge;

const SETUP: &[u8] = br#"{"bot":{"login":"temper","email":"temper@localhost"}}"#;

/// A kept forge at forge-N.test, with setup for Owned and Fake modes.
#[must_use]
pub fn record(number: u16, kind: forge::Kind, mode: forge::Mode) -> forge::ForgeRecord {
    let host = format!("forge-{number}.test").into_bytes().into_boxed_slice();
    let base = format!("https://forge-{number}.test").into_bytes().into_boxed_slice();
    forge::ForgeRecord {
        number,
        kind,
        mode,
        api: forge::Api {
            host: host.clone(),
            port: 443,
            transport: forge::ApiTransport::Tls { server_name: host, trust: None },
        },
        git: base.clone(),
        git_program: None,
        web: Some(base),
        read_only: false,
        bot: forge::Bot {
            login: b"temper".as_slice().into(),
            name: b"Temper".as_slice().into(),
            email: b"temper@localhost".as_slice().into(),
        },
        setup: match mode {
            forge::Mode::Owned | forge::Mode::Fake => Some(SETUP.into()),
            forge::Mode::Managed | forge::Mode::Connected => None,
        },
        first: None,
        playground: false,
        state: forge::ForgeState::Kept,
    }
}

/// A record matching the configured forge's kind, mode, addresses and read-only state.
#[must_use]
pub fn from_configured(configured: &forge::Configured) -> forge::ForgeRecord {
    let mut row = record(configured.number, configured.kind, configured.mode);
    row.git.clone_from(&configured.git);
    row.web.clone_from(&configured.web);
    row.read_only = configured.read_only;
    row.bot.login.clone_from(&configured.bot);
    row.setup = if configured.asked || !configured.bot.is_empty() { Some(SETUP.into()) } else { None };
    row
}

/// Each `Forge` row and a `ForgeNumbers` row past the highest number.
#[must_use]
pub fn rows(records: &[forge::ForgeRecord]) -> Box<[temper_engine_domain::Record]> {
    let mut rows = Vec::with_capacity(records.len() + 1);
    let mut highest = 0;
    for row in records {
        highest = highest.max(row.number);
        rows.push(temper_engine_domain::Record::Forge { row: Box::new(forge::Stored::Forge(row.clone())) });
    }
    rows.push(temper_engine_domain::Record::Forge {
        row: Box::new(forge::Stored::ForgeNumbers {
            next: highest.checked_add(1).expect("fixture has a next forge number"),
        }),
    });
    rows.into_boxed_slice()
}
