//! Private preparation record with a versioned, strict JSON envelope.
use alloc::boxed::Box;
use jig_protocol_config::{self as config, Path, read};
use skein_json::{
    Document, Kind,
    writer::{Encoder, Limits},
};
use skein_lib::Secret;
/// Forgejo's own secrets, drawn once at the first preparation.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Secrets {
    pub secret_key: Secret,     // Forgejo's secret key.
    pub internal_token: Secret, // Internal authentication token.
    pub jwt_secret: Secret,     // OAuth signing secret.
}
/// Owned's prepared versions, completion marker and secrets.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct State {
    pub forgejo: Box<[u8]>,        // Prepared Forgejo pin.
    pub runner: Option<Box<[u8]>>, // Prepared runner pin, if configured.
    pub prepared: bool,            // The first preparation ended.
    pub secrets: Secrets,          // Reused at later preparations.
}
/// Why a preparation record cannot be read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Json,
    Version(u64),
    Member,
}
fn emit(e: &mut Encoder, state: &State) {
    e.object_start();
    e.key(b"version");
    e.number(b"1");
    e.key(b"forgejo");
    e.string(&state.forgejo);
    e.key(b"runner");
    match &state.runner {
        Some(v) => e.string(v),
        None => e.null(),
    }
    e.key(b"prepared");
    e.boolean(state.prepared);
    for (key, value) in [
        (b"secret_key".as_slice(), &state.secrets.secret_key),
        (b"internal_token", &state.secrets.internal_token),
        (b"jwt_secret", &state.secrets.jwt_secret),
    ] {
        e.key(key);
        e.string(value.expose());
    }
    e.object_end();
}
/// Encodes the whole private record.
#[must_use]
pub fn encode(state: &State) -> Secret {
    let limits = Limits { depth: 1, length: 16_384 };
    let mut measure = Encoder::measure(&limits);
    emit(&mut measure, state);
    let mut write = Encoder::write(measure.measured().expect("bounded state"), &limits);
    emit(&mut write, state);
    Secret::new(write.finish())
}
fn text(
    d: &Document,
    o: &mut read::Object,
    key: &[u8],
    p: &Path,
    ps: &mut config::Problems,
) -> Result<Box<[u8]>, Refused> {
    let at = o.required(d, key, p, ps).ok_or(Refused::Member)?;
    read::text(d, at, 4096).or(Err(Refused::Member))
}
/// Refuses malformed JSON, unknown or absent members and other versions.
pub fn decode(bytes: &[u8]) -> Result<State, Refused> {
    let d = config::document(bytes, &config::DocumentLimits { bytes: 16_384, tokens: 32, depth: 2 })
        .or(Err(Refused::Json))?;
    let mut o = read::Object::new(&d, 0).or(Err(Refused::Member))?;
    let p = Path::root();
    let mut ps = config::Problems::empty();
    let at = o.required(&d, b"version", &p, &mut ps).ok_or(Refused::Member)?;
    let version = read::integer(&d, at, u64::MAX).or(Err(Refused::Member))?;
    if version != 1 {
        return Err(Refused::Version(version));
    }
    let forgejo = text(&d, &mut o, b"forgejo", &p, &mut ps)?;
    let at = o.required(&d, b"runner", &p, &mut ps).ok_or(Refused::Member)?;
    let runner = if d.token(at).ok_or(Refused::Member)?.kind == Kind::Null {
        None
    } else {
        Some(read::text(&d, at, 4096).or(Err(Refused::Member))?)
    };
    let at = o.required(&d, b"prepared", &p, &mut ps).ok_or(Refused::Member)?;
    let prepared = read::boolean(&d, at).or(Err(Refused::Member))?;
    let secret_key = Secret::new(text(&d, &mut o, b"secret_key", &p, &mut ps)?);
    let internal_token = Secret::new(text(&d, &mut o, b"internal_token", &p, &mut ps)?);
    let jwt_secret = Secret::new(text(&d, &mut o, b"jwt_secret", &p, &mut ps)?);
    o.finish(&d, &p, &mut ps);
    if !ps.is_empty() {
        return Err(Refused::Member);
    }
    Ok(State { forgejo, runner, prepared, secrets: Secrets { secret_key, internal_token, jwt_secret } })
}
