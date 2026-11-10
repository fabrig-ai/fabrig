//! The first-run document names machine parts and seeds no project
//! (temper/docs/design/protocol/setup/init.md, 3), with whole JSON string placeholders.
//! The pinned jig and smith parsers require positive prices, so demo uses their
//! minimum (one per million) until they admit protocol/setup/init.md 3's zero prices.
//! Embedding these fixed templates is init's job (protocol/setup/init.md, 2); they are not runtime inputs.
#![expect(
    clippy::disallowed_macros,
    reason = "init embeds its default documents, as protocol/setup/init.md 2 requires"
)]

use crate::{Bare, Directories, Facts, path};
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits};
use skein_lib::{Decimal, List, Writer, bytes};

pub(crate) const TEMPER: &[u8] = include_bytes!("../template/temper.json");
pub(crate) const AGENT: &[u8] = include_bytes!("../template/agent.json");

pub(crate) const BARE: &[u8] = include_bytes!("../template/bare.json");
pub(crate) const BARE_AGENT: &[u8] = include_bytes!("../template/bare-agent.json");

pub(crate) fn documents(facts: &Facts<'_>, directories: &Directories<'_>, login: &[u8]) -> (Box<[u8]>, Box<[u8]>) {
    let common = common_values(
        &Bare {
            binary: facts.binary,
            git: facts.git,
            claude: facts.claude,
            clients: facts.ports.clients,
            webhooks: facts.ports.webhooks,
        },
        directories,
    );
    let forge = listener(facts.ports.forge);
    let mut forge_git = Writer::new(7_usize.checked_add(forge.len()).expect("an HTTP base fits"));
    forge_git.put(b"http://").expect("the scheme was counted");
    forge_git.put(&forge).expect("the socket was counted");
    let forge_git = forge_git.finish();
    let owned = listener(facts.ports.owned);
    let mut owned_git = Writer::new(7_usize.checked_add(owned.len()).expect("an HTTP base fits"));
    owned_git.put(b"http://").expect("the scheme was counted");
    owned_git.put(&owned).expect("the socket was counted");
    let owned_git = owned_git.finish();
    let mut email = Writer::new(login.len().checked_add(10).expect("a login and @localhost fit"));
    email.put(login).expect("the login was counted");
    email.put(b"@localhost").expect("the domain was counted");
    let email = email.finish();
    let llm = listener(facts.ports.llm);
    let provider = if facts.sign_in {
        let redirect = path::join(&owned_git, b"user/oauth2/temper/callback");
        json(Value::Provider { redirect: &redirect })
    } else {
        json(Value::Text(None))
    };

    let values = [
        (b"provider".as_slice(), provider),
        (b"owned_sign_in".as_slice(), json(Value::Text(if facts.sign_in { Some(b"forgejo") } else { None }))),
        (b"forge".as_slice(), json(Value::Text(Some(&forge)))),
        (b"forge_git".as_slice(), json(Value::Text(Some(&forge_git)))),
        (b"owned".as_slice(), json(Value::Text(Some(&owned)))),
        (b"owned_git".as_slice(), json(Value::Text(Some(&owned_git)))),
        (b"login".as_slice(), json(Value::Text(Some(login)))),
        (b"email".as_slice(), json(Value::Text(Some(&email)))),
        (b"llm".as_slice(), json(Value::Text(Some(&llm)))),
    ];
    (fill(TEMPER, &values, &common), fill(AGENT, &values, &common))
}

pub(crate) fn bare_documents(facts: &Bare<'_>, directories: &Directories<'_>) -> (Box<[u8]>, Box<[u8]>) {
    let values = common_values(facts, directories);
    (fill(BARE, &values, &[]), fill(BARE_AGENT, &values, &[]))
}

