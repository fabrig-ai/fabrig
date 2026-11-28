//! Spanned TOML checked against the rows before any document is constructed (file.md, 2 and 6).
mod sites;

use crate::{
    FILE_BYTES, Facts, File, ForgeSignIn, Internal, Keys, Member, OpenId, OpenIdClient, Programs, Push, Read, Refusal,
    Shape, SignIn, Test, TestGitHub, TestGoogle, Transport, binary, document,
    formats::{FIRST, Formats, MoveKind, MovedOut, PRODUCTION},
    rows::{self, Form, Required},
    source::{Lines, at, diagnostic},
};
use serde::Deserialize;
use sites::{Children, Fields, Site};
use std::{
    borrow::Borrow,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
};
use toml::{Table, Value, de::DeTable};

pub fn parse(text: &[u8], shape: Shape, facts: &Facts) -> Result<Read, Refusal> {
    parse_at(text, shape, facts, PRODUCTION)
}

pub(crate) fn parse_at(text: &[u8], shape: Shape, facts: &Facts, formats: Formats) -> Result<Read, Refusal> {
    read(text, shape, || Ok(facts), None, formats)
}

pub(super) fn parse_with_facts(
    text: &[u8],
    shape: Shape,
    facts: impl FnOnce() -> Result<Facts, Refusal>,
) -> Result<Read, Refusal> {
    read(text, shape, facts, None, PRODUCTION)
}

/// Each top-level key supplied replaces the whole key; omitted keys stay.
/// Required keys may come from the file (testing/qa/deployment.md, 6.2).
pub fn patch(file: &File, text: &[u8], shape: Shape, facts: &Facts) -> Result<Read, Refusal> {
    let patched = read(text, shape, || Ok(facts), Some(file), PRODUCTION)?;
    // Check inherited keys too and return the lines of the file that will be written.
    parse(&crate::render(&patched.file, shape), shape, facts)
}

pub(super) fn table(text: &[u8]) -> Result<(&str, toml::Spanned<DeTable<'_>>), Refusal> {
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
    Ok((text, spanned))
}

fn read<F: Borrow<Facts>>(
    text: &[u8],
    shape: Shape,
    facts: impl FnOnce() -> Result<F, Refusal>,
    base: Option<&File>,
    formats: Formats,
) -> Result<Read, Refusal> {
    let (text, spanned) = table(text)?;
    let mut decoded = Table::deserialize(toml::de::Deserializer::from(spanned.clone())).expect("parsed TOML");
    let format = formats.read(text, spanned.get_ref(), &decoded, shape)?;
    let lines = Lines::new(text);
    let mut fields = sites::fields(spanned.get_ref());
    // A patch supplies newest-format keys, not another version of the file.
    let changes = if base.is_none() && format < formats.newest {
        Changes::apply(&lines, &mut decoded, &mut fields, format, formats)?
    } else {
        Changes::default()
    };
    let replaces_environment = decoded.contains_key("environment");
    if let Some(base) = base {
        let rendered = crate::render(base, shape);
        let mut table: toml::Table =
            toml::from_str(std::str::from_utf8(&rendered).expect("rendered UTF-8")).expect("rendered TOML");
        table.extend(decoded);
        decoded = table;
    }
    let value = Value::Table(decoded);
    let mut reader =
        Reader { lines: &lines, shape, keys: Keys::default(), refusal: Refusal::empty(), environment: Vec::new() };
    reader.table(&fields, &value, "");
    reader.groups(&value);
    if !reader.refusal.problems.is_empty() {
        return Err(reader.refusal);
    }
    let facts = facts()?;
    let facts = facts.borrow();
    let mut file = file(&value, reader.environment);
    if let Some(base) = base
        && !replaces_environment
    {
        file.environment.clone_from(&base.environment);
    }
    if shape == Shape::WorkerAlone && file.name.is_none() {
        let bound = temper_service_core::derived::stock().link.name;
        let bytes = facts.hostname.len();
        if bytes > usize::try_from(bound).expect("worker name bound fits usize") {
            let words = format!("the host's name is {bytes} bytes, past its bound of {bound}; give name explicitly");
            return Err(Refusal::one(reader.keys.render_key("name", &words)));
        }
    }
    reader.keys.record_rows(shape, &file, facts);
    let document = document::build(&file, shape, facts)?;
    Ok(Read { file, document, keys: reader.keys, format, notices: changes.notices, moved_out: changes.moved_out })
}

