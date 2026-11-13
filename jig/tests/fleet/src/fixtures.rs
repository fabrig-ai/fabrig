//! Shared host bounds for the domain world.
use jig_domain_link::{Bounds, EndpointBound, Form, Grant, Hosting, Membership, Phase, Scheme};
#[must_use]
pub fn bounds() -> Bounds {
    use skein_lib::Duration;
    Bounds {
        program_endpoints: Box::new([]),
        endpoints: Box::new([]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_secs(3600),
        waiting: Duration::from_secs(3600),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
        reply_bytes: None,
    }
}

/// A live run and an answered run named by the memory scenario.
#[must_use]
pub fn hosting_pair(
    r1: skein_lib::Token,
    a1: skein_lib::Token,
    r2: skein_lib::Token,
    a2: skein_lib::Token,
) -> Vec<Hosting> {
    vec![
        Hosting { run: r1, attempt: a1, phase: Phase::Active },
        Hosting { run: r2, attempt: a2, phase: Phase::Answered },
    ]
}

/// The account granted to the fleet world's scripted attempts.
pub const ANALYST: Grant = Grant {
    form: Form::Credential(Scheme::Bearer),
    account: 1,
    generation: 1,
    valid: skein_lib::Duration::from_secs(60),
};

#[must_use]
pub fn needs() -> jig_domain_fleet::Needs {
    jig_domain_fleet::Needs { charters: Box::new([]), spend: 0, message_bytes: 0, start_messages: 0, unacknowledged: 0 }
}

pub const MODEL_CHARTER: u32 = 17;
pub const OTHER_CHARTER: u32 = 23;
pub const MODEL_WINDOW: u32 = 8192;
pub const MODEL_OUTPUT: u32 = 1024;

/// Distinct charters, one requiring both its main and alternative models.
#[must_use]
pub fn model_needs() -> jig_domain_fleet::Needs {
    use jig_domain_fleet::{CharterNeeds, Kinds, ModelNeed};
    let charters = [
        (MODEL_CHARTER, vec![b"main".as_slice(), b"alternative".as_slice()]),
        (OTHER_CHARTER, vec![b"other".as_slice()]),
    ];
    jig_domain_fleet::Needs {
        charters: charters
            .into_iter()
            .map(|(charter, models)| CharterNeeds {
                version: jig_domain_fleet::Version { charter, revision: 1 },
                kinds: Kinds::Both,
                models: models
                    .into_iter()
                    .map(|name| ModelNeed {
                        endpoint: model_endpoint(name),
                        name: name.into(),
                        window: MODEL_WINDOW,
                        output: MODEL_OUTPUT,
                    })
                    .collect(),
                turns: 0,
                time: skein_lib::Duration::ZERO,
                waiting: skein_lib::Duration::ZERO,
                change_reserve: None,
            })
            .collect(),
        ..needs()
    }
}

#[must_use]
pub fn model_bounds(names: &[&[u8]]) -> Bounds {
    Bounds {
        program_endpoints: Box::new([]),
        endpoints: names
            .iter()
            .map(|name| EndpointBound { endpoint: model_endpoint(name), window: MODEL_WINDOW, output: MODEL_OUTPUT })
            .collect(),
        ..bounds()
    }
}

/// Fixed configured membership for the fleet worlds, independent of live channels.
#[must_use]
pub fn domain(limits: &jig_domain_fleet::Limits, needs: jig_domain_fleet::Needs) -> jig_domain_fleet::Domain {
    let mut domain = restored_fleet(limits, needs);
    let names =
        (0..limits.configured_workers).map(|index| format!("worker-{index}").into_bytes().into_boxed_slice()).collect();
    let names =
        Membership::new(names, limits.configured_workers, limits.worker_name_bytes).expect("fixture membership");
    domain.configure_workers(names);
    domain
}

/// A runtime fleet fixture has heard its start's restored needs.
#[must_use]
pub fn restored_fleet(limits: &jig_domain_fleet::Limits, needs: jig_domain_fleet::Needs) -> jig_domain_fleet::Domain {
    let mut domain = jig_domain_fleet::Domain::new(limits, 0, needs.clone());
    let mut out = skein_lib::Queue::uncapped();
    jig_domain_fleet::step(
        &mut domain,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: *limits },
        jig_domain_fleet::Event::Needs(needs),
        &mut out,
    );
    assert!(out.is_empty(), "fixture has no early hellos");
    settle_fleet(&mut domain, limits);
    domain
}

fn settle_fleet(domain: &mut jig_domain_fleet::Domain, limits: &jig_domain_fleet::Limits) {
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: *limits };
    for _ in 0_u8..4 {
        if !domain.is_ready() {
            return;
        }
        let mut out = skein_lib::Queue::uncapped();
        jig_domain_fleet::resume(domain, &env, true, &mut out);
        assert!(out.is_empty(), "fixture fleet pass must be empty: {out:?}");
    }
    assert!(!domain.is_ready(), "fixture fleet readiness settles within four passes");
}

/// Independent served endpoints for the three model fixtures.
#[must_use]
pub fn model_endpoint(name: &[u8]) -> u32 {
    match name {
        b"main" => 1,
        b"alternative" => 2,
        b"other" => 3,
        _ => panic!("unknown model fixture"),
    }
}
