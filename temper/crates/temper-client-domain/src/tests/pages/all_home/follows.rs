//! Home's follows reads and boxes (client/pages/home.md, 1, *What follows what*).
use super::*;
use all_home::Relation;

fn subject(fixture: &Across, project: u32) -> Watchable {
    forge::follows::watch(project, &fixture.limits.forge).unwrap()
}
fn opened(fixture: &Across) -> Box<[u32]> {
    let mut projects = List::uncapped();
    for (_, subject) in &fixture.opened {
        if let Watchable::Family { record, .. } = subject
            && let w::Watch::Follows(key) = wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch()
        {
            projects.push(key.project()).unwrap();
        }
    }
    projects.into_boxed()
}
#[test]
fn follows_reads_keep_the_first_four_until_the_page_leaves() {
    // The membership order, rather than project number, picks the four.
    let mut fixture = Across::new(&[FOURTH, SECOND, FIRST, THIRD, FIFTH], Address::AllHome);
    fixture.ack();
    assert_eq!(all_home::FOLLOWED, 4);
    assert_eq!(opened(&fixture).as_ref(), [FOURTH, SECOND, FIRST, THIRD]);
    let removed = fixture.watch(subject(&fixture, SECOND));
    let retained = [FOURTH, FIRST, THIRD].map(|project| fixture.watch(subject(&fixture, project)));
    let out = fixture.frame(&[THIRD, FIRST, FOURTH, FIFTH]);
    assert!(closes(&out).contains(&removed));
    assert_eq!(opened(&fixture).as_ref(), [FOURTH, SECOND, FIRST, THIRD, FIFTH]);
    for token in retained {
        assert!(!closes(&out).contains(&token));
    }
    fixture.frame(&[THIRD, FIRST, FOURTH, FIFTH]);
    assert_eq!(opened(&fixture).len(), 5, "a kept watch isn't asked twice");
    fixture.ack();
    let held = [THIRD, FIRST, FOURTH, FIFTH].map(|project| fixture.watch(subject(&fixture, project)));
    let out = fixture.go(Address::Accounts);
    for token in held {
        assert!(closes(&out).contains(&token));
    }
}
#[test]
fn project_home_needs_you_and_launchpad_hold_no_follows() {
    for (address, launchpad) in [
        (Address::Home { project: FIRST }, false),
        (Address::Inbox { filter: Filter::ALL }, false),
        (Address::Home { project: FIRST }, true),
        (Address::AllHome, true),
    ] {
        let mut fixture = Across::waiting(Address::Accounts);
        if launchpad {
            fixture.event(Event::Launchpad);
        }
        fixture.frame(&[FIRST, SECOND, THIRD, FOURTH, FIFTH]);
        fixture.go(address.clone());
        assert!(opened(&fixture).is_empty(), "{address:?}, launchpad {launchpad}");
    }
    // Entering a project's Home or Needs you also releases the open page's reads.
    for address in [Address::Home { project: FIRST }, Address::Inbox { filter: Filter::ALL }] {
        let mut fixture = Across::new(&[FIRST, SECOND], Address::AllHome);
        fixture.ack();
        let held = [FIRST, SECOND].map(|project| fixture.watch(subject(&fixture, project)));
        let out = fixture.go(address);
        for token in held {
            assert!(closes(&out).contains(&token));
        }
    }
    let mut fixture = Across::new(&[FIRST, SECOND], Address::AllHome);
    fixture.ack();
    let held = [FIRST, SECOND].map(|project| fixture.watch(subject(&fixture, project)));
    let out = fixture.event(Event::Launchpad);
    for token in held {
        assert!(closes(&out).contains(&token), "launchpad drops page-only follows");
    }
}
fn follow(stopped: Option<&[u8]>) -> w::FollowView {
    let mut row =
        super::super::how::follow_row(&[], w::FollowStart::Owner, 2, Some(w::FollowVerdict::Passed)).into_parts();
    row.source = b"acme/vsr".as_slice().into();
    row.stopped = match stopped {
        Some(words) => Some(Box::from(words)),
        None => None,
    };
    let mut parity = row.parity.unwrap().into_parts();
    let mut source = [0; 32];
    for (byte, value) in source.iter_mut().zip([0x9c, 0x04, 0xd7, 0xe1, 0xa2, 0xb3]) {
        *byte = value;
    }
    parity.source = source;
    row.parity = Some(w::FollowParity::new(&w::CEILINGS, parity).unwrap());
    w::FollowView::new(&w::CEILINGS, row).unwrap()
}
fn panel(project: u32, follows: &[w::FollowView], dependencies: &[u32]) -> w::FollowsPanel {
    let mut rows = List::with_capacity(w::CEILINGS.follows_panel_dependencies);
    for (index, project) in dependencies.iter().enumerate() {
        rows.push(
            w::DependencyView::new(
                &w::CEILINGS,
                w::DependencyViewParts {
                    number: u32::try_from(index.checked_add(1).unwrap()).unwrap(),
                    project: *project,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    w::FollowsPanel::new(
        &w::CEILINGS,
        w::FollowsPanelParts { project, follows: List::from_boxed(follows.into()), dependencies: rows },
    )
    .unwrap()
}
fn me() -> c::Me {
    let mut me = f::chats::me();
    let template = me.projects.get(0).unwrap().clone();
    me.projects = List::uncapped();
    for (project, name) in [(FIRST, b"vsr".as_slice()), (SECOND, b"vsr-rs"), (THIRD, b"cowsql"), (FOURTH, b"fabrig")] {
        let mut row = template.clone();
        row.project = project;
        row.name = c::DocumentText::complete(c::Text::new(name.into(), 64).unwrap());
        me.projects.push(row).unwrap();
    }
    me
}
fn expected_follow(stopped: Option<&[u8]>) -> Relation {
    Relation::Follow {
        project: SECOND,
        name: b"vsr-rs".as_slice().into(),
        source: b"acme/vsr".as_slice().into(),
        line: "2 behind · check passes at 9c04d7e1a2b3".as_bytes().into(),
        stopped: match stopped {
            Some(words) => Some(Box::from(words)),
            None => None,
        },
    }
}
fn expected_dependency(project: u32, name: &[u8], dependency: &[u8]) -> Relation {
    Relation::Dependency { project, name: name.into(), dependency: dependency.into() }
}
#[test]
fn relation_boxes_keep_project_and_document_order_and_words() {
    let me = me();
    // Reversed delivery order must not reorder the boxes.
    assert_eq!(
        all_home::relations(
            &[panel(FOURTH, &[], &[SECOND]), panel(SECOND, &[follow(None)], &[])],
            me.projects.as_slice()
        )
        .as_ref(),
        [expected_follow(None), expected_dependency(FOURTH, b"fabrig", b"vsr-rs"),]
    );
    assert_eq!(
        all_home::relations(
            &[panel(FOURTH, &[], &[SECOND]), panel(SECOND, &[follow(Some(b"Check is missing"))], &[7, FIRST]),],
            me.projects.as_slice()
        )
        .as_ref(),
        [
            expected_follow(Some(b"Check is missing")),
            expected_dependency(SECOND, b"vsr-rs", b"project 7"),
            expected_dependency(SECOND, b"vsr-rs", b"vsr"),
            expected_dependency(FOURTH, b"fabrig", b"vsr-rs"),
        ]
    );
    assert_eq!(
        all_home::relations(&[panel(FOURTH, &[], &[SECOND])], me.projects.as_slice()).as_ref(),
        [expected_dependency(FOURTH, b"fabrig", b"vsr-rs"),],
        "an unanswered project's boxes are absent, not its neighbours'"
    );
    let mut second = follow(None).into_parts();
    second.number = 2;
    second.source = b"aaa/second".as_slice().into();
    let mut expected_second = expected_follow(None);
    if let Relation::Follow { source, .. } = &mut expected_second {
        *source = b"aaa/second".as_slice().into();
    }
    assert_eq!(
        all_home::relations(
            &[panel(SECOND, &[follow(None), w::FollowView::new(&w::CEILINGS, second).unwrap()], &[FIRST])],
            me.projects.as_slice()
        )
        .as_ref(),
        [expected_follow(None), expected_second, expected_dependency(SECOND, b"vsr-rs", b"vsr"),],
        "a document's follows keep their order before its dependencies"
    );
    let mut reversed = me.projects.into_boxed();
    reversed.reverse();
    assert_eq!(
        all_home::relations(&[panel(SECOND, &[follow(None)], &[]), panel(FOURTH, &[], &[SECOND])], &reversed).as_ref(),
        [expected_dependency(FOURTH, b"fabrig", b"vsr-rs"), expected_follow(None),]
    );
}
#[test]
fn unanswered_refused_and_ended_panels_leave_the_other_boxes_standing() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::Closed] {
        let mut fixture = Across::new(&[FIRST, SECOND, THIRD, FOURTH], Address::AllHome);
        fixture.ack();
        let failed = fixture.watch(subject(&fixture, SECOND));
        let other = fixture.watch(subject(&fixture, FOURTH));
        fixture.family(other, wire::follows_panel(&panel(FOURTH, &[], &[SECOND])));
        let expected = expected_dependency(FOURTH, &name(FOURTH), &name(SECOND));
        assert_eq!(all_home::facts(&fixture.root, Wall::EPOCH).follows.as_ref(), [expected]);
        fixture.family(failed, wire::follows_panel(&panel(SECOND, &[follow(None)], &[])));
        assert_eq!(all_home::facts(&fixture.root, Wall::EPOCH).follows.len(), 2);
        fixture.up(Up::WatchEnded { watch: failed, end });
        assert_eq!(
            all_home::facts(&fixture.root, Wall::EPOCH).follows.as_ref(),
            [expected_dependency(FOURTH, &name(FOURTH), &name(SECOND)),]
        );
    }
}
