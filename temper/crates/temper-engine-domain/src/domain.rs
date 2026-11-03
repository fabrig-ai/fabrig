//! The admitted step and ordered journal release (jig's domain/root.md, 3).
use crate::boundary::{
    BriefConnector, BriefSection, Event, Fact, Payload, PreparedWorkspace, Read, Released, Request, Work,
};
use crate::limits::{Limits, backlog, core_limits, environment_core, payload_slots, run_policy_bound, valid};
use crate::restart::{connector_restarting, request_load};
use crate::route::{
    LandingRule, forge_route, host_route, landing, policy_translate, route_core_requests, route_into, step_routed,
};
use crate::translate::{Config, RootConfig, environment_forge};
use crate::{CallKey, Decision, Delivery, Journal, JournalOutput as Output, Range, Write, loads};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core::Core;
use jig_domain_fleet as fleet;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Id, List, Map, Queue, ReplyTo, Slab, Time, Token, Wall};
use temper_engine_domain_calendar as calendar;
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change as forge_change;

/// Why a start stopped before decisions opened (temper/docs/design/domain/root.md, 9.1).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum StartStop {
    /// jig's start refused a fit at this phase (jig/docs/design/domain/engine.md, 6).
    Core { phase: Option<jig_domain_core::start::Phase>, failure: jig_domain_core::start::Failure },
    /// A startup page could not be read.
    Load(loads::RestoreFailure),
    /// The forge's restored records do not fit its limits.
    ForgeFit { phase: Option<jig_domain_core::start::Phase>, failure: forge::RestoreFit },
    /// The deployment's namespace binding was refused.
    Binding,
}

/// Core, forge, journal and bounded child-requested continuation bindings.
/// The core owns policy and current proofs; the forge owns connector payloads
/// and timer decisions. Every outbound envelope passes through the journal
/// (jig's domain/root.md, sections 2–4).
#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the journal door, header, pending switch, sweep page and opened milestone are independent"
)]
pub struct Domain {
    revision: u64,
    held_bytes: u64,
    /// Retired M1 reset marker; readiness never reads it. Keeping the marker
    /// needs no clock pair beside the retained restore failure.
    pub(crate) settled: Option<()>,
    pub(crate) settling: crate::settling::Phase,
    #[cfg(test)]
    pub(crate) empty_resumes: u32,
    #[cfg(test)]
    pub(crate) start_phases: Queue<jig_domain_core::start::Phase>,
    pub(crate) limits: Limits,
    /// Step/fire scratch, empty between calls.
    scratch: Queue<Request>,
    pub(crate) config: RootConfig,
    pub(crate) core: Core,
    pub(crate) calendar: calendar::Calendar,
    pub(crate) journal: Journal,
    /// Store credit: sent and cumulatively answered commit numbers.
    pub(crate) sent_commit: u64,
    pub(crate) durable_commit: u64,
    pub(crate) stop_pending: bool,
    pub(crate) door_pass: bool,
    pub(crate) door_count: u32,
    pub(crate) startup_range: Option<Range>,
    pub(crate) start_wall: Option<Wall>,
    pub(crate) start_stop: Option<StartStop>,
    pub(crate) opened: bool,
    pub(crate) start_pending: bool,
    pub(crate) startup_fill: Option<u64>,
    pub(crate) sweep_rows: Queue<crate::Record>,
    pub(crate) sweep_page: bool,
    pub(crate) sweep_next: Option<crate::Key>,
    pub(crate) core_header_loaded: bool,
    pub(crate) restored_commits: Option<u64>,
    pub(crate) brief_connectors: Slab<BriefConnector>,
    pub(crate) forge_briefs: Map<Token, BriefSection>,
    pub(crate) run_workspaces: Map<u64, PreparedWorkspace>,
    pub(crate) forge: forge::Domain,
    pub(crate) application: Option<(Token, crate::Application)>,
    pub(crate) application_carried: Option<jig_domain_core::Carried>,
    pub(crate) application_playground: Option<crate::route::playground::Context>,
    pub(crate) playground_contexts: Map<Token, crate::route::playground::Context>,
    pub(crate) playground_start: Option<crate::route::playground::Start>,
    pub(crate) playground_replies: Map<Token, (Option<Token>, crate::route::playground::Reply)>,
    pub(crate) next_playground_reply: u64,
    pub(crate) playground_provider: Option<u32>,
    pub(crate) playground_policy: Option<u32>,
    pub(crate) next_application_owner: u64,
    pub(crate) returns: Map<Token, (Time, Wall)>,
    pub(crate) adoption_restore: Map<Token, (Token, Option<forge::Repository>)>,
    pub(crate) forge_subscribing: Map<Token, forge::Subscriber>,
    pub(crate) forge_unsubscribing: Map<Token, (u64, forge::Topic)>,
    pub(crate) forge_reading: Map<Token, (ReplyTo, CallKey)>,
    pub(crate) forge_left: Map<(u64, u64), Box<[forge::Pushed]>>,
    pub(crate) forge_effecting: Map<u64, (ReplyTo, CallKey)>,
    pub(crate) forge_effects: Map<Token, forge_route::EffectFlight>,
    pub(crate) next_effect_owner: u64,
    pub(crate) forge_delegating: Map<(u64, u64), forge_change::Delegate>,
    pub(crate) loads: loads::Loads,
    pub(crate) hub_link: jig_domain_hub_link::Link,
    pub(crate) assignment_parts: Map<u64, crate::boundary::AssignmentParts>,
    pub(crate) answer_parts: Map<Token, crate::boundary::Parts>,
    pub(crate) refused_calls: Map<Token, Box<[u8]>>,
    /// Loads of durable rows outside the working set: a run's preparation
    /// (transcript, dependencies, inputs) and historical queries.
    pub(crate) archive_reads: Slab<Option<Read>>,
    /// Read loads' answers that waited for journal room (jig's `domain/root.md`, 3).
    pub(crate) store_answers: Queue<Event>,
    pub(crate) call_renders: Map<Token, host_route::Flight>,
    pub(crate) work: Queue<Work>,
    pub(crate) facts: Queue<Fact>,
    pub(crate) facts_lost: u64,
}

impl Domain {
    /// The phase the start is in, none before the start and once decisions open.
    #[must_use]
    pub fn start_phase(&self) -> Option<jig_domain_core::start::Phase> {
        self.core.start_phase()
    }

    /// First unmade core run-proof link before startup can open.
    #[must_use]
    pub fn core_missing_proof_link(&self) -> Option<(jig_domain_core::Key, jig_domain_core::Key)> {
        self.core.missing_proof_link()
    }

    /// First task's charter absent from its held settings at startup.
    #[must_use]
    pub fn core_missing_charter_link(&self) -> Option<(jig_domain_core::Key, u32, u32)> {
        self.core.missing_charter_link()
    }

