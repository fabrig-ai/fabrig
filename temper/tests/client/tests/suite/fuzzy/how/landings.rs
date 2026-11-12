//! Random landing documents and the shared drawing (client/pages/settings.md, 2).
use jig_client::{Down, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Rng, Time, Wall};
use skein_ui::Role;
use temper_client_domain::{Address, Domain, Event, Place, Request, pages::how};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_world::fixtures;

pub(super) struct Drawing {
    root: Domain,
    view: temper_client_view::View,
}
fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    temper_client_domain::step(
        root,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: fixtures::domain_limits() },
        event,
        &mut out,
    );
    let replies: Vec<_> = out
        .iter()
        .filter_map(|request| match request {
            Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } => {
                Some(Up::WatchEnded { watch: *watch, end: c::WatchEnd::Closed })
            }
            Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Read { read, .. } } } => {
                Some(Up::ReadDone { read: *read, result: jig_client::Readout::Limit })
            }
            _ => None,
        })
        .collect();
    for reply in replies {
        up(root, reply);
    }
    out
}
fn up(root: &mut Domain, up: Up) {
    event(root, Event::Client { event: jig_client::Event::Up { up } });
}
impl Drawing {
    pub(super) fn new() -> Self {
        let (_, root) = temper_client_world::front::account_acts::fixture(fixtures::SEED);
        let temper_client_world::app::Root::Person(root) = root else { panic!("person shell") };
        let limits = fixtures::domain_limits();
        let words = fixtures::phrasebooks(&limits);
        Self {
            root: *root,
            view: temper_client_view::View::new(
                &fixtures::view_bounds(&limits, &words),
                &limits,
                words,
                fixtures::FORGE,
            ),
        }
    }
    #[expect(clippy::too_many_lines, reason = "one seed drives watches, random documents and the shared drawing")]
    pub(super) fn check(&mut self, seed: u64, limits: &w::Limits) {
        let mut rng = Rng::new(seed);
        event(&mut self.root, Event::Went { place: Place::Address(Address::Repositories { project: 1 }) });
        let out = event(&mut self.root, Event::Went { place: Place::Address(Address::Settings { project: 1 }) });
        let (mut follows_watch, mut ways_watch, mut roles_watch) = (None, None, None);
        for request in &out {
            if let Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject } },
            } = request
            {
                match subject {
                    Watchable::Family { record, .. } => {
                        match wire::decode_watching(&record.bytes, limits).expect("watch").watch() {
                            w::Watch::Follows(_) => follows_watch = Some(*watch),
                            w::Watch::Ways(_) => ways_watch = Some(*watch),
                            _ => {}
                        }
                    }
                    Watchable::Jig(c::Subject::Tasks { query, .. }) if query.label == Some(3) => {
                        roles_watch = Some(*watch);
                    }
                    Watchable::Jig(_) => {}
                }
            }
        }
        let mut follows: Vec<_> = (0..rng.below(5))
            .map(|number| {
                let paths: Vec<_> = (0..rng.below(4)).map(|_| b"src".as_slice()).collect();
                let mut row = follow_row(
                    &paths,
                    if rng.chance(500) { w::FollowStart::Owner } else { w::FollowStart::Code },
                    u32::try_from(rng.below(10)).expect("lag"),
                    None,
                )
                .into_parts();
                row.number = u32::try_from(number).expect("number");
                row.source = format!("source-{number}").into_bytes().into_boxed_slice();
                w::FollowView::new(limits, row).expect("random follow")
            })
            .collect();
        if follows.len() > 1 && rng.chance(500) {
            follows.reverse();
        }
        let panel = rng.chance(700).then(|| {
            w::FollowsPanel::new(
                limits,
                w::FollowsPanelParts {
                    project: 1,
                    follows: List::from_boxed(follows.into_boxed_slice()),
                    dependencies: List::with_capacity(0),
                },
            )
            .expect("panel")
        });
        let route = rng.chance(700).then(|| super::path(&mut rng, limits)).filter(|path| !path.stops().is_empty());
        let members = rng.chance(500).then(|| -> Box<[c::ProjectMember]> {
            Box::new([c::ProjectMember {
                party: 4,
                kind: c::PartyKind::Person,
                role: c::HeldRole::Member,
                login: text(b"pat"),
                name: text(if rng.chance(500) { b"Pat" } else { b"" }),
            }])
        });
        let expected = how::follows(panel.as_ref(), route.as_ref(), members.as_deref(), rng.below(5), &[]);
        let sources: Vec<_> =
            panel.as_ref().map_or_else(Vec::new, |panel| panel.follows().iter().map(w::FollowView::source).collect());
        assert_eq!(
            expected.follows.iter().map(|row| row.source.as_ref()).collect::<Vec<_>>(),
            sources,
            "seed {seed}: follow order"
        );
        let tiers = route.as_ref().is_some_and(|path| {
            path.stops()
                .iter()
                .any(|stop| *stop.step() == w::PathStep::Verify && matches!(stop.opens(), w::Opens::Tier(_)))
        });
        assert_eq!(expected.tiers.is_some(), tiers, "seed {seed}: verify tiers only");
        for (watch, bytes) in [
            (follows_watch.expect("follows watch"), panel.as_ref().map(wire::follows_panel)),
            (
                ways_watch.expect("ways watch"),
                route.map(|path| {
                    wire::ways_panel(
                        &w::WaysPanel::new(
                            limits,
                            w::WaysPanelParts {
                                project: 1,
                                words: None,
                                ways: List::with_capacity(0),
                                current: None,
                                own: path,
                                rules: w::PathRules::new(
                                    limits,
                                    w::PathRulesParts { rows: List::with_capacity(0), note: Box::default() },
                                )
                                .expect("empty rules"),
                                may_change: false,
                            },
                        )
                        .expect("own path"),
                    )
                }),
            ),
        ] {
            if let Some(bytes) = bytes {
                up(
                    &mut self.root,
                    Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                    },
                );
            } else {
                up(&mut self.root, Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded });
            }
        }
        let link =
            jig_client_protocol_link::Limits::new(fixtures::domain_limits().client.domain, 8).expect("task limits");
        let task = jig_client_protocol_link::translate::documents::task(jig_client_test_records::task_record(), &link)
            .expect("role prototype");
        let mut rows = Vec::new();
        for number in 0..rng.below(4) {
            let mut chip = task.chip.clone();
            chip.task = number;
            chip.title = text(format!("Role {number}").as_bytes());
            rows.push(c::TaskLine {
                chip,
                project: 1,
                requester: c::Requester::Person(1),
                tracked: None,
                label: 3,
                flags: task.flags,
                changed: task.changed,
                last_activity: task.changed,
                ended_at: rng.chance(500).then_some(task.changed),
                budget: task.budget.clone(),
                waits_on: c::WaitsOn::Nothing,
                phases: c::PhaseCounts::ZERO,
                delegates_done: 0,
                delegates_total: 0,
                held_below: 0,
                conversation: None,
            });
        }
        let live = rows.iter().filter(|row| row.ended_at.is_none()).count();
        up(
            &mut self.root,
            Up::Watched {
                watch: roles_watch.expect("roles watch"),
                event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                    rows: List::from_boxed(rows.into_boxed_slice()),
                    more: false,
                }))),
            },
        );
        temper_client_view::build(&mut self.view, &self.root, Wall::EPOCH, &mut Queue::uncapped());
        let nodes = self.view.tree().nodes();
        let titles: Vec<_> = nodes
            .iter()
            .filter(|node| {
                node.role == Role::Heading { level: 3 }
                    && node.name.as_deref().is_some_and(|name| name.starts_with(b"cowsql/vsr-rs follows source-"))
            })
            .map(|node| node.name.as_deref().expect("title"))
            .collect();
        let wanted: Vec<_> =
            sources.iter().map(|source| [b"cowsql/vsr-rs follows ".as_slice(), source].concat()).collect();
        assert_eq!(titles, wanted, "seed {seed}: view keeps follows once, in order");
        assert_eq!(
            nodes
                .iter()
                .any(|node| node.name.as_deref() == Some(b"Tiers on main") && node.role == Role::Heading { level: 3 }),
            tiers
        );
        assert_eq!(
            nodes.iter().any(|node| node.name.as_deref() == Some(b"What follows a landing")
                && node.role == Role::Heading { level: 2 }),
            !sources.is_empty() || tiers
        );
        let parts = how::parts(&how::facts(&self.root, 1));
        assert_eq!(parts.standing.len(), live);
        assert_eq!(
            nodes
                .iter()
                .any(|node| node.name.as_deref() == Some(b"Budgets") && node.role == Role::Heading { level: 2 }),
            live > 0 || parts.budget.is_some()
        );
    }
}
fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), 64).expect("short words"))
}

