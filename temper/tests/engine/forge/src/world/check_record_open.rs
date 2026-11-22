//! Evidence exposed to the scripted repair agent (domain/forge/change/repairs.md).
use super::{World, raw, top};
use temper_engine_domain_forge_change as change;

impl World {
    /// Render the instruction the root gives its repair, using the connector's snapshot.
    #[must_use]
    pub fn repair_words(&self, task: u64, why: change::Repair) -> Box<[u8]> {
        let failure = self.top.landing_failure(task, why);
        top::change_words::repair_words(self.change(task), why, failure.as_ref(), &[], 4096)
            .expect("repair instruction fits")
    }

    /// Inspect the fake's answer, including a refusal, independently of the connector.
    pub fn try_inspect(&self, read: &raw::Read) -> Result<raw::Answer, raw::Error> {
        self.forge.inspect(&self.forge_env.limits, b"org/repo", read)
    }
}