    /// The start's stop, kept until the service reports it.
    #[must_use]
    pub fn start_stop(&self) -> Option<StartStop> {
        self.start_stop.clone()
    }

    /// The transcript and turn bytes the root and its core hold now
    /// (domain/root.md, 4.1): the root's running total and the core's
    /// `transcript_bytes`. A person's read pages waiting in `store_answers`
    /// are not counted: they are not transcripts, and a preparation's
    /// transcript pages (`StoreInput::Loaded`) never wait there.
    #[must_use]
    pub fn held_bytes(&self) -> u64 {
        self.held_bytes.checked_add(self.core.transcript_bytes()).expect("held transcript and turn bytes fit u64")
    }

    pub(crate) fn hold_bytes(&mut self, bytes: u64) {
        self.held_bytes = self.held_bytes.checked_add(bytes).expect("root transcript and turn bytes fit u64");
    }

    pub(crate) fn release_bytes(&mut self, bytes: u64, released: &str) {
        self.held_bytes = self.held_bytes.checked_sub(bytes).expect(released);
    }

    /// Deployment models in the order settings authority and model choices use.
    #[must_use]
    pub fn models(&self) -> &[jig_domain_core::Model] {
        &self.core.settings.models
    }

    /// The live task arena, for a world to inspect between iterations.
    #[must_use]
    pub const fn tasks(&self) -> &tasks::Domain {
        &self.core.tasks
    }

    /// Tasks whose prepared assignment has not yet been handed to a worker.
    #[must_use]
    pub fn pending_assignments(&self) -> List<u64> {
        let mut tasks = List::with_capacity(self.assignment_parts.len());
        for (task, _) in &self.assignment_parts {
            tasks.push(*task).expect("one task per assignment");
        }
        tasks
    }

