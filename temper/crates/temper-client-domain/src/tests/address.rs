use crate::{
    ChatsFilter, ModelWay, PlanFilter, Step, WayKind,
    address::{Address, Filter, Kind, Place, parse, print},
};

#[test]
fn each_page_round_trips_through_its_path() {
    let pages: [(Address, &[u8]); 5] = [
        (Address::Inbox { filter: Filter::ALL }, b"/"),
        (Address::Chats { filter: ChatsFilter::All, start: false }, b"/chats"),
        (Address::Chat { task: 7 }, b"/chats/7"),
        (Address::Task { task: 7, filter: PlanFilter::All }, b"/tasks/7"),
        (Address::Adopt { project: 3 }, b"/projects/3/adopt"),
    ];
    for (address, path) in pages {
        assert_eq!(&*print(&address), path, "{address:?}");
        assert_eq!(parse(path), Place::Address(address), "{path:?}");
    }
}

#[test]
fn each_inbox_filter_round_trips_through_its_query() {
    let filters: [(Filter, &[u8]); 9] = [
        (Filter { project: Some(3), ..Filter::ALL }, b"/?project=3"),
        (Filter { kind: Some(Kind::Proposal), ..Filter::ALL }, b"/?kind=proposal"),
        (Filter { kind: Some(Kind::Question), ..Filter::ALL }, b"/?kind=question"),
        (Filter { kind: Some(Kind::Held), ..Filter::ALL }, b"/?kind=held"),
        (Filter { kind: Some(Kind::PersonTask), ..Filter::ALL }, b"/?kind=person-task"),
        (Filter { goal: Some(9), ..Filter::ALL }, b"/?goal=9"),
        (
            Filter { project: Some(3), kind: Some(Kind::PersonTask), goal: Some(9) },
            b"/?project=3&kind=person-task&goal=9",
        ),
        (Filter { project: Some(u32::MAX), ..Filter::ALL }, b"/?project=4294967295"),
        (Filter { goal: Some(u64::MAX), ..Filter::ALL }, b"/?goal=18446744073709551615"),
    ];
    for (filter, path) in filters {
        let address = Address::Inbox { filter };
        assert_eq!(&*print(&address), path, "{filter:?}");
        assert_eq!(parse(path), Place::Address(address), "{path:?}");
    }
}

#[test]
fn the_inbox_prints_its_set_fields_in_one_order() {
    let filters: [(Filter, &[u8]); 2] =
        [(Filter::ALL, b"/"), (Filter { project: Some(3), goal: Some(9), ..Filter::ALL }, b"/?project=3&goal=9")];
    for (filter, path) in filters {
        assert_eq!(&*print(&Address::Inbox { filter }), path, "{filter:?}");
    }
}

#[test]
fn another_pages_query_is_dropped() {
    assert_eq!(parse(b"/chats/7?x=1"), Place::Address(Address::Chat { task: 7 }));
}

#[test]
fn a_field_it_cannot_read_is_dropped() {
    let project = Filter { project: Some(3), ..Filter::ALL };
    let fields: &[(&[u8], Filter)] = &[
        (b"/?secret=s", Filter::ALL),
        (b"/?kind=reply&project=3", project),
        (b"/?kind=Question", Filter::ALL),
        (b"/?project=07", Filter::ALL),
        (b"/?project=4294967296", Filter::ALL),
        (b"/?goal=", Filter::ALL),
        (b"/?kind", Filter::ALL),
        (b"/?=3", Filter::ALL),
        (b"/?", Filter::ALL),
        (b"/?&&", Filter::ALL),
        (b"/?project=3&project=4", project),
        (b"/?project=x&project=4", Filter { project: Some(4), ..Filter::ALL }),
        (b"/?goal=9&project=3", Filter { goal: Some(9), ..project }),
        (b"/?goal=18446744073709551616", Filter::ALL),
        (b"/?kind=question=held", Filter::ALL),
        (b"/?kind=held&kind=question", Filter { kind: Some(Kind::Held), ..Filter::ALL }),
        (b"/?goal=x&goal=9&goal=10", Filter { goal: Some(9), ..Filter::ALL }),
        (b"/?kind=reply&kind=question", Filter { kind: Some(Kind::Question), ..Filter::ALL }),
    ];
    for &(path, filter) in fields {
        assert_eq!(parse(path), Place::Address(Address::Inbox { filter }), "{path:?}");
    }
}

