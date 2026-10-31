//! Validated account metadata, without credential values (shell.md, 5.2 and 7.1).
//! The shared catalog groups settings once; each account retains typed source
//! and provider choices beside the fields used by registration and diagnostics.
use super::{Settings, Sourced};
use std::collections::BTreeMap;

/// A configured source, retained after the declaration checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountSource {
    /// No source declaration, so the account cannot be selected for a run.
    Unspecified,
    /// A saved sign-in registration and token record.
    SignIn,
    /// An access-only credential in a named environment variable.
    Env,
    /// A private key file beneath the credentials directory.
    Key,
    /// A program's own sign-in state.
    Program,
    /// An endpoint that needs no credential.
    Nothing,
}

impl AccountSource {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::SignIn | Self::Unspecified => "sign_in",
            Self::Env => "env",
            Self::Key => "key",
            Self::Program => "program",
            Self::Nothing => "none",
        }
    }
}

/// A provider's preset registration, independent of the selected endpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    /// `OpenAI`'s registered plan sign-in.
    OpenAi,
    /// Anthropic's program or key source.
    Anthropic,
}

impl Provider {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
        }
    }
}

/// Named source metadata, without any borrowed or refreshed value.
#[derive(Debug)]
pub struct Account {
    pub name: String,
    pub number: u32,
    pub fields: BTreeMap<String, Sourced>,
    source: AccountSource,
    provider: Option<Provider>,
}

impl Account {
    pub(crate) fn from_fields(name: String, number: u32, fields: BTreeMap<String, Sourced>) -> Self {
        let text = |key| fields.get(key).and_then(|field| field.value.as_str());
        let source = match text("source") {
            Some("sign_in") => AccountSource::SignIn,
            None => AccountSource::Unspecified,
            Some("env") => AccountSource::Env,
            Some("key") => AccountSource::Key,
            Some("program") => AccountSource::Program,
            Some("none") => AccountSource::Nothing,
            Some(_) => unreachable!("account source was validated"),
        };
        let provider = match text("provider") {
            Some("openai") => Some(Provider::OpenAi),
            Some("anthropic") => Some(Provider::Anthropic),
            None => None,
            Some(_) => unreachable!("account provider was validated"),
        };
        Self { name, number, fields, source, provider }
    }

    pub(crate) const fn credential_number(&self) -> Option<u32> {
        match self.source {
            AccountSource::Nothing => None,
            AccountSource::Unspecified
            | AccountSource::SignIn
            | AccountSource::Env
            | AccountSource::Key
            | AccountSource::Program => Some(self.number),
        }
    }

    /// The validated source choice, absent accounts included.
    #[must_use]
    pub const fn source(&self) -> AccountSource {
        self.source
    }

    /// The optional provider preset selected by the account.
    #[must_use]
    pub const fn provider(&self) -> Option<Provider> {
        self.provider
    }
}

pub(crate) fn accounts(settings: &Settings) -> BTreeMap<String, Account> {
    let mut grouped = BTreeMap::<String, BTreeMap<String, Sourced>>::new();
    for (key, value) in &settings.values {
        if let Some(rest) = key.strip_prefix("accounts.") {
            let (name, field) = named_field(rest).expect("validated account key");
            grouped.entry(name).or_default().insert(field.into(), value.clone());
        }
    }
    grouped
        .into_iter()
        .map(|(name, fields)| {
            let number = super::resolve::numbers::number(&name);
            (name.clone(), Account::from_fields(name, number, fields))
        })
        .collect()
}

pub(crate) fn name(part: &str) -> String {
    if part.is_empty() || part.contains(['.', '"', '\\']) {
        serde_json::to_string(part).expect("setting name")
    } else {
        part.into()
    }
}

/// Split one encoded setting name from its field without dropping malformed input.
pub(crate) fn named_field(path: &str) -> Result<(String, &str), ()> {
    if path.starts_with('"') {
        let mut parts = serde_json::Deserializer::from_str(path).into_iter::<String>();
        let name = parts.next().ok_or(())?.map_err(|_| ())?;
        let field = path.get(parts.byte_offset()..).and_then(|rest| rest.strip_prefix('.')).ok_or(())?;
        Ok((name, field))
    } else {
        path.split_once('.').map(|(name, field)| (name.into(), field)).ok_or(())
    }
}