    /// Conservative mutation revision, including refused inputs, empty probes,
    /// observation drains and reclamation. Nested public calls may bump twice.
    /// Owners checkpoint it only after their release and reclaim pass settles.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) fn changed(&mut self) {
        self.revision = self.revision.checked_add(1).expect("root revision space");
    }

    /// Wake deployment-row subscribers after volatile routing diagnostics change.
    pub fn mark_forges(&mut self) {
        self.changed();
        let mut subjects = List::with_capacity(1);
        subjects.push(Token::new(0)).expect("one forges subject");
        self.work.push(Work::Core(jig_domain_core::Event::Marked { connector: self.config.forge_connector, subjects }));
    }

    /// Routing capacities are fixed before the first input and priced by the service.
    pub fn connection_limits(&mut self, forges: u32, installations: u32) {
        self.changed();
        self.settled = None;
        self.forge.connection_limits(forges, installations);
    }
    /// The routing's latest connection information; no connection state is restored.
    #[must_use]
    pub fn forge_connection(&self, forge: u16) -> forge::Connection {
        self.forge.connection(forge)
    }
    /// The current deployment document, including volatile setup progress.
    #[must_use]
    pub fn forges(&self) -> forge::Forges {
        self.forge.forges()
    }
    /// The playground row as the service reads it (temper/docs/design/protocol/setup.md, 8.4).
    #[must_use]
    pub fn playground(&self) -> Option<&forge::Playground> {
        self.forge.playground()
    }
    /// Whether the connector has ended the forge at removal's decision.
    #[must_use]
    pub fn forge_ended(&self, forge: u16) -> bool {
        self.forge.forge_ended(forge)
    }
    /// Restored provider ids for the setup host's start-time reconcile.
    #[must_use]
    pub fn adopted_ids(&self, forge: u16) -> Box<[u64]> {
        self.forge.adopted_ids(forge)
    }
    #[must_use]
    pub fn forge_installation(&self, forge: u16, installation: u64) -> Option<&forge::Installation> {
        self.forge.installation(forge, installation)
    }

    /// Allocate the root and fixed child room from shell-supplied configuration and validated
    /// cross-child limits. Checks bounded authority/bootstrap configuration; issues no request.
    /// Startup pages/account setup begin only on `Event::Restart`.
    /// # Errors
    /// A configured worker count or name exceeds its priced membership bound.
    #[expect(clippy::too_many_lines, reason = "the root allocates every child and bounded handoff table together")]
    pub fn new(mut config: Config, limits: &Limits) -> Result<Domain, fleet::WorkerProblem> {
        assert!(valid(limits), "root limits are valid");
        assert!(
            config.core.settings.deployment_provider != 0,
            "deployment service provider differs from forge sign-in provider 0"
        );
        assert!(config.core.connectors.len() == 1, "the forge is the deployment's one connector");
        let forge_connector = *config.core.connectors.first().expect("one connector");
        for charter in &config.core.defaults.charters {
            assert!(
                match run_policy_bound(&charter.policy, limits) {
                    Some(bytes) => bytes <= u64::from(limits.journal.run_bytes),
                    None => false,
                },
                "each charter's run policy fits the assignment bound: charter {} needs {:?}, bound {}",
                charter.number,
                run_policy_bound(&charter.policy, limits),
                limits.journal.run_bytes
            );
        }
        let mut forge_charter = false;
        for charter in &config.core.defaults.charters {
            forge_charter |= charter.number == config.forge_charter;
        }
        assert!(forge_charter, "the forge's agent work runs under a configured charter");
        assert!(
            config.core.settings.resume_bytes > 0
                && config.core.settings.resume_bytes <= limits.journal.transcript_bytes,
            "configured task transcript bound fits the root's owned-byte limit"
        );
        assert!(
            limits.forge.judge_projects >= limits.authority.projects
                && limits.forge.judge_criteria >= limits.authority.requirements,
            "forge judge table covers authority policy bounds"
        );
        assert!(
            config.landing.projects.capacity() <= limits.authority.projects
                && config.landing.landings.capacity() <= limits.authority.projects,
            "typed landing project table fits the policy bound"
        );

        assert!(
            landing_within(&config.landing.deployment, limits.journal.transcript_bytes),
            "deployment landing policy fits owned-byte bound"
        );
        for (_, rules) in &config.landing.projects {
            assert!(
                landing_within(rules, limits.journal.transcript_bytes),
                "project landing policy fits owned-byte bound"
            );
        }
        assert!(
            config.core.seed_projects.len() <= usize::try_from(limits.authority.projects).expect("project bound"),
            "permission policy project bound"
        );
        for project in &config.core.seed_projects {
            let mappings = &project.policy.permissions;
            let count = u64::try_from(mappings.len()).expect("usize fits u64");
            let unit = u64::try_from(size_of::<people::PermissionRole>()).expect("type size fits u64");
            assert!(
                match count.checked_mul(unit) {
                    Some(bytes) => bytes <= u64::from(limits.journal.transcript_bytes),
                    None => false,
                },
                "permission policy bound"
            );
        }
        for owner in &config.core.owners {
            if owner.project == people::DEPLOYMENT {
                continue;
            }
            assert!(
                seed_policy(&config.core.seed_projects, owner.project).is_some(),
                "bootstrap owner names configured policy"
            );
            assert!(
                match seed_policy(&config.core.seed_projects, owner.project).expect("known policy").escalation_role {
                    Some(role) => role <= 3,
                    None => false,
                },
                "current root requires final escalation role"
            );
        }
        let mut forge_config = core::mem::replace(
            &mut config.forge,
            skein_forge_client::Config { namespace: Box::new([]), writers: Box::new([]) },
        );
        forge_config.namespace = Box::from(config.core.deployment);
        let mut forge = forge::Domain::new(
            &limits.forge,
            config.core.seed,
            forge_config,
            &config.identities,
            &config.forges,
            config.under_box,
        )
        .expect("valid forge configuration");
        if let Some(machine) = config.machine.take() {
            forge.set_machine(machine);
        }
        #[expect(clippy::manual_map, reason = "the strict subset uses explicit matches")]
        let playground_provider = match &config.playground {
            Some(config) => Some(config.provider),
            None => None,
        };
        #[expect(clippy::manual_map, reason = "the strict subset uses explicit matches")]
        let playground_policy = match config.core.defaults.policies.first() {
            Some((number, _)) => Some(*number),
            None => None,
        };
        forge.configure_playground(config.playground.take());
        let built = policy_translate::build_landing(
            &config.landing.deployment,
            false,
            limits.authority.requirements,
            forge_connector,
        )
        .expect("deployment landing requirements bounded");
        assert!(
            add_configured_requirements(&mut config.core.defaults.rules, &built.requirements, &limits.authority),
            "deployment landing requirements fit authority configuration"
        );
        assert!(forge.deployment_judges(built.criteria), "deployment judge table bounded");
        for (&project, rules) in &config.landing.projects {
            let built = policy_translate::build_landing(rules, true, limits.authority.requirements, forge_connector)
                .expect("project landing requirements bounded");
            let policy =
                seed_policy_mut(&mut config.core.seed_projects, project).expect("landing project has a policy");
            assert!(
                policy_translate::landing_roles(policy, &config.landing.deployment),
                "deployment landing roles configured"
            );
            assert!(policy_translate::landing_roles(policy, rules), "project landing roles configured");
            let combined = policy_translate::combine_requirements(
                &policy.requirements,
                &built.requirements,
                limits.authority.requirements,
            )
            .expect("project landing requirements fit authority policy");
            policy.requirements = combined;
            assert!(forge.project_judges(project, built.criteria), "project judge table bounded");
        }
        let gate_policy = policy_translate::gate_policy(
            &config.landing,
            limits.authority.requirements,
            limits.forge.change_policy.gates,
        )
        .expect("landing gate templates bounded");
        assert!(forge.gate_policy(gate_policy, &limits.forge), "forge owns bounded landing templates");
        for (&project, landing) in &config.landing.landings {
            assert!(
                crate::translate::project_landing(&mut forge, project, landing),
                "project landing configuration bounded and supported"
            );
        }
        let root_config =
            RootConfig { forge_connector, forge_charter: config.forge_charter, landings: config.landing.landings };
        let link_rng = skein_lib::Rng::new(config.core.seed);
        let mut core = Core::new(config.core, &core_limits(limits))?;
        // temper/docs/design/domain/agent.md, 8.2: the person answers in their inbox.
        core.person_answers();
        let mut configured = Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
            tasks::Event::Kinds {
                connector: root_config.forge_connector,
                kinds: Box::new([tasks::Kind {
                    connector: root_config.forge_connector,
                    kind: 1,
                    hold: match forge::resources::BRANCH_HOLD {
                        forge::resources::HoldKind::Exclusive { wait } => tasks::HoldKind::Exclusive {
                            taken: if wait { tasks::Taken::Waits } else { tasks::Taken::Refuses },
                        },
                        forge::resources::HoldKind::Shared => unreachable!("private branch kind takes exclusive holds"),
                    },
                }]),
            },
            &mut configured,
        );
        assert!(configured.is_empty(), "configuration emits no request");
        Ok(Domain {
            revision: 0,
            held_bytes: 0,
            calendar: calendar::Calendar::new(
                &calendar::Limits { projects: limits.authority.projects },
                config.period_span,
            ),
            scratch: Queue::uncapped(),
            settled: None,
            settling: crate::settling::Phase::Running,
            #[cfg(test)]
            empty_resumes: 0,
            #[cfg(test)]
            start_phases: Queue::uncapped(),
            journal: Journal::uncapped(1, 0),
            sent_commit: 0,
            durable_commit: 0,
            stop_pending: false,
            door_pass: false,
            door_count: 0,
            startup_range: None,
            start_wall: None,
            start_stop: None,
            opened: false,
            start_pending: false,
            startup_fill: None,
            sweep_rows: Queue::uncapped(),
            sweep_page: false,
            sweep_next: None,
            core_header_loaded: false,
            restored_commits: None,
            core,
            brief_connectors: Slab::with_capacity(
                limits
                    .brief
                    .briefs
                    .checked_mul(limits.brief.sections)
                    .expect("validated brief count")
                    .checked_mul(2)
                    .expect("validated brief connector room"),
            ),
            forge_briefs: Map::uncapped(),
            run_workspaces: Map::with_capacity(limits.tasks.tasks),
            forge,
            application: None,
            application_carried: None,
            application_playground: None,
            playground_contexts: Map::with_capacity(limits.forge.applications),
            playground_start: None,
            playground_replies: Map::with_capacity(limits.forge.applications),
            next_playground_reply: u64::MAX,
            playground_provider,
            playground_policy,
            next_application_owner: 1,
            returns: Map::with_capacity(limits.people.requests),
            adoption_restore: Map::with_capacity(limits.forge.adoptions),
            forge_subscribing: Map::with_capacity(limits.fleet.calls),
            forge_unsubscribing: Map::with_capacity(limits.fleet.calls),
            forge_reading: Map::with_capacity(limits.fleet.calls),
            forge_left: Map::with_capacity(limits.tasks.tasks),
            forge_effecting: Map::with_capacity(limits.fleet.calls),
            forge_effects: Map::with_capacity(
                limits.call_records.checked_add(limits.tasks.tasks).expect("effect handoff capacity"),
            ),
            next_effect_owner: 1,
            forge_delegating: Map::with_capacity(limits.forge.changes),
            loads: loads::Loads::new(&limits.loads),
            hub_link: jig_domain_hub_link::Link::new(
                jig_domain_hub_link::Limits {
                    assignments: limits.tasks.tasks,
                    payloads: payload_slots(limits).expect("valid payload room"),
                    turns: limits.tasks.tasks,
                },
                link_rng,
            ),
            assignment_parts: Map::with_capacity(limits.tasks.tasks),
            answer_parts: Map::uncapped(),
            refused_calls: Map::uncapped(),
            archive_reads: Slab::with_capacity(limits.loads.loads),
            store_answers: Queue::with_capacity(limits.loads.loads),
            call_renders: Map::with_capacity(limits.fleet.calls),
            work: Queue::uncapped(),
            facts: Queue::with_capacity(crate::limits::FACTS),
            facts_lost: 0,
            config: root_config,
            limits: *limits,
        })
    }

    /// First connector-owned configured-fit refusal during startup.
    #[must_use]
    pub fn forge_fit_failure(&self) -> Option<&forge::RestoreFit> {
        self.forge.restore_fit_failure()
    }
    /// First unmade connector link before startup reconciliation.
    #[must_use]
    pub fn forge_missing_link(&self) -> Option<forge::MissingLink> {
        self.forge.missing_link(&self.core.tasks)
    }
    /// Decisions open and the journal is not stopped (temper/docs/design/domain/root.md, 9.1).
    #[must_use]
    pub fn ready(&self) -> bool {
        self.opened && self.start_stop().is_none() && !self.journal.stopped()
    }

    pub(crate) fn start_runtime(&self) -> bool {
        use jig_domain_core::start::Phase;
        match self.start_phase() {
            Some(
                Phase::LoadCore
                | Phase::LoadInbox
                | Phase::RestoreConnector { .. }
                | Phase::Link
                | Phase::Sweep
                | Phase::Switch
                | Phase::Wires,
            ) => false,
            Some(Phase::AdoptRuns | Phase::ReadAfresh { .. } | Phase::SettleOutbox { .. } | Phase::Open) => true,
            None => self.opened,
        }
    }

    pub(crate) fn stop_start(&mut self, stop: StartStop) {
        if self.start_stop.is_none() {
            self.start_stop = Some(stop);
            self.stop_pending = true;
        }
    }

    pub(crate) fn stop_core(&mut self) {
        let failure = self.core.start_failure().expect("a refused row records its start fit failure");
        self.stop_start(StartStop::Core { phase: self.start_phase(), failure });
    }

    /// Pure shell idle query: no immediate internal handoff or issued store operation
    /// remains unfinished. Call after the iteration's reclaim; sessions, idle
    /// workers, assigned workers awaiting external answers and future task/account
    /// timers are permitted. Story completion also requires the shell's independent
    /// final result condition. This query never cancels work.
    #[must_use]
    pub fn quiescent(&self) -> bool {
        self.ready()
            && self.core.quiescent(self.journal.idle())
            && self.loads.quiescent()
            && self.work.is_empty()
            && self.assignment_parts.is_empty()
            && self.hub_link.payload_count() == 0
            && self.answer_parts.is_empty()
            && self.refused_calls.is_empty()
            && self.forge_briefs.is_empty()
            && self.archive_reads.is_empty()
            && self.store_answers.is_empty()
            && self.adoption_restore.is_empty()
            && self.forge_subscribing.is_empty()
            && self.forge_unsubscribing.is_empty()
            && self.forge_reading.is_empty()
            && self.forge.briefs_idle()
            && self.call_renders.is_empty()
            && self.forge_effecting.is_empty()
            && self.forge_effects.is_empty()
            && self.brief_connectors.is_empty()
            && !self.forge.is_ready()
    }

    /// Pure snapshot query for the latest allocated deployment counters; these may run ahead of
    /// store durability. Exposes neither children nor owned handoff bodies and emits no effect.
    #[must_use]
    pub fn deployment(&self) -> crate::Deployment {
        self.core.counters.deployment()
    }

    /// Retired IO/body/child slots are reclaimed at iteration end, after every event and ready
    /// pass. Bounded child/slab bookkeeping releases only retired entries; issued loads still
    /// awaiting a terminal remain owned. Emits no request or terminal.
    pub fn reclaim(&mut self) {
        self.changed();
        self.settled = None;
        self.core.reclaim();
        self.brief_connectors.reclaim();
        self.forge.reclaim();
        self.archive_reads.reclaim();
        loads::reclaim(&mut self.loads);
    }

    /// Shell/root caller discards every currently queued child observation, scanning at most each
    /// child's configured fact capacity. Emits no effect or terminal and changes no decision state.
    pub fn drain_facts(&mut self) {
        self.changed();
        self.core.drain_facts(&core_limits(&self.limits));
    }

    /// The oldest of the root's own facts not drained yet.
    pub fn pop_fact(&mut self) -> Option<Fact> {
        self.changed();
        self.facts.pop()
    }

    /// Live pieces dropped for an unkept kind or an offset beyond the views' u32
    /// (jig's `domain/hosts.md`, 6.5; `domain/engine.md`, 11).
    #[must_use]
    pub const fn pieces_dropped(&self) -> u64 {
        self.hub_link.pieces_dropped()
    }

    /// How many of the root's own facts were dropped for want of room.
    #[must_use]
    pub const fn facts_lost(&self) -> u64 {
        self.facts_lost
    }
}

