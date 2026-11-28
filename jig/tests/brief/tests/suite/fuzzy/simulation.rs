//! Small deterministic inventories sweep sizes, priorities and budgets.

use std::convert::Infallible;

use jig_brief_world::{LIMITS, World, referee};
use jig_domain_brief::{Core, GatherEvent, GatherRequest, Planned};
use skein_lib::{Duration, Time, Token};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};

#[test]
fn seeded_connector_sizes_and_priorities_never_exceed_the_budget_or_leak_a_token() -> Result<(), Case<Infallible>> {
    cases(this_test!("seeded_connector_sizes_and_priorities_never_exceed_the_budget_or_leak_a_token"), 0..64, |seed| {
        let mut world = World::new(LIMITS);
        let size = usize::try_from(seed % 64 + 1).expect("small size");
        let budget = u32::try_from(12 + seed % 37).expect("small budget");
        let token = Token::new(2);
        world.put(token, &vec![b'x'; size]);
        let asked = world.step(GatherEvent::Plan {
            brief: Token::new(1),
            budget,
            deadline: Time::ZERO.saturating_add(Duration::from_secs(10)),
            sections: Box::new([
                Planned::Connector {
                    connector: 3,
                    section: jig_domain_contract::parts::Section {
                        name: b"section",
                        kind: 7,
                        token,
                        size: 0,
                        limit: 80,
                        priority: u16::try_from(seed % 3).expect("small priority"),
                        required: seed % 2 == 0,
                        document: None,
                    },
                },
                Planned::Core {
                    kind: Core::Task,
                    text: b"do work".as_slice().into(),
                    limit: 32,
                    priority: 1,
                    required: true,
                },
            ]),
        });
        let done = world.settle(asked);
        assert!(referee::within_budget(&done, budget), "brief exceeds its budget");
        assert!(
            matches!(done.as_slice(), [GatherRequest::Complete { .. } | GatherRequest::Failed { .. }]),
            "expected one ending, got {done:?}"
        );
        assert!(world.closed(token), "connector token not closed");
        Ok(())
    })
}
