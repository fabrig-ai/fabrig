mod tests {
    use jig_core_kit::testing::{fork_cut, oracle_closed};
    use jig_core_kit::world::{Cut, World};

    #[test]
    fn shared_setup_children_keep_the_parent_and_report_case_failures() {
        let mut world = World::new(super::super::Setup { seed: 190, ..super::super::Setup::default() });
        world.turn(1, 1, None, b"durable answer");
        world.end(jig_domain_tasks::End::Parked, 1);
        let rows = world.store.rows.clone();
        let commits = world.store.applied;
        for name in ["first restart", "second restart"] {
            world
                .case(name, |child| {
                    child.restart(190, false);
                    assert!(child.domain.ready());
                    child.say(71);
                    assert!(child.store.applied > commits);
                })
                .expect("case passed");
        }
        assert_eq!(world.store.rows, rows, "children leave the held setup unchanged");
        assert_eq!(world.store.applied, commits);
        assert_eq!(
            world.case("failed route", |_| panic!("case check")),
            Err("forked case failed route failed: Code(101)".into())
        );
        assert_eq!(
            world.case("unread transcript", |child| child.store.unread_transcript(1)),
            Err("forked case unread transcript failed: Code(101)".into()),
            "a child cannot pass before its closing oracle"
        );
        assert!(!oracle_closed(&world), "the parent's closing oracle still waits");
    }

    #[test]
    fn a_child_failure_names_its_cut() {
        let mut world = World::new(super::super::Setup::default());
        let cut = Cut::Submitted(world.store.applied + 1);
        world.sweep(vec![cut]);
        let result = fork_cut(&mut world, cut);
        if result == Ok(true) {
            std::process::exit(23);
        }
        assert_eq!(result, Err(format!("forked cut {cut:?} failed: Code(23)")));
    }
}
