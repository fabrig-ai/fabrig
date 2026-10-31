//! Setup limits (temper/docs/design/draft/playground-reset.md, 3).
use crate::{
    given, limits,
    problem::{Problems, What},
    setup::Setup,
};
use alloc::boxed::Box;
use jig_protocol_config::Path;
use skein_forge_setup::limits::Limits;
use skein_lib::Writer;
/// Setup entries and limits supplied to the service.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Checked {
    pub fetch_bytes: u64,
    pub setup: Setup,
    pub limits: Limits,
}
pub(crate) fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut len = 0_usize;
    for p in parts {
        len = len.checked_add(p.len()).expect("bounded text");
    }
    let mut w = Writer::new(len);
    for p in parts {
        w.put(p).expect("measured text");
    }
    w.finish()
}
/// Applies the machine's setup limits (temper/docs/design/draft/playground-reset.md, 3).
pub fn limits(overrides: &[(Box<[u8]>, u64)]) -> Result<(Limits, u64), Problems> {
    let mut ps = Problems::empty();
    let mut limits = limits::small();
    let mut fetch_bytes = limits::FETCH_BYTES;
    let lp = Path::root().member(b"application").member(b"limits").member(b"overrides");
    for (name, value) in overrides {
        let result = if name.as_ref() == b"setup.fetch_bytes" {
            match limits::fetch_bytes(*value) {
                Ok(value) => {
                    fetch_bytes = value;
                    Ok(())
                }
                Err(why) => Err(why),
            }
        } else {
            limits::apply(&mut limits, name, *value)
        };
        if let Err(w) = result {
            ps.push(
                lp.member(name),
                What::Reason(match w {
                    limits::Refused::Unknown => b"no limit of temper's has this name",
                    limits::Refused::Range => b"outside its type",
                }),
            );
        }
    }
    if let Err(why) = skein_forge_setup::limits::check(&limits) {
        ps.extend(given::refusal(0, why));
    }
    if ps.is_empty() { Ok((limits, fetch_bytes)) } else { Err(ps) }
}
