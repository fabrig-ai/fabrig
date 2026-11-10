//! A repository's link to its page on its forge (temper/docs/design/client/forge/deployment.md).
use crate::{
    forge_sign_in::Forges,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::Writer;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::forge::forges::{Kind, Mode, Row};

pub(crate) fn address(row: &Row, sign_in: &Forges, owner: &[u8], name: &[u8]) -> Option<Box<[u8]>> {
    let address = if row.kind == Kind::Forgejo
        && let Some(forge) = sign_in.number(row.number)
    {
        forge.open_repository(owner, name)
    } else {
        web_address(row.web.as_deref()?, owner, name)
    }?;
    if skein_ui::markdown::safe_url(&address) { Some(address) } else { None }
}
/// Opens no node of its own: the link, or the words, in the node the caller holds open.
pub(crate) fn name(b: &mut Builder, w: &Words, row: Option<&Row>, sign_in: &Forges, owner: &[u8], name: &[u8]) {
    if w.shell == Shell::Browser
        && let Some(row) = row
        && row.mode != Mode::Fake
        && let Some(address) = address(row, sign_in, owner, name)
    {
        let label = w.say(Word::RepositoryLink, &[Hole::Name(owner.into()), Hole::Name(name.into())]);
        b.open(Role::Link);
        b.style(crate::look::REPOSITORY_LINK);
        b.name(&label);
        b.external(&address);
        b.text(Run::Plain, &label);
        b.close();
    } else {
        w.text(b, Word::SwitcherRepository, &[Hole::Name(owner.into()), Hole::Name(name.into())]);
    }
}
pub(crate) fn web_address(web: &[u8], owner: &[u8], name: &[u8]) -> Option<Box<[u8]>> {
    if owner.is_empty() || name.is_empty() {
        return None;
    }
    let mut web = web;
    for _ in 0..web.len() {
        let Some(prefix) = web.strip_suffix(b"/") else { break };
        web = prefix;
    }
    let owner = encoded(owner, true)?;
    let name = encoded(name, true)?;
    let bytes = web.len().checked_add(owner.len())?.checked_add(name.len())?.checked_add(2)?;
    if bytes > usize::try_from(temper_client_domain::forge::change::ADDRESS_BYTES).ok()? {
        return None;
    }
    let mut out = Writer::new(bytes);
    for part in [web, b"/", &owner, b"/", &name] {
        out.put(part).ok()?;
    }
    Some(out.finish())
}
pub(crate) fn encoded(value: &[u8], segment: bool) -> Option<Box<[u8]>> {
    // Browsers normalize percent-encoded dot segments too; quote their escapes.
    if segment && value == b"." {
        return Some(b"%252E".as_slice().into());
    }
    if segment && value == b".." {
        return Some(b"%252E%252E".as_slice().into());
    }
    let mut bytes = 0_usize;
    for byte in value {
        bytes = bytes.checked_add(if kept(*byte) { 1 } else { 3 })?;
    }
    if bytes > usize::try_from(temper_client_domain::forge::change::ADDRESS_BYTES).ok()? {
        return None;
    }
    let mut out = Writer::new(bytes);
    for byte in value {
        if kept(*byte) {
            out.put(&[*byte]).ok()?;
        } else {
            let hex = b"0123456789ABCDEF";
            out.put(&[b'%', *hex.get(usize::from(*byte >> 4_u8))?, *hex.get(usize::from(*byte & 15))?]).ok()?;
        }
    }
    Some(out.finish())
}
fn kept(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || b"-_~".contains(&byte) || byte == b'.'
}
