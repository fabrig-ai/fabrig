//! Provider setting JSON (jig/docs/design/domain/engine/llm-providers.md, 6).
use super::{Endpoint, EndpointAddress, ProviderSetting, Registration, Transport};
use crate::{Directories, Path, Problems, What, check, read};
use alloc::boxed::Box;
use skein_json::{Document, writer::Encoder};
use skein_lib::{Decimal, List};

pub fn provider_setting(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<ProviderSetting> {
    let provider = super::providers::setting(document, at, path, problems)?;
    let mut object = object(document, at, path, problems)?;
    let mut endpoints = List::with_capacity(super::ENDPOINTS);
    if let Some(items) = check::list(&mut object, document, b"endpoints", super::ENDPOINTS, path, problems) {
        for (index, at) in items.iter().enumerate() {
            let path = check::index(&path.member(b"endpoints"), index);
            if let Some(value) =
                super::endpoint(document, *at, &path, &Directories::working(), Some(&provider.name), problems)
            {
                endpoints.push(value).expect("bounded endpoints");
            }
        }
    }
    let withdrawn = match object.member(document, b"withdrawn") {
        Some(at) => match read::boolean(document, at) {
            Ok(value) => value,
            Err(what) => {
                problems.push(path.member(b"withdrawn"), what);
                return None;
            }
        },
        None => false,
    };
    let value = ProviderSetting { provider, endpoints: endpoints.into_boxed(), withdrawn };
    if value.provider.number > super::PROVIDERS {
        problems.push(path.clone(), What::Bound);
        return None;
    }
    if !jig_domain_core_vocabulary::settings::providers::fits(&value) {
        problems.push(path.clone(), What::Form);
        return None;
    }
    Some(value)
}
pub fn providers_setting(
    document: &Document,
    at: u32,
    path: &Path,
    problems: &mut Problems,
) -> Option<Box<[ProviderSetting]>> {
    let mut object = object(document, at, path, problems)?;
    let items = check::list(&mut object, document, b"providers", super::PROVIDERS, path, problems)?;
    let mut values = List::with_capacity(items.len());
    let mut endpoints = 0_usize;
    for (index, at) in items.iter().enumerate() {
        let path = check::index(&path.member(b"providers"), index);
        let value = provider_setting(document, *at, &path, problems)?;
        endpoints = endpoints.saturating_add(value.endpoints.len());
        if endpoints > skein_lib::usize_of(super::ENDPOINTS) {
            problems.push(path.clone(), What::Bound);
            return None;
        }
        for previous in &values {
            if !jig_domain_core_vocabulary::settings::providers::distinct(&value, previous) {
                problems.push(path.clone(), What::Unique);
                return None;
            }
        }
        values.push(value).expect("bounded providers");
    }
    object.finish(document, path, problems);
    Some(values.into_boxed())
}
pub fn render_providers(values: &[ProviderSetting], writer: &mut Encoder) {
    writer.object_start();
    writer.key(b"providers");
    writer.array_start();
    for value in values {
        render_provider(value, writer);
    }
    writer.array_end();
    writer.object_end();
}
pub fn render_provider(value: &ProviderSetting, writer: &mut Encoder) {
    let provider = &value.provider;
    writer.object_start();
    writer.key(b"number");
    writer.unsigned(u64::from(provider.number));
    writer.key(b"name");
    writer.string(&provider.name);
    optional(b"endpoint", provider.endpoint.as_deref(), writer);
    optional(b"direct", provider.direct.as_deref(), writer);
    writer.key(b"signed_in_plan");
    writer.boolean(provider.signed_in_plan);
    optional(b"manage_usage", provider.manage_usage.as_deref(), writer);
    if let Some(list) = &provider.list {
        texts(b"keep", &list.keep, writer);
        texts(b"drop", &list.drop, writer);
    }
    writer.key(b"sources");
    writer.object_start();
    if provider.keys {
        writer.key(b"keys");
        writer.boolean(true);
    }
    if !provider.programs.is_empty() {
        texts(b"programs", &provider.programs, writer);
    }
    if let Some(value) = &provider.sign_in {
        writer.key(b"sign_in");
        render_registration(value, writer);
    }
    writer.object_end();
    writer.key(b"endpoints");
    writer.array_start();
    for endpoint in &value.endpoints {
        render_endpoint(endpoint, writer);
    }
    writer.array_end();
    if value.withdrawn {
        writer.key(b"withdrawn");
        writer.boolean(true);
    }
    writer.object_end();
}
fn optional(name: &[u8], value: Option<&[u8]>, writer: &mut Encoder) {
    if let Some(value) = value {
        writer.key(name);
        writer.string(value);
    }
}
fn texts(name: &[u8], values: &[Box<[u8]>], writer: &mut Encoder) {
    writer.key(name);
    writer.array_start();
    for value in values {
        writer.string(value);
    }
    writer.array_end();
}
fn render_transport(value: &Transport, writer: &mut Encoder) {
    writer.key(b"transport");
    match value {
        Transport::Plaintext => writer.string(b"plaintext"),
        Transport::Tls { server_name, trust } => {
            writer.string(b"tls");
            writer.key(b"server_name");
            writer.string(server_name);
            optional(b"trust", trust.as_deref(), writer);
        }
    }
}
fn render_registration(value: &Registration, writer: &mut Encoder) {
    writer.object_start();
    for (name, text) in [
        (b"issuer".as_slice(), &value.issuer),
        (b"client", &value.client),
        (b"authorization", &value.authorization),
        (b"token", &value.token),
        (b"keys", &value.keys),
        (b"redirect_path", &value.redirect_path),
    ] {
        writer.key(name);
        writer.string(text);
    }
    texts(b"scopes", &value.scopes, writer);
    texts(b"must_grant", &value.must_grant, writer);
    optional(b"name_hint", value.name_hint.as_deref(), writer);
    optional(b"revocation", value.revocation.as_deref(), writer);
    optional(b"resource", value.resource.as_deref(), writer);
    if let Some(port) = value.redirect_port {
        writer.key(b"redirect_port");
        writer.unsigned(u64::from(port));
    }
    render_transport(&value.endpoint_transport, writer);
    writer.object_end();
}
fn render_endpoint(value: &Endpoint, writer: &mut Encoder) {
    writer.object_start();
    writer.key(b"name");
    writer.string(&value.name);
    writer.key(b"number");
    writer.unsigned(u64::from(value.number));
    writer.key(b"dialect");
    writer.unsigned(u64::from(value.dialect));
    optional(b"program", value.program.as_deref(), writer);
    writer.key(b"address");
    writer.object_start();
    match &value.address {
        EndpointAddress::Host { name, port } => {
            writer.key(b"host");
            writer.string(name);
            writer.key(b"port");
            writer.unsigned(u64::from(*port));
        }
        EndpointAddress::Socket(socket) => {
            writer.key(b"socket");
            writer.string(&socket_text(*socket));
        }
    }
    writer.object_end();
    render_transport(&value.transport, writer);
    writer.key(b"headers");
    writer.array_start();
    for header in &value.headers {
        writer.object_start();
        writer.key(b"name");
        writer.string(&header.name);
        writer.key(b"value");
        writer.string(&header.value);
        writer.object_end();
    }
    writer.array_end();
    writer.key(b"connect_ms");
    writer.unsigned(value.connect.as_millis());
    writer.key(b"handshake_ms");
    writer.unsigned(value.handshake.as_millis());
    writer.object_end();
}
fn socket_text(socket: core::net::SocketAddr) -> Box<[u8]> {
    let mut writer = List::with_capacity(128);
    match socket {
        core::net::SocketAddr::V4(value) => {
            for (at, byte) in value.ip().octets().iter().enumerate() {
                if at != 0 {
                    socket_put(&mut writer, b".");
                }
                socket_put(&mut writer, Decimal::of(u64::from(*byte)).as_bytes());
            }
        }
        core::net::SocketAddr::V6(value) => {
            socket_put(&mut writer, b"[");
            for (at, word) in value.ip().segments().iter().enumerate() {
                if at != 0 {
                    socket_put(&mut writer, b":");
                }
                for shift in [12_u32, 8, 4, 0] {
                    let digit = u8::try_from(word.checked_shr(shift).expect("hextet shift") & 15).expect("hex digit");
                    let byte = if digit < 10 {
                        b'0'.checked_add(digit).expect("digit")
                    } else {
                        b'a'.checked_add(digit.checked_sub(10).expect("hex letter")).expect("hex letter")
                    };
                    socket_put(&mut writer, &[byte]);
                }
            }
            if value.scope_id() != 0 {
                socket_put(&mut writer, b"%");
                socket_put(&mut writer, Decimal::of(u64::from(value.scope_id())).as_bytes());
            }
            socket_put(&mut writer, b"]");
        }
    }
    socket_put(&mut writer, b":");
    socket_put(&mut writer, Decimal::of(u64::from(socket.port())).as_bytes());
    writer.into_boxed()
}

pub(super) fn object(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<read::Object> {
    match read::Object::new(document, at) {
        Ok(value) => Some(value),
        Err(what) => {
            problems.push(path.clone(), what);
            None
        }
    }
}

fn socket_put(writer: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        writer.push(*byte).expect("socket text bound");
    }
}
