//! The root immediately answers the project release placeholder (contracts J5).
use jig_domain_core as core;

#[test]
fn a_release_ask_is_answered_at_once() {
    for known in [true, false] {
        let (mut domain, env) = super::resume_fixture::root();
        let connector = if known { domain.config.forge_connector } else { u16::MAX };
        if !known {
            let _stopping = core::step(&mut domain.core, &crate::limits::environment_core(&env), core::Event::Stopping);
            assert!(domain.core.stopping());
        }
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        crate::route::asks::ask(&mut domain, &env, &mut decision, connector, core::Ask::ReleaseProject { project: 1 });
        let Some(crate::Work::Core(event)) = domain.work.pop() else {
            panic!("release answered within the decision");
        };
        let core::Event::ProjectReleased { project, connector: answered } = event else {
            panic!("release terminal");
        };
        assert_eq!((project, answered), (1, connector));
        assert!(domain.work.is_empty());
        let event = core::Event::ProjectReleased { project, connector: answered };
        assert!(!event.new_work());
        let core::Requests::Out(mut requests) =
            core::step(&mut domain.core, &crate::limits::environment_core(&env), event);
        let Some(core::Request::Decided) = requests.pop() else {
            panic!("release acknowledgement decides nothing");
        };
        assert!(requests.is_empty());
    }
}