fn common_values(facts: &Bare<'_>, directories: &Directories<'_>) -> [(&'static [u8], Box<[u8]>); 6] {
    let agent = path::join(directories.config, b"agent.json");
    let programs = path::join(directories.data, b"programs");
    let clients = listener(facts.clients);
    let webhooks = listener(facts.webhooks);
    [
        (b"binary", json(Value::Text(Some(facts.binary)))),
        (b"agent", json(Value::Text(Some(&agent)))),
        (b"git", json(Value::Text(facts.git))),
        (b"listener", json(Value::Text(Some(&clients)))),
        (b"webhooks", json(Value::Text(Some(&webhooks)))),
        (b"programs", json(Value::Programs { claude: facts.claude, directory: &programs })),
    ]
}

fn listener(port: core::num::NonZeroU16) -> Box<[u8]> {
    let port = Decimal::of(u64::from(port.get()));
    let mut listener = Writer::new(10_usize.checked_add(port.as_bytes().len()).expect("a socket address fits"));
    listener.put(b"127.0.0.1:").expect("the host and colon were counted");
    listener.put(port.as_bytes()).expect("the port was counted");
    listener.finish()
}

#[derive(Clone, Copy, Debug)]
enum Value<'a> {
    Text(Option<&'a [u8]>),
    Provider { redirect: &'a [u8] },
    Programs { claude: Option<&'a [u8]>, directory: &'a [u8] },
}

fn encode(json: &mut Encoder, value: Value<'_>) {
    match value {
        Value::Text(Some(text)) => json.string(text),
        Value::Text(None) => json.null(),
        Value::Provider { redirect } => {
            json.object_start();
            json.key(b"key");
            json.string(b"provider.key");
            json.key(b"clients");
            json.array_start();
            json.object_start();
            json.key(b"id");
            json.string(b"forgejo");
            json.key(b"secret");
            json.null();
            json.key(b"redirect_uri");
            json.string(redirect);
            json.object_end();
            json.array_end();
            json.key(b"addresses");
            json.array_start();
            json.unsigned(1);
            json.array_end();
            json.object_end();
        }
        Value::Programs { claude, directory } => {
            json.object_start();
            json.key(b"claude_code");
            json.object_start();
            json.key(b"path");
            json.string(claude.unwrap_or(crate::FETCHED));
            json.key(b"directory");
            json.string(directory);
            json.object_end();
            json.object_end();
        }
    }
}

fn json(value: Value<'_>) -> Box<[u8]> {
    // Two nested objects and two paths, each at 4096 bytes and at most six
    // escaped bytes per byte, plus keys and punctuation, fit within 50,000.
    let limits = Limits { depth: 3, length: 50_000 };
    let mut measure = Encoder::measure(&limits);
    encode(&mut measure, value);
    let len = measure.measured().expect("checked paths and the fixed shape fit the JSON limits");
    let mut writer = Encoder::write(len, &limits);
    encode(&mut writer, value);
    writer.finish()
}

fn fill(source: &[u8], values: &[(&[u8], Box<[u8]>)], common: &[(&[u8], Box<[u8]>)]) -> Box<[u8]> {
    let count = bytes::count(source, b"\"$", u32::MAX);
    let capacity =
        count.checked_mul(2).expect("two pieces per placeholder fit").checked_add(1).expect("one final piece fits");
    let mut pieces = List::with_capacity(capacity);
    let mut cursor = 0;
    for _ in 0..count {
        let start = bytes::find_from(source, b"\"$", cursor).expect("the placeholders were counted");
        let name_start = start.checked_add(2).expect("an embedded template's offset fits");
        let tail = source.get(name_start..).expect("the placeholder starts within the template");
        let name_len = bytes::find(tail, b"\"").expect("each placeholder is a whole JSON string");
        let name = tail.get(..name_len).expect("the closing quote was found");
        pieces.push(source.get(cursor..start).expect("placeholders are disjoint")).expect("two pieces per placeholder");
        pieces.push(replacement(name, values, common)).expect("two pieces per placeholder");
        cursor = name_start
            .checked_add(name_len)
            .expect("the name ends within the template")
            .checked_add(1)
            .expect("the closing quote ends within the template");
    }
    pieces.push(source.get(cursor..).expect("the last placeholder ends within the template")).expect("one final piece");
    let mut length = 0_usize;
    for piece in &pieces {
        length = length.checked_add(piece.len()).expect("the fixed templates and bounded replacements fit");
    }
    let mut writer = Writer::new(length);
    for piece in &pieces {
        writer.put(piece).expect("every piece was counted");
    }
    writer.finish()
}

fn replacement<'a>(name: &[u8], values: &'a [(&[u8], Box<[u8]>)], common: &'a [(&[u8], Box<[u8]>)]) -> &'a [u8] {
    for replacements in [values, common] {
        for (known, value) in replacements {
            if name == *known {
                return value;
            }
        }
    }
    unreachable!("the template's placeholders are all known, checked by the crate's tests")
}