pub(crate) fn internal(number: u64) -> ReplyTo {
    ReplyTo::new(Token::new(number))
}

pub(crate) fn emit(decision: &mut Decision, limits: &Limits, delivery: Delivery) {
    decision.carry_delivery(&limits.journal, delivery);
}

/// Return replaced turn bytes to their root owner. A refused write means an
/// overrun stops the root, which never activates again; its total is then moot.
pub(crate) fn save(decision: &mut Decision, limits: &Limits, write: Write) -> u64 {
    decision.carry_write(&limits.journal, write).unwrap_or_default()
}

/// Route one admitted input and all synchronous child callbacks in one decision; store terminals
/// are accepted even under pressure. The shell supplies
/// unchanged configured limits with injected time. Refused web/worker calls get terminal/busy
/// notices before child mutation; admitted effects may wait in the journal until store durability.
/// Account operations keep their own secret-free terminal contract.
pub fn step(domain: &mut Domain, env: &Env<Limits>, event: Event) {
    domain.changed();
    let mut out = core::mem::replace(&mut domain.scratch, Queue::with_capacity(0));
    step_routed(domain, env, event, &mut out);
    stage_now(domain, &mut out);
    assert!(out.is_empty(), "step stages every immediate output");
    domain.scratch = out;
}

/// Whether a due task may start its preparation: the held transcript and
/// turn bytes are under the budget (domain/root.md, 4.1). Only an
/// activation asks; nothing else waits on bytes.
pub(crate) fn within_budget(domain: &Domain, limits: &Limits) -> bool {
    domain.held_bytes() < limits.memory_bytes
}