#[test]
fn one_trailing_slash_is_ignored() {
    for (path, expected) in [
        (&b"/chats/7/"[..], Place::Address(Address::Chat { task: 7 })),
        (b"/chats/", Place::Address(Address::Chats { filter: ChatsFilter::All, start: false })),
        (b"/chats/7//", Place::Missing),
    ] {
        assert_eq!(parse(path), expected, "{path:?}");
    }
}

#[test]
fn a_path_that_is_no_page_is_missing() {
    let paths: [&[u8]; 12] = [
        b"/jig/v1/sign-in/link",
        b"/bundle/x",
        b"/test/panic",
        b"/robots.txt",
        b"chats",
        b"/chats/x",
        b"/tasks",
        b"/projects/1",
        b"/projects/1/board",
        b"/projects/3/adopt/x",
        b"//",
        b"/chats/%37",
    ];
    for path in paths {
        assert_eq!(parse(path), Place::Missing, "{path:?}");
    }
}

#[test]
fn a_number_not_in_canonical_decimal_is_missing() {
    let paths: [&[u8]; 5] = [b"/chats/07", b"/chats/+7", b"/chats/-7", b"/chats/7a", b"/chats/123456789012345678901"];
    for path in paths {
        assert_eq!(parse(path), Place::Missing, "{path:?}");
    }
}

#[test]
fn each_number_spans_its_types_range() {
    let task = Place::Address(Address::Chat { task: u64::MAX });
    let project = Place::Address(Address::Adopt { project: u32::MAX });
    assert_eq!(parse(b"/chats/18446744073709551615"), task);
    assert_eq!(parse(b"/projects/4294967295/adopt"), project);
    assert_eq!(&*print(&Address::Chat { task: u64::MAX }), b"/chats/18446744073709551615");
    assert_eq!(&*print(&Address::Adopt { project: u32::MAX }), b"/projects/4294967295/adopt");
    assert_eq!(parse(b"/chats/18446744073709551616"), Place::Missing);
    assert_eq!(parse(b"/projects/4294967296/adopt"), Place::Missing);
}

#[test]
fn the_welcome_prints_and_parses_each_step() {
    for (step, path) in [
        (Step::Welcome, b"/welcome".as_slice()),
        (Step::Forge { forge: None }, b"/welcome?at=forge"),
        (Step::Model { way: None }, b"/welcome?at=model"),
        (Step::Ready, b"/welcome?at=ready"),
    ] {
        let address = Address::Welcome { step };
        assert_eq!(&*print(&address), path);
        assert_eq!(parse(path), Place::Address(address));
    }
}
#[test]
fn a_welcome_query_it_cannot_read_opens_its_first_step() {
    for path in [b"/welcome?at=done".as_slice(), b"/welcome?x=1", b"/welcome?at=", b"/welcome?at=Forge"] {
        assert_eq!(parse(path), Place::Address(Address::Welcome { step: Step::Welcome }));
    }
    assert_eq!(
        parse(b"/welcome?x=1&at=model&at=forge"),
        Place::Address(Address::Welcome { step: Step::Model { way: None } })
    );
    // A first that doesn't read is dropped, and the next that reads is the step (browser.md, 2.3).
    assert_eq!(
        parse(b"/welcome?at=done&at=forge"),
        Place::Address(Address::Welcome { step: Step::Forge { forge: None } })
    );
}

