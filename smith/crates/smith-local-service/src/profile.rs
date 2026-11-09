//! The standard host profile keeps the spawned-agent and delivery bounds.
//! It never knows a chat or an agent's charter. `host_limits` grants a window
//! large enough for the agent's largest turn. Contract: shell.md, section 2.1;
//! protocol/limits.md, sections 3.6 and 3.8.

mod effective;

pub use effective::effective;
pub use smith_agent_profile::host_limits;

#[cfg(test)]
mod tests {
    use super::host_limits;
    use skein_lib::Duration;
    use smith_agent_service::profile::{Check, Layer, LimitName, Refusal, Value, standard};

    #[test]
    fn the_host_exit_grace_is_derived_from_its_agents_teardown_and_one_close() {
        let profile = standard();
        let mut agent = smith_agent_process_world::limits();
        agent.events = Some(
            smith_agent_service::event_limits(
                &agent,
                smith_protocol_events::sinks::Capture::None,
                1,
                0,
                &[],
                profile.policy.write_deadline,
            )
            .expect("bounded event stream"),
        );
        let host = host_limits(&profile, &agent).expect("derived grace");
        assert_eq!(host.teardown, Duration::from_secs(40));
        assert_eq!(host.exit_grace, Duration::from_secs(45));
        assert_eq!(host.cancel_grace, profile.policy.cancel_grace);
    }

    #[test]
    fn a_policy_exit_grace_below_the_agents_bound_names_grace_over_teardown() {
        let mut profile = standard();
        profile.policy.exit_grace = Duration::from_secs(9);
        let agent = smith_agent_process_world::limits();
        assert_eq!(
            host_limits(&profile, &agent),
            Err(Refusal::Broken {
                check: Check::GraceOverTeardown,
                left: LimitName::deployment(Layer::Policy, "exit_grace").value(Value::Duration(Duration::from_secs(9))),
                right: LimitName::deployment(Layer::Service, "teardown")
                    .value(Value::Duration(Duration::from_secs(10))),
            }),
        );
    }

    #[test]
    fn host_payload_bounds_follow_the_agents_admitted_ownership() {
        let profile = standard();
        let mut agent = smith_agent_process_world::limits();
        for payload in [256, 4096, 40_960] {
            agent.domain.run.host_input_bytes = payload;
            agent.domain.run.host_reply_bytes = payload;
            agent.domain.run.outcome_bytes = u64::from(payload);
            let host = host_limits(&profile, &agent).expect("derived host bounds");
            assert_eq!(host.charter_bytes, agent.domain.run.run_bytes);
            assert_eq!(host.outcome_bytes, u64::from(payload));
            assert_eq!(host.call_bytes, u64::from(payload));
            assert!(host.answer_bytes >= u64::from(payload));
            assert!(host.answer_bytes >= smith_host_domain::Delivered::owned_bound());
            assert_eq!(host.fact_bytes, 17, "one kind and two u64 fields");
            assert_eq!(host.unacknowledged_bytes, host.turn_bytes);
            assert_eq!(host.transcript_bytes, host.turn_bytes);
        }
    }

    #[test]
    fn an_unrepresentable_host_window_names_its_derived_limit() {
        let mut agent = smith_agent_process_world::limits();
        agent.domain.session.history_bytes = u64::MAX;
        assert_eq!(
            host_limits(&standard(), &agent),
            Err(Refusal::Derived { limit: LimitName::deployment(Layer::Service, "turn_bytes") }),
        );
    }
}
