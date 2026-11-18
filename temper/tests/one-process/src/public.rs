//! Visitors through the service and a caching proxy (temper/docs/design/protocol/sign-in.md, 2.5).
mod abandoned;
mod checks;
mod deployment;
mod http;
mod load;
mod observation;
mod owners;
mod previous;
mod programs;
mod proxy;
mod requests;
mod revocation;
mod service;
mod service_checks;
mod visitor;
mod wire;

use std::cell::Cell;
thread_local! { static ACTIVE: Cell<bool> = const { Cell::new(false) }; }
// The forge's largest public record is PublicLandings; two reservations fit, three do not.
pub(super) const PUBLIC_ANSWER_BYTES: u32 = 67_405;
pub(super) const PUBLIC_BYTES: u32 = PUBLIC_ANSWER_BYTES * 2 + 1;
pub(crate) fn configure(value: &mut temper_playground::documents::Configuration) {
    if !ACTIVE.with(Cell::get) {
        return;
    }
    if value["world_settings"]["policies"][0]["policy"]["ceiling"]["grants"]
        .as_array()
        .is_some_and(|grants| !grants.is_empty())
    {
        for project in value["world_settings"]["policies"].as_array_mut().expect("policies") {
            let job = serde_json::json!({"kind":"job","number":1});
            let kinds = project["policy"]["ceiling"]["delegation"]["kinds"].as_array_mut().expect("kinds");
            if !kinds.contains(&job) {
                kinds.push(job);
            }
            for pointer in ["/policy/ceiling/grants", "/policy/roles/0/authority/grants"] {
                let grants = project.pointer_mut(pointer).expect("world grants");
                let prototype = grants[0].clone();
                *grants = temper_engine_domain_forge::CHANGE_KINDS
                    .into_iter()
                    .chain([15])
                    .map(|kind| {
                        let mut grant = prototype.clone();
                        grant["kind"] = kind.into();
                        grant
                    })
                    .collect();
            }
        }
        for (name, count) in [
            ("tasks.authority_grants", 8),
            ("tasks.executor_kinds", 3),
            ("authority.executors", 3),
            ("authority.grants", 8),
            ("notes.scopes", 12),
            ("people.amendment_bytes", 2048),
            ("tasks.authority_bytes", 256),
        ] {
            value.jig_over(name, count);
        }
    }
    value.temper_over("forge.judge_projects", 3);
    // Workflow discovery can share the fixture's four foreground logical-call slots.
    value.temper_over("forge.client.pending", 5);
    value.temper_over("forge.client.op_bytes", 4096);
    value.temper_over("checkout.message_bytes", 4098);
    // Merge setup and workflow discovery fit one forge rate window before visitor reads.
    value.temper_over("forge.client.rate", 1000);
    value.jig_over("core.call_answer_bytes", 4098);
    value.jig_over("hub.outcome_bytes", 4098);
    for worker in value["workers"].as_array_mut().expect("workers") {
        worker["limits"]["overrides"]["core.call_answer_bytes"] = 4098.into();
        worker["limits"]["overrides"]["hub.outcome_bytes"] = 4098.into();
        worker["limits"]["overrides"]["hub.event_bytes"] = 4096.into();
        worker["limits"]["overrides"]["tasks.spec_bytes"] = 16_384.into();
        worker["limits"]["overrides"]["tasks.result_bytes"] = 4096.into();
        worker["limits"]["overrides"]["tasks.message_bytes"] = 4096.into();
        worker["limits"]["overrides"]["tasks.inbox_bytes"] = 32_768.into();
    }
    value["application"]["clients"]["public"] = "https://one.example".into();
    value["world_settings"]["settings"] =
        serde_json::json!({"items":[{"setting":{"kind":"project-landing","project":1},"set":{"by":"turn"}}]});
    for (name, count) in [
        ("clients.public.reads", 4),
        ("clients_http.public_calls", 4),
        ("clients.public.rate", 8),
        ("clients.public.rate_burst", 8),
        ("clients.public.bytes", PUBLIC_BYTES),
        ("clients.public.bytes_burst", PUBLIC_BYTES),
        ("clients.public.answer_bytes", PUBLIC_ANSWER_BYTES),
        ("clients.public.max_age", 2),
        ("core.public.goals", 4),
        ("core.public.ended", 4),
        ("core.public.lines", 8),
        ("core.public.depth", 4),
        ("people.projects", 4),
        ("authority.projects", 3),
        ("tasks.spec_bytes", 16_384),
        ("tasks.batch_bytes", 32_768),
        ("tasks.result_bytes", 4096),
        ("tasks.message_bytes", 4096),
        ("tasks.inbox_bytes", 32_768),
    ] {
        value.jig_over(name, u64::from(count));
    }
    temper_playground::documents::turn_bounds::apply(value, service::agent_settings().limits.bodies.turn_body);
    previous::configure(value);
}

/// Replay a visitor's negative walk with failures naming its seed.
pub fn walk(seed: u64, name: &str) {
    eprintln!("public walk {name}, seed {seed}");
    if name.starts_with("the_proxy_")
        || matches!(
            name,
            "a_stopped_upstream_serves_copies_until_their_freshness_ends"
                | "two_deployments_behind_one_proxy_keep_their_own_copies"
                | "a_refused_host_is_never_answered_from_the_cache"
        )
    {
        checks::run(seed, name);
    } else if name == "a_store_written_before_public_reading_reads_every_project_private" {
        previous::run(seed);
    } else {
        service::run(seed, name);
    }
    ACTIVE.with(|active| active.set(false));
}
