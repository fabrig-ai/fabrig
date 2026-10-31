/// Bounds for infrastructure's live records and system values.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Identifiers shared with the deployment's specs, contracts and authority.
    pub vocabulary: Vocabulary,
    /// Services in the working set.
    pub services: u32,
    /// Environments in the working set.
    pub environments: u32,
    /// Pools in the working set.
    pub pools: u32,
    /// Live tasks with named resources.
    pub tasks: u32,
    /// Live procedure states.
    pub procedures: u32,
    /// Staged effects and, separately, prepared application values.
    pub staged: u32,
    /// Durable proposed effects and, separately, retained application outcomes.
    pub proposals: u32,
    /// Unsettled outbox entries.
    pub effects: u32,
    /// Creations, including their outstanding effects and unanswered closes.
    pub made: u32,
    /// Maximum resources one task names.
    pub resources_per_task: u32,
    /// Maximum bytes in one name or version.
    pub name_bytes: u32,
    /// Maximum attempts before holding uncertainty.
    pub max_attempts: u32,
    /// Seconds before an uncertain attempt may be retried.
    pub retry_after_seconds: u64,
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.name_bytes != 0
        && limits.max_attempts != 0
        && limits.retry_after_seconds != 0
        && closing_capacity(limits).is_some()
        && refresh_capacity(limits).is_some()
        && crate::drift_accounts::capacity(limits).is_some()
}

fn closing_capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.checked_add(limits.procedures)?.checked_add(limits.effects)
}

fn refresh_capacity(limits: &Limits) -> Option<u32> {
    limits
        .tasks
        .checked_mul(limits.resources_per_task)?
        .checked_add(limits.procedures)?
        .checked_add(limits.environments)?
        .checked_add(limits.made)
}

/// Configured connector vocabulary (jig/docs/design/examples.md, 3–6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Vocabulary {
    pub environment_parameter: u32,
    pub name_parameter: u32,
    pub environment_until_parameter: u32,
    pub environment_price_parameter: u32,
    pub environment_deadline_parameter: u32,
    pub creation_task_parameter: u32,
    pub creation_purpose_parameter: u32,
    pub operation_parameter: u32,
    pub replicas_parameter: u32,
    pub service_deadline_parameter: u32,
    pub scale_procedure: u32,
    pub remediate_procedure: u32,
    pub service_hold: u16,
    pub environment_hold: u16,
    pub pool_slot_hold: u16,
    pub restart_effect: u16,
    pub scale_effect: u16,
    pub rollback_effect: u16,
    pub create_environment_effect: u16,
    pub tear_down_effect: u16,
}

impl Vocabulary {
    /// Identifiers used by the reference deployment.
    #[must_use]
    pub const fn reference() -> Self {
        Self {
            environment_parameter: 1,
            name_parameter: 2,
            environment_until_parameter: 3,
            environment_price_parameter: 4,
            environment_deadline_parameter: 5,
            creation_task_parameter: 3,
            creation_purpose_parameter: 4,
            operation_parameter: 3,
            replicas_parameter: 3,
            service_deadline_parameter: 4,
            scale_procedure: 1,
            remediate_procedure: 2,
            service_hold: 1,
            environment_hold: 2,
            pool_slot_hold: 3,
            restart_effect: 1,
            scale_effect: 2,
            rollback_effect: 3,
            create_environment_effect: 4,
            tear_down_effect: 5,
        }
    }
}
