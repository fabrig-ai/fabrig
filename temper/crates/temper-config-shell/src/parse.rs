//! Spanned TOML checked against the rows before any document is constructed (file.md, 2 and 6).
use crate::{
    FILE_BYTES, Facts, File, ForgeSignIn, Internal, Keys, Member, OpenId, OpenIdClient, Programs, Push, Read, Refusal,
    Shape, SignIn, Test, TestGitHub, TestGoogle, Transport, binary, document,
    rows::{self, Form, Required},
    source::at,
};
use serde::Deserialize;
use std::{
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};
use toml::{
    Value,
    de::{DeTable, DeValue},
};

pub fn parse(text: &[u8], shape: Shape, facts: &Facts) -> Result<Read, Refusal> {
    if text.len() > FILE_BYTES {
        return Err(Refusal::one("temper.toml: file exceeds 1 MiB".into()));
    }
    let text = std::str::from_utf8(text).map_err(|why| {
        let line = text.get(..why.valid_up_to()).expect("UTF-8 prefix").split(|byte| *byte == b'\n').count();
        Refusal::one(format!("temper.toml:{line}: must be UTF-8"))
    })?;
    let spanned = DeTable::parse(text).map_err(|why| {
        Refusal::one(format!("temper.toml:{}: {}", at(text, why.span().map_or(0, |span| span.start)), why.message()))
    })?;
    let decoded = toml::Table::deserialize(toml::de::Deserializer::from(spanned.clone())).expect("parsed TOML");
    let mut reader = Reader { text, shape, keys: Keys::default(), refusal: Refusal::empty(), environment: Vec::new() };
    reader.table(spanned.get_ref(), &Value::Table(decoded.clone()), "");
    reader.groups(&decoded);
    if !reader.refusal.problems.is_empty() {
        return Err(reader.refusal);
    }
    let file = file(&decoded, reader.environment);
    reader.keys.record_rows(shape, &file, facts);
    let document = document::build(&file, shape, facts)?;
    Ok(Read { file, document, keys: reader.keys })
}
struct Reader<'a> {
    text: &'a str,
    shape: Shape,
    keys: Keys,
    refusal: Refusal,
    environment: Vec<(usize, String, String)>,
}
impl Reader<'_> {
    fn table(&mut self, table: &DeTable<'_>, value: &Value, parent: &str) {
        for (key, spanned) in table {
            let name = key.get_ref().as_ref();
            let path = if parent.is_empty() { name.to_owned() } else { format!("{parent}.{name}") };
            let line = at(self.text, key.span().start);
            self.keys.sources.push((path.clone(), line));
            let value = value.get(name).expect("same parsed key");
            if parent == "environment" {
                if name.is_empty() || name.contains(['=', '\0']) || value.as_str().is_none_or(|s| s.contains('\0')) {
                    self.refusal.push(&path, Some(line), &Form::Environment.words());
                } else {
                    self.environment.push((key.span().start, name.into(), value.as_str().expect("string").into()));
                }
                continue;
            }
            let normalized = normalized(&path);
            let Some(row) = rows::find(&normalized) else {
                let words = match rows::MOVED.iter().find(|(key, _)| *key == path) {
                    Some((_, words)) => format!("moved: {}", words.replace("{binary}", binary(self.shape))),
                    None => format!("unknown; {} help config lists the keys", binary(self.shape)),
                };
                self.refusal.push(&path, Some(line), &words);
                continue;
            };
            if !row.shapes.contains(&self.shape) {
                let names =
                    row.shapes.iter().map(|shape| format!("{}'s", binary(*shape))).collect::<Vec<_>>().join(" or ");
                self.refusal.push(&path, Some(line), &format!("a key of {names} file, not {}'s", binary(self.shape)));
                continue;
            }
            if !valid(value, row.form) {
                self.refusal.push(&path, Some(line), &row.form.words());
                continue;
            }
            match spanned.get_ref() {
                DeValue::Table(child) => self.table(child, value, &path),
                DeValue::Array(array) if row.form == Form::Members || row.form == Form::Records => {
                    for (index, item) in array.iter().enumerate() {
                        let entry = format!("{path}[{index}]");
                        if let DeValue::Table(fields) = item.get_ref() {
                            self.keys.sources.push((entry.clone(), at(self.text, item.span().start)));
                            self.table(fields, value.get(index).expect("same array"), &entry);
                        } else {
                            self.refusal.push(&entry, Some(at(self.text, item.span().start)), "a table");
                        }
                    }
                }
                DeValue::String(_)
                | DeValue::Integer(_)
                | DeValue::Float(_)
                | DeValue::Boolean(_)
                | DeValue::Datetime(_)
                | DeValue::Array(_) => {}
            }
        }
        for row in rows::ROWS {
            let (prefix, name) = row.key.rsplit_once('.').unwrap_or(("", row.key));
            if prefix != normalized(parent) || !row.shapes.contains(&self.shape) {
                continue;
            }
            let required = match row.required {
                Required::No => false,
                Required::Parent => true,
                Required::Engine => self.shape == Shape::EngineAlone,
                Required::Worker => self.shape == Shape::WorkerAlone,
            };
            if required && value.get(name).is_none() {
                let path = if parent.is_empty() { name.to_owned() } else { format!("{parent}.{name}") };
                self.refusal.push(&path, None, &format!("required in {}'s file", binary(self.shape)));
            }
        }
    }
    fn groups(&mut self, root: &toml::Table) {
        let root = Value::Table(root.clone());
        if self.shape != Shape::WorkerAlone
            && root.get("webhooks").is_none()
            && root
                .get("listen")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<SocketAddr>().ok())
                .is_some_and(|a| a.port() == u16::MAX)
        {
            self.refusal.push("webhooks (its default)", None, "listen uses port 65535; give webhooks explicitly");
        }
        let google = lookup(&root, "sign_in.google").is_some_and(Value::is_table);
        for key in ["sign_in.members", "sign_in.forges", "test.google"] {
            if lookup(&root, key).is_some() && !google {
                self.refusal.push(key, self.keys.line(key), "requires [sign_in.google]");
            }
        }
        if google && lookup(&root, "sign_in.members").is_none() {
            self.refusal.push("sign_in.members", None, "required with [sign_in.google]");
        }
        if lookup(&root, "test.push").is_some() && lookup(&root, "push").is_none() {
            self.refusal.push("test.push", self.keys.line("test.push"), "requires [push]");
        }
        if let Some(github) = lookup(&root, "test.github")
            && github.get("transport").and_then(Value::as_str) == Some("plaintext")
            && !github
                .get("api")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<SocketAddr>().ok())
                .is_some_and(|a| a.ip().is_loopback())
        {
            self.refusal.push(
                "test.github.transport",
                self.keys.line("test.github.transport"),
                "plaintext requires a loopback socket",
            );
        }
    }
}
fn normalized(path: &str) -> String {
    let mut out = String::new();
    let mut index = false;
    for c in path.chars() {
        if c == '[' {
            index = true;
            out.push(c);
        } else if c == ']' {
            index = false;
            out.push(c);
        } else if !index {
            out.push(c);
        }
    }
    out
}
fn visible(s: &str, max: usize) -> bool {
    !s.is_empty() && s.len() <= max && s.bytes().all(|b| b.is_ascii_graphic())
}
fn data_path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 4096
        && !s.contains('\0')
        && (Path::new(s).is_absolute() || !Path::new(s).components().any(|c| c == Component::ParentDir))
}
fn address(s: &str) -> bool {
    visible(s, 254)
        && s.len() >= 3
        && s.split_once('@').is_some_and(|(a, b)| !a.is_empty() && !b.is_empty() && !b.contains('@'))
}
fn endpoint(s: &str, max: usize, origin: bool) -> bool {
    s.len() <= max
        && temper_service_core::startup::url::url(s.as_bytes())
            .is_some_and(|(base, _, _, _)| !origin || base == s.as_bytes())
}
fn valid(value: &Value, form: Form) -> bool {
    match form {
        Form::Table | Form::Environment => value.is_table(),
        Form::Members => value.as_array().is_some_and(|a| (1..=64).contains(&a.len())),
        Form::Records => value.is_array(),
        Form::Forge => value.as_integer().is_some_and(|n| (1..=65535).contains(&n)),
        Form::Addresses => {
            value.as_array().is_some_and(|a| a.iter().all(|v| v.as_integer() == Some(1) || v.as_integer() == Some(3)))
        }
        Form::Services => value.as_array().is_some_and(|a| {
            (1..=8).contains(&a.len()) && a.iter().all(|v| v.as_str().is_some_and(|s| endpoint(s, 1024, true)))
        }),
        Form::Transport => value.as_str() == Some("plaintext") || value.is_table(),
        Form::Socket => value.as_str().and_then(|s| s.parse::<SocketAddr>().ok()).is_some_and(|a| a.ip().is_loopback()),
        Form::Origin => value.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 267),
        Form::Name => value.as_str().is_some_and(|s| !s.is_empty() && s.len() <= 32),
        Form::Address => value.as_str().is_some_and(address),
        Form::Visible(max) => value.as_str().is_some_and(|s| visible(s, max)),
        Form::DataPath | Form::ConfigPath => value.as_str().is_some_and(data_path),
        Form::AbsolutePath => {
            value.as_str().is_some_and(|s| Path::new(s).is_absolute() && s.len() <= 1_048_576 && !s.contains('\0'))
        }
        Form::Login => value.as_str().is_some_and(|s| {
            !s.is_empty()
                && s.len() <= 40
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                && s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
                && s.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
                && !s.eq_ignore_ascii_case("temper")
        }),
        Form::Redirect => value.as_str().is_some_and(|s| endpoint(s, 2048, false)),
        Form::Subject => value.as_str().is_some_and(|s| {
            visible(s, 256) && (s.starts_with("mailto:") || s.starts_with("https:")) && !s.contains(['"', '\\'])
        }),
        Form::Endpoint => value.as_str().is_some_and(|s| endpoint(s, 2048, false)),
        Form::Api => value.as_str().is_some_and(|s| {
            s.parse::<SocketAddr>().is_ok()
                || s.rsplit_once(':').is_some_and(|(h, p)| visible(h, 253) && p.parse::<u16>().is_ok())
        }),
        Form::Base => value.as_str().is_some_and(|s| !s.ends_with('/') && endpoint(s, 4096, false)),
    }
}
fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(value, |v, key| v.get(key))
}
fn text(value: &Value, path: &str) -> Option<String> {
    lookup(value, path).and_then(Value::as_str).map(str::to_owned)
}
fn path(value: &Value, key: &str) -> Option<PathBuf> {
    text(value, key).map(PathBuf::from)
}
fn items<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    lookup(value, key).and_then(Value::as_array).map_or(&[], Vec::as_slice)
}
fn transport(value: &Value) -> Transport {
    if value.is_str() {
        Transport::Plaintext
    } else {
        Transport::Tls {
            server_name: text(value, "server_name").expect("checked server name"),
            trust: path(value, "trust"),
        }
    }
}
fn file(table: &toml::Table, mut environment: Vec<(usize, String, String)>) -> File {
    let value = Value::Table(table.clone());
    environment.sort_by_key(|(offset, _, _)| *offset);
    let sign_in = lookup(&value, "sign_in.google").map(|google| SignIn {
        client: text(google, "client").expect("checked client"),
        secret: path(google, "secret").expect("checked secret"),
        members: items(&value, "sign_in.members")
            .iter()
            .map(|v| Member {
                address: text(v, "address").expect("checked address"),
                login: text(v, "login").expect("checked login"),
            })
            .collect(),
        forges: items(&value, "sign_in.forges")
            .iter()
            .map(|v| ForgeSignIn {
                forge: u16::try_from(v.get("forge").and_then(Value::as_integer).expect("checked forge"))
                    .expect("bounded forge"),
                client: text(v, "client").expect("checked client"),
            })
            .collect(),
    });
    let openid = lookup(&value, "openid").map(|v| OpenId {
        key: path(v, "key"),
        addresses: v.get("addresses").map(|v| {
            v.as_array()
                .expect("checked array")
                .iter()
                .map(|n| u16::try_from(n.as_integer().expect("checked number")).expect("bounded number"))
                .collect()
        }),
        clients: items(v, "clients")
            .iter()
            .map(|v| OpenIdClient {
                id: text(v, "id").expect("checked id"),
                secret: path(v, "secret"),
                redirect: text(v, "redirect").expect("checked redirect"),
            })
            .collect(),
    });
    File {
        listen: text(&value, "listen").map(|s| s.parse().expect("checked socket")),
        public: text(&value, "public"),
        webhooks: text(&value, "webhooks").map(|s| s.parse().expect("checked socket")),
        owner: text(&value, "owner"),
        workers: text(&value, "workers").map(|s| s.parse().expect("checked socket")),
        engine: text(&value, "engine").map(|s| s.parse().expect("checked socket")),
        name: text(&value, "name"),
        sign_in,
        openid,
        push: lookup(&value, "push").map(|v| Push {
            key: path(v, "key").expect("checked key"),
            subject: text(v, "subject").expect("checked subject"),
        }),
        programs: Programs {
            git: path(&value, "programs.git"),
            claude: path(&value, "programs.claude"),
            forgejo: path(&value, "programs.forgejo"),
            runner: path(&value, "programs.runner"),
        },
        environment: environment.into_iter().map(|(_, k, v)| (k, v)).collect(),
        internal: Internal { bundle: path(&value, "internal.bundle") },
        test: Test {
            github: lookup(&value, "test.github").map(|v| TestGitHub {
                api: text(v, "api").expect("checked API"),
                transport: transport(v.get("transport").expect("checked transport")),
                remote: text(v, "remote").expect("checked remote"),
                web: text(v, "web").expect("checked web"),
                git: path(v, "git"),
            }),
            google: lookup(&value, "test.google").map(|v| TestGoogle {
                issuer: text(v, "issuer").expect("checked issuer"),
                authorization: text(v, "authorization").expect("checked authorization"),
                token: text(v, "token").expect("checked token"),
                keys: text(v, "keys").expect("checked keys"),
                transport: transport(v.get("transport").expect("checked transport")),
            }),
            push_services: lookup(&value, "test.push.services").map(|v| {
                v.as_array()
                    .expect("checked services")
                    .iter()
                    .map(|v| v.as_str().expect("checked service").into())
                    .collect()
            }),
        },
    }
}
