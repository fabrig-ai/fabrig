//! Owner joins, bounded lag entries and the follows watch (client/pages/team.md, 1).
use super::*;
use team::follows::Owner;

const AFTER: [u8; 32] =
    [0x9c, 0x04, 0xd7, 0xe1, 0xa2, 0xb3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn row(owner: Owner, lag: u32) -> w::FollowView {
    let mut row = super::super::how::follow_row(&[], w::FollowStart::Owner, lag, None).into_parts();
    let mut setting = row.setting.into_parts();
    setting.owner = match owner {
        Owner::Task(task) => {
            w::FollowOwner::Task(w::FollowOwnerTask::new(&w::CEILINGS, w::FollowOwnerTaskParts { task }).unwrap())
        }
        Owner::Person(person) => {
            w::FollowOwner::Person(w::FollowPerson::new(&w::CEILINGS, w::FollowPersonParts { person }).unwrap())
        }
    };
    row.setting = w::Follow::new(&w::CEILINGS, setting).unwrap();
    row.source =
        if owner == Owner::Task(52) { b"acme/upstream".as_slice().into() } else { b"acme/tools".as_slice().into() };
    if lag > 0 {
        row.parity = Some(
            w::FollowParity::new(
                &w::CEILINGS,
                w::FollowParityParts {
                    source: AFTER,
                    head: [0; 32],
                    verdict: w::FollowVerdict::Passed,
                    run: b"check".as_slice().into(),
                },
            )
            .unwrap(),
        );
        row.entries = List::from_boxed(Box::new([
            w::FollowEntry::new(
                &w::CEILINGS,
                w::FollowEntryParts {
                    after: AFTER,
                    change: None,
                    state: w::FollowState::HeldByOwner(
                        w::FollowWords::new(
                            &w::CEILINGS,
                            w::FollowWordsParts { words: b"waits for the handoff fix".as_slice().into() },
                        )
                        .unwrap(),
                    ),
                },
            )
            .unwrap(),
            w::FollowEntry::new(
                &w::CEILINGS,
                w::FollowEntryParts {
                    after: [0xab; 32],
                    change: Some(812),
                    state: w::FollowState::WaitsFor(
                        w::FollowWait::new(&w::CEILINGS, w::FollowWaitParts { change: 812 }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        ]));
    }
    w::FollowView::new(&w::CEILINGS, row).unwrap()
}
fn panel(rows: &[w::FollowView]) -> w::FollowsPanel {
    w::FollowsPanel::new(
        &w::CEILINGS,
        w::FollowsPanelParts {
            project: PROJECT,
            follows: List::from_boxed(rows.into()),
            dependencies: List::with_capacity(0),
        },
    )
    .unwrap()
}
fn facts() -> Facts {
    let mut facts = team::gather(&read_people(&[YOU]), YOU, None, &[], None, Wall::EPOCH);
    facts.roles = List::from_boxed(Box::new([team::Role { name: b"Prover".as_slice().into(), ..role(52) }, role(61)]));
    facts.selected = Some(52);
    facts
}

#[test]
fn follows_join_only_listed_owners_and_the_selected_card() {
    let mut facts = facts();
    assert!(team::parts(&facts).follows.is_empty(), "unanswered watch");
    facts.follows = team::follows::gather(Some(&panel(&[
        row(Owner::Task(52), 2),
        row(Owner::Person(YOU), 0),
        row(Owner::Task(99), 0),
        row(Owner::Person(99), 0),
    ])));
    let parts = team::parts(&facts);
    assert_eq!(parts.follows.len(), 2, "unlisted owners have no row");
    let role = parts.follows.get(0).unwrap();
    assert_eq!(role.owner, Owner::Task(52));
    assert_eq!(role.line.as_ref(), b"Follows acme/upstream \xc2\xb7 2 behind \xc2\xb7 check passes at 9c04d7e1a2b3");
    let person = parts.follows.get(1).unwrap();
    assert_eq!(person.owner, Owner::Person(YOU));
    assert_eq!(person.line.as_ref(), b"Follows acme/tools \xc2\xb7 caught up \xc2\xb7 no check yet");
    let card = parts.card_follows.get(0).unwrap();
    assert_eq!(card.entries.len(), 2);
    assert_eq!(card.entries.get(0).unwrap().after, *b"9c04d7e1a2b3");
    assert_eq!(card.entries.get(0).unwrap().state.as_ref(), b"held by its owner: waits for the handoff fix");
    assert_eq!(card.entries.get(1).unwrap().after, *b"abababababab");
    assert_eq!(card.entries.get(1).unwrap().state.as_ref(), b"waits for change 812");
    assert_eq!(card.more, 0);
    facts.selected = Some(61);
    assert!(team::parts(&facts).card_follows.is_empty(), "selected role owns none");
    facts.selected = Some(999);
    assert_eq!(team::parts(&facts).card_follows.len(), 1, "unlisted selection falls back to first role");
    facts.follows = team::follows::gather(None);
    assert!(team::parts(&facts).follows.is_empty(), "refused watch");
}

#[test]
fn follows_card_caps_entries_at_eight_and_keeps_caught_up_and_stopped_lines() {
    let mut facts = facts();
    for count in [0, 8, 10] {
        let mut follow = row(Owner::Task(52), 2).into_parts();
        follow.lag = count;
        follow.entries = List::with_capacity(count);
        for n in 0..count {
            follow
                .entries
                .push(
                    w::FollowEntry::new(
                        &w::CEILINGS,
                        w::FollowEntryParts {
                            after: AFTER,
                            change: Some(u64::from(n)),
                            state: w::FollowState::NotMade,
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        facts.follows = team::follows::gather(Some(&panel(&[w::FollowView::new(&w::CEILINGS, follow).unwrap()])));
        let parts = team::parts(&facts);
        let follow = parts.card_follows.get(0).unwrap();
        assert_eq!(follow.entries.len(), count.min(8));
        assert_eq!(follow.more, count.saturating_sub(8));
    }
    let mut stopped = row(Owner::Task(52), 2).into_parts();
    stopped.stopped = Some(b"source unavailable".as_slice().into());
    facts.follows = team::follows::gather(Some(&panel(&[w::FollowView::new(&w::CEILINGS, stopped).unwrap()])));
    assert_eq!(team::parts(&facts).follows.get(0).unwrap().line.as_ref(), b"Follows acme/upstream \xc2\xb7 stopped");
}

#[test]
fn team_follows_watch_delivers_clears_and_closes_on_leave() {
    let mut f = reads_fixture();
    let mut out = Queue::uncapped();
    pages::step(
        &mut f.root.pages,
        &f.env,
        &f.root.client,
        Event::Went { address: Address::Team { project: PROJECT, role: None } },
        &mut out,
    );
    let mut follows = None;
    for request in &out {
        if let Request::Client(jig_client::Event::Read { handle, read: Readable::Jig(c::Read::Project { .. }) }) =
            request
        {
            f.read = Some(*handle);
        }
        if let Request::Client(jig_client::Event::Watch { handle, subject }) = request
            && Some(subject.clone()) == crate::forge::follows::watch(PROJECT, &f.env.limits.forge)
        {
            assert!(follows.is_none(), "one follows watch");
            follows = Some((*handle, subject.clone()));
        }
    }
    let (handle, subject) = follows.expect("Team opens follows");
    let opened = client(&mut f, jig_client::Event::Watch { handle, subject });
    let watch = opened_token(&opened);
    feed(&mut f, Feed::Project(Box::new(read_people(&[YOU]))));
    assert!(team::shown(&f.root, PROJECT).facts().unwrap().follows.is_empty());
    client(
        &mut f,
        jig_client::Event::Up {
            up: jig_client::Up::FamilyDelivered {
                watch,
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::follows_panel(&panel(&[row(Owner::Person(YOU), 0)])),
                },
            },
        },
    );
    page(&mut f, Event::Told { told: Told::Changed { handle } });
    let shown = team::shown(&f.root, PROJECT);
    assert_eq!(team::parts(shown.facts().unwrap()).follows.len(), 1);
    client(&mut f, jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::NotAllowed } });
    page(&mut f, Event::Told { told: Told::Ended { handle, why: jig_client::Why::Unknown } });
    assert!(team::shown(&f.root, PROJECT).facts().unwrap().follows.is_empty(), "refused watch clears its answer");
    page(&mut f, Event::Went { address: Address::Home { project: PROJECT } });
    assert!(f.closed.as_slice().contains(&handle), "leaving Team drops follows");
}
