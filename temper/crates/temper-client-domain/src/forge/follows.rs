//! Follow lines shared by ctl and pages (temper/docs/design/domain/forge/follows.md, 7).
use alloc::boxed::Box;
use skein_lib::{Decimal, bytes::concat};
use temper_client_protocol_wire::v1 as w;

#[must_use]
pub fn watch(project: u32, limits: &super::Limits) -> Option<jig_client::Watchable> {
    let limits = super::wire_limits(limits)?;
    let subject = w::FollowsSubject::new(&limits, w::FollowsSubjectParts { project }).ok()?;
    let watching = w::Watching::new(&limits, w::WatchingParts { watch: w::Watch::Follows(subject) }).ok()?;
    Some(jig_client::Watchable::Family {
        scope: jig_client::contract::Scope::Project(project),
        window: jig_client::contract::Window { rows: 0, after: None },
        record: jig_client::Family {
            family: temper_client_protocol_wire::FORGE,
            version: temper_client_protocol_wire::CLIENT_VERSION,
            bytes: temper_client_protocol_wire::watching(&watching),
        },
    })
}
#[must_use]
pub fn after12(bytes: &[u8; 32]) -> [u8; 12] {
    let mut digits = [0; 12];
    for (i, digit) in digits.iter_mut().enumerate() {
        let byte = *bytes.get(i / 2).expect("twelve digits in 32 bytes");
        let nibble = if i % 2 == 0 { byte >> 4_u8 } else { byte & 15 };
        *digit = *b"0123456789abcdef".get(usize::from(nibble)).expect("hex nibble");
    }
    digits
}
#[must_use]
pub fn parity(value: Option<&w::FollowParity>) -> Box<[u8]> {
    let Some(value) = value else { return Box::from(b"no check yet".as_slice()) };
    let verdict = match value.verdict() {
        w::FollowVerdict::Passed => Box::from(b"check passes".as_slice()),
        w::FollowVerdict::Failing(failing) => {
            concat(&[Decimal::of(u64::from(failing.count())).as_bytes(), b" failing"])
        }
        w::FollowVerdict::Many => Box::from(b"many failing".as_slice()),
    };
    concat(&[&verdict, b" at ", &after12(value.source())])
}
/// Shared binding lag words (domain/forge/follows.md, 7).
#[must_use]
pub fn lag(count: u32) -> Box<[u8]> {
    if count == 0 {
        Box::from(b"caught up".as_slice())
    } else {
        concat(&[Decimal::of(u64::from(count)).as_bytes(), b" behind"])
    }
}
/// One binding line, without a trailing newline.
#[must_use]
pub fn line(row: &w::FollowView) -> Box<[u8]> {
    let lag = lag(row.lag());
    concat(&[
        b"follow ",
        Decimal::of(u64::from(row.number())).as_bytes(),
        b": ",
        row.source(),
        b" ",
        row.source_branch(),
        b" \xe2\x86\x92 ",
        row.target(),
        b" ",
        row.target_branch(),
        b" \xc2\xb7 ",
        &lag,
        b" \xc2\xb7 ",
        &parity(row.parity().as_ref()),
    ])
}
#[must_use]
pub fn state(entry: &w::FollowEntry) -> Box<[u8]> {
    match entry.state() {
        w::FollowState::NotMade => Box::from(b"not made yet".as_slice()),
        w::FollowState::WaitsWord => Box::from(b"waits for its owner's word".as_slice()),
        w::FollowState::HeldByOwner(words) => concat(&[b"held by its owner: ", words.words()]),
        w::FollowState::WaitsFor(wait) => concat(&[b"waits for change ", Decimal::of(wait.change()).as_bytes()]),
        w::FollowState::Producing => match entry.change() {
            Some(change) => concat(&[b"producing as change ", Decimal::of(*change).as_bytes()]),
            None => Box::from(b"not made yet".as_slice()),
        },
        w::FollowState::Held(words) => concat(&[b"held: ", words.words()]),
        w::FollowState::Ended(words) => concat(&[b"ended without landing: ", words.words()]),
    }
}
