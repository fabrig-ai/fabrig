//! Independent printed and local links, fresh bearer tokens, and durable digest admission (protocol/client/sign-in.md).
use crate::{Kept, SecretOwner, routes::Route};
use alloc::boxed::Box;
use jig_client_protocol_wire::v21 as w;
use skein_lib::{CryptoRng, Map, Secret, Time, Token, Writer, length::len_u64};
/// How many past links a component keeps the digests of (protocol/client/sign-in.md).
pub(crate) const PAST_LINKS: u32 = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Past {
    Used,
    Expired,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Which {
    Printed,
    Local,
}
#[derive(Debug)]
pub(crate) struct Slot {
    pub live: Option<Link>,
    pub spending: Option<(Token, Link)>,
}
impl Slot {
    pub const fn new() -> Self {
        Self { live: None, spending: None }
    }
}
#[derive(Debug)]
pub(crate) struct Link {
    pub secret: Secret,
    pub until: Time,
}
#[derive(Debug)]
pub(crate) enum Waiting {
    ApprovalWhose { digest: [u8; 32] },
    ApprovalCore { digest: [u8; 32] },
    Authorize { sign_in: u64, authorization: skein_oauth_issuer::Authorization },
    Core { held: Option<u64> },
    LinkGet { sign_in: u64, digest: Secret },
    SignOut { sign_in: u64, committed: bool },
    Watch { sign_in: u64, watcher: Token, opening: bool },
    Keep { sign_in: u64, owner: SecretOwner, answer: Answer },
    Get { owner: crate::component::services::GetOwner, digest: Secret, body: Box<[u8]>, route: Route },
    Made { sign_in: u64, party: u64 },
    Moving { sign_in: u64, account: u32, number: u32 },
    Read { sign_in: u64, what: Box<jig_domain_core_vocabulary::reads::Read> },
    ApplicationRead { sign_in: u64, family: u16, version: u32 },
    Ask { sign_in: u64, family: Option<u16>, version: u32 },
    // A request whose carried value the store is keeping: the core sees
    // `request` only once the value is durable (protocol/store/secrets.md).
    Carry { sign_in: u64, family: Option<u16>, version: u32, request: Box<CarriedRequest> },
    Checked { sign_in: u64, body: Box<[u8]>, route: Route },
}
/// An admitted value's request, without the value or a speculative carried number.
#[derive(Debug)]
pub(crate) enum CarriedRequest {
    Application {
        key: [u8; 16],
        family: u16,
        version: u32,
        scope: jig_domain_people::ApplicationScope,
        record: Box<[u8]>,
        digest: [u8; 32],
        carried_digest: [u8; 32],
    },
    AccountKey {
        key: [u8; 16],
        ask: Box<jig_domain_people::Ask>,
        digest: [u8; 32],
    },
}
#[derive(Debug)]
pub(crate) enum Answer {
    Approval(w::SignedIn),
    SignedIn(w::SignedIn),
    Minted { credential: Secret, expires: skein_lib::Wall },
    Service(w::KeyedAnswer),
}
#[derive(Debug)]
pub(crate) struct Pending {
    pub exchanges: Map<Token, Waiting>,
    pub mints: Map<Token, Token>,
    /// The values this component's decodes kept, by exchange: none while
    /// the store keeps it, then its number until the core answers or
    /// leaves it unused (protocol/store/secrets.md).
    pub carried: Map<Token, Option<u64>>,
}
pub(crate) fn draw(rng: &mut CryptoRng) -> Secret {
    let mut bytes = [0; 32];
    rng.fill(&mut bytes);
    Secret::new(Box::new(bytes))
}
pub(crate) fn token(rng: &mut CryptoRng, number: u64) -> Secret {
    let mut out = Writer::new(40);
    out.put(&number.to_be_bytes()).expect("eight bytes");
    out.put(draw(rng).expose()).expect("32 bytes");
    Secret::new(out.finish())
}
pub(crate) fn digest(bytes: &[u8]) -> Secret {
    Secret::new(Box::new(skein_lib::sha256::digest(bytes)))
}
/// A carrying request's digest: its record and its value, each after its
/// length, so the same key sent with another value is a conflict.
pub(crate) fn carrying(record: &[u8], value: &Secret) -> [u8; 32] {
    let mut hash = skein_lib::sha256::Sha256::new();
    for part in [record, value.expose()] {
        hash.update(&len_u64(part.len()).to_be_bytes());
        hash.update(part);
    }
    hash.finish()
}
pub(crate) fn credential(value: Option<&Secret>) -> Option<(u64, Secret)> {
    let bytes = value?.expose();
    if bytes.len() != 40 {
        return None;
    }
    let number = u64::from_be_bytes(bytes.get(..8)?.try_into().ok()?);
    Some((number, digest(bytes)))
}
pub(crate) fn accepted(value: Option<&Kept>, digest: &Secret, wall: skein_lib::Wall) -> bool {
    match value {
        Some(value) => value.expires > wall && constant_time_eq(value.digest.expose(), digest.expose()),
        None => false,
    }
}
pub(crate) fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut different = 0_u8;
    for (a, b) in left.iter().zip(right) {
        different |= a ^ b;
    }
    different == 0
}

impl Waiting {
    pub(crate) fn approval_take(&self) -> bool {
        match self {
            Self::ApprovalWhose { .. } | Self::ApprovalCore { .. } | Self::Keep { answer: Answer::Approval(_), .. } => {
                true
            }
            Self::Authorize { .. }
            | Self::Core { .. }
            | Self::LinkGet { .. }
            | Self::SignOut { .. }
            | Self::Watch { .. }
            | Self::Keep { .. }
            | Self::Get { .. }
            | Self::Made { .. }
            | Self::Moving { .. }
            | Self::Read { .. }
            | Self::ApplicationRead { .. }
            | Self::Ask { .. }
            | Self::Carry { .. }
            | Self::Checked { .. } => false,
        }
    }
}
