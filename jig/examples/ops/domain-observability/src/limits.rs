/// Bounds for observability's working set and system values.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Identifiers shared with the deployment's specs, contracts and authority.
    pub vocabulary: Vocabulary,
    /// Cached service facts.
    pub services: u32,
    /// Live subscriptions.
    pub subscriptions: u32,
    /// Standing watches.
    pub watches: u32,
    /// Staged silences.
    pub staged: u32,
    /// Outstanding silences.
    pub effects: u32,
    /// Pending verdicts.
    pub judges: u32,
    /// Pending reads.
    pub reads: u32,
    /// Maximum subscribers reached by one alert.
    pub subscribers_per_alert: u32,
    /// Maximum load samples per service.
    pub samples_per_service: u32,
    /// Maximum services in one watch.
    pub services_per_watch: u32,
    /// Maximum alert numbers in one batch.
    pub alerts_per_batch: u32,
    /// Maximum bytes in a name or rule.
    pub name_bytes: u32,
    /// Maximum bytes in a read answer.
    pub answer_bytes: u32,
    /// Maximum attempts before an uncertain effect is held.
    pub max_attempts: u32,
    /// Seconds from sending an attempt to its absolute retry deadline.
    pub retry_after_seconds: u64,
}

/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.name_bytes != 0
        && limits.answer_bytes != 0
        && limits.max_attempts != 0
        && limits.retry_after_seconds != 0
        && limits.subscribers_per_alert <= limits.subscriptions
        && closing_capacity(limits).is_some()
        && refresh_capacity(limits).is_some()
        && crate::incidents::capacity(limits).is_some()
        && crate::watch_steps::capacity(limits).is_some()
        && crate::watch_steps::message_capacity(limits).is_some()
}

fn closing_capacity(limits: &Limits) -> Option<u32> {
    limits.subscriptions.checked_add(limits.watches)?.checked_add(limits.effects)
}

fn refresh_capacity(limits: &Limits) -> Option<u32> {
    limits
        .subscriptions
        .checked_add(limits.watches.checked_mul(limits.services_per_watch)?)?
        .checked_add(crate::incidents::capacity(limits)?)
}

/// Configured connector vocabulary (jig/docs/design/examples.md, 3–6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Vocabulary {
    pub environment_parameter: u32,
    pub service_parameter: u32,
    pub silence_effect: u16,
    pub logs_read: u16,
    pub series_read: u16,
    pub fired_read: u16,
    pub noise_verdict: u32,
    pub incident_verdict: u32,
    pub watch_template_parameter: u32,
}

impl Vocabulary {
    /// Identifiers used by the reference deployment.
    #[must_use]
    pub const fn reference() -> Self {
        Self {
            environment_parameter: 1,
            service_parameter: 2,
            silence_effect: 1,
            logs_read: 2,
            series_read: 3,
            fired_read: 4,
            noise_verdict: 1,
            incident_verdict: 2,
            watch_template_parameter: 3,
        }
    }
}
