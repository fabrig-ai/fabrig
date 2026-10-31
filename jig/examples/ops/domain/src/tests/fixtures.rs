//! Local reference-root fixtures (`jig/docs/design/domain/testing.md`, 2).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use jig_protocol_charter::{Tool, ToolEffect};
use skein_lib::{Duration, List};

#[must_use]
pub fn limits() -> obs::Limits {
    obs::Limits {
        services: 4,
        subscriptions: 8,
        watches: 2,
        staged: 2,
        effects: 2,
        judges: 4,
        reads: 4,
        subscribers_per_alert: 8,
        samples_per_service: 16,
        services_per_watch: 3,
        alerts_per_batch: 8,
        name_bytes: 64,
        answer_bytes: 128,
        max_attempts: 3,
        retry_after_seconds: 30,
    }
}

#[must_use]
pub fn service() -> obs::Service {
    obs::Service::new(Box::from(*b"production"), Box::from(*b"checkout"))
}

#[must_use]
pub fn infrastructure_limits() -> infra::Limits {
    infra::Limits {
        services: 4,
        environments: 4,
        pools: 2,
        tasks: 4,
        procedures: 4,
        staged: 4,
        proposals: 4,
        effects: 4,
        made: 4,
        resources_per_task: 4,
        name_bytes: 64,
        max_attempts: 3,
        retry_after_seconds: 30,
    }
}

#[must_use]
pub fn infra_service() -> infra::Service {
    infra::Service::new(Box::from(*b"production"), Box::from(*b"checkout"))
}

#[must_use]
pub fn agent_bounds() -> host::Bounds {
    host::Bounds {
        program_endpoints: Box::new([]),
        models: Box::new([host::ModelBound {
            name: b"scripted".as_slice().into(),
            window: u32::MAX,
            output: u32::MAX,
        }]),
        turns: u32::MAX,
        spend: u64::MAX,
        time: Duration::from_nanos(u64::MAX),
        waiting: Duration::from_nanos(u64::MAX),
        checks: Duration::from_nanos(1),
        message_bytes: u32::MAX,
        inbox: u32::MAX,
        unacknowledged: u32::MAX,
    }
}

pub fn tools(timeout: Duration) -> Box<[Tool]> {
    let declarations = jig_protocol_tools::declarations();
    let count =
        u32::try_from(declarations.len()).expect("declared tools").checked_add(5).expect("five connector tools");
    let mut tools = List::with_capacity(count);
    for tool in declarations {
        tools
            .push(Tool {
                name: tool.name.into(),
                description: tool.description.into(),
                schema: tool.schema.into(),
                effect: if tool.writes { ToolEffect::Write } else { ToolEffect::Read },
                timeout,
            })
            .expect("declared tools fit");
    }
    for tool in connector_tools(timeout) {
        tools.push(tool).expect("five connector tools fit");
    }
    tools.into_boxed()
}
fn connector_tools(timeout: Duration) -> Box<[Tool]> {
    let mut tools = List::with_capacity(5);
    for name in [b"read_logs".as_slice(), b"read_series", b"read_alerts", b"effect_restart", b"propose_restart"] {
        tools.push(Tool {
            name: name.into(), description: name.into(),
            schema: if name == b"propose_restart" {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"operation":{"type":"integer"},"reason":{"type":"string"}},"required":["environment","service","operation","reason"]}"#.as_slice().into()
            } else if name == b"effect_restart" {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"operation":{"type":"integer"}},"required":["environment","service","operation"]}"#.as_slice().into()
            } else if name == b"read_logs" {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"from":{"type":"integer"},"through":{"type":"integer"},"max_bytes":{"type":"integer"},"filter":{"type":"string"}},"required":["environment","service","from","through","max_bytes","filter"]}"#.as_slice().into()
            } else {
                br#"{"type":"object","properties":{"environment":{"type":"string"},"service":{"type":"string"},"from":{"type":"integer"},"through":{"type":"integer"},"max_bytes":{"type":"integer"}},"required":["environment","service","from","through","max_bytes"]}"#.as_slice().into()
            },
            effect: if name.starts_with(b"read_") { ToolEffect::Read } else { ToolEffect::Write }, timeout,
        }).expect("five declared tools fit");
    }
    tools.into_boxed()
}
