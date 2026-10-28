//! Members and issuer configuration (protocol/sign-in.md, 3.1).
mod forges;
use super::{Transport, read, resolved_transport};
use crate::{
    limits::SignIn as Limits,
    settings,
    startup::{Problems, What},
};
use alloc::boxed::Box;
pub use forges::ForgeSignIn;
pub(super) use forges::check as check_forges;
use jig_protocol_config::{self as config, Path};
use skein_json::{Document, document::Kind};
use skein_lib::{List, bytes};

#[derive(Clone, Debug)]
pub struct Member {
    pub address: Box<[u8]>,
    pub login: Box<[u8]>,
}
#[derive(Debug)]
pub struct SignIn {
    pub provider: u16,
    pub client: Box<[u8]>,
    pub secret: Box<[u8]>,
    pub endpoints: Endpoints,
    pub members: Box<[Member]>,
    pub forges: Box<[ForgeSignIn]>,
}
#[derive(Debug)]
pub struct Endpoints {
    pub issuer: Box<[u8]>,
    pub authorization: Box<[u8]>,
    pub token: Box<[u8]>,
    pub keys: Box<[u8]>,
    /// None selects TLS with each endpoint's host and the machine's roots.
    pub transport: Option<Transport>,
}
#[must_use]
pub fn path() -> Path {
    Path::root().member(b"application").member(b"sign_in")
}
#[must_use]
pub fn visible(value: &[u8]) -> bool {
    for byte in value {
        if !(b'!'..=b'~').contains(byte) {
            return false;
        }
    }
    !value.is_empty()
}
#[must_use]
pub fn address(value: &[u8]) -> bool {
    if !(3..=254).contains(&value.len()) || !visible(value) {
        return false;
    }
    let Some(at) = bytes::find(value, b"@") else { return false };
    at > 0 && at.saturating_add(1) < value.len() && !value.get(at.saturating_add(1)..).expect("address").contains(&b'@')
}
#[must_use]
pub fn login(value: &[u8]) -> bool {
    if value.is_empty() || value.len() > 40 {
        return false;
    }
    for byte in value {
        if !byte.is_ascii_alphanumeric() && !b"-_.".contains(byte) {
            return false;
        }
    }
    value.first().expect("nonempty").is_ascii_alphanumeric() && value.last().expect("nonempty").is_ascii_alphanumeric()
}
/// Split a bounded HTTP URL into its origin, hostname and port. No userinfo,
/// fragment, controls or backslashes can change the configured destination.
#[expect(clippy::disallowed_methods, reason = "configuration parses bounded URL authorities")]
#[must_use]
pub fn url(value: &[u8]) -> Option<(&[u8], &[u8], u16, bool)> {
    if !visible(value) || value.contains(&b'\\') || value.contains(&b'#') {
        return None;
    }
    let (start, default, tls) = if value.starts_with(b"https://") {
        (8_usize, 443, true)
    } else if value.starts_with(b"http://") {
        (7, 80, false)
    } else {
        return None;
    };
    let rest = value.get(start..)?;
    let mut end = rest.len();
    for separator in [b"/".as_slice(), b"?"] {
        if let Some(at) = bytes::find(rest, separator) {
            end = end.min(at);
        }
    }
    let authority = rest.get(..end)?;
    if authority.is_empty() || authority.contains(&b'@') {
        return None;
    }
    let (host, port) = if let Some(rest) = authority.strip_prefix(b"[") {
        let at = bytes::find(rest, b"]")?;
        let host = rest.get(..at)?;
        let suffix = rest.get(at.checked_add(1)?..)?;
        let port = if suffix.is_empty() { default } else { port(suffix.strip_prefix(b":")?)? };
        core::str::from_utf8(host).ok()?.parse::<core::net::Ipv6Addr>().ok()?;
        (host, port)
    } else if let Some(at) = bytes::find(authority, b":") {
        (authority.get(..at)?, port(authority.get(at.checked_add(1)?..)?)?)
    } else {
        (authority, default)
    };
    if host.is_empty() || port == 0 {
        return None;
    }
    for byte in host {
        if !byte.is_ascii_alphanumeric() && !b".-:".contains(byte) {
            return None;
        }
    }
    if !tls && !loopback(host) {
        return None;
    }
    Some((value.get(..start.checked_add(end)?)?, host, port, tls))
}
fn port(value: &[u8]) -> Option<u16> {
    if value.is_empty() {
        return None;
    }
    let mut number = 0_u16;
    for byte in value {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    if number == 0 { None } else { Some(number) }
}
#[expect(clippy::disallowed_methods, reason = "configuration checks numeric loopback hosts")]
fn loopback(host: &[u8]) -> bool {
    if host == b"localhost" {
        return true;
    }
    let Ok(host) = core::str::from_utf8(host) else { return false };
    match host.parse::<core::net::IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => false,
    }
}
fn endpoint(
    document: &Document,
    object: &mut config::read::Object,
    name: &[u8],
    path: &Path,
    limits: Limits,
    problems: &mut Problems,
) -> Option<Box<[u8]>> {
    let at = read::required(object, document, name, path, problems)?;
    let value = match config::read::text(document, at, limits.url_bytes) {
        Ok(value) => value,
        Err(what) => {
            problems.push(path.member(name), What::Jig(what));
            return None;
        }
    };
    read::form(url(&value).is_some(), path.member(name), problems);
    Some(value)
}
fn endpoints(
    document: &Document,
    at: u32,
    path: &Path,
    limits: Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<Endpoints> {
    if document.token(at)?.kind == Kind::Null {
        return Some(Endpoints {
            issuer: bytes::copy_of(b"https://accounts.google.com"),
            authorization: bytes::copy_of(b"https://accounts.google.com/o/oauth2/v2/auth"),
            token: bytes::copy_of(b"https://oauth2.googleapis.com/token"),
            keys: bytes::copy_of(b"https://www.googleapis.com/oauth2/v3/certs"),
            transport: None,
        });
    }
    let mut object = read::object(document, at, path, problems)?;
    let issuer = endpoint(document, &mut object, b"issuer", path, limits, problems);
    let authorization = endpoint(document, &mut object, b"authorization", path, limits, problems);
    let token = endpoint(document, &mut object, b"token", path, limits, problems);
    let keys = endpoint(document, &mut object, b"keys", path, limits, problems);
    let at = read::required(&mut object, document, b"transport", path, problems);
    let transport = super::transport(document, at?, &path.member(b"transport"), problems);
    read::finish(object, document, path, problems);
    let mut transport = transport?;
    if let Transport::Tls { trust: Some(trust), .. } = &mut transport
        && let Some(value) = super::joined(
            directories,
            config::Directory::Config,
            trust,
            &path.member(b"transport").member(b"tls").member(b"trust"),
            problems,
        )
    {
        *trust = value;
    }
    let endpoints = Endpoints {
        issuer: issuer?,
        authorization: authorization?,
        token: token?,
        keys: keys?,
        transport: Some(transport),
    };
    for (name, value) in endpoints.values() {
        if let Some((_, host, _, tls)) = url(value) {
            let plain = match &endpoints.transport {
                Some(Transport::Plaintext) => true,
                Some(Transport::Tls { .. }) | None => false,
            };
            read::form(!plain || (!tls && loopback(host)), path.member(name), problems);
            read::form(plain || tls, path.member(name), problems);
        }
    }
    Some(endpoints)
}
impl Endpoints {
    #[must_use]
    pub fn values(&self) -> [(&'static [u8], &[u8]); 4] {
        [
            (b"issuer", &self.issuer),
            (b"authorization", &self.authorization),
            (b"token", &self.token),
            (b"keys", &self.keys),
        ]
    }
}
fn members(
    document: &Document,
    members: Option<List<u32>>,
    jig: &config::Engine,
    limits: Limits,
    path: &Path,
    problems: &mut Problems,
) -> List<Member> {
    let mut members_out = List::<Member>::with_capacity(limits.members);
    if let Some(ats) = members {
        read::form(!ats.is_empty(), path.member(b"members"), problems);
        for (index, at) in ats.iter().enumerate() {
            let path = read::index(&path.member(b"members"), index);
            let Some(mut object) = read::object(document, *at, &path, problems) else { continue };
            let address = read::text(&mut object, document, b"address", &path, problems);
            let login = read::text(&mut object, document, b"login", &path, problems);
            read::finish(object, document, &path, problems);
            if let (Some(address), Some(login)) = (address, login) {
                read::form(self::address(&address), path.member(b"address"), problems);
                read::form(self::login(&login), path.member(b"login"), problems);
                let identity_bytes = u64::try_from(address.len())
                    .expect("bounded address")
                    .saturating_add(u64::try_from(login.len()).expect("bounded login").saturating_mul(2));
                for (limit, bound) in [
                    (b"engine.limits.clients.identity_bytes".as_slice(), jig.limits.clients.identity_bytes),
                    (b"engine.limits.people.identity_bytes", jig.limits.core.people.identity_bytes),
                ] {
                    if identity_bytes > u64::from(bound) {
                        problems
                            .push(path.clone(), What::Within { bytes: identity_bytes, limit, bound: u64::from(bound) });
                    }
                }

                for member in &members_out {
                    for (name, duplicate) in [
                        (b"address".as_slice(), member.address.eq_ignore_ascii_case(&address)),
                        (b"login", member.login.eq_ignore_ascii_case(&login)),
                    ] {
                        if duplicate {
                            problems.push(path.member(name), What::Jig(config::What::Unique));
                        }
                    }
                }
                members_out.push(Member { address, login }).expect("bounded members");
            }
        }
    }
    members_out
}
pub(super) fn parse(
    document: &Document,
    at: Option<u32>,
    jig: &config::Engine,
    limits: Limits,
    directories: &config::Directories,
    problems: &mut Problems,
) -> Option<SignIn> {
    let at = at?;
    if document.token(at)?.kind == Kind::Null {
        return None;
    }
    let path = path();
    let mut object = read::object(document, at, &path, problems)?;
    let google = read::required(&mut object, document, b"google", &path, problems);
    let members = read::list(&mut object, document, b"members", limits.members, &path, problems);
    let forges = forges::parse(document, &mut object, &path, problems);
    read::finish(object, document, &path, problems);
    let members_out = self::members(document, members, jig, limits, &path, problems);
    let path = path.member(b"google");
    let mut object = read::object(document, google?, &path, problems)?;
    let provider = read::number(&mut object, document, b"provider", &path, problems);
    let client = read::text(&mut object, document, b"client", &path, problems);
    let secret = read::text(&mut object, document, b"secret", &path, problems);
    let at = read::required(&mut object, document, b"endpoints", &path, problems);
    let endpoints = endpoints(document, at?, &path.member(b"endpoints"), limits, directories, problems);
    if let Some(endpoints) = &endpoints {
        for (name, value) in endpoints.values() {
            if value.len() > usize::try_from(limits.url_bytes).expect("URL bound") {
                problems.push(path.member(b"endpoints").member(name), What::Jig(config::What::Bound));
            }
        }
    }

    read::finish(object, document, &path, problems);
    let provider = provider?;
    let Ok(provider) = u16::try_from(provider) else {
        problems.push(path.member(b"provider"), What::Jig(config::What::Bound));
        return None;
    };
    let mut application = false;
    for configured in &jig.providers {
        if configured.number == provider {
            match configured.kind {
                config::ProviderKind::Application => application = true,
                config::ProviderKind::Link => {
                    problems.push(Path::root().member(b"engine").member(b"clients"), What::Jig(config::What::Refers));
                }
                config::ProviderKind::Deployment => {}
            }
        }
    }
    if !application {
        problems.push(path.member(b"provider"), What::Jig(config::What::Refers));
    }
    let client = client?;
    read::form(client.len() <= 256 && visible(&client), path.member(b"client"), problems);
    let mut owners = 0_u32;
    for (index, owner) in jig.owners.iter().enumerate() {
        if owner.identity.provider == provider {
            owners = owners.saturating_add(1);
            let mut found = false;
            for member in &members_out {
                found |= member.address == owner.identity.subject;
            }
            if !found {
                problems.push(
                    read::index(&Path::root().member(b"engine").member(b"owners"), index),
                    What::Jig(config::What::Refers),
                );
            }
        }
    }
    if owners == 0 {
        problems.push(Path::root().member(b"engine").member(b"owners"), What::Jig(config::What::Refers));
    }
    let secret = super::joined(directories, config::Directory::Data, &secret?, &path.member(b"secret"), problems)?;
    Some(SignIn { provider, client, secret, endpoints: endpoints?, members: members_out.into_boxed(), forges })
}
pub(super) fn files(sign_in: &SignIn, limits: Limits, files: &mut List<config::File>) {
    let path = path().member(b"google");
    files
        .push(config::File {
            at: path.member(b"secret"),
            path: sign_in.secret.clone(),
            max: limits.secret_bytes,
            private: true,
        })
        .expect("sign-in secret");
    if let Some(Transport::Tls { trust: Some(trust), .. }) = &sign_in.endpoints.transport {
        files
            .push(config::File {
                at: path.member(b"endpoints").member(b"transport").member(b"tls").member(b"trust"),
                path: trust.clone(),
                max: 1_048_576,
                private: false,
            })
            .expect("issuer trust");
    }
}
#[expect(clippy::manual_map, reason = "the programming model keeps branches explicit without closures")]
pub(super) fn resolve(
    sign_in: &SignIn,
    limits: Limits,
    files: &settings::Files,
    problems: &mut Problems,
) -> Option<settings::SignIn> {
    let path = path().member(b"google");
    let secret =
        config::secret(super::contents(files, &sign_in.secret, limits.secret_bytes, path.member(b"secret"), problems)?);
    read::form(visible(secret.expose()), path.member(b"secret"), problems);
    let transport = match &sign_in.endpoints.transport {
        Some(transport) => Some(resolved_transport(transport, files, &path.member(b"endpoints"), problems)),
        None => None,
    };
    Some(settings::SignIn {
        provider: sign_in.provider,
        client: sign_in.client.clone(),
        secret,
        issuer: sign_in.endpoints.issuer.clone(),
        authorization: sign_in.endpoints.authorization.clone(),
        token: sign_in.endpoints.token.clone(),
        keys: sign_in.endpoints.keys.clone(),
        transport,
        members: sign_in.members.clone(),
        forges: sign_in.forges.clone(),
    })
}
