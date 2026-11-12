//! Adapter collection and archives retain the attempt's evidence in runner order.
//! Source selection checks the planned identity (../../../../docs/design/benchmarks/README.md, sections 6 and 10).

use crate::agent::Ended;
use crate::run::{Archive, Work};
use crate::{AttemptResult, Fault, Identity, Measure, credential, input, mismatch};

pub(in crate::agent) fn home(source: &std::path::Path) -> Result<std::path::PathBuf, Fault> {
    Ok(source.parent().ok_or_else(|| credential("source login needs its harness parent"))?.join("smith"))
}

pub(in crate::agent) fn takes_arguments(arguments: &[String]) -> bool {
    arguments.len().is_multiple_of(2) && arguments.as_chunks::<2>().0.iter().all(|pair| pair[0] == "-c")
}

pub(in crate::agent) fn source_arm(work: Work<'_>, identity: &Identity) -> Result<(crate::arm::ArmRef, String), Fault> {
    let build = crate::arm::read_build(&work.prepared.program)?;
    if build.arm.commit != identity.agent_version || !matches!(identity.smith_face, Some(crate::SmithFace::Events)) {
        return Err(mismatch("Smith built arm differs from the planned full source version"));
    }
    let version = build.arm.commit.clone();
    Ok((crate::arm::ArmRef::Built(build), version))
}

impl super::Prepared {
    pub(crate) fn archive(&self, archive: Archive<'_>, result: &mut AttemptResult) -> Result<(), Fault> {
        if let Some(check) = self.check_document() {
            for (path, name) in
                [(check.clone(), "check.json"), (check.with_file_name("check-stderr.log"), "check-stderr.log")]
            {
                archive.put(result, name, &std::fs::read(path)?)?;
            }
        }
        result.identity.user_configuration_sha256 = Measure::unavailable("Smith uses frozen harness settings only");
        archive.put(result, "settings.json", &serde_json::to_vec_pretty(&self.rendered)?)?;
        archive.put(result, "face.json", &serde_json::to_vec_pretty(&self.registry)?)?;
        Ok(())
    }

    pub(crate) fn check_identity(identity: &Identity) -> Result<(), Fault> {
        if identity.smith_face != Some(crate::SmithFace::Events) {
            return Err(input("Smith identity names a different face than its source registry"));
        }
        Ok(())
    }
}

impl super::Observing {
    pub(crate) fn settle(&mut self, result: &mut AttemptResult, ended: Ended<'_>) {
        let Ended { exit, forced, task, workspace } = ended;
        if let Some(health) = self.health(exit, forced) {
            result.health = health;
        }
        result.outcome.checks.extend(self.checks_in_workspace(&task.event, &task.prompt, workspace));
    }

    pub(crate) fn collect(&mut self, result: &mut AttemptResult) -> Result<(), Fault> {
        self.events().resolve_model(&mut result.identity.model);
        self.events().timing(&mut result.timing);
        result.tokens = self.tokens()?;
        result.spend = self.spend(&result.tokens)?;
        result.spend.priced = crate::result::priced_spend(result, &self.events().response_tokens()?);
        result.resources.own_peak_rss_bytes = self.observer.own_peak_rss_bytes();
        result.counts = self.counts();
        result.delegation = self.delegation();
        Ok(())
    }
}
