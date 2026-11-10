//! Adapter collection and archives retain the attempt's evidence in runner order.
//! Source selection checks the planned identity (../../../../docs/design/benchmarks/README.md, sections 6 and 10).

use crate::run::{Archive, Work};
use crate::{
    AttemptResult, CacheObservation, CheckVerdict, Fault, Identity, Input, Measure, format, input, mismatch, refuse_if,
};

pub(in crate::agent) fn source_arm(work: Work<'_>, identity: &Identity) -> Result<(crate::arm::ArmRef, String), Fault> {
    let pin = crate::read_configuration(
        work.prepared.configuration.as_ref().ok_or_else(|| input("Codex requires its pin"))?,
    )?;
    if pin.pin.sha256.as_deref() != Some(identity.binary_sha256.as_str()) {
        return Err(mismatch("Codex executable digest differs from its pin"));
    }
    let version = crate::run::raw(
        work.clock,
        work.machine,
        work.private,
        &[
            work.prepared.program.to_str().ok_or_else(|| format("program path must be UTF-8"))?.into(),
            "--version".into(),
        ],
        work.prepared.setup_deadline_seconds,
        work.prepared,
    )?;
    refuse_if(!crate::run::succeeded(&version), Input, "cannot verify the installed agent version")?;
    let version = std::str::from_utf8(&version.stdout)?.to_owned();
    if version.trim().strip_prefix("codex-cli ").unwrap_or(version.trim()) != identity.agent_version {
        return Err(mismatch("installed version differs from the planned identity"));
    }
    Ok((crate::arm::ArmRef::Installed { program: work.prepared.program.clone() }, version))
}

impl super::PreparedCodex {
    pub(crate) fn archive(&self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), Fault> {
        result.identity.user_configuration_sha256 =
            Measure::unavailable("attempt-owned Codex home; user configuration is not read");
        archive.put(result, "overrides.json", &serde_json::to_vec_pretty(&self.overrides)?)?;
        Ok(())
    }
}

impl super::Observing {
    pub(crate) fn archive_trace(archive: Archive<'_>, result: &mut AttemptResult, trace: &[u8]) -> Result<(), Fault> {
        if !trace.is_empty() {
            archive.put(result, "trace.jsonl", trace)?;
        }
        Ok(())
    }

    pub(crate) fn cache_check() -> CheckVerdict {
        CheckVerdict {
            check: "cache-reuse".into(),
            passed: Measure::unavailable("cache reuse is judged on a complete three-repetition triplet"),
            evidence: vec![],
        }
    }

    pub(crate) fn settle(&mut self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), Fault> {
        if let Some(prepared) = &mut self.prepared {
            result.delegation = crate::Delegation::unavailable("Codex does not report child shares or parent blocking");
            if let Some(borrowed) = prepared.borrowed.take() {
                let proof = borrowed.finish(archive.guard)?;
                archive.put(result, "credential-proof.json", &serde_json::to_vec_pretty(&proof)?)?;
            }
        }
        Ok(())
    }

    pub(crate) fn collect_result(&mut self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), Fault> {
        if self.prepared.is_some() {
            result.delegation = crate::Delegation::unavailable("Codex does not report child shares or parent blocking");
            if let Some(thread) = self.observer.thread().map(str::to_owned) {
                let collection = self.collect().map(|_| ());
                if let Some(rollouts) = &self.collected {
                    for file in &rollouts.files {
                        if !crate::run::valid_name(&file.thread) {
                            return Err(input("rollout thread id cannot name an archive file"));
                        }
                        archive.put(result, &format!("rollout-{}.jsonl", file.thread), &file.bytes)?;
                    }
                }
                collection?;
                let rollouts = self.collected.as_ref().expect("completed collection");
                rollouts.resolve_model(&thread, &mut result.identity.model)?;
                rollouts.timing(&thread, &mut result.timing)?;
                let ledger = rollouts.token_ledger(&thread)?;
                result.tokens = ledger.records()?;
                result.spend.priced = crate::result::priced_spend(result, ledger.responses());
                rollouts.counts(&thread, &mut result.counts)?;
            }
        }
        Ok(())
    }

    pub(crate) fn cache_reuse(&self, threshold: f64, miss_limit: u32) -> Measure<CacheObservation> {
        if self.prepared.is_none() {
            return Measure::unavailable("cache probe requires its selected adapter");
        }
        self.collected.as_ref().map_or_else(
            || Measure::unavailable("named Codex rollout collection is absent"),
            |rollouts| {
                self.observer.thread().map_or_else(
                    || Measure::unavailable("Codex root thread is absent"),
                    |root| rollouts.cache_reuse(root, threshold, miss_limit),
                )
            },
        )
    }
}