fn follow_row(paths: &[&[u8]], start: w::FollowStart, lag: u32, verdict: Option<w::FollowVerdict>) -> w::FollowView {
    let repo = w::FollowRepository::new(&w::CEILINGS, w::FollowRepositoryParts { forge: 1, repository: 7 })
        .expect("fuzzy follow");
    let setting = w::Follow::new(
        &w::CEILINGS,
        w::FollowParts {
            source: w::FollowSource::new(
                &w::CEILINGS,
                w::FollowSourceParts {
                    repository: repo.clone(),
                    branch: b"main".as_slice().into(),
                    paths: List::from_boxed(paths.iter().map(|p| Box::from(*p)).collect()),
                },
            )
            .expect("fuzzy follow"),
            target: w::FollowTarget::new(
                &w::CEILINGS,
                w::FollowTargetParts { repository: repo, branch: b"main".as_slice().into() },
            )
            .expect("fuzzy follow"),
            owner: w::FollowOwner::Person(
                w::FollowPerson::new(&w::CEILINGS, w::FollowPersonParts { person: 4 }).expect("fuzzy follow"),
            ),
            start,
            producer: None,
            verification: w::FollowCheck::new(
                &w::CEILINGS,
                w::FollowCheckParts {
                    workflow: b".forgejo/workflows/vsr-tests.yml".as_slice().into(),
                    artifact: b"results".as_slice().into(),
                    format: w::FollowFormat::Junit,
                },
            )
            .expect("fuzzy follow"),
        },
    )
    .expect("fuzzy follow");
    let parity = verdict.map(|verdict| {
        w::FollowParity::new(
            &w::CEILINGS,
            w::FollowParityParts { source: [0xab; 32], head: [0xcd; 32], verdict, run: b"check".as_slice().into() },
        )
        .expect("fuzzy follow")
    });
    w::FollowView::new(
        &w::CEILINGS,
        w::FollowViewParts {
            number: 1,
            setting,
            source: b"cowsql/vsr".as_slice().into(),
            source_branch: b"main".as_slice().into(),
            target: b"cowsql/vsr-rs".as_slice().into(),
            target_branch: b"main".as_slice().into(),
            task: None,
            stopped: None,
            lag,
            entries: List::with_capacity(0),
            skips: List::with_capacity(0),
            parity,
        },
    )
    .expect("fuzzy follow")
}