pub(crate) fn admits(domain: &Domain, limits: &Limits) -> bool {
    backlog(domain) < u64::from(limits.watermark)
        && domain.core.counters.deployment().messages
            <= u64::MAX.checked_sub(u64::from(limits.tasks.tasks)).expect("task count fits u64")
        && domain.work.is_empty()
}

// Whether this close was empty, including the automatic deployment header.
pub(crate) fn close(domain: &mut Domain, env: &Env<Limits>, decision: Decision, out: &mut Queue<Request>) -> bool {
    finish_close(domain, env, decision, out, false)
}

fn finish_close(
    domain: &mut Domain,
    env: &Env<Limits>,
    mut decision: Decision,
    out: &mut Queue<Request>,
    finishing_only: bool,
) -> bool {
    if domain.start_stop().is_some() {
        out.push(Request::Stop);
        return false;
    }
    let requests = if domain.start_runtime() {
        jig_domain_core::finish_decision(&mut domain.core, &environment_core(env))
    } else {
        Queue::uncapped()
    };
    let finished = requests.is_empty();
    route_core_requests(domain, env, &mut decision, requests);
    route_into(domain, env, &mut decision);
    let persist_header = domain.core_header_loaded
        && (domain.start_phase() != Some(jig_domain_core::start::Phase::LoadCore) || domain.startup_range.is_none());
    let empty = finished
        && decision.empty()
        && !(persist_header && domain.core.counters.dirty())
        && domain.work.is_empty()
        && domain.store_answers.is_empty()
        && out.is_empty();
    if finishing_only && !decision.overrun() {
        assert!(decision.write_free() && !domain.core.counters.dirty(), "stopping finishing makes no commit");
    }
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.core.counters,
        &env.limits.journal,
        decision,
        persist_header,
    )
    .expect("root decision uses its configured store edge");
    if let Some(commits) = domain.restored_commits.take() {
        assert!(domain.journal.idle(), "deployment is the first cold-start row");
        domain.journal = Journal::uncapped(1, commits);
        domain.sent_commit = commits;
        domain.durable_commit = commits;
    }
    if domain.journal.stopped() {
        // Immediate outputs assembled alongside this failed cohort must not
        // reach the journal's door either.
        for _ in 0..out.len() {
            let _discarded = out.pop();
        }
        out.push(Request::Stop);
        return false;
    }
    empty
}

pub(crate) fn invalidate_input(domain: &mut Domain, event: &Event) {
    match event {
        Event::Resume => {}
        Event::Setup { .. }
        | Event::Released(_)
        | Event::Forge(_)
        | Event::Stop
        | Event::Credential { .. }
        | Event::Account(_)
        | Event::Party(_)
        | Event::Store(_)
        | Event::Restart { .. }
        | Event::Worker(_) => domain.settled = None,
    }
}

pub(crate) fn journal_outputs(journal_out: &mut Queue<Output>, out: &mut Queue<Request>) {
    for _ in 0..journal_out.len() {
        match journal_out.pop().expect("journal output count") {
            Output::Now(request) => out.push(request),
            Output::Commit { number, writes } => {
                out.push(Request::Store(crate::StoreRequest::Commit { number, writes }));
            }
            Output::Stop => out.push(Request::Stop),
            Output::Deliver(delivery) => out.push(crate::boundary::delivery_output(delivery)),
        }
    }
}

/// The only admission call for outputs that have no durability dependency.
pub(crate) fn now(domain: &mut Domain, request: Request) -> bool {
    domain.journal.now(request).is_ok()
}

/// Admit immediate requests through the journal's door. The caller takes
/// them on a later release pass; no step returns an outbound queue.
pub(crate) fn stage_now(domain: &mut Domain, out: &mut Queue<Request>) {
    domain.door_pass = true;
    for _ in 0..out.len() {
        let request = out.pop().expect("original output count");
        match request {
            Request::Stop => domain.stop_pending = true,
            request @ (Request::Worker(_)
            | Request::Party(_)
            | Request::Connect { .. }
            | Request::Replace { .. }
            | Request::Manifest { .. }
            | Request::Reachable { .. }
            | Request::Install { .. }
            | Request::Accept { .. }
            | Request::EraseCarried { .. }
            | Request::Setup { .. }
            | Request::ForgeKept(_)
            | Request::ForgeGone { .. }
            | Request::BoxGroup { .. }
            | Request::PlaygroundActs { .. }
            | Request::ResetAct { .. }
            | Request::BoxRemove { .. }
            | Request::SetUp { .. }
            | Request::Bound { .. }
            | Request::Return { .. }
            | Request::Forge { .. }
            | Request::Store(
                crate::StoreRequest::Switch | crate::StoreRequest::Load { .. } | crate::StoreRequest::Read { .. },
            )
            | Request::ToChild(_)
            | Request::Busy(_)
            | Request::Unsettled(_)
            | Request::Drained
            | Request::Account(_)) => {
                assert!(now(domain, request), "root door reserves one route's immediate outputs");
                domain.door_count = domain.door_count.checked_add(1).expect("bounded immediate route");
            }
            Request::Store(crate::StoreRequest::Commit { .. }) => {
                unreachable!("release takes commits from the journal")
            }
        }
    }
}

/// Drain the journal only; child continuations enter through `Event::Released`
/// and owner progress through `Event::Resume`, each with its own admission.
pub fn release(domain: &mut Domain, _env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.changed();
    if domain.stop_pending {
        domain.stop_pending = false;
        domain.door_pass = false;
        domain.door_count = 0;
        out.push(Request::Stop);
        return;
    }
    if domain.start_stop().is_some() {
        return;
    }
    if domain.door_pass {
        domain.door_pass = false;
        send_commit(domain, out);
        for _ in 0..domain.door_count {
            assert_eq!(release_one(domain, out), skein_lib::Released::Some, "one admitted immediate output");
        }
        domain.door_count = 0;
        return;
    }
    // The journal's release limit is one held output before the next commit.
    let _status: skein_lib::Released = release_one(domain, out);
    send_commit(domain, out);
}

// Moving an assignment into a held delivery changes no ownership. Only the
// journal's released request leaves the root, on either release pass.
fn release_one(domain: &mut Domain, out: &mut Queue<Request>) -> skein_lib::Released {
    let before = usize::try_from(out.len()).expect("output count fits usize");
    let status = domain.journal.release(out);
    let mut bytes = 0_u64;
    for request in out.iter().skip(before) {
        if let Request::Worker(crate::WorkerRequest::Deliver(Delivery::Assigned { assignment, .. })) = request {
            bytes = bytes
                .checked_add(crate::budget::assignment_bytes(assignment))
                .expect("released assignment bytes fit u64");
        }
    }
    domain.release_bytes(bytes, "released assignment turns and sections were held");
    status
}