#[test]
fn all_model_way_addresses_print_and_read_including_the_welcome() {
    for (kind, path, welcome) in [
        (
            WayKind::SignIn,
            b"/accounts/connect?provider=17&way=sign-in".as_slice(),
            b"/welcome?at=model&provider=17&way=sign-in".as_slice(),
        ),
        (WayKind::Program, b"/accounts/connect?provider=17&way=program", b"/welcome?at=model&provider=17&way=program"),
        (WayKind::Key, b"/accounts/connect?provider=17&way=key", b"/welcome?at=model&provider=17&way=key"),
    ] {
        let way = Some(ModelWay { source: None, provider: 17, kind });
        for (address, expected) in
            [(Address::ConnectModel { way }, path), (Address::Welcome { step: Step::Model { way } }, welcome)]
        {
            assert_eq!(print(&address).as_ref(), expected);
            assert_eq!(parse(expected), Place::Address(address));
        }
    }
    assert_eq!(print(&Address::ConnectModel { way: None }).as_ref(), b"/accounts/connect");
}
#[test]
fn incomplete_or_unreadable_way_fields_are_dropped() {
    for path in [
        b"/accounts/connect?provider=17".as_slice(),
        b"/accounts/connect?way=key",
        b"/accounts/connect?provider=017&way=key",
        b"/accounts/connect?provider=4294967296&way=key",
        b"/accounts/connect?provider=17&way=unknown",
        b"/accounts/connect?other=17&way=key",
    ] {
        assert_eq!(parse(path), Place::Address(Address::ConnectModel { way: None }));
    }
    assert_eq!(
        parse(b"/welcome?at=model&provider=no&way=key"),
        Place::Address(Address::Welcome { step: Step::Model { way: None } })
    );
}

#[test]
fn connect_forge_prints_and_parses_its_choice() {
    use crate::Choice;
    for (forge, path) in [
        (None, b"/forges/connect".as_slice()),
        (Some(Choice::Forge(3)), b"/forges/connect?forge=3"),
        (Some(Choice::Forge(u16::MAX)), b"/forges/connect?forge=65535"),
        (Some(Choice::Elsewhere), b"/forges/connect?forge=elsewhere"),
    ] {
        let address = Address::ConnectForge { forge };
        assert_eq!(print(&address).as_ref(), path);
        assert_eq!(parse(path), Place::Address(address));
    }
    assert_eq!(print(&Address::Forges).as_ref(), b"/forges");
    assert_eq!(parse(b"/forges"), Place::Address(Address::Forges));
}
#[test]
fn a_forge_choice_it_cannot_read_opens_choose() {
    for path in [
        b"/forges/connect?forge=unknown".as_slice(),
        b"/forges/connect?forge=65536",
        b"/forges/connect?forge=03",
        b"/forges/connect?forge=-1",
        b"/forges/connect?x=1",
    ] {
        assert_eq!(parse(path), Place::Address(Address::ConnectForge { forge: None }));
    }
    assert_eq!(
        parse(b"/forges/connect?forge=3&forge=4&x=1"),
        Place::Address(Address::ConnectForge { forge: Some(crate::Choice::Forge(3)) })
    );
    assert_eq!(
        parse(b"/forges/connect?forge=&forge=3"),
        Place::Address(Address::ConnectForge { forge: Some(crate::Choice::Forge(3)) })
    );
    assert_eq!(parse(b"/forges/3/github/setup"), Place::Missing);
}
#[test]
fn the_welcomes_forge_step_carries_its_choice() {
    for (forge, path) in [
        (Some(crate::Choice::Forge(3)), b"/welcome?at=forge&forge=3".as_slice()),
        (Some(crate::Choice::Elsewhere), b"/welcome?at=forge&forge=elsewhere"),
        (None, b"/welcome?at=forge"),
    ] {
        let address = Address::Welcome { step: Step::Forge { forge } };
        assert_eq!(print(&address).as_ref(), path);
        assert_eq!(parse(path), Place::Address(address));
    }
    assert_eq!(
        parse(b"/welcome?forge=3&at=model"),
        Place::Address(Address::Welcome { step: Step::Model { way: None } })
    );
}

