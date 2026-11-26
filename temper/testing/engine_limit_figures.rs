//! The small engine figures used by typed worlds.
#[must_use]
pub fn over(base: &jig_protocol_config::Limits) -> jig_protocol_config::Limits {
    over_keeping_job_bounds(base, false)
}

/// Optionally keep the profile's task spec and result bounds in worlds that run jobs.
#[must_use]
pub fn over_keeping_job_bounds(
    base: &jig_protocol_config::Limits,
    keep_job_bounds: bool,
) -> jig_protocol_config::Limits {
    let mut rows = skein_lib::List::with_capacity(21);
    for (name, value) in [
        (b"authority.executors".as_slice(), 22),
        (b"authority.grants".as_slice(), 16),
        (b"core.call_answer_bytes".as_slice(), 2048),
        (b"core.policy_bytes".as_slice(), 8192),
        (b"core.run_bytes".as_slice(), 32768),
        (b"core.templates".as_slice(), 7),
        (b"fleet.slots".as_slice(), 4),
        (b"hub.turns".as_slice(), 1),
        (b"notes.recalled".as_slice(), 1),
        (b"notes.scopes".as_slice(), 32),
        (b"people.amendment_bytes".as_slice(), 64),
        (b"people.words".as_slice(), 512),
        (b"tasks.authority_bytes".as_slice(), 64),
        (b"tasks.charters".as_slice(), 13),
        (b"tasks.executor_kinds".as_slice(), 22),
        (b"tasks.hold_bytes".as_slice(), 64),
        (b"tasks.inbox_bytes".as_slice(), 4096),
        (b"tasks.message_bytes".as_slice(), 512),
        (b"tasks.result_bytes".as_slice(), 32),
        (b"tasks.run_inbox".as_slice(), 2),
        (b"tasks.spec_bytes".as_slice(), 128),
    ] {
        if keep_job_bounds && (name == b"tasks.spec_bytes" || name == b"tasks.result_bytes") {
            continue;
        }
        rows.push(jig_domain_core::settings::Override {
            name: name.into(),
            value,
            reason: b"world's figure".as_slice().into(),
        })
        .expect("twenty-one figures");
    }
    let rows = rows.into_boxed();
    jig_protocol_config::overrides::with_overrides(base, &rows).expect("checked world figures")
}
