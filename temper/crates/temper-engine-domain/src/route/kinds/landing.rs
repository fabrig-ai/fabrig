//! Cross-domain landing judgements (temper/docs/design/domain/landing-settings.md, 1.1).
use crate::Domain;
mod repairs;
use alloc::boxed::Box;
pub(crate) use repairs::restored;
use skein_lib::{Decimal, List};
use temper_engine_domain_forge::settings;

pub(super) fn refusal(domain: &Domain, asked: &settings::Asked) -> Option<Box<[u8]>> {
    let settings::Edit::Set(value) = &asked.edit else { return None };
    if let settings::Value::Landing(landing) = &value.value
        && landing.by == temper_engine_domain_forge_change::Way::Turn
    {
        if domain.config.prepare_kind.is_none() {
            return Some(words(&[b"by: turn needs prepare job kind 1"]));
        }
        let (spec, result) = domain.config.prepare_capacity;
        for (field, has, needs) in
            [(b"tasks.spec_bytes".as_slice(), spec, 16_384), (b"tasks.result_bytes".as_slice(), result, 4_096)]
        {
            if has < needs {
                let has = Decimal::of(u64::from(has));
                let needs = Decimal::of(u64::from(needs));
                return Some(words(&[
                    field,
                    b": ",
                    has.as_bytes(),
                    b" bytes; by: turn prepare job kind 1 needs ",
                    needs.as_bytes(),
                ]));
            }
        }
        if let Some(why) = domain.config.turn_capacity {
            return Some(why.into());
        }
    }
    if let settings::Value::Landing(landing) = &value.value
        && let Some(settings) = &landing.repairs
    {
        if let Err(field) = repairs::resolve(domain, settings) {
            return Some(words(&[b"repairs.", field, b": not offered by the deployment or charter"]));
        }
        if settings.goal.budget <= domain.engine.core().authority.rules().minimum_run_spend {
            return Some(words(&[b"repairs.goal.budget: must be above minimum_run_spend"]));
        }
    }
    if let settings::Value::Gates(gates) = &value.value {
        for row in &gates.repositories {
            let project = match domain.forge.repository(row.repository) {
                Some(repo) => Some(repo.project),
                None => None,
            };
            if project != Some(asked.scope) {
                let forge = Decimal::of(u64::from(row.repository.forge));
                let repo = Decimal::of(u64::from(row.repository.repository));
                let project = Decimal::of(u64::from(asked.scope));
                return Some(words(&[
                    b"repositories: repository ",
                    forge.as_bytes(),
                    b"/",
                    repo.as_bytes(),
                    b" is not of project ",
                    project.as_bytes(),
                ]));
            }
        }
    }
    None
}
fn words(parts: &[&[u8]]) -> Box<[u8]> {
    let mut out = List::uncapped();
    for part in parts {
        for &byte in *part {
            out.push(byte).expect("bounded landing refusal");
        }
    }
    out.into_boxed()
}
