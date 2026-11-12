use skein_fake_forge_forgejo::Route;
use skein_forge::forgejo::request::{Grant, Request};
use skein_forge_setup::{
    item::Item,
    reconcile::{Stage, Why},
};
use skein_forge_setup_world::{Ended, HostWorld, fixtures};
#[test]
fn random_initial_forges_converge_after_hook_refusals_and_each_further_run_remakes_hooks() {
    let mut refused_routes = Vec::new();
    for seed in 0..32 {
        let mut world = fixtures::random(seed);
        let Hooks { url, expected } = seed_hooks(&mut world, seed);
        let refused = match seed % 3 {
            0 => Some(Route::DeleteRepoHook),
            1 => Some(Route::CreateRepoHook),
            2 => None,
            _ => unreachable!("remainder below three"),
        };
        if let Some(route) = refused
            && refused_hook_run(&mut world, route, seed)
        {
            refused_routes.push(route);
        }
        assert_eq!(world.reconcile(), Ended::Done, "seed {seed}");
        for &id in &expected {
            let hooks: Vec<_> =
                world.store().repository_hooks(u32::try_from(id).expect("small id")).filter(|h| h.url == url).collect();
            assert_eq!(hooks.len(), 1, "seed {seed}, repository {id}");
            assert_eq!(&hooks[0].secret, world.file(skein_forge_setup::item::Item::WebhookSecret).expect("secret"));
            assert!(hooks[0].active);
            assert_eq!(
                hooks[0].events.iter().map(AsRef::as_ref).collect::<Vec<&[u8]>>(),
                skein_forge::forgejo::delivery::SUBSCRIBED
            );
        }
        fixtures::assert_bot(&world);
        for org in [b"acme", b"work"] {
            fixtures::assert_org(&world, org);
        }
        for &id in world.adopted_ids() {
            assert!(world.requests().contains(&Request::RepositoryById { id }), "seed {seed}, repository {id}");
            let name = match id {
                1 => b"one".as_slice(),
                2 => b"two",
                3 => b"three",
                4 => continue,
                _ => unreachable!("scenario repositories"),
            };
            let at = world
                .requests()
                .iter()
                .find_map(|request| {
                    if let Request::Permission { at, .. } = request
                        && at.name.as_ref() == name
                    {
                        Some(at)
                    } else {
                        None
                    }
                })
                .expect("every adopted known repository has its permission read");
            let permission = world.store().collaborator(u32::try_from(id).expect("scenario id"), b"kit");
            if at.owner.as_ref() == b"kit" {
                assert_eq!(permission, None, "seed {seed}, owned repository {id}");
            } else {
                assert!(matches!(permission, Some(Grant::Write | Grant::Admin)), "seed {seed}, repository {id}");
            }
        }
        let since = world.requests().len();
        let files = fixtures::file_writes(&world);
        assert_eq!(world.reconcile(), Ended::Done, "seed {seed}, second run");
        assert_eq!(
            fixtures::write_routes(&world, since),
            expected.iter().flat_map(|_| [Route::DeleteRepoHook, Route::CreateRepoHook]).collect::<Vec<_>>(),
            "seed {seed}"
        );
        let remade: Vec<_> = world.requests()[since..]
            .iter()
            .filter_map(|request| match request {
                Request::DeleteRepoHook { at, .. } | Request::CreateRepoHook { at, .. } => Some(at.name.as_ref()),
                _ => None,
            })
            .collect();
        let names: Vec<&[u8]> = expected
            .iter()
            .flat_map(|id| {
                [match id {
                    1 => b"one".as_slice(),
                    2 => b"two",
                    3 => b"three",
                    _ => unreachable!("known ids"),
                }; 2]
            })
            .collect();
        assert_eq!(remade, names, "seed {seed}");
        assert!(world.secrets_absent(), "seed {seed}");
        assert_eq!(fixtures::file_writes(&world), files, "seed {seed}");
    }
    for route in [Route::DeleteRepoHook, Route::CreateRepoHook] {
        assert!(refused_routes.contains(&route), "a seed must exercise {route:?}'s refusal");
    }
}

fn refused_hook_run(world: &mut HostWorld, route: Route, seed: u64) -> bool {
    world.refuse(route);
    let stopped = match world.reconcile() {
        Ended::Stop(failure) => {
            assert_eq!(failure.step, Stage::Repository, "seed {seed}");
            assert_eq!(failure.why, Why::Refused(403), "seed {seed}");
            true
        }
        Ended::Done => false,
    };
    assert_eq!(world.requests().iter().any(|r| skein_fake_forge_forgejo::route(r) == route), stopped, "seed {seed}");
    assert!(world.secrets_absent(), "seed {seed}, refused run");
    world.permit();
    stopped
}

struct Hooks {
    url: Box<[u8]>,
    expected: Vec<u64>,
}
fn seed_hooks(world: &mut skein_forge_setup_world::HostWorld, seed: u64) -> Hooks {
    let mut rng = skein_lib::Rng::new(seed);
    let address =
        std::net::SocketAddr::from(([127, 0, 0, 1], 8000 + u16::try_from(rng.next_u64() % 1000).expect("small port")));
    world.listening(address);
    let url = skein_forge_setup::reconcile::hook_url(address, 1).expect("loopback");
    let ids = world.adopted_ids().to_vec();
    let mut expected = Vec::new();
    let kept = fixtures::secret(b"kept-webhook-secret");
    world.put(Item::WebhookSecret, kept.clone());
    for id in ids.iter().copied().filter(|id| *id <= 3) {
        for _ in 0..rng.next_u64() % 3 {
            let discarded = world.with_hook(
                u32::try_from(id).expect("small id"),
                b"http://discarded",
                fixtures::secret(b"discarded-secret"),
            );
            world.delete_repository_hook(u32::try_from(id).expect("small id"), discarded);
        }
        for _ in 0..rng.next_u64() % 4 {
            let target = if rng.next_u64() & 1 == 1 { url.as_ref() } else { b"http://another-service/hooks/2" };
            let secret = if rng.next_u64() & 1 == 1 { kept.clone() } else { fixtures::secret(b"stale-hook-secret") };
            world.with_hook(u32::try_from(id).expect("small id"), target, secret);
        }
        expected.push(id);
    }
    Hooks { url, expected }
}
