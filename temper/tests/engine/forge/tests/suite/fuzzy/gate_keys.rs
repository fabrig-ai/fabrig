//! Rebasing equal filtered candidates on different generated bases must not
//! invalidate a non-base gate (domain/landing.md, 6.1).
use skein_lib::Rng;
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as landing;
use temper_engine_forge_world::{REPO, landings};

#[test]
fn non_base_gate_keys_ignore_generated_bases_and_ignored_paths() {
    for seed in 0..32 {
        let mut rng = Rng::new(seed);
        let ignored = format!("docs/{}.md", rng.below(100));
        let visible = format!("lib-{}.rs", rng.below(100));
        let content = rng.next_u64().to_be_bytes();
        let mut keys = Vec::new();
        for base in [b"first base".as_slice(), b"different base".as_slice()] {
            let mut world = landings::turn_world();
            assert!(world.project_landing(
                landings::PROJECT,
                change::Way::Turn,
                Box::new([top::RepositoryGates {
                    repository: REPO,
                    gates: Box::new([landing::Gate {
                        name: b"gate".as_slice().into(),
                        workflow: b"gate.yml".as_slice().into(),
                        ignored: Box::new([ignored.as_bytes().into()]),
                        blocking: true,
                        base: false,
                    }]),
                    tiers: Box::new([]),
                }])
            ));
            world.push_file(b"main", b"file", base);
            world.push_file(b"main", ignored.as_bytes(), base);
            world.push_file(b"main", visible.as_bytes(), &content);
            if base == b"different base" {
                // Different ancestry as well as different ignored content.
                world.push_file(b"main", ignored.as_bytes(), b"one more ignored change");
            }
            world.produce_on(b"temper/51", b"main");
            landings::ready_change(&mut world, landings::CHANGE, b"temper/51");
            world.event(top::Event::Landing { row: landings::landing_row(world.change(landings::CHANGE)) });
            landings::drive_landing(&mut world, landings::LANDING);
            let landing::Proof::Passed { key } = world.landing(landings::LANDING).landing.members[0].gates[0].proof
            else {
                panic!("seed {seed}: gate passed");
            };
            keys.push(key);
        }
        assert_eq!(keys[0], keys[1], "seed {seed}: a non-base key depends only on its filtered candidate");
    }
}
