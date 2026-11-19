//! The file's canonical spelling, with comments from the rows (file.md, 7).
use crate::{
    File, Shape, Transport, binary,
    rows::{self, Form},
};
use std::{fmt::Write, path::Path};
use toml::{Table, Value};

#[must_use]
pub fn render(file: &File, shape: Shape) -> Box<[u8]> {
    let about = match shape {
        Shape::Standalone => "configuration",
        Shape::EngineAlone => "engine",
        Shape::WorkerAlone => "worker",
    };
    let mut out = format!("# temper's {about}. `{} help config` says what each line does.\n", binary(shape));
    if shape != Shape::WorkerAlone {
        out.push_str("# Everything else is changed by asking temper.\n");
    }
    out.push('\n');
    let value = values(file);
    table(&mut out, &value, "", "", shape, file);
    out.into_bytes().into_boxed_slice()
}
fn table(out: &mut String, value: &Table, template: &str, path: &str, shape: Shape, file: &File) {
    let children: Vec<_> = rows::ROWS
        .iter()
        .filter(|row| {
            row.shapes.contains(&shape) && row.key.rsplit_once('.').map_or("", |(parent, _)| parent) == template
        })
        .collect();
    for row in &children {
        let key = row.key.rsplit('.').next().expect("key");
        if let Some(value) = value.get(key)
            && row.form != Form::Table
            && row.form != Form::Members
            && row.form != Form::Records
            && row.form != Form::Environment
        {
            writeln!(out, "{key} = {}   # {}", spelling(value), row.line).expect("string");
        }
    }
    for row in children {
        let key = row.key.rsplit('.').next().expect("key");
        let Some(value) = value.get(key) else {
            continue;
        };
        let path = if path.is_empty() { key.to_owned() } else { format!("{path}.{key}") };
        match row.form {
            Form::Table => {
                let value = value.as_table().expect("typed table");
                let direct = rows::ROWS.iter().any(|child| {
                    child
                        .key
                        .rsplit_once('.')
                        .is_some_and(|(p, k)| p == row.key && child.form != Form::Table && value.contains_key(k))
                });
                if direct || value.is_empty() {
                    writeln!(out, "\n[{path}]   # {}", row.line).expect("string");
                }
                table(out, value, row.key, &path, shape, file);
            }
            Form::Members | Form::Records => {
                for value in value.as_array().expect("typed records") {
                    writeln!(out, "\n[[{path}]]   # {}", row.line).expect("string");
                    table(out, value.as_table().expect("typed record"), &format!("{}[]", row.key), &path, shape, file);
                }
            }
            Form::Environment => {
                writeln!(out, "\n[{path}]   # {}", row.line).expect("string");
                for (key, value) in &file.environment {
                    writeln!(out, "{} = {}", quote(key), quote(value)).expect("string");
                }
            }
            Form::Socket
            | Form::Origin
            | Form::Address
            | Form::Name
            | Form::Visible(_)
            | Form::DataPath
            | Form::AbsolutePath
            | Form::ConfigPath
            | Form::Login
            | Form::Forge
            | Form::Addresses
            | Form::Redirect
            | Form::Subject
            | Form::Endpoint
            | Form::Api
            | Form::Base
            | Form::Transport
            | Form::Services => {}
        }
    }
}
pub(super) fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                write!(out, "\\u{:04X}", u32::from(c)).expect("string");
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
fn spelling(value: &Value) -> String {
    match value {
        Value::String(s) => quote(s),
        Value::Integer(n) => n.to_string(),
        Value::Array(a) => format!("[{}]", a.iter().map(spelling).collect::<Vec<_>>().join(", ")),
        Value::Table(t) => {
            format!("{{ {} }}", t.iter().map(|(k, v)| format!("{k} = {}", spelling(v))).collect::<Vec<_>>().join(", "))
        }
        Value::Float(_) | Value::Boolean(_) | Value::Datetime(_) => unreachable!("typed configuration values"),
    }
}
fn put(table: &mut Table, key: &str, value: Option<String>) {
    if let Some(value) = value {
        table.insert(key.into(), Value::String(value));
    }
}
fn path(table: &mut Table, key: &str, value: Option<&Path>) {
    put(table, key, value.map(|p| p.to_str().expect("file paths are UTF-8").into()));
}
fn group(table: &mut Table, key: &str, value: Table) {
    table.insert(key.into(), Value::Table(value));
}
fn transport(value: &Transport) -> Value {
    match value {
        Transport::Plaintext => Value::String("plaintext".into()),
        Transport::Tls { server_name, trust } => {
            let mut t = Table::new();
            put(&mut t, "server_name", Some(server_name.clone()));
            path(&mut t, "trust", trust.as_deref());
            Value::Table(t)
        }
    }
}
fn strings(values: &[String]) -> Value {
    Value::Array(values.iter().cloned().map(Value::String).collect())
}
fn values(file: &File) -> Table {
    let mut root = Table::new();
    put(&mut root, "listen", file.listen.map(|a| a.to_string()));
    put(&mut root, "public", file.public.clone());
    put(&mut root, "webhooks", file.webhooks.map(|a| a.to_string()));
    put(&mut root, "owner", file.owner.clone());
    put(&mut root, "workers", file.workers.map(|a| a.to_string()));
    put(&mut root, "engine", file.engine.map(|a| a.to_string()));
    put(&mut root, "name", file.name.clone());
    if let Some(sign_in) = &file.sign_in {
        group(&mut root, "sign_in", sign_in_values(sign_in));
    }
    if let Some(openid) = &file.openid {
        let mut t = Table::new();
        path(&mut t, "key", openid.key.as_deref());
        if let Some(addresses) = &openid.addresses {
            t.insert(
                "addresses".into(),
                Value::Array(addresses.iter().map(|n| Value::Integer(i64::from(*n))).collect()),
            );
        }
        if !openid.clients.is_empty() {
            t.insert(
                "clients".into(),
                Value::Array(
                    openid
                        .clients
                        .iter()
                        .map(|c| {
                            let mut t = Table::new();
                            put(&mut t, "id", Some(c.id.clone()));
                            path(&mut t, "secret", c.secret.as_deref());
                            put(&mut t, "redirect", Some(c.redirect.clone()));
                            Value::Table(t)
                        })
                        .collect(),
                ),
            );
        }
        group(&mut root, "openid", t);
    }
    if let Some(push) = &file.push {
        let mut t = Table::new();
        path(&mut t, "key", Some(&push.key));
        put(&mut t, "subject", Some(push.subject.clone()));
        group(&mut root, "push", t);
    }
    let mut t = Table::new();
    path(&mut t, "git", file.programs.git.as_deref());
    path(&mut t, "claude", file.programs.claude.as_deref());
    path(&mut t, "forgejo", file.programs.forgejo.as_deref());
    path(&mut t, "runner", file.programs.runner.as_deref());
    if !t.is_empty() {
        group(&mut root, "programs", t);
    }
    if !file.environment.is_empty() {
        group(&mut root, "environment", Table::new());
    }
    if let Some(bundle) = &file.internal.bundle {
        let mut t = Table::new();
        path(&mut t, "bundle", Some(bundle));
        group(&mut root, "internal", t);
    }
    let mut t = Table::new();
    if let Some(g) = &file.test.github {
        let mut v = Table::new();
        put(&mut v, "api", Some(g.api.clone()));
        v.insert("transport".into(), transport(&g.transport));
        put(&mut v, "remote", Some(g.remote.clone()));
        put(&mut v, "web", Some(g.web.clone()));
        path(&mut v, "git", g.git.as_deref());
        group(&mut t, "github", v);
    }
    if let Some(g) = &file.test.google {
        let mut v = Table::new();
        put(&mut v, "issuer", Some(g.issuer.clone()));
        put(&mut v, "authorization", Some(g.authorization.clone()));
        put(&mut v, "token", Some(g.token.clone()));
        put(&mut v, "keys", Some(g.keys.clone()));
        v.insert("transport".into(), transport(&g.transport));
        group(&mut t, "google", v);
    }
    if let Some(services) = &file.test.push_services {
        let mut v = Table::new();
        v.insert("services".into(), strings(services));
        group(&mut t, "push", v);
    }
    if !t.is_empty() {
        group(&mut root, "test", t);
    }
    root
}

fn sign_in_values(sign_in: &crate::SignIn) -> Table {
    let mut t = Table::new();
    let mut g = Table::new();
    put(&mut g, "client", Some(sign_in.client.clone()));
    path(&mut g, "secret", Some(&sign_in.secret));
    group(&mut t, "google", g);
    t.insert(
        "members".into(),
        Value::Array(
            sign_in
                .members
                .iter()
                .map(|m| {
                    let mut t = Table::new();
                    put(&mut t, "address", Some(m.address.clone()));
                    put(&mut t, "login", Some(m.login.clone()));
                    Value::Table(t)
                })
                .collect(),
        ),
    );
    t.insert(
        "forges".into(),
        Value::Array(
            sign_in
                .forges
                .iter()
                .map(|f| {
                    let mut t = Table::new();
                    t.insert("forge".into(), Value::Integer(i64::from(f.forge)));
                    put(&mut t, "client", Some(f.client.clone()));
                    Value::Table(t)
                })
                .collect(),
        ),
    );
    t
}
