//! The App's state and bearer check (fake-forge.md, 10.1).
use alloc::boxed::Box;
use skein_forge::github::{document::Permission, jwt};
use skein_json::{
    Document,
    document::{self, Kind},
    tokenizer,
};
use skein_lib::{List, Secret, Wall, base64, bytes};

// The fake checks the dialect's RS256/JWT header independently (forge-github.md, 8.1).
pub(crate) const HEADER: &[u8] = b"eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9";

/// The App the layer holds. Its routes belong to the service.
pub struct App {
    pub id: u64,
    pub slug: Box<[u8]>,
    pub owner: u64,
    pub permissions: Box<[Permission]>,
    pub events: Box<[Box<[u8]>]>,
    pub key: &'static [u8],
    pub webhook: Secret,
}

/// The grants setup asks for (forge-github.md, 11): a real App holds them, and
/// its bot's repository permission is its installation's (4.1, 8.2).
#[must_use]
pub fn wanted() -> Box<[Permission]> {
    let mut permissions =
        List::with_capacity(u32::try_from(skein_forge::github::setup::PERMISSIONS.len()).expect("wanted count"));
    for (name, access) in skein_forge::github::setup::PERMISSIONS {
        permissions.push(Permission { name: Box::from(name), access }).expect("wanted room");
    }
    permissions.into_boxed()
}

// The key and webhook bytes must never appear in a trace.
impl core::fmt::Debug for App {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("App")
            .field("id", &self.id)
            .field("slug", &self.slug)
            .field("owner", &self.owner)
            .field("permissions", &self.permissions)
            .field("events", &self.events)
            .field("key_bytes", &self.key.len())
            .field("webhook", &self.webhook)
            .finish()
    }
}

/// Why a JSON Web Token is refused; each is a `401` at the route.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Malformed,
    Header,
    Signature,
    Issuer,
    Expired,
    TooLong,
    Early,
}

/// One verified JWT, retained as its exact bytes and parsed claims.
/// The token is bounded by [`jwt::JWT_BYTES`]; claims are checked on every use.
pub struct Verified {
    key: &'static [u8],
    entry: Option<Entry>,
}

struct Entry {
    token: Secret,
    claims: Claims,
}

impl core::fmt::Debug for Verified {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Verified").field("key_bytes", &self.key.len()).field("cached", &self.entry.is_some()).finish()
    }
}

impl Verified {
    #[must_use]
    pub const fn empty() -> Self {
        Self { key: &[], entry: None }
    }

    /// Drops the entry when its owner replaces or revokes the App's key.
    pub fn forget(&mut self) {
        self.entry = None;
        self.key = &[];
    }

    /// Checks the signature once per distinct JWT and key, and claims every time.
    pub fn check(&mut self, token: &[u8], app: &App, now: Wall) -> Result<(), Refused> {
        if self.key != app.key {
            self.forget();
        }
        if let Some(entry) = &self.entry
            && entry.token.expose() == token
        {
            return entry.claims.check(app, now);
        }
        let claims = verified_claims(token, app)?;
        self.forget();
        self.key = app.key;
        self.entry = Some(Entry { token: Secret::new(bytes::copy_of(token)), claims });
        claims.check(app, now)
    }
}

/// Checks the header, RS256 signature, then the claims on the
/// forge's clock. Issuance may be at most 60 seconds ahead, and the life
/// must be positive and at most [`jwt::LIFE`]. Expiry is exclusive.
pub fn check(token: &[u8], app: &App, now: Wall) -> Result<(), Refused> {
    verified_claims(token, app)?.check(app, now)
}

fn verified_claims(token: &[u8], app: &App) -> Result<Claims, Refused> {
    if token.len() > jwt::JWT_BYTES {
        return Err(Refused::Malformed);
    }
    let first = bytes::find(token, b".").ok_or(Refused::Malformed)?;
    let claims_at = first.checked_add(1).ok_or(Refused::Malformed)?;
    let last = bytes::find_from(token, b".", claims_at).ok_or(Refused::Malformed)?;
    let signature_at = last.checked_add(1).ok_or(Refused::Malformed)?;
    if bytes::find_from(token, b".", signature_at).is_some() {
        return Err(Refused::Malformed);
    }
    if token.get(..first) != Some(HEADER) {
        return Err(Refused::Header);
    }
    let signature = base64::decode(base64::Alphabet::Url, token.get(signature_at..).ok_or(Refused::Malformed)?, 256)
        .or(Err(Refused::Malformed))?;
    let input = token.get(..last).ok_or(Refused::Malformed)?;
    let public = skein_tls::rs256_public_pkcs1(app.key).ok_or(Refused::Signature)?;
    if !skein_tls::verify_rs256(&public.modulus, &public.exponent, input, &signature) {
        return Err(Refused::Signature);
    }
    let claims = base64::decode(
        base64::Alphabet::Url,
        token.get(claims_at..last).ok_or(Refused::Malformed)?,
        u32::try_from(jwt::JWT_BYTES).expect("JWT bound fits u32"),
    )
    .or(Err(Refused::Malformed))?;
    claims_read(&claims)
}

