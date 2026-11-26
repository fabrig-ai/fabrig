//! Refusals a worker's Keep plans can make (jig/docs/design/protocol/channel/layouts.md, 5.4).
use super::{Cause, Nested, Peer, Value, table, words};
use crate::{Carried, KINDS};
use alloc::boxed::Box;
use skein_codec::layout::{Misconverted, Mode, Plan, Refused};
use skein_lib::List;

/// The value type whose Keep plan can refuse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Site {
    Frame,
    Charter,
    Turn,
    Carried(Carried),
}

impl Site {
    fn value(self) -> Value {
        match self {
            Self::Frame => Value::Frame,
            Self::Charter => Value::Part(Nested::Charter),
            Self::Turn => Value::Part(Nested::Turn),
            Self::Carried(carried) => Value::Part(Nested::Carried(carried)),
        }
    }
}

/// A possible refusal, named by the engine's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub wire: Box<str>,
    pub site: Site,
    pub why: Refused,
    pub declaration: Box<str>,
    pub part: Box<str>,
}

impl Gap {
    /// The words used when the worker's layout cannot keep a value at this site.
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        words::kept(&self.wire, &self.declaration, &self.part, Cause::from_refused(self.why))
    }
}

impl Peer {
    /// Lists possible refusals in the outgoing value types' order, then by plan index.
    #[must_use]
    pub fn gaps(&self) -> List<Gap> {
        let mut gaps = List::uncapped();
        for site in [
            Site::Frame,
            Site::Charter,
            Site::Turn,
            Site::Carried(Carried::Items),
            Site::Carried(Carried::JobSpec),
            Site::Carried(Carried::Grant),
            Site::Carried(Carried::Answer),
        ] {
            for pass in &self.plans {
                if pass.mode != Mode::Keep || pass.value != site.value() {
                    continue;
                }
                let Some(plan) = &pass.plan else { continue };
                for index in 0..=u16::MAX {
                    let Some((why, declaration, part)) = plan.refusal(index) else { break };
                    if site == Site::Frame && why == Refused::Variant && optional_variant(plan, index) {
                        continue;
                    }
                    gaps.push(Gap {
                        wire: pass.wire.clone(),
                        site,
                        why,
                        declaration: Box::from(declaration),
                        part: Box::from(part),
                    })
                    .expect("uncapped gaps");
                }
            }
        }
        gaps
    }
}

fn optional_variant(plan: &Plan, index: u16) -> bool {
    // Check the refusal for the tag itself, not any refusals in its payload.
    for rule in KINDS {
        if rule.body.optional_at_worker()
            && let Some(tag) = table::tag(table::DOWN, rule.kind)
            && plan.variant_tag(tag) == Err(Misconverted::Refused(index))
        {
            return true;
        }
    }
    false
}
