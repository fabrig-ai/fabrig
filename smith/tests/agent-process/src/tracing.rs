//! Output-pressure interposition around the shipped agent and diagnostic bridge.
//! It retains only kernel completions and observations; startup records, roots,
//! deadlines and settlement belong to `Agent` (smith/docs/design/testing.md, 4).

use skein_io::kernel::{self, Complete, Submit};
use skein_lib::{List, Queue, Time, Wall};
use skein_shell::Host;
use smith_agent_service as agent;

pub(super) struct Tracing {
    pub(super) agent: skein_world::PipeHost<smith_agent_shell::Agent>,
    pub(super) pressure: Option<super::world::orders::Pressure>,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}

impl Tracing {
    pub(super) fn new(
        mut configuration: agent::Config,
        inherited: &skein_world::Inherited,
        capture: smith_protocol_events::sinks::Capture,
    ) -> Self {
        for _ in 0..2 {
            let header =
                effective_header(&configuration.limits, &configuration.channel_endpoints, &configuration.llm_endpoints);
            configuration.limits.events = Some(
                smith_agent_service::event_limits(
                    &configuration.limits,
                    capture,
                    1,
                    0,
                    header.as_slice(),
                    skein_lib::Duration::from_secs(1),
                )
                .expect("world event bounds"),
            );
        }
        let queue = configuration.limits.queue;
        let mut configuration = crate::shell_configuration(configuration);
        configuration.trace = Some(smith_agent_shell::trace::TraceConfig {
            path: "event-trace/events.jsonl".into(),
            capture,
            delivery: smith_protocol_events::sinks::Delivery::Complete,
        });
        let pipe = |child| inherited.pipes.iter().find(|(number, _)| *number == child).expect("inherited pipe").1;
        let (errors, bridge) = crate::AgentErrors::pipe(pipe(2));
        let agent = smith_agent_shell::Agent::new(
            configuration,
            smith_agent_shell::Resources {
                input: pipe(0),
                output: pipe(1),
                signals: inherited.signal,
                seed: 7,
                pid: 101,
                trace: Some(inherited.appends.first().expect("startup trace").1),
                error: Some(pipe(2)),
                roots: Some(inherited.roots.iter().map(|(_, fd)| *fd).collect()),
                programs: None,
            },
            Box::new(errors),
        )
        .expect("observed shipped agent");
        Self {
            agent: bridge.host(agent),
            pressure: None,
            completions: Queue::with_capacity(queue),
            submissions: Queue::with_capacity(queue),
        }
    }
}

// Pressure changes the completion and submission passes, so this interposer
// implements Host; Delegate would forward past those controls.
impl Host for Tracing {
    fn drain(&mut self) {
        self.agent.drain();
        while let Some(submission) = self.agent.submissions().pop() {
            if let Some(pressure) = &mut self.pressure {
                pressure.observe(&submission);
            }
            self.submissions.push(submission);
        }
    }

    fn iterate(&mut self, now: Time, wall: Wall) {
        if let Some(pressure) = &mut self.pressure
            && pressure.released
            && let Some(complete) = pressure.held.take()
        {
            self.agent.completions().push(complete);
        }
        while let Some(complete) = self.completions.pop() {
            if let Some(pressure) = &mut self.pressure
                && pressure.holds(&complete)
            {
                assert!(complete.result.is_ok(), "pressure retains a successful completion");
                pressure.held = Some(complete);
            } else {
                self.agent.completions().push(complete);
            }
        }
        self.agent.iterate(now, wall);
    }

    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }

    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }

    fn work_pending(&self, now: Time) -> bool {
        self.agent.work_pending(now) || !self.completions.is_empty() || !self.submissions.is_empty()
    }

    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.agent.next_mechanism_deadline()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.agent.next_policy_deadline()
    }

    fn is_empty(&self) -> bool {
        self.agent.is_empty() && self.completions.is_empty() && self.submissions.is_empty()
    }

    fn exit(&self) -> Option<kernel::Exit> {
        if self.is_empty() { self.agent.exit() } else { None }
    }

    fn operations(&self) -> u32 {
        self.agent.operations()
    }
}

/// Receiving bounds derived from the same producer configuration.
pub(super) fn event_limits(limits: &agent::Limits) -> smith_protocol_events::EventLimits {
    let configuration = crate::configuration();
    let header = effective_header(limits, &configuration.channel_endpoints, &configuration.llm_endpoints);
    smith_agent_service::event_limits(
        limits,
        smith_protocol_events::sinks::Capture::Everything,
        1,
        0,
        header.as_slice(),
        skein_lib::Duration::from_secs(1),
    )
    .expect("world event bounds")
}

fn effective_header(
    limits: &agent::Limits,
    names: &smith_protocol_translate::Endpoints,
    endpoints: &smith_protocol_llm::Endpoints,
) -> List<smith_events::EffectiveLimit> {
    let configuration = crate::declarations(limits.machine.env_bytes);
    let inventory =
        agent::profile::effective(&crate::profile(), &configuration, limits, endpoints).expect("world inventory");
    agent::profile::header(&inventory, names, &configuration).expect("world effective names")
}