pub(crate) fn send_commit(domain: &mut Domain, out: &mut Queue<Request>) {
    // Unsent decisions stay in the journal until a store answer frees credit.
    if domain.journal.stopped()
        || domain.sent_commit.checked_sub(domain.durable_commit).expect("store answers only sent commits")
            >= u64::from(domain.limits.journal.commits)
    {
        return;
    }
    match crate::commit(&mut domain.journal, &domain.limits.journal) {
        Some(Output::Commit { number, writes }) => {
            for write in &writes {
                domain.release_bytes(crate::budget::turn_bytes(write), "issued commit's turn body was held");
            }
            domain.sent_commit = number;
            out.push(Request::Store(crate::StoreRequest::Commit { number, writes }));
        }
        Some(Output::Now(_) | Output::Deliver(_) | Output::Stop) => unreachable!("journal commit has writes only"),
        None => {}
    }
}

pub(crate) fn resume_routed(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.journal.stopped() || domain.start_stop().is_some() {
        return;
    }
    if !domain.store_answers.is_empty() && domain.work.is_empty() {
        let event = domain.store_answers.pop().expect("queued store answer");
        step_routed(domain, env, event, out);
        return;
    }
    if !domain.work.is_empty() {
        let decision = route(domain, env);
        close(domain, env, decision, out);
        return;
    }
    if backlog(domain) >= u64::from(env.limits.watermark) {
        return;
    }
    // From the stop on, nothing that would make a commit is taken: connector
    // work stays with the connector and no run is placed (jig's
    // `domain/engine.md`, 6.1).
    if domain.core.stopping() {
        if domain.core.finish_owed() {
            domain.settled = None;
            let decision = Decision::decide(&mut domain.journal, &env.limits.journal);
            finish_close(domain, env, decision, out, true);
        }
        return;
    }
    if domain.ready() || connector_restarting(domain) {
        if domain.forge.is_ready() {
            domain.settled = None;
            let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
            let mut child = Queue::uncapped();
            forge::resume(&mut domain.forge, &environment_forge(env), &mut child);
            forge_route::outputs(domain, env, &mut decision, &mut child);
            route_into(domain, env, &mut decision);
            close(domain, env, decision, out);
            return;
        }
        if !domain.ready() {
            return;
        }
        if !domain.settling.active()
            && within_budget(domain, &env.limits)
            && let Some(context) = domain.core.take_due()
        {
            domain.work.push(Work::Activate(context));
            let decision = route(domain, env);
            close(domain, env, decision, out);
            return;
        }
        if !domain.core.fleet.is_ready() && !domain.core.finish_owed() {
            let due = match domain.next_deadline(env) {
                Some(deadline) => deadline <= env.now,
                None => false,
            };
            if !due {
                return;
            }
        }
        domain.settled = None;
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        let routed = jig_domain_core::resume_fleet(&mut domain.core, &environment_core(env));
        let fleet_empty = routed.is_empty();
        route_core_requests(domain, env, &mut decision, routed);
        route_into(domain, env, &mut decision);
        if close(domain, env, decision, out) && fleet_empty {
            #[cfg(test)]
            {
                domain.empty_resumes = domain.empty_resumes.checked_add(1).expect("test decision count");
            }
        }
    }
}

/// Fire participating child timers through the same barrier; store durability and inputs run before
/// this pass. The shell supplies injected monotonic/wall time. Account
/// timers may emit bounded protocol actions independently; root routes that mutate tasks/fleet wait
/// for whole-decision admission. Their store and account outcomes enter later through `step`.
pub fn fire(domain: &mut Domain, env: &Env<Limits>) {
    domain.changed();
    let mut out = core::mem::replace(&mut domain.scratch, Queue::with_capacity(0));
    fire_routed(domain, env, &mut out);
    stage_now(domain, &mut out);
    assert!(out.is_empty(), "fire stages every immediate output");
    domain.scratch = out;
}

pub(crate) fn fire_routed(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    domain.settled = None;
    if domain.journal.stopped() || domain.start_stop().is_some() {
        return;
    }
    if !domain.start_runtime() {
        return;
    }
    // A timer that cannot be admitted stays due and fires again next pass.
    if !admits(domain, &env.limits) {
        return;
    }
    domain.settled = None;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    if let Some(timer) = domain.core.due(env.now, env.wall) {
        let routed = jig_domain_core::fire(&mut domain.core, &environment_core(env), timer);
        route_core_requests(domain, env, &mut decision, routed);
    }
    let restarting = connector_restarting(domain);
    if (restarting || domain.ready() || domain.settling.active()) && !domain.core.stopping() {
        let mut forge_out = Queue::uncapped();
        forge::fire(&mut domain.forge, &environment_forge(env), &mut forge_out);
        forge_route::outputs(domain, env, &mut decision, &mut forge_out);
    }
    route_into(domain, env, &mut decision);
    if domain.ready() {
        if !domain.core.stopping()
            && let Some(next) = domain.calendar.next_start()
            && next <= env.wall
        {
            crate::route::calendar::route(domain, env, calendar::Event::Due, None);
            route_into(domain, env, &mut decision);
        }
        let routed =
            jig_domain_core::step(&mut domain.core, &environment_core(env), jig_domain_core::Event::EffectDeadline);
        route_core_requests(domain, env, &mut decision, routed);
        route_into(domain, env, &mut decision);
    }
    close(domain, env, decision, out);
}

pub(crate) fn route(domain: &mut Domain, env: &Env<Limits>) -> Decision {
    domain.settled = None;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    route_into(domain, env, &mut decision);
    decision
}

pub(crate) fn forge_release_ending(ending: tasks::Ending) -> forge::ReleaseEnding {
    match ending {
        tasks::Ending::Done(_) => forge::ReleaseEnding::Done,
        tasks::Ending::Failed { .. } => forge::ReleaseEnding::Failed,
        tasks::Ending::Cancelled { .. } => forge::ReleaseEnding::Cancelled,
    }
}
/// A released continuation advances admitted work and is always taken.
pub(crate) fn step_released(domain: &mut Domain, env: &Env<Limits>, released: Released, out: &mut Queue<Request>) {
    // Retire the notification while stopping. The entry stays durable for
    // the next start's outbox settlement; returning it would retry forever.
    if let Delivery::ForgeCommitted { .. } = released.0
        && (domain.core.stopping() || domain.settling.active())
    {
        return;
    }
    domain.settled = None;
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    released_into(domain, env, &mut decision, released, out);
    route_into(domain, env, &mut decision);
    close(domain, env, decision, out);
}

