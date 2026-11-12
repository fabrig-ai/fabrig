//! Browser state kept across restarts (testing/fakes/github.md, 1.7).
use super::{Record, Store};
use crate::{
    Limits,
    image::{
        Refusal,
        codec::{Encoder, count, data, flag, num, optional_data, small, strings},
    },
    manifest::{Hook, Manifest},
};
use skein_forge::github::document::{Access, Permission};
use skein_lib::{List, Reader, Secret, Wall, usize_of};

pub(crate) fn permissions(out: &mut Encoder, values: &[Permission]) {
    out.num(skein_lib::len_u64(values.len()));
    for value in values {
        out.bytes(&value.name);
        out.bytes(value.access.text());
    }
}
pub(crate) fn read_permissions(input: &mut Reader<'_>, limits: &Limits) -> Result<List<Permission>, Refusal> {
    let length = count(input, limits.github.inner, "permissions")?;
    let mut values = List::with_capacity(length);
    for _ in 0..length {
        let name = data(input, usize_of(limits.github.string_bytes), "permission name")?;
        let access =
            Access::read(&data(input, 5, "permission access")?).ok_or(Refusal::Malformed("permission access"))?;
        values.push(Permission { name, access }).expect("admitted permissions");
    }
    Ok(values)
}
impl Store {
    pub(crate) fn encode_image(&self, out: &mut Encoder) {
        out.num(self.rng.state());
        out.num(u64::from(self.key));
        out.optional_data(self.setup.as_deref());
        out.num(u64::from(self.states.len()));
        for (state, record) in &self.states {
            out.bytes(state);
            write_manifest(out, &record.manifest);
            out.bytes(record.code.expose());
            out.num(record.expires.as_nanos());
            out.flag(record.used);
        }
    }
    pub(crate) fn decode_image(input: &mut Reader<'_>, limits: &Limits) -> Result<Self, Refusal> {
        let mut restored = Self::new(limits.pages, num(input, "pages rng")?);
        restored.key = small(input, "page key rotation")?;
        if usize_of(restored.key) >= crate::keys::KEYS.len() {
            return Err(Refusal::Malformed("page key rotation"));
        }
        restored.setup = optional_data(input, usize_of(limits.github.string_bytes), "setup url")?;
        let length = count(input, limits.pages.states, "page states")?;
        for _ in 0..length {
            let state = data(input, usize_of(limits.github.target_bytes), "page state")?;
            if restored.states.contains_key(state.as_ref()) {
                return Err(Refusal::Malformed("duplicate page state"));
            }
            let manifest = read_manifest(input, limits)?;
            if manifest_bytes(&manifest).ok_or(Refusal::Limits("manifest bytes"))?
                > usize_of(limits.pages.manifest_bytes)
            {
                return Err(Refusal::Limits("manifest bytes"));
            }
            let code = Secret::new(data(input, 20, "conversion code")?);
            if code.len() != 20 {
                return Err(Refusal::Malformed("conversion code"));
            }
            let expires = Wall::from_nanos(num(input, "code expiry")?);
            let used = flag(input, "code used")?;
            drop(restored.states.insert(state, Record { manifest, code, expires, used }).expect("admitted states"));
        }
        Ok(restored)
    }
}
// Retained text cannot exceed the admitted manifest's bytes. The account is
// supplied by the service, not by that document, and is bounded separately.
fn manifest_bytes(m: &Manifest) -> Option<usize> {
    let mut length = m.name.len().checked_add(m.url.len())?.checked_add(m.redirect_url.len())?;
    if let Some(url) = &m.setup_url {
        length = length.checked_add(url.len())?;
    }
    if let Some(hook) = &m.hook_attributes
        && let Some(url) = &hook.url
    {
        length = length.checked_add(url.len())?;
    }
    for permission in &m.default_permissions {
        length = length.checked_add(permission.name.len())?;
    }
    for event in &m.default_events {
        length = length.checked_add(event.len())?;
    }
    Some(length)
}
fn write_manifest(out: &mut Encoder, m: &Manifest) {
    out.bytes(&m.name);
    out.bytes(&m.url);
    out.flag(m.public);
    out.bytes(&m.redirect_url);
    out.optional_data(m.setup_url.as_deref());
    out.flag(m.setup_on_update);
    out.flag(m.hook_attributes.is_some());
    if let Some(hook) = &m.hook_attributes {
        out.optional_data(hook.url.as_deref());
        out.flag(hook.active);
    }
    permissions(out, m.default_permissions.as_slice());
    out.strings(m.default_events.as_slice());
    out.bytes(&m.account);
}
fn read_manifest(input: &mut Reader<'_>, limits: &Limits) -> Result<Manifest, Refusal> {
    let string = usize_of(limits.github.string_bytes);
    let name = data(input, string, "manifest name")?;
    let url = data(input, string, "manifest url")?;
    let public = flag(input, "manifest public")?;
    let redirect_url = data(input, string, "manifest redirect")?;
    let setup_url = optional_data(input, string, "manifest setup url")?;
    let setup_on_update = flag(input, "manifest setup on update")?;
    let hook_attributes = if flag(input, "manifest hook")? {
        Some(Hook {
            url: optional_data(input, string, "manifest hook url")?,
            active: flag(input, "manifest hook active")?,
        })
    } else {
        None
    };
    let default_permissions = read_permissions(input, limits)?;
    let events = strings(input, limits.github.inner, string, "manifest events")?;
    let mut default_events = List::with_capacity(limits.github.inner);
    for event in events {
        default_events.push(event).expect("admitted events");
    }
    let account = data(input, string, "manifest account")?;
    Ok(Manifest {
        name,
        url,
        public,
        redirect_url,
        setup_url,
        setup_on_update,
        hook_attributes,
        default_permissions,
        default_events,
        account,
    })
}