#[derive(Clone, Copy)]
struct Claims {
    issued: u64,
    expires: u64,
    issuer: u64,
}

impl Claims {
    fn check(self, app: &App, now: Wall) -> Result<(), Refused> {
        if self.issuer != app.id {
            return Err(Refused::Issuer);
        }
        let life = self.expires.checked_sub(self.issued).ok_or(Refused::TooLong)?;
        if life == 0 || life > jwt::LIFE.as_nanos().div_euclid(1_000_000_000) {
            return Err(Refused::TooLong);
        }
        if now.as_secs() >= self.expires {
            return Err(Refused::Expired);
        }
        if self.issued > now.as_secs().checked_add(60).ok_or(Refused::Early)? {
            return Err(Refused::Early);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Field {
    Issued,
    Expires,
    Issuer,
    Ignore,
}

fn claims_read(input: &[u8]) -> Result<Claims, Refused> {
    let document = read(input)?;
    if document.token(0).ok_or(Refused::Malformed)?.kind != Kind::ObjectStart {
        return Err(Refused::Malformed);
    }
    let mut depth = 0_u32;
    let mut field = Field::Ignore;
    let mut issued = None;
    let mut expires = None;
    let mut app_id = None;
    for at in 0..document.len() {
        let record = document.token(at).ok_or(Refused::Malformed)?;
        match record.kind {
            Kind::ObjectStart | Kind::ArrayStart => {
                if depth == 1 && field != Field::Ignore {
                    return Err(Refused::Malformed);
                }
                depth = depth.checked_add(1).ok_or(Refused::Malformed)?;
            }
            Kind::ObjectEnd | Kind::ArrayEnd => {
                depth = depth.checked_sub(1).ok_or(Refused::Malformed)?;
            }
            Kind::Key => {
                if depth == 1 {
                    field = match document.text(record).ok_or(Refused::Malformed)? {
                        b"iat" => Field::Issued,
                        b"exp" => Field::Expires,
                        b"iss" => Field::Issuer,
                        _ => Field::Ignore,
                    };
                }
            }
            Kind::Number | Kind::String => {
                if depth == 1 && field != Field::Ignore {
                    if record.kind == Kind::String && field != Field::Issuer {
                        return Err(Refused::Malformed);
                    }
                    let text = document.text(record).ok_or(Refused::Malformed)?;
                    let value = bytes::decimal(text).ok_or(Refused::Malformed)?;
                    let slot = match field {
                        Field::Issued => &mut issued,
                        Field::Expires => &mut expires,
                        Field::Issuer => &mut app_id,
                        Field::Ignore => unreachable!("known claim field"),
                    };
                    if slot.replace(value).is_some() {
                        return Err(Refused::Malformed);
                    }
                }
            }
            Kind::Long => return Err(Refused::Malformed),
            Kind::True | Kind::False | Kind::Null => {
                if depth == 1 && field != Field::Ignore {
                    return Err(Refused::Malformed);
                }
            }
        }
    }
    Ok(Claims {
        issued: issued.ok_or(Refused::Malformed)?,
        expires: expires.ok_or(Refused::Malformed)?,
        issuer: app_id.ok_or(Refused::Malformed)?,
    })
}

// Retention, nesting and the driving loop are bounded by the admitted token size.
fn read(input: &[u8]) -> Result<Document, Refused> {
    let bound = u32::try_from(jwt::JWT_BYTES).expect("JWT bound fits u32");
    let limits = tokenizer::Limits { depth: bound, string: bound, number: bound, chunk: bound, length: bound };
    skein_json::parse(input, &limits, &document::Limits { tokens: bound, text: bound }).or(Err(Refused::Malformed))
}
