//! The start's document, in the existing decoders' forms (file.md, 4).
use crate::{Facts, File, Refusal, Shape, Transport};
use skein_json::writer::{Encoder, Limits};
use std::{net::SocketAddr, path::Path};

pub(crate) fn build(file: &File, shape: Shape, facts: &Facts) -> Result<Box<[u8]>, Refusal> {
    let listen = file.listen.unwrap_or_else(|| "127.0.0.1:7100".parse().expect("constant socket"));
    let webhooks = if let Some(socket) = file.webhooks {
        socket
    } else {
        let port = if listen.port() == 0 { 0 } else { listen.port().checked_add(1).expect("webhooks default checked") };
        SocketAddr::new(listen.ip(), port)
    };
    let limits = Limits { depth: 32, length: 16 << 20 };
    let mut encoder = Encoder::measure(&limits);
    encode(&mut encoder, file, shape, facts, listen, webhooks);
    let length = encoder.measured().map_err(|why| Refusal::one(format!("temper.toml: start's document: {why:?}")))?;
    let mut encoder = Encoder::write(length, &limits);
    encode(&mut encoder, file, shape, facts, listen, webhooks);
    Ok(encoder.finish())
}
fn text(json: &mut Encoder, key: &str, value: &str) {
    json.key(key.as_bytes());
    json.string(value.as_bytes());
}
fn path(json: &mut Encoder, key: &str, value: Option<&Path>) {
    json.key(key.as_bytes());
    match value {
        Some(value) => json.string(value.to_str().expect("file paths are UTF-8").as_bytes()),
        None => json.null(),
    }
}
fn socket(json: &mut Encoder, value: SocketAddr) {
    json.object_start();
    text(json, "socket", &value.to_string());
    json.object_end();
}
fn channel(json: &mut Encoder, in_process: bool, value: Option<SocketAddr>) {
    if in_process {
        json.object_start();
        text(json, "in_process", "workers");
        json.object_end();
    } else {
        socket(json, value.expect("required socket"));
    }
}
fn encode(json: &mut Encoder, file: &File, shape: Shape, facts: &Facts, listen: SocketAddr, webhooks: SocketAddr) {
    json.object_start();
    if shape != Shape::WorkerAlone {
        json.key(b"engine");
        json.object_start();
        json.key(b"hosts");
        json.object_start();
        json.key(b"listener");
        json.array_start();
        channel(json, shape == Shape::Standalone, file.workers);
        json.array_end();
        json.object_end();
        json.key(b"owners");
        json.array_start();
        json.object_start();
        json.key(b"provider");
        json.unsigned(if file.sign_in.is_some() { 3 } else { 1 });
        text(json, "subject", &file.owner.clone().unwrap_or_else(|| format!("{}@localhost", facts.login)));
        json.object_end();
        json.array_end();
        json.key(b"clients");
        json.object_start();
        if let Some(provider) = &file.openid {
            json.key(b"provider");
            json.object_start();
            path(json, "key", Some(provider.key.as_deref().unwrap_or_else(|| Path::new("provider.key"))));
            json.key(b"clients");
            json.array_start();
            for client in &provider.clients {
                json.object_start();
                text(json, "id", &client.id);
                path(json, "secret", client.secret.as_deref());
                text(json, "redirect_uri", &client.redirect);
                json.object_end();
            }
            json.array_end();
            json.key(b"addresses");
            json.array_start();
            for address in provider.addresses.as_deref().unwrap_or(if file.sign_in.is_some() { &[3] } else { &[1] }) {
                json.unsigned(u64::from(*address));
            }
            json.array_end();
            json.object_end();
        } else if shape == Shape::Standalone && facts.provider_key {
            json.key(b"provider");
            json.object_start();
            text(json, "key", "provider.key");
            json.key(b"clients");
            json.array_start();
            json.array_end();
            json.key(b"addresses");
            json.array_start();
            json.unsigned(1);
            json.array_end();
            json.object_end();
        }
        json.object_end();
        json.object_end();
    }
    json.key(b"workers");
    json.array_start();
    if shape != Shape::EngineAlone {
        json.object_start();
        json.key(b"engine");
        channel(json, shape == Shape::Standalone, file.engine);
        text(
            json,
            "name",
            if shape == Shape::Standalone { "local" } else { file.name.as_deref().unwrap_or(&facts.hostname) },
        );
        text(json, "secret", if shape == Shape::Standalone { "worker.secret" } else { "secrets/worker.secret" });
        json.key(b"slots");
        json.unsigned(1);
        json.object_end();
    }
    json.array_end();
    json.key(b"application");
    json.object_start();
    if shape != Shape::WorkerAlone {
        application(json, file, listen, webhooks);
    }
    if shape != Shape::EngineAlone && !file.environment.is_empty() {
        json.key(b"environment");
        json.array_start();
        for (name, value) in &file.environment {
            json.object_start();
            text(json, "name", name);
            text(json, "value", value);
            json.object_end();
        }
        json.array_end();
    }
    json.object_end();
    json.object_end();
}
fn application(json: &mut Encoder, file: &File, listen: SocketAddr, webhooks: SocketAddr) {
    json.key(b"clients");
    json.object_start();
    json.key(b"listener");
    socket(json, listen);
    path(json, "bundle", file.internal.bundle.as_deref());
    json.key(b"public");
    match &file.public {
        Some(s) => json.string(s.as_bytes()),
        None => json.null(),
    }
    json.object_end();
    json.key(b"webhooks");
    json.object_start();
    json.key(b"listener");
    socket(json, webhooks);
    json.object_end();
    sign_in(json, file);
    if let Some(push) = &file.push {
        json.key(b"push");
        json.object_start();
        path(json, "key", Some(&push.key));
        text(json, "subject", &push.subject);
        json.key(b"services");
        if let Some(services) = &file.test.push_services {
            json.array_start();
            for service in services {
                json.string(service.as_bytes());
            }
            json.array_end();
        } else {
            json.null();
        }
        json.object_end();
    }
    if let Some(github) = &file.test.github {
        json.key(b"test");
        json.object_start();
        json.key(b"github");
        json.object_start();
        json.key(b"api");
        if let Ok(address) = github.api.parse::<SocketAddr>() {
            socket(json, address);
        } else {
            let (host, port) = github.api.rsplit_once(':').expect("checked API host and port");
            json.object_start();
            text(json, "host", host);
            json.key(b"port");
            json.unsigned(port.parse().expect("checked API port"));
            json.object_end();
        }
        json.key(b"transport");
        transport(json, &github.transport);
        text(json, "remote", &github.remote);
        text(json, "web", &github.web);
        if github.git.is_some() {
            path(json, "git", github.git.as_deref());
        }
        json.object_end();
        json.object_end();
    }
}
fn sign_in(json: &mut Encoder, file: &File) {
    json.key(b"sign_in");
    if let Some(sign_in) = &file.sign_in {
        json.object_start();
        json.key(b"google");
        json.object_start();
        json.key(b"provider");
        json.unsigned(3);
        text(json, "client", &sign_in.client);
        path(json, "secret", Some(&sign_in.secret));
        json.key(b"endpoints");
        if let Some(google) = &file.test.google {
            json.object_start();
            text(json, "issuer", &google.issuer);
            text(json, "authorization", &google.authorization);
            text(json, "token", &google.token);
            text(json, "keys", &google.keys);
            json.key(b"transport");
            transport(json, &google.transport);
            json.object_end();
        } else {
            json.null();
        }
        json.object_end();
        json.key(b"members");
        json.array_start();
        for member in &sign_in.members {
            json.object_start();
            text(json, "address", &member.address);
            text(json, "login", &member.login);
            json.object_end();
        }
        json.array_end();
        json.key(b"forges");
        json.array_start();
        for forge in &sign_in.forges {
            json.object_start();
            json.key(b"forge");
            json.unsigned(u64::from(forge.forge));
            text(json, "client", &forge.client);
            json.object_end();
        }
        json.array_end();
        json.object_end();
    } else {
        json.null();
    }
}
fn transport(json: &mut Encoder, transport: &Transport) {
    match transport {
        Transport::Plaintext => json.string(b"plaintext"),
        Transport::Tls { server_name, trust } => {
            json.object_start();
            json.key(b"tls");
            json.object_start();
            text(json, "server_name", server_name);
            path(json, "trust", trust.as_deref());
            json.object_end();
            json.object_end();
        }
    }
}
