//! Typed test builders that write settings kinds 4 and 5, not a document.
//!
//! temper/docs/design/testing/README.md, 5.
use temper_engine_domain_forge_vocabulary as forge;
/// A project's landing way and repository checks (protocol/configuration/engine.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectLanding {
    pub gate_retries: u8,
    pub gate_timeout: skein_lib::Duration,
    pub check_timeout: Option<skein_lib::Duration>,
    pub by: Way,
    pub batch: u16,
    pub combine_main: temper_engine_domain_forge_landing::Combine,
    pub combine_other: temper_engine_domain_forge_landing::Combine,
    pub merge_job: Option<Box<[u8]>>,
    pub close_job: Option<Box<[u8]>>,
    pub landing_attempts: u8,
    pub repositories: Box<[RepositoryChecks]>,
    pub tier_retries: u8,
    pub verification_bound_ms: u64,
    pub wait: temper_engine_domain_forge_change::Wait,
}
pub use temper_engine_domain_forge_change::Way;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RepositoryChecks {
    pub repository: skein_forge::api::Repository,
    pub gates: Box<[GateCheck]>,
    pub tiers: Box<[TierCheck]>,
}
pub use temper_engine_domain_forge_landing::Gate as GateCheck;
/// A repository's tier and the branch it follows (domain/after-landing.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TierCheck {
    pub name: Box<[u8]>,
    pub workflow: Box<[u8]>,
    pub ignored: Box<[Box<[u8]>]>,
    pub artifact: Box<[u8]>,
    pub format: temper_engine_domain_forge_tiers::Format,
    pub follows: forge::Follows,
    pub span_ms: u64,
    pub description: Box<[u8]>,
}
/// Settings values for a test world's typed landing builder (domain/landing-settings.md, 1.1).
impl ProjectLanding {
    #[must_use]
    pub fn values(self) -> (forge::settings::ProjectLanding, forge::settings::ProjectGates) {
        let landing = &self;
        let mut repositories =
            skein_lib::List::with_capacity(u32::try_from(landing.repositories.len()).expect("repositories"));
        for repository in &landing.repositories {
            let mut gates = skein_lib::List::with_capacity(u32::try_from(repository.gates.len()).expect("gates"));
            for gate in &repository.gates {
                gates
                    .push(temper_engine_domain_forge_landing::Gate {
                        name: gate.name.clone(),
                        workflow: gate.workflow.clone(),
                        ignored: gate.ignored.clone(),
                        blocking: gate.blocking,
                        base: gate.base,
                        report: gate.report.clone(),
                    })
                    .expect("counted gates");
            }
            let mut tiers = skein_lib::List::with_capacity(u32::try_from(repository.tiers.len()).expect("tiers"));
            for tier in &repository.tiers {
                tiers
                    .push(forge::Tier {
                        name: tier.name.clone(),
                        workflow: tier.workflow.clone(),
                        ignored: tier.ignored.clone(),
                        artifact: tier.artifact.clone(),
                        format: tier.format,
                        follows: tier.follows.clone(),
                        span: match tier.span_ms {
                            0 => None,
                            millis => Some(skein_lib::Duration::from_millis(millis)),
                        },
                        description: tier.description.clone(),
                    })
                    .expect("counted tiers");
            }
            repositories
                .push(forge::RepositoryGates {
                    repository: repository.repository,
                    gates: gates.into_boxed(),
                    tiers: tiers.into_boxed(),
                })
                .expect("counted repositories");
        }
        (
            forge::settings::ProjectLanding {
                by: landing.by,
                batch: landing.batch,
                combine_main: landing.combine_main,
                combine_other: landing.combine_other,
                merge_job: landing.merge_job.clone(),
                close_job: landing.close_job.clone(),
                landing_attempts: landing.landing_attempts,
                check_timeout_ms: landing.check_timeout.map(skein_lib::Duration::as_millis),
                gate_retries: landing.gate_retries,
                gate_timeout_ms: landing.gate_timeout.as_millis(),
                tier_retries: landing.tier_retries,
                verification_bound_ms: landing.verification_bound_ms,
                wait: landing.wait,
                repairs: None,
            },
            forge::settings::ProjectGates { repositories: repositories.into_boxed() },
        )
    }
}