fn released_into(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    released: Released,
    out: &mut Queue<Request>,
) {
    match released.0 {
        Delivery::ProjectReleased { connector, project } => {
            domain.work.push(Work::Core(jig_domain_core::Event::ProjectReleased { connector, project }));
        }
        Delivery::ProjectMade { project } => {
            if let Some(row) = domain.core.tasks.funding(tasks::Funder::Period { project, period: 1 })
                && let Some(times) = row.period
            {
                crate::route::calendar::route(
                    domain,
                    env,
                    calendar::Event::Began { project, period: 1, began: times.began },
                    None,
                );
            }
            // jig's domain/engine.md, "a project made": the root hears it after commit.
            // LandingPolicy.projects, forge Judges.projects and GatePolicy.projects
            // hold optional seeded overrides; deployment rules already cover this project.
            // Repositories and homes arise on adoption, panels on watching.
            // None needs an empty-project row.
        }
        Delivery::ForgeCommitted { entry } => {
            domain.work.push(Work::Forge(forge::Event::Committed { entry }));
        }
        Delivery::Start(continuation) => {
            let mut output = Queue::uncapped();
            let action = domain.core.start_resume(&environment_core(env), continuation, &mut output);
            route_core_requests(domain, env, decision, output);
            domain.work.push(Work::Start(action));
        }
        Delivery::Fleet(event) => domain.work.push(Work::Fleet(event)),
        Delivery::NotesLoad { owner, range } => crate::route::held::notes_load(domain, env, decision, owner, range),
        Delivery::Relay { task, attempt, word, .. } => {
            let name = Token::new(word.number);
            let payload = domain.hub_link.retain(Payload::InboxWord(word));
            domain.work.push(Work::Fleet(fleet::Event::Inbound {
                run: Token::new(task),
                attempt: Token::new(attempt),
                message: fleet::Message { name, sender: payload, words: payload },
            }));
        }
        Delivery::Load { waiter, range, after } => {
            request_load(domain, waiter, range, after, out);
        }
        Delivery::ReadEscalationDecision { waiter } => {
            let Some(Some(read)) = domain.archive_reads.get(Id::from_token(waiter)) else { return };
            let (task, revision) = match read {
                Read::Escalation(query) => (query.task, query.revision),
                Read::Ended(_)
                | Read::Transcript { .. }
                | Read::GoalWords(_)
                | Read::InputCheck(_)
                | Read::Notes { .. }
                | Read::Proposal(_)
                | Read::TaskProposal { .. } => {
                    unreachable!("decision archive waiter")
                }
            };
            request_load(
                domain,
                waiter,
                Range::Core(crate::CoreRange::EscalationDecision { task, revision }),
                None,
                out,
            );
        }
        delivery @ (Delivery::PlaygroundNote { .. }
        | Delivery::Host(_)
        | Delivery::AssignedJob { .. }
        | Delivery::Link { .. }
        | Delivery::ForgeCall { .. }
        | Delivery::Bound { .. }
        | Delivery::Procedure { .. }
        | Delivery::Assigned { .. }
        | Delivery::Forges { .. }
        | Delivery::ForgeDocument { .. }
        | Delivery::Panel { .. }
        | Delivery::Application { .. }
        | Delivery::WebReply { .. }
        | Delivery::CarriedUnused { .. }
        | Delivery::Read { .. }
        | Delivery::ReadLoad { .. }
        | Delivery::Document { .. }
        | Delivery::WatchEnded { .. }
        | Delivery::Drained
        | Delivery::Refuse { .. }
        | Delivery::TurnBusy { .. }
        | Delivery::SourceSignedOut { .. }
        | Delivery::Result { .. }) => out.push(crate::boundary::delivery_output(delivery)),
    }
}

/// Configuration `Domain::new` asserts against (or, for `Workers`, returns),
/// refused at start before the root is made (domain/root.md, 6), each naming
/// the part that breaks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// Configured worker membership exceeds its priced count or name bound.
    Workers(fleet::WorkerProblem),
    /// The charter at this index: its run policy's bytes, `None` past `u64`,
    /// beyond `journal.run_bytes`.
    Run { charter: usize, bytes: Option<u64> },
    /// The configured `core.resume_bytes` is 0 or beyond `journal.transcript_bytes`.
    Resume { bytes: u32 },
    /// The forge's judges hold fewer projects than the authority's policies.
    JudgeProjects,
    /// The forge's judges hold fewer criteria than the authority's requirements.
    JudgeCriteria,
    /// A seeded owner's project names no escalation role from 0 to 3.
    Escalation { project: u32 },
    /// The deployment's landing rules (`None`) or a project's: beyond
    /// `journal.transcript_bytes`, the authority's requirements, or the
    /// forge's judges and gate templates.
    Landing { project: Option<u32> },
    /// A project's permission mappings, `None` past `u64`, beyond
    /// `journal.transcript_bytes`.
    Permissions { project: u32, bytes: Option<u64> },
    /// The full seed policy after joining landing requirements, `None` past
    /// accounting bounds, beyond `policy_bytes`.
    Policy { project: u32, bytes: Option<u64> },
    /// The forges' writers or git identities beyond the forge's repositories,
    /// or the namespace and writers beyond the forge client's `op_bytes`.
    Forges,
}

