//! First preparation's flags (temper/docs/design/protocol/setup/place.md, 4).
use std::{ffi::OsString, os::unix::ffi::OsStrExt};
use temper_forge_setup::{
    machine::OwnedMachine,
    setup::{Account, Fetch, Runner},
    setup_file::{self, SetupFile},
};

#[derive(Debug)]
pub(super) struct Flags(Vec<(String, Vec<u8>)>);
impl Flags {
    pub(super) fn parse(arguments: &[OsString]) -> Result<Self, String> {
        let mut values = Vec::new();
        let mut args = arguments.iter();
        while let Some(flag) = args.next() {
            let flag = flag.to_str().ok_or("flag is not UTF-8")?;
            if ![
                "--listen",
                "--remote",
                "--public",
                "--person",
                "--forgejo",
                "--runner-label",
                "--runner-binary",
                "--runner-capacity",
                "--runner-path",
                "--sign-in",
            ]
            .contains(&flag)
            {
                return Err(format!("{flag}: unknown flag"));
            }
            if flag != "--runner-label" && values.iter().any(|(name, _)| name == flag) {
                return Err(format!("{flag}: repeated"));
            }
            let value = args.next().ok_or_else(|| format!("{flag}: missing value"))?;
            let mut value = value.as_bytes().to_vec();
            if flag == "--person" {
                value.push(0);
                value.extend_from_slice(args.next().ok_or("--person: missing EMAIL")?.as_bytes());
            }
            values.push((flag.to_string(), value));
        }
        Ok(Self(values))
    }
    fn value(&self, flag: &str) -> Option<&[u8]> {
        self.0.iter().find(|(name, _)| name == flag).map(|(_, value)| value.as_slice())
    }
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub(super) fn make(&self, directories: &jig_protocol_config::Directories) -> Result<SetupFile, String> {
        let listen = self.value("--listen").ok_or("--listen: required for the first prepare")?;
        let listen = std::str::from_utf8(listen)
            .ok()
            .and_then(|v| v.parse::<std::net::SocketAddr>().ok())
            .ok_or("--listen: invalid form")?;
        let person = self.value("--person").ok_or("--person: required for the first prepare")?;
        let at = person.iter().position(|b| *b == 0).expect("parsed person has two values");
        let (login, tail) = person.split_at(at);
        let email = tail.strip_prefix(&[0]).expect("person separator");
        let mut labels = Vec::new();
        for (name, value) in &self.0 {
            if name == "--runner-label" {
                labels.push(value.clone().into_boxed_slice());
            }
        }
        let capacity = match self.value("--runner-capacity") {
            Some(v) => std::str::from_utf8(v)
                .ok()
                .and_then(|v| v.parse::<u32>().ok())
                .ok_or("--runner-capacity: invalid form")?,
            None => 1,
        };
        let runner = if labels.is_empty() {
            None
        } else {
            {
                let mut runner = Runner::stock(
                    self.value("--runner-binary").unwrap_or(b"fetched").into(),
                    labels.into_boxed_slice(),
                );
                runner.capacity = capacity;
                if let Some(path) = self.value("--runner-path") {
                    runner.path = path.into();
                }
                Some(runner)
            }
        };
        if runner.is_none() {
            for flag in ["--runner-binary", "--runner-capacity", "--runner-path"] {
                if self.value(flag).is_some() {
                    return Err(format!("{flag}: requires --runner-label"));
                }
            }
        }
        let public = self.value("--public").map(|v| temper_forge_setup::setup::Origin { host: v.into(), port: 443 });
        // The encoder takes a typed origin; validate the flag before constructing it.
        let public = if let Some(v) = public {
            let parsed =
                skein_forge::github::redirect::parse(&v.host, 4096, 1).map_err(|_| "--public: invalid form")?;
            if !v.host.starts_with(b"https://")
                || v.host.strip_prefix(b"https://").is_some_and(|tail| tail.iter().any(|b| b"/?#".contains(b)))
                || parsed.origin.port == 0
            {
                return Err("--public: invalid form".into());
            }
            Some(temper_forge_setup::setup::Origin { host: parsed.origin.host, port: parsed.origin.port })
        } else {
            None
        };
        let file = SetupFile::Owned(OwnedMachine {
            listen,
            remote: self
                .value("--remote")
                .map_or_else(|| format!("http://{listen}").into_bytes().into_boxed_slice(), Into::into),
            person: Account { login: login.into(), email: email.into() },
            public,
            forgejo: self.value("--forgejo").unwrap_or(b"fetched").into(),
            fetch: Fetch {
                forgejo: self.value("--forgejo").is_none_or(|v| v == b"fetched"),
                runner: self.value("--runner-binary").is_none_or(|v| v == b"fetched"),
                mirror: None,
            },
            runner,
            sign_in: self.value("--sign-in").map(Into::into),
        });
        setup_file::decode(&setup_file::encode(&file), directories).map_err(|why| match why {
            setup_file::Refused::Members(ps) => {
                let mut lines = temper_forge_setup::problem::lines(&ps);
                // Configuration paths become the flag spelling at this boundary.
                for (member, flag) in [
                    ("listen", "--listen"),
                    ("public", "--public"),
                    ("person.login", "--person"),
                    ("person.email", "--person"),
                    ("runner.labels", "--runner-label"),
                    ("runner.binary", "--runner-binary"),
                    ("runner.capacity", "--runner-capacity"),
                    ("runner.path", "--runner-path"),
                    ("sign_in", "--sign-in"),
                    ("forgejo", "--forgejo"),
                    ("remote", "--remote"),
                ] {
                    let text = String::from_utf8_lossy(&lines).replace(member, flag);
                    lines = text.into_bytes().into_boxed_slice();
                }
                String::from_utf8_lossy(&lines).into_owned()
            }
            setup_file::Refused::Json | setup_file::Refused::Version(_) => "invalid setup flags".into(),
        })
    }
    pub(super) fn compare(
        &self,
        kept: &SetupFile,
        directories: &jig_protocol_config::Directories,
    ) -> Result<(), String> {
        if self.is_empty() {
            return Ok(());
        }
        let SetupFile::Owned(owned) = kept else {
            return Err("setup is fake".into());
        };
        let mut full = Self(Vec::new());
        for flag in ["--listen", "--remote", "--person", "--forgejo"] {
            full.0.push((flag.into(), field(owned, flag)));
        }
        for flag in ["--public", "--sign-in"] {
            let value = field(owned, flag);
            if !value.is_empty() {
                full.0.push((flag.into(), value));
            }
        }
        if let Some(runner) = &owned.runner {
            for label in &runner.labels {
                full.0.push(("--runner-label".into(), label.to_vec()));
            }
            for flag in ["--runner-binary", "--runner-capacity", "--runner-path"] {
                full.0.push((flag.into(), field(owned, flag)));
            }
        }
        for (name, _) in &self.0 {
            full.0.retain(|(kept, _)| kept != name);
        }
        full.0.extend(self.0.clone());
        let SetupFile::Owned(candidate) = full.make(directories)? else { unreachable!("prepare flags make owned") };
        for (flag, _) in &self.0 {
            let old = field(owned, flag);
            let new = field(&candidate, flag);
            if old != new {
                let member = match flag.as_str() {
                    "--runner-label" => "runner.labels",
                    "--runner-binary" => "runner.binary",
                    "--runner-capacity" => "runner.capacity",
                    "--runner-path" => "runner.path",
                    "--sign-in" => "sign_in",
                    other => other.trim_start_matches("--"),
                };
                return Err(format!(
                    "setup differs at {member}: the file has {}, the flag {}",
                    String::from_utf8_lossy(&old).replace('\0', " "),
                    String::from_utf8_lossy(&new).replace('\0', " ")
                ));
            }
        }
        Ok(())
    }
}
fn field(owned: &OwnedMachine, flag: &str) -> Vec<u8> {
    match flag {
        "--listen" => owned.listen.to_string().into_bytes(),
        "--remote" => owned.remote.to_vec(),
        "--public" => owned.public.as_ref().map_or_else(Vec::new, |o| setup_file::origin(o).into_vec()),
        "--person" => [owned.person.login.as_ref(), b"\0", owned.person.email.as_ref()].concat(),
        "--forgejo" => {
            if owned.fetch.forgejo {
                b"fetched".to_vec()
            } else {
                owned.forgejo.to_vec()
            }
        }
        "--runner-binary" => owned
            .runner
            .as_ref()
            .map_or_else(Vec::new, |r| if owned.fetch.runner { b"fetched".to_vec() } else { r.binary.to_vec() }),
        "--runner-capacity" => owned.runner.as_ref().map_or_else(Vec::new, |r| r.capacity.to_string().into_bytes()),
        "--runner-path" => owned.runner.as_ref().map_or_else(Vec::new, |r| r.path.to_vec()),
        "--sign-in" => owned.sign_in.as_ref().map_or_else(Vec::new, |v| v.to_vec()),
        "--runner-label" => owned.runner.as_ref().map_or_else(Vec::new, |r| {
            format!(
                "[{}]",
                r.labels
                    .iter()
                    .map(|label| format!("{:?}", String::from_utf8_lossy(label)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .into_bytes()
        }),
        _ => unreachable!("parsed flag"),
    }
}
