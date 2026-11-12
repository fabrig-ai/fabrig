use super::*;

pub(in crate::tests) struct Fixture<'a> {
    pub roles: Option<&'a [&'a [u8]]>,
    pub areas: bool,
    pub you: bool,
}
fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), f::limits().client.domain.text).unwrap())
}
pub(in crate::tests) fn roles(names: &[&[u8]]) -> c::Tasks {
    let mut rows = List::uncapped();
    for (index, name) in names.iter().enumerate() {
        let mut row = line(40_u64.checked_add(u64::try_from(index).unwrap()).unwrap(), name, 0, false);
        row.label = 3;
        rows.push(row).unwrap();
    }
    c::Tasks { rows, more: false }
}
pub(in crate::tests) fn project() -> c::Project {
    c::Project {
        acts: List::uncapped(),
        number: PROJECT,
        name: text(b"Hello"),
        resources: List::uncapped(),
        policy: c::ProjectPolicy {
            templates: List::uncapped(),
            ceiling: c::AuthorityDocument::complete(c::Authority {
                models: None,
                tools: 0,
                grants: List::uncapped(),
                delegation: c::Delegation { kinds: List::uncapped(), tasks: 0, depth: 0 },
                spend: 0,
                deadline: None,
                notes: 0,
                note_resources: List::uncapped(),
            }),
            period_spend: 0,
            escalation_role: None,
            projections: List::uncapped(),
            roles: List::uncapped(),
            requirements: List::uncapped(),
            permissions: None,
            permissions_cut: false,
            cut: false,
        },
        members: List::from_boxed(Box::new([c::ProjectMember {
            party: PARTY,
            kind: c::PartyKind::Person,
            role: c::HeldRole::Owner,
            login: text(b"reader"),
            name: text(b"Reader"),
        }])),
        current_period: None,
        pools: List::uncapped(),
        configured_period: 0,
        configured_pools: List::uncapped(),
        configured_pools_cut: false,
        spend: List::uncapped(),
        recurring: List::uncapped(),
        cut: false,
    }
}
pub(in crate::tests) fn now(scope: c::NoteScope) -> c::NotePage {
    c::NotePage {
        rows: List::from_boxed(Box::new([c::Note {
            number: 1,
            revision: c::Revision { value: 1 },
            scope,
            description: text(b"now"),
            body: text(b"NOW LINE\nNot this line"),
            writer: c::NoteAuthor::Task { task: 40, attempt: 1 },
            tasks: List::uncapped(),
            changed: None,
            acts: List::uncapped(),
            cut: false,
        }])),
        before: None,
        after: None,
        cut: false,
    }
}
pub(in crate::tests) fn findings(fixture: &Fixture<'_>) -> w::Findings {
    let task = |number| w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).unwrap());
    let mut areas = List::with_capacity(2);
    for (name, holder, enabled) in [
        (b"tests".as_slice(), task(40), fixture.areas),
        (
            b"core",
            w::Holder::Party(w::HolderParty::new(&w::CEILINGS, w::HolderPartyParts { number: PARTY }).unwrap()),
            fixture.you,
        ),
    ] {
        if enabled {
            areas
                .push(
                    w::AreaRow::new(
                        &w::CEILINGS,
                        w::AreaRowParts {
                            name: name.into(),
                            words: name.into(),
                            owner: w::AreaOwner::Task(
                                w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 40 }).unwrap(),
                            ),
                            holder: Some(holder),
                        },
                    )
                    .unwrap(),
                )
                .unwrap();
        }
    }
    w::Findings::new(
        &w::CEILINGS,
        w::FindingsParts {
            project: PROJECT,
            rows: List::with_capacity(0),
            areas,
            triager: fixture.areas.then_some(w::AreaOwner::Task(
                w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 41 }).unwrap(),
            )),
            holder: task(41),
            cut: false,
        },
    )
    .unwrap()
}
pub(in crate::tests) fn root(fixture: Fixture<'_>) -> Domain {
    home_team(
        Some(false),
        &[],
        &[],
        true,
        None,
        Address::Home { project: PROJECT },
        LEDE.as_bytes(),
        false,
        Some(fixture),
    )
}
#[test]
fn the_team_rows_follow_the_reads_and_keep_you_last() {
    let two: &[&[u8]] = &[b"Prover", b"Triager"];
    let five: &[&[u8]] = &[b"Prover", b"Triager", b"Writer", b"Fourth", b"Fifth"];
    for (label, roles, areas, you, expected) in [
        ("unread", None, true, true, &[][..]),
        ("empty", Some(&[][..]), false, false, &[][..]),
        (
            "held",
            Some(two),
            true,
            true,
            &[
                "Prover    tests    NOW LINE",
                "Triager   triage   NOW LINE",
                "You       core     Two questions wait on you.",
            ][..],
        ),
        ("no areas", Some(two), false, false, &["Prover    NOW LINE", "Triager   NOW LINE"][..]),
        (
            "five and you",
            Some(five),
            false,
            true,
            &[
                "Prover           NOW LINE",
                "Triager          NOW LINE",
                "Writer           NOW LINE",
                "You       core   Two questions wait on you.",
            ][..],
        ),
    ] {
        let root = root(Fixture { roles, areas, you });
        let view = build(&root, &Tui::new(f::ADDRESS));
        let mut rows = List::uncapped();
        let mut heads = List::uncapped();
        for node in view.tree().nodes() {
            let words = f::text(view.tree(), node);
            if node.styles.as_slice() == [look::HEAD] {
                heads.push(words.clone()).unwrap();
            }
            if node.role == Role::Item
                && (words.starts_with(b"Prover")
                    || words.starts_with(b"Triager")
                    || words.starts_with(b"Writer")
                    || words.starts_with(b"You"))
            {
                rows.push(words).unwrap();
            }
        }
        assert_eq!(usize::try_from(rows.len()).unwrap(), expected.len(), "{label}: {rows:?}");
        for (row, expected) in rows.iter().zip(expected) {
            assert_eq!(row.as_ref(), expected.as_bytes(), "{label}");
        }
        assert_eq!(heads.as_slice().contains(&b"Team on Hello".as_slice().into()), !expected.is_empty(), "{label}");
        if !expected.is_empty() {
            let mut needs = None;
            let mut team = None;
            let mut start = None;
            for (index, head) in heads.iter().enumerate() {
                if head.as_ref() == b"Needs you" {
                    needs = Some(index);
                }
                if head.as_ref() == b"Team on Hello" {
                    team = Some(index);
                }
                if head.as_ref() == b"Start" {
                    start = Some(index);
                }
            }
            assert!(needs.unwrap() < team.unwrap() && team.unwrap() < start.unwrap(), "{heads:?}");
        }
    }
}
#[test]
fn a_team_row_is_cut_at_a_word() {
    let root = root(Fixture { roles: Some(&[b"Prover", b"Triager"]), areas: true, you: true });
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Size { columns: 40 });
    let mut part = launchpad::read(&root, &tui, &f::words(), NOW);
    part.team.get_mut(0).unwrap().line = b"A long sentence that runs beyond the room".as_slice().into();
    let view = draw(part, 40, 2);
    let mut row = None;
    for node in view.tree().nodes() {
        if node.role == Role::Item && f::text(view.tree(), node).starts_with(b"Prover") {
            row = Some(node);
        }
    }
    assert_eq!(f::text(view.tree(), row.unwrap()).as_ref(), "Prover    tests    A long sentence…".as_bytes());
}

pub(in crate::tests) fn entries() -> List<c::InboxEntry> {
    let mut entries = super::entries();
    for entry in entries.iter_mut() {
        entry.object = c::InboxObject::Question { message: 1 };
        let c::TaskLink::Stored { number, .. } = entry.task else { panic!("stored task") };
        entry.cursor = c::InboxCursor::Inbox { task: number, object: entry.object, at: entry.at };
    }
    entries
}