/// Hold the configuration to the root's limits as `Domain::new` does, and
/// return each part it would assert on; empty, the root can be made.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one pass mirrors the root's construction step by step")]
pub fn refused(mut config: Config, limits: &Limits) -> Box<[Refused]> {
    let room = u32::try_from(config.core.defaults.charters.len().saturating_add(config.core.owners.len()))
        .expect("bounded configuration")
        .saturating_add(u32::try_from(config.core.seed_projects.len()).expect("bounded projects").saturating_mul(2))
        .saturating_add(config.landing.projects.len())
        .saturating_add(config.landing.landings.len())
        .saturating_add(7);
    let mut refused = List::with_capacity(room);
    if let Err(problem) = fleet::Membership::check(
        &config.core.worker_names,
        limits.fleet.configured_workers,
        limits.fleet.worker_name_bytes,
    ) {
        refused.push(Refused::Workers(problem)).expect("counted refusals");
    }
    for (charter, configured) in config.core.defaults.charters.iter().enumerate() {
        let bytes = run_policy_bound(&configured.policy, limits);
        let fits = match bytes {
            Some(bytes) => bytes <= u64::from(limits.journal.run_bytes),
            None => false,
        };
        if !fits {
            refused.push(Refused::Run { charter, bytes }).expect("counted refusals");
        }
    }
    let resume = config.core.settings.resume_bytes;
    if resume == 0 || resume > limits.journal.transcript_bytes {
        refused.push(Refused::Resume { bytes: resume }).expect("counted refusals");
    }
    // Judges' tables smaller than the authority's refuse each landing set
    // past them, so only the tables are named.
    let judges = limits.forge.judge_projects >= limits.authority.projects
        && limits.forge.judge_criteria >= limits.authority.requirements;
    if limits.forge.judge_projects < limits.authority.projects {
        refused.push(Refused::JudgeProjects).expect("counted refusals");
    }
    if limits.forge.judge_criteria < limits.authority.requirements {
        refused.push(Refused::JudgeCriteria).expect("counted refusals");
    }
    for seed in &config.core.seed_projects {
        let project = seed.number;
        let mappings = &seed.policy.permissions;
        let bytes = u64::try_from(mappings.len())
            .expect("usize fits u64")
            .checked_mul(u64::try_from(size_of::<people::PermissionRole>()).expect("type size fits u64"));
        let fits = match bytes {
            Some(bytes) => bytes <= u64::from(limits.journal.transcript_bytes),
            None => false,
        };
        if !fits {
            refused.push(Refused::Permissions { project, bytes }).expect("counted refusals");
        }
    }
    for (index, owner) in config.core.owners.iter().enumerate() {
        if owner.project == people::DEPLOYMENT {
            continue;
        }
        let mut named = false;
        for earlier in config.core.owners.iter().take(index) {
            named |= earlier.project == owner.project;
        }
        let escalates = match seed_policy(&config.core.seed_projects, owner.project) {
            Some(policy) => match policy.escalation_role {
                Some(role) => role <= 3,
                None => false,
            },
            None => false,
        };
        if !named && !escalates {
            refused.push(Refused::Escalation { project: owner.project }).expect("counted refusals");
        }
    }
    let Some(&connector) = config.core.connectors.first() else {
        return refused.into_boxed();
    };
    let mut forge_config = core::mem::replace(
        &mut config.forge,
        skein_forge_client::Config { namespace: Box::new([]), writers: Box::new([]) },
    );
    forge_config.namespace = Box::from(config.core.deployment);
    let Ok(mut forge) = forge::Domain::new(
        &limits.forge,
        config.core.seed,
        forge_config,
        &config.identities,
        &config.forges,
        config.under_box,
    ) else {
        refused.push(Refused::Forges).expect("counted refusals");
        return refused.into_boxed();
    };
    let deployment = match policy_translate::build_landing(
        &config.landing.deployment,
        false,
        limits.authority.requirements,
        connector,
    ) {
        Some(built) => {
            landing_within(&config.landing.deployment, limits.journal.transcript_bytes)
                && add_configured_requirements(&mut config.core.defaults.rules, &built.requirements, &limits.authority)
                && forge.deployment_judges(built.criteria)
                && gates_fit(&mut forge, &config.landing.deployment, false, limits)
        }
        None => false,
    };
    if !deployment && judges {
        refused.push(Refused::Landing { project: None }).expect("counted refusals");
    }
    for (&project, rules) in &config.landing.projects {
        let joined = landing_project(
            &mut config.core.seed_projects,
            &mut forge,
            (project, rules, &config.landing.deployment),
            connector,
            limits,
        );
        if !joined && judges {
            refused.push(Refused::Landing { project: Some(project) }).expect("counted refusals");
        }
    }
    if config.landing.landings.capacity() > limits.authority.projects {
        refused.push(Refused::Landing { project: None }).expect("counted refusals");
    }
    for (&project, landing) in &config.landing.landings {
        if !crate::translate::project_landing(&mut forge, project, landing) {
            refused.push(Refused::Landing { project: Some(project) }).expect("counted refusals");
        }
    }
    let gates = match policy_translate::gate_policy(
        &config.landing,
        limits.authority.requirements,
        limits.forge.change_policy.gates,
    ) {
        Some(gates) => forge.gate_policy(gates, &limits.forge),
        None => false,
    };
    if !gates && refused.is_empty() {
        refused.push(Refused::Landing { project: None }).expect("counted refusals");
    }
    for seed in &config.core.seed_projects {
        let bytes = match jig_domain_core::policy_snapshot(
            &seed.policy.policy,
            &seed.policy.permissions,
            &seed.policy.templates,
        ) {
            Some(snapshot) => people::policy_bytes(&snapshot),
            None => None,
        };
        let fits = match bytes {
            Some(bytes) => bytes <= limits.policy_bytes,
            None => false,
        };
        if !fits {
            refused.push(Refused::Policy { project: seed.number, bytes }).expect("counted refusals");
        }
    }
    refused.into_boxed()
}

/// One project's landing rules joined to its policy, as `Domain::new` joins them.
fn landing_project(
    policies: &mut [jig_domain_core::SeedProject],
    forge: &mut forge::Domain,
    (project, rules, deployment): (u32, &[LandingRule], &[LandingRule]),
    connector: u16,
    limits: &Limits,
) -> bool {
    if !landing_within(rules, limits.journal.transcript_bytes) {
        return false;
    }
    let Some(built) = policy_translate::build_landing(rules, true, limits.authority.requirements, connector) else {
        return false;
    };
    let Some(policy) = seed_policy_mut(policies, project) else { return false };
    if !policy_translate::landing_roles(policy, deployment) || !policy_translate::landing_roles(policy, rules) {
        return false;
    }
    let Some(combined) = policy_translate::combine_requirements(
        &policy.requirements,
        &built.requirements,
        limits.authority.requirements,
    ) else {
        return false;
    };
    policy.requirements = combined;
    forge.project_judges(project, built.criteria) && gates_fit(forge, rules, true, limits)
}

/// One set of rules' gate templates within the forge's, alone.
fn gates_fit(forge: &mut forge::Domain, rules: &[LandingRule], project: bool, limits: &Limits) -> bool {
    match policy_translate::gate_templates(
        rules,
        project,
        limits.authority.requirements,
        limits.forge.change_policy.gates,
    ) {
        Some(templates) => forge
            .gate_policy(forge::GatePolicy { deployment: templates, projects: Map::with_capacity(0) }, &limits.forge),
        None => false,
    }
}

fn landing_within(rules: &[LandingRule], bytes: u32) -> bool {
    match landing::landing_rules_bytes(rules) {
        Some(held) => held <= u64::from(bytes),
        None => false,
    }
}

fn seed_policy(seeds: &[jig_domain_core::SeedProject], project: u32) -> Option<&authority::Policy> {
    for seed in seeds {
        if seed.number == project {
            return Some(&seed.policy.policy);
        }
    }
    None
}

fn seed_policy_mut(seeds: &mut [jig_domain_core::SeedProject], project: u32) -> Option<&mut authority::Policy> {
    for seed in seeds {
        if seed.number == project {
            return Some(&mut seed.policy.policy);
        }
    }
    None
}

// Augment the binary's defaults before jig constructs authority from Current.
fn add_configured_requirements(
    rules: &mut authority::Rules,
    additions: &[authority::Requirement],
    limits: &authority::Limits,
) -> bool {
    let Some(combined) = policy_translate::combine_requirements(&rules.requirements, additions, limits.requirements)
    else {
        return false;
    };
    rules.requirements = combined;
    authority::valid_rules(rules, limits)
}