#[derive(Default)]
struct Changes {
    notices: Vec<String>,
    moved_out: Vec<MovedOut>,
}
impl Changes {
    fn apply(
        lines: &Lines,
        table: &mut Table,
        fields: &mut Fields<'_>,
        written: i64,
        formats: Formats,
    ) -> Result<Self, Refusal> {
        let mut notices = Vec::new();
        let mut moved_out = Vec::new();
        for movement in formats.moves.iter().filter(|movement| movement.format > written) {
            let key = movement.key;
            let Some((value, site)) = sites::take(table, fields, key, "") else { continue };
            let offset = site.offset;
            let line = lines.at(offset);
            let refuse = |words: &str| Refusal::one(diagnostic(key, Some(line), words, false));
            let destination = match movement.kind {
                MoveKind::Renamed { new } => Some((new, value, site)),
                MoveKind::Converted { new, convert, .. } => {
                    let value = convert(value).map_err(refuse)?;
                    let site = Site::converted(offset, &value);
                    Some((new, value, site))
                }
                MoveKind::MovedOut { .. } => {
                    moved_out.push(MovedOut { key, line, value, movement });
                    None
                }
                MoveKind::Gone { .. } => None,
            };
            if let Some((new, value, site)) = destination
                && !sites::put(table, fields, new, value, site)
            {
                return Err(refuse(&format!("cannot move to {new}; already given")));
            }
            notices.push((offset, diagnostic(key, Some(line), &movement.notice(written, formats.newest), false)));
        }
        notices.sort_by_key(|(offset, _)| *offset);
        Ok(Self { notices: notices.into_iter().map(|(_, notice)| notice).collect(), moved_out })
    }
}

