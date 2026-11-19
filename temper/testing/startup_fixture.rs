#![expect(clippy::disallowed_methods, reason = "ordinary fixture code edits typed settings and JSON documents")]
//! A world's document and its typed start limits (configuration/limit-settings.md, 8).
use crate::{limits, startup};
use jig_protocol_config as config;
use serde_json::Value;

#[derive(Clone)]
#[expect(
    clippy::partial_pub_fields,
    reason = "worlds edit the typed figures; their explicit-name provenance stays private"
)]
pub struct Start {
    pub document: Box<[u8]>,
    pub jig: config::Limits,
    pub own: limits::Limits,
    jig_given: skein_lib::List<Box<[u8]>>,
}
impl Start {
    pub fn stock(document: &[u8]) -> Self {
        let stock = limits::stock();
        let (jig, mut own) =
            limits::base(stock.live_goals, stock.tool_payload, temper_service_core::derived::STOCK).unwrap();
        if let Ok(value) = serde_json::from_slice::<Value>(document)
            && let Some(slots) = value.pointer("/workers/0/slots").and_then(Value::as_u64)
        {
            own.checkout.workspaces = u32::try_from(slots).unwrap();
            own.git_component.operations = own.checkout.workspaces;
        }
        Self::given(document, &jig, &own)
    }
    pub fn given(document: &[u8], jig: &config::Limits, own: &limits::Limits) -> Self {
        Self {
            document: document.into(),
            jig: *jig,
            own: *own,
            jig_given: skein_lib::List::with_capacity(u32::try_from(config::names::NAMES.len()).unwrap()),
        }
    }
    pub fn limits(&self, path: &[u8]) -> Option<Box<[u8]>> {
        if path.starts_with(b"application.limits")
            && serde_json::from_slice::<Value>(&self.document).ok()?.get("engine").is_none()
        {
            return None;
        }
        let (engine, overrides) = match path {
            b"engine.limits" => (true, false),
            b"engine.limits.overrides" => (true, true),
            b"application.limits" => (false, false),
            b"application.limits.overrides" => (false, true),
            _ => return None,
        };
        let mut values = serde_json::Map::new();
        if engine {
            for row in &config::names::NAMES {
                values.insert(row.name.into(), Value::from((row.value)(&self.jig)));
            }
        } else {
            for row in limits::NAMES {
                values.insert(row.name.into(), Value::from((row.value)(&self.own)));
            }
        }
        let value = if overrides { Value::Object(values) } else { serde_json::json!({"overrides":values}) };
        Some(serde_json::to_vec(&value).unwrap().into())
    }
    pub fn over(&mut self, path: &[u8], input: &[u8]) -> bool {
        if path.starts_with(b"application.limits")
            && serde_json::from_slice::<Value>(&self.document).is_ok_and(|value| value.get("engine").is_none())
        {
            return false;
        }
        let engine = match path {
            b"engine.limits" | b"engine.limits.overrides" => true,
            b"application.limits" | b"application.limits.overrides" => false,
            _ => return false,
        };
        let mut values: Value = serde_json::from_slice(input).unwrap();
        if !path.ends_with(b"overrides") {
            values = values["overrides"].take();
        }
        for (name, value) in values.as_object().expect("a world's numeric limits") {
            let value = value.as_u64().expect("a world's limit figure");
            if engine {
                config::overrides::apply(&mut self.jig, name.as_bytes(), value).unwrap();
                if !self.jig_given.iter().any(|given| given.as_ref() == name.as_bytes()) {
                    self.jig_given.push(name.as_bytes().into()).unwrap();
                }
            } else {
                limits::apply(&mut self.own, name.as_bytes(), value).unwrap();
            }
        }
        if engine {
            let mut rows = skein_lib::List::with_capacity(self.jig_given.len());
            for name in &self.jig_given {
                rows.push(jig_domain_core::settings::Override {
                    name: name.clone(),
                    value: (config::names::find(name).unwrap().value)(&self.jig),
                    reason: b"world figure".as_slice().into(),
                })
                .unwrap();
            }
            let mut rows = rows.into_boxed();
            rows.sort_unstable_by(compare);
            if let Ok(jig) = config::overrides::with_overrides(&self.jig, &rows) {
                self.jig = jig;
            }
        } else {
            let values = values.as_object().unwrap();
            if values.contains_key("forge.client.op_bytes") && !values.contains_key("forge_component.body_bytes") {
                self.own.forge_component.body_bytes =
                    temper_service_core::derived::forge_body(self.own.forge.client.op_bytes).unwrap();
            }
            if values.contains_key("forge.repositories") && !values.contains_key("forge_component.repositories") {
                self.own.forge_component.repositories = self.own.forge.repositories;
            }
        }
        true
    }
    pub fn named(&self, directories: &config::Directories) -> Result<startup::Named, startup::Problems> {
        self.named_with(directories, &crate::stock::Given::Stock)
    }
    pub fn named_with(
        &self,
        directories: &config::Directories,
        given: &crate::stock::Given,
    ) -> Result<startup::Named, startup::Problems> {
        startup::named_with_limits(&self.document, directories, given, &self.jig, &self.own)
    }
}
impl core::ops::Deref for Start {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.document
    }
}
/// Byte-only document edits keep the existing typed figures; raw documents start at the stock.
pub trait Input {
    fn configured(&self) -> Start;
}
impl Input for Start {
    fn configured(&self) -> Start {
        self.clone()
    }
}
impl Input for [u8] {
    fn configured(&self) -> Start {
        Start::stock(self)
    }
}
impl Input for Box<[u8]> {
    fn configured(&self) -> Start {
        Start::stock(self)
    }
}
#[expect(clippy::disallowed_types, reason = "ordinary test inputs include edited JSON bytes")]
impl Input for Vec<u8> {
    fn configured(&self) -> Start {
        Start::stock(self)
    }
}
impl<const N: usize> Input for [u8; N] {
    fn configured(&self) -> Start {
        Start::stock(self)
    }
}

fn compare(
    left: &jig_domain_core::settings::Override,
    right: &jig_domain_core::settings::Override,
) -> core::cmp::Ordering {
    left.name.cmp(&right.name)
}