#[test]
fn a_return_source_is_printed_and_read_only_on_a_sign_in_way() {
    for base in [b"/accounts/connect?".as_slice(), b"/welcome?at=model&".as_slice()] {
        for (query, canonical) in [
            ("provider=1&way=sign-in&source=7", "provider=1&way=sign-in&source=7"),
            ("source=7&provider=1&way=sign-in", "provider=1&way=sign-in&source=7"),
            ("provider=1&way=sign-in&source=4294967295", "provider=1&way=sign-in&source=4294967295"),
            ("provider=1&way=sign-in&source=4294967296", "provider=1&way=sign-in"),
            ("provider=1&way=sign-in&source=07", "provider=1&way=sign-in"),
            ("provider=1&way=sign-in&source=bad", "provider=1&way=sign-in"),
            ("provider=1&way=key&source=7", "provider=1&way=key"),
            ("provider=1&way=program&source=7", "provider=1&way=program"),
        ] {
            let mut input = skein_lib::Writer::new(base.len().checked_add(query.len()).expect("query fits"));
            input.put(base).expect("base fits");
            input.put(query.as_bytes()).expect("query fits");
            let input = input.finish();
            let Place::Address(address) = parse(&input) else { panic!("known path") };
            assert_eq!(print(&address).strip_prefix(base).expect("same path"), canonical.as_bytes(), "{query}");
        }
    }
}

#[test]
fn chats_and_plan_filters_round_trip_with_the_starts_focus() {
    use crate::{ChatsFilter as Chats, PlanFilter as Plan};
    let cases: &[(Address, &[u8])] = &[
        (Address::Chats { filter: Chats::All, start: false }, b"/chats"),
        (Address::Chats { filter: Chats::Live, start: false }, b"/chats?state=live"),
        (Address::Chats { filter: Chats::Closed, start: false }, b"/chats?state=closed"),
        (Address::Chats { filter: Chats::All, start: true }, b"/chats?at=start"),
        (Address::Chats { filter: Chats::Live, start: true }, b"/chats?state=live&at=start"),
        (Address::Chats { filter: Chats::Closed, start: true }, b"/chats?state=closed&at=start"),
        (Address::Task { task: 7, filter: Plan::All }, b"/tasks/7"),
        (Address::Task { task: 7, filter: Plan::Live }, b"/tasks/7?state=live"),
    ];
    for (address, path) in cases {
        assert_eq!(print(address).as_ref(), *path);
        assert_eq!(parse(path), Place::Address(address.clone()));
    }
}

#[test]
fn chats_and_plan_drop_each_unreadable_field_independently() {
    for (path, canonical) in [
        (b"/chats?state=nope".as_slice(), b"/chats".as_slice()),
        (b"/tasks/7?state=nope", b"/tasks/7"),
        (b"/tasks/7?state=closed", b"/tasks/7"),
        (b"/chats?at=start&state=live", b"/chats?state=live&at=start"),
        (b"/chats?at=start&state=nope", b"/chats?at=start"),
        (b"/chats?at=nope&state=closed", b"/chats?state=closed"),
        (b"/chats?state=nope&state=live&state=closed&at=nope&at=start", b"/chats?state=live&at=start"),
        (b"/tasks/7?state=closed&state=live&at=start", b"/tasks/7?state=live"),
        (b"/chats?state=Live&at=Start", b"/chats"),
        (b"/chats?state=live=closed&at=start=nope", b"/chats"),
        (b"/chats?state&at&unknown=live", b"/chats"),
    ] {
        assert_eq!(parse(path), parse(canonical), "{path:?}");
    }
}
