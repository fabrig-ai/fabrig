//! The link follows the sole owner and the forge's account read (temper/docs/design/domain/forge/adoption.md, 6).
use super::*;

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the link owner and forge decisions")]
fn a_link_follows_the_sole_owner_and_what_the_forge_says() {
    const P: u64 = 31;
    const Q: u64 = 32;
    const W: u64 = 17;
    const SEVEN: Option<Link> = Some(Link { user: 7, person: P });
    enum Owner {
        NotHanded,
        Handed(Option<u64>),
    }
    struct Row {
        name: &'static str,
        before: Option<Link>,
        under_box: bool,
        kind: Kind,
        mode: Mode,
        state: Option<ForgeState>,
        owner: Owner,
        answer: Option<PersonAnswer>,
        after: Option<Link>,
        saved: bool,
        notes: &'static [LinkNote],
    }
    const MAKE: Row = Row {
        name: "make",
        before: None,
        under_box: true,
        kind: Kind::Forgejo,
        mode: Mode::Owned,
        state: Some(ForgeState::Kept),
        owner: Owner::Handed(Some(P)),
        answer: Some(PersonAnswer::Is(7)),
        after: SEVEN,
        saved: true,
        notes: &[],
    };
    for row in [
        MAKE,
        Row { name: "keep", before: SEVEN, saved: false, ..MAKE },
        Row {
            name: "replace",
            before: SEVEN,
            answer: Some(PersonAnswer::Is(9)),
            after: Some(Link { user: 9, person: P }),
            notes: &[LinkNote::Replaced],
            ..MAKE
        },
        Row {
            name: "gone",
            before: SEVEN,
            answer: Some(PersonAnswer::Gone),
            saved: false,
            notes: &[LinkNote::Gone],
            ..MAKE
        },
        Row {
            name: "gone without link",
            answer: Some(PersonAnswer::Gone),
            after: None,
            saved: false,
            notes: &[LinkNote::Gone],
            ..MAKE
        },
        Row {
            name: "not theirs",
            kind: Kind::GitHub,
            before: SEVEN,
            answer: Some(PersonAnswer::NotTheirs),
            after: None,
            notes: &[LinkNote::Cleared],
            ..MAKE
        },
        Row {
            name: "not theirs without link",
            kind: Kind::GitHub,
            answer: Some(PersonAnswer::NotTheirs),
            after: None,
            saved: false,
            ..MAKE
        },
        Row { name: "GitHub", kind: Kind::GitHub, ..MAKE },
        Row { name: "managed GitHub", kind: Kind::GitHub, mode: Mode::Managed, ..MAKE },
        Row { name: "connected GitHub", kind: Kind::GitHub, mode: Mode::Connected, ..MAKE },
        Row { name: "fake GitHub", kind: Kind::GitHub, mode: Mode::Fake, ..MAKE },
        Row {
            name: "writer",
            before: SEVEN,
            answer: Some(PersonAnswer::Is(W)),
            saved: false,
            notes: &[LinkNote::Writer],
            ..MAKE
        },
        Row {
            name: "writer without link",
            answer: Some(PersonAnswer::Is(W)),
            after: None,
            saved: false,
            notes: &[LinkNote::Writer],
            ..MAKE
        },
        Row { name: "without box", under_box: false, after: None, saved: false, ..MAKE },
        Row {
            name: "restored without box",
            before: SEVEN,
            under_box: false,
            owner: Owner::Handed(Some(P)),
            answer: None,
            after: None,
            saved: true,
            notes: &[LinkNote::Cleared],
            ..MAKE
        },
        Row { name: "owner not handed", owner: Owner::NotHanded, after: None, saved: false, ..MAKE },
        Row { name: "no sole owner", owner: Owner::Handed(None), after: None, saved: false, ..MAKE },
        Row { name: "managed Forgejo", mode: Mode::Managed, after: None, saved: false, ..MAKE },
        Row { name: "fake Forgejo", mode: Mode::Fake, after: None, saved: false, ..MAKE },
        Row { name: "connected Forgejo", mode: Mode::Connected, after: None, saved: false, ..MAKE },
        Row { name: "unknown forge", state: None, after: None, saved: false, ..MAKE },
        Row {
            name: "ended forge",
            state: Some(ForgeState::Ended { secrets_erased: false, box_removed: false }),
            before: SEVEN,
            saved: false,
            ..MAKE
        },
        Row {
            name: "different owner",
            before: SEVEN,
            owner: Owner::Handed(Some(Q)),
            answer: None,
            after: None,
            notes: &[LinkNote::Cleared],
            ..MAKE
        },
        Row {
            name: "owner removed",
            before: SEVEN,
            owner: Owner::Handed(None),
            answer: None,
            after: None,
            notes: &[LinkNote::Cleared],
            ..MAKE
        },
        Row { name: "same owner", before: SEVEN, answer: None, saved: false, ..MAKE },
    ] {
        let mut d = setup_domain(&[], row.under_box);
        assert_eq!(d.owner, None, "owner is volatile");
        d.writers.insert(1, W).expect("fixture writer");
        let mut record = records::record(1);
        record.kind = row.kind;
        record.mode = row.mode;
        record.person = row.before;
        if let Some(state) = row.state {
            record.state = state;
            assert!(outputs(&mut d, Event::Restore { record: Stored::Forge(record.clone()) }).is_empty());
        }
        let mut sent = Queue::uncapped();
        match row.owner {
            Owner::NotHanded => {}
            Owner::Handed(party) => step(&mut d, &env(), Event::Owner { party }, &mut sent),
        }
        if let Some(answer) = row.answer {
            step(&mut d, &env(), Event::PersonOf { forge: 1, answer }, &mut sent);
        }
        let after = match d.record(1) {
            Some(record) => record.person,
            None => None,
        };
        assert_eq!(after, row.after, "{}: link", row.name);
        record.person = row.after;
        let mut expected = Queue::uncapped();
        if row.saved {
            expected.push(Request::Save { record: Stored::Forge(record), release: None, read_afresh: None });
        }
        for note in row.notes {
            expected.push(Request::LinkNoted { forge: 1, note: *note });
        }
        for _ in 0..expected.len() {
            assert_eq!(sent.pop(), expected.pop(), "{}: saves and notes", row.name);
        }
        assert!(sent.is_empty(), "{}: unexpected saves or notes: {sent:?}", row.name);
    }
}

#[test]
fn a_closed_box_pipe_keeps_the_restored_link_when_the_owner_is_handed_again() {
    const P: u64 = 31;
    const SEVEN: Option<Link> = Some(Link { user: 7, person: P });
    let mut d = setup_domain(&[], true);
    let mut record = records::record(1);
    record.mode = Mode::Owned;
    record.person = SEVEN;
    assert!(outputs(&mut d, Event::Restore { record: Stored::Forge(record) }).is_empty());
    assert!(outputs(&mut d, Event::Owner { party: Some(P) }).is_empty());
    assert_eq!(d.link(1), SEVEN);

    assert_eq!(outputs(&mut d, Event::BoxClosed).as_ref(), &[Request::MarkedForges]);
    assert!(outputs(&mut d, Event::Owner { party: Some(P) }).is_empty());
    assert_eq!(d.link(1), SEVEN);
}
