use crate::joined_world::Joined;
use jig_protocol_worker_link::Request;
use jig_protocol_worker_wire as w;
use jig_worker_link_world::fixtures as f;
use skein_lib::{Duration, List, Time};

fn figures(value: u64) -> w::Figures {
    let limit =
        w::Limit::new(&w::CEILINGS, w::LimitParts { name: b"hub.slots".as_slice().into(), value }).expect("limit");
    let agents = w::Agents::new(
        &w::CEILINGS,
        w::AgentsParts { profile: List::with_capacity(0), endpoints: List::with_capacity(0) },
    )
    .expect("agents");
    w::Figures::new(&w::CEILINGS, w::FiguresParts { places: 999, limits: List::from_boxed(Box::new([limit])), agents })
        .expect("figures")
}
fn figures_world(secret: &[u8]) -> Joined {
    let mut credential = f::credential();
    credential.opening = w::Opening::Figures;
    credential.secret = skein_lib::Secret::new(secret.into());
    Joined::with_credential(&f::joined_limits(), 1, credential)
}
fn received(world: &Joined) -> Vec<&w::Figures> {
    world
        .worker_root
        .iter()
        .filter_map(|r| if let Request::Figures { figures } = r { Some(figures) } else { None })
        .collect()
}
#[test]
fn a_figures_link_reports_once_closes_and_never_redials() {
    let mut world = figures_world(f::SECRET_A);
    let owner = figures(4);
    world.figures(owner.clone());
    world.run();
    let got = received(&world);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].limits(), owner.limits());
    assert_eq!(got[0].agents(), owner.agents());
    assert_eq!(got[0].places(), 4);
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Closed)));
    assert!(!world.worker_root.iter().any(|r| matches!(r, Request::Ready | Request::Lost)));
    assert!(world.engine_root.is_empty(), "no fleet channel");
    assert!(matches!(
        world.engine_openings.as_slice(),
        [jig_protocol_engine_workers::Request::FiguresTaken { worker: 0 }]
    ));
    world.advance(Duration::from_secs(60));
    assert_eq!(received(&world).len(), 1);
    assert_eq!(world.dials, [Time::ZERO]);
    world.finish();
}
#[test]
fn a_figures_link_redials_after_starting_until_the_owner_gives_figures() {
    let mut world = figures_world(f::SECRET_A);
    world.run();
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Refused { reason: 262, .. })));
    assert_eq!(received(&world).len(), 0);
    assert_eq!(world.dials, [Time::ZERO]);
    world.figures(figures(4));
    world.advance(Duration::from_millis(200));
    assert_eq!(world.dials.len(), 2);
    assert!(world.dials[1] > Time::ZERO);
    assert_eq!(received(&world).len(), 1);
    world.advance(Duration::from_secs(60));
    assert_eq!(world.dials.len(), 2);
    world.finish();
}
#[test]
fn a_figures_link_with_the_wrong_secret_is_unauthorized() {
    let mut world = figures_world(b"wrong");
    world.figures(figures(4));
    world.run();
    assert!(world.worker_root.iter().any(|r| matches!(r, Request::Refused { reason: 260, .. })));
    assert_eq!(received(&world).len(), 0);
    assert!(world.engine_root.is_empty());
    world.finish();
}
#[test]
fn a_link_reports_every_figures_at_opening_live_and_after_redial() {
    let mut world = Joined::new(&f::joined_limits(), 1);
    world.figures(figures(4));
    world.run();
    assert_eq!(world.engine_records().get(..2), Some([w::Kind::Wires.number(), w::Kind::Figures.number()].as_slice()));
    assert_eq!(received(&world).len(), 1);
    assert_eq!(world.engine_channels().len(), 1);
    world.figures(figures(8));
    world.run();
    assert_eq!(received(&world).len(), 2);
    assert_eq!(received(&world)[1].limits().get(0).expect("limit").value(), 8);
    world.cut();
    world.advance(Duration::from_millis(200));
    assert_eq!(received(&world).len(), 3);
    assert_eq!(received(&world)[2].limits().get(0).expect("limit").value(), 8);
    world.finish();
}
