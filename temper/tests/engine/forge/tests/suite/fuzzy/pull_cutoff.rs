use skein_fake_forge_domain::api as raw;
use skein_forge::api;
use skein_lib::{Duration, Rng, Token};
use temper_engine_domain_forge as top;
use temper_engine_domain_forge_change as change;
use temper_engine_forge_world::{REPO, World, changes, translate};

#[test]
fn seeded_pull_reads_size_once_within_the_cutoff_and_bytes() {
    let mut observed = 0;
    let mut cut = 0;
    let mut oversized = 0;
    for seed in 0..128 {
        let mut rng = Rng::new(seed);
        let mut world = World::with_comparison_room(seed);
        world.adopt();
        world.produce_on(b"temper/51", b"main");
        let tree = (0..20)
            .map(|file| raw::File {
                path: format!("file-{file}").into_bytes().into_boxed_slice(),
                content: b"small change".as_slice().into(),
            })
            .collect();
        world.push_tree(b"temper/51", tree);
        let pull = world.open_pull(b"temper/51", b"main");
        let head = translate::commit(world.branch(b"temper/51"));
        let mut row = changes::change_row();
        row.pull = Some(pull);
        row.change.last_head = Some(head);
        row.change.state = change::State::Gating { head, asked: Box::new([]) };
        world.event(top::Event::Change { row });
        // Draw both the refused page and whether this pull has a refusal.
        let page = u32::try_from(rng.below(3)).expect("three pages") + 1;
        if rng.below(2) == 0 {
            world.reject_file_page(page, api::Excess::at_least(api::Bound::AnswerBytes, 4096, 4097));
        }
        world.call_latencies(Duration::ZERO, Duration::from_secs(40));
        world.take_seen();
        world.take_calls();
        let section = Token::new(930);
        world.event(top::Event::PlanBrief {
            section,
            partner: None,
            source: top::BriefSource::Pull {
                item: top::BriefItem { repository: REPO.repository, number: pull },
                head: top::BriefCommit(head),
            },
        });
        let start = world.now();
        let cutoff = start.saturating_add(Duration::from_millis(22_500));
        world.event(top::Event::GatherPlanned {
            task: 9,
            section,
            parts: 1,
            bytes: 2048,
            ci_budget: 2048,
            deadline: start.saturating_add(Duration::from_secs(30)),
        });
        let mut pulled_before_cutoff = false;
        for _ in 0..20_000 {
            let calls = world.take_calls();
            if calls.iter().any(|op| matches!(op, api::Op::Read(api::Read::PullFiles { .. }))) && world.now() < cutoff {
                pulled_before_cutoff = true;
            }
            if world.seen().iter().any(|request| matches!(request, top::Request::BriefSized { .. })) {
                break;
            }
            assert!(world.step_until(cutoff.saturating_add(Duration::from_secs(100))), "seed {seed}: no size");
        }
        if world.seen().iter().any(|request| {
            matches!(
                request,
                top::Request::BriefReadRefused { stage: "files", error: Some(api::Error::TooLarge(_)), .. }
            )
        }) {
            oversized += 1;
        }
        if pulled_before_cutoff {
            observed += 1;
            assert!(world.now() <= cutoff, "seed {seed}: sized {:?}, cutoff {cutoff:?}", world.now());
            if world.seen().iter().any(|request| {
                matches!(request, top::Request::BriefReadRefused { reason: "not read by the section's cutoff", .. })
            }) {
                cut += 1;
            }
            world.run_for(120);
            let sizes: Vec<_> = world
                .seen()
                .iter()
                .filter_map(|request| match request {
                    top::Request::BriefSized { section: found, size } if *found == section => Some(size),
                    _ => None,
                })
                .collect();
            assert_eq!(sizes.len(), 1, "seed {seed}: exactly once");
            let size = *sizes[0].as_ref().unwrap_or_else(|error| panic!("seed {seed}: {error:?}"));
            assert!(size <= 2048, "seed {seed}: {size}");
            world.event(top::Event::TakeBrief { section });
            assert!(
                world.seen().iter().any(|request| matches!(request,
                    top::Request::BriefTaken { section: found, bytes: Some(bytes) }
                        if *found == section && bytes.len() == usize::try_from(size).expect("bounded size")
                )),
                "seed {seed}: sized bytes retained"
            );
        }
    }
    assert!(observed > 0, "the seeds must exercise identity reads before the cutoff");
    assert!(cut > 0, "the seeds must leave files or comparison at the cutoff");
    assert!(oversized > 0, "the seeds must exercise oversized files pages");
}