struct Reader<'a> {
    lines: &'a Lines,
    shape: Shape,
    keys: Keys,
    refusal: Refusal,
    environment: Vec<(usize, String, String)>,
}
impl Reader<'_> {
    fn table(&mut self, table: &Fields<'_>, value: &Value, parent: &str) {
        for (name, site) in table {
            let name = name.as_ref();
            let path = if parent.is_empty() { name.to_owned() } else { format!("{parent}.{name}") };
            let line = self.lines.at(site.offset);
            self.keys.sources.push((path.clone(), line));
            let value = value.get(name).expect("same parsed key");
            if parent == "environment" {
                if name.is_empty() || name.contains(['=', '\0']) || value.as_str().is_none_or(|s| s.contains('\0')) {
                    self.refusal.push(&path, Some(line), &Form::Environment.words());
                } else {
                    self.environment.push((site.offset, name.into(), value.as_str().expect("string").into()));
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
            match &site.children {
                Children::Table(child) => self.table(child, value, &path),
                Children::Array(array) if row.form == Form::Members || row.form == Form::Records => {
                    for (index, item) in array.iter().enumerate() {
                        let entry = format!("{path}[{index}]");
                        if let Children::Table(fields) = &item.children {
                            self.keys.sources.push((entry.clone(), self.lines.at(item.offset)));
                            self.table(fields, value.get(index).expect("same array"), &entry);
                        } else {
                            self.refusal.push(&entry, Some(self.lines.at(item.offset)), "a table");
                        }
                    }
                }
                Children::Leaf | Children::Array(_) => {}
            }
        }
        let template = normalized(parent);
        for row in rows::ROWS.iter().filter(|row| row.required != Required::No) {
            let (prefix, name) = row.key.rsplit_once('.').unwrap_or(("", row.key));
            if prefix != template || !row.shapes.contains(&self.shape) {
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
    fn groups(&mut self, root: &Value) {
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
        let google = lookup(root, "sign_in.google").is_some_and(Value::is_table);
        for key in ["sign_in.members", "sign_in.forges", "test.google"] {
            if lookup(root, key).is_some() && !google {
                self.refusal.push(key, self.keys.line(key), "requires [sign_in.google]");
            }
        }
        if google && lookup(root, "sign_in.members").is_none() {
            self.refusal.push("sign_in.members", None, "required with [sign_in.google]");
        }
        if lookup(root, "test.push").is_some() && lookup(root, "push").is_none() {
            self.refusal.push("test.push", self.keys.line("test.push"), "requires [push]");
        }
        if let Some(github) = lookup(root, "test.github")
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
        Form::Format => value.as_integer().is_some_and(|n| n >= FIRST),
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
fn file(value: &Value, mut environment: Vec<(usize, String, String)>) -> File {
    environment.sort_by_key(|(offset, _, _)| *offset);
    let sign_in = lookup(value, "sign_in.google").map(|google| SignIn {
        client: text(google, "client").expect("checked client"),
        secret: path(google, "secret").expect("checked secret"),
        members: items(value, "sign_in.members")
            .iter()
            .map(|v| Member {
                address: text(v, "address").expect("checked address"),
                login: text(v, "login").expect("checked login"),
            })
            .collect(),
        forges: items(value, "sign_in.forges")
            .iter()
            .map(|v| ForgeSignIn {
                forge: u16::try_from(v.get("forge").and_then(Value::as_integer).expect("checked forge"))
                    .expect("bounded forge"),
                client: text(v, "client").expect("checked client"),
            })
            .collect(),
    });
    let openid = lookup(value, "openid").map(|v| OpenId {
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
        listen: text(value, "listen").map(|s| s.parse().expect("checked socket")),
        public: text(value, "public"),
        webhooks: text(value, "webhooks").map(|s| s.parse().expect("checked socket")),
        owner: text(value, "owner"),
        workers: text(value, "workers").map(|s| s.parse().expect("checked socket")),
        engine: text(value, "engine").map(|s| s.parse().expect("checked socket")),
        name: text(value, "name"),
        sign_in,
        openid,
        push: lookup(value, "push").map(|v| Push {
            key: path(v, "key").expect("checked key"),
            subject: text(v, "subject").expect("checked subject"),
        }),
        programs: Programs {
            git: path(value, "programs.git"),
            claude: path(value, "programs.claude"),
            forgejo: path(value, "programs.forgejo"),
            runner: path(value, "programs.runner"),
        },
        environment: environment.into_iter().map(|(_, k, v)| (k, v)).collect(),
        internal: Internal { bundle: path(value, "internal.bundle") },
        test: Test {
            github: lookup(value, "test.github").map(|v| TestGitHub {
                api: text(v, "api").expect("checked API"),
                transport: transport(v.get("transport").expect("checked transport")),
                remote: text(v, "remote").expect("checked remote"),
                web: text(v, "web").expect("checked web"),
                git: path(v, "git"),
            }),
            google: lookup(value, "test.google").map(|v| TestGoogle {
                issuer: text(v, "issuer").expect("checked issuer"),
                authorization: text(v, "authorization").expect("checked authorization"),
                token: text(v, "token").expect("checked token"),
                keys: text(v, "keys").expect("checked keys"),
                transport: transport(v.get("transport").expect("checked transport")),
            }),
            push_services: lookup(value, "test.push.services").map(|v| {
                v.as_array()
                    .expect("checked services")
                    .iter()
                    .map(|v| v.as_str().expect("checked service").into())
                    .collect()
            }),
        },
    }
}
