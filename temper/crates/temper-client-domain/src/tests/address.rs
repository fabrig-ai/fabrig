use crate::{
    ChatsFilter, GoalsFilter, ModelWay, Step, WayKind, WorkFilter,
    address::{Address, Filter, Kind, Place, parse, print},
};

#[test]
fn each_page_round_trips_through_its_path() {
    let pages: [(Address, &[u8]); 42] = [
        (Address::Work { project: 3, filter: WorkFilter::All }, b"/projects/3/work"),
        (Address::Work { project: 3, filter: WorkFilter::Drafts }, b"/projects/3/work?show=drafts"),
        (Address::Members, b"/deployment/members"),
        (Address::Remembered, b"/remembered"),
        (Address::Limits, b"/deployment/limits"),
        (Address::Inbox { filter: Filter::ALL }, b"/"),
        (Address::Chats { filter: ChatsFilter::All }, b"/chats"),
        (Address::Chat { task: 7 }, b"/chats/7"),
        (Address::Task { task: 7 }, b"/tasks/7"),
        (Address::Team { project: 7, role: None }, b"/projects/7/team"),
        (Address::Team { project: 7, role: Some(52) }, b"/projects/7/team?role=52"),
        (Address::Team { project: 7, role: Some(u64::MAX) }, b"/projects/7/team?role=18446744073709551615"),
        (Address::Team { project: 0, role: None }, b"/projects/0/team"),
        (Address::Team { project: 3, role: None }, b"/projects/3/team"),
        (Address::Team { project: u32::MAX, role: None }, b"/projects/4294967295/team"),
        (Address::Changes { project: 0 }, b"/projects/0/changes"),
        (Address::Changes { project: 3 }, b"/projects/3/changes"),
        (Address::Changes { project: u32::MAX }, b"/projects/4294967295/changes"),
        (Address::Home { project: 0 }, b"/projects/0"),
        (Address::AllHome, b"/home"),
        (Address::Home { project: u32::MAX }, b"/projects/4294967295"),
        (Address::Settings { project: 0 }, b"/projects/0/settings"),
        (Address::Settings { project: u32::MAX }, b"/projects/4294967295/settings"),
        (Address::Home { project: 3 }, b"/projects/3"),
        (Address::Goals { project: 0, filter: GoalsFilter::All }, b"/projects/0/goals"),
        (Address::Goals { project: u32::MAX, filter: GoalsFilter::All }, b"/projects/4294967295/goals"),
        (Address::Goals { project: 3, filter: GoalsFilter::All }, b"/projects/3/goals"),
        (Address::Settings { project: 3 }, b"/projects/3/settings"),
        (Address::Models { project: 0 }, b"/projects/0/settings/models"),
        (Address::Models { project: 3 }, b"/projects/3/settings/models"),
        (Address::Models { project: u32::MAX }, b"/projects/4294967295/settings/models"),
        (Address::Builders { project: 0 }, b"/projects/0/settings/builders"),
        (Address::Builders { project: 3 }, b"/projects/3/settings/builders"),
        (Address::Builders { project: u32::MAX }, b"/projects/4294967295/settings/builders"),
        (Address::Role { project: 3, charter: 4 }, b"/projects/3/settings/roles/4"),
        (Address::Rule { project: 3, kind: 2, number: 1 }, b"/projects/3/settings/rules/2/1"),
        (Address::Rule { project: 0, kind: 1, number: 7 }, b"/projects/0/settings/rules/1/7"),
        (Address::Repositories { project: 3 }, b"/projects/3/settings/repositories"),
        (Address::Adopt { project: 3 }, b"/projects/3/adopt"),
        (Address::Ask { project: 0 }, b"/projects/0/ask"),
        (Address::Ask { project: u32::MAX }, b"/projects/4294967295/ask"),
        (Address::Ask { project: 3 }, b"/projects/3/ask"),
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
    for path in [b"/home".as_slice(), b"/home?x=1"] {
        assert_eq!(parse(path), Place::Address(Address::AllHome));
        assert_eq!(print(&Address::AllHome).as_ref(), b"/home");
    }
    assert_eq!(parse(b"/chats/7?x=1"), Place::Address(Address::Chat { task: 7 }));
    assert_eq!(parse(b"/projects/3/ask?x=1"), Place::Address(Address::Ask { project: 3 }));
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
        (b"/home/", Place::Address(Address::AllHome)),
        (b"/chats/", Place::Address(Address::Chats { filter: ChatsFilter::All })),
        (b"/chats/7//", Place::Missing),
        (b"/projects/3/team/", Place::Address(Address::Team { project: 3, role: None })),
        (b"/projects/3/ask/", Place::Address(Address::Ask { project: 3 })),
        (b"/projects/3/ask//", Place::Missing),
    ] {
        assert_eq!(parse(path), expected, "{path:?}");
    }
}

#[test]
fn a_path_that_is_no_page_is_missing() {
    let paths: &[&[u8]] = &[
        b"/jig/v1/sign-in/link",
        b"/bundle/x",
        b"/test/panic",
        b"/robots.txt",
        b"chats",
        b"/chats/x",
        b"/chats/newer",
        b"/projects/3/team/1",
        b"/tasks",
        b"/projects/01",
        b"/projects/1/other",
        b"/projects/99999999999",
        b"/projects/4294967296",
        b"/projects/4294967296/settings",
        b"/projects/01/settings",
        b"/projects/01/goals",
        b"/projects/4294967296/goals",
        b"/projects/x/goals",
        b"/projects/3/goals/extra",
        b"/projects/99999999999/settings",
        b"/projects/4294967296/ask",
        b"/projects/01/ask",
        b"/projects/x/ask",
        b"/projects//ask",
        b"/projects/-1/ask",
        b"/projects/+1/ask",
        b"/projects/3/ask/extra",
        b"/projects/3/settings/configure",
        b"/projects/3/settings/roles/0",
        b"/projects/3/settings/roles/04",
        b"/projects/3/settings/roles/4294967296",
        b"/projects/3/settings/roles/4/extra",
        b"/projects/3/settings/rules/3/1",
        b"/projects/3/settings/rules/0/1",
        b"/projects/3/settings/rules/2/0",
        b"/projects/3/settings/rules/2/01",
        b"/projects/3/settings/rules/2",
        b"/projects/3/settings/rules/2/1/extra",
        b"/projects/1/board",
        b"/projects/3/adopt/x",
        b"//",
        b"/chats/%37",
        b"/nowhere",
        b"/home/1",
        b"/chats/abc",
        b"/chats/-1",
        b"/chats/99999999999999999999999999",
        b"/chats/1/extra",
        b"/chats/%zz",
        b"/chats//1",
        b"/tasks/x",
        b"/projects/x/adopt",
        b"/projects//adopt",
        b"/CHATS",
        b"/test/other",
        b"/test/panic/",
        b"/test/Panic",
    ];
    for path in paths {
        assert_eq!(parse(path), Place::Missing, "{path:?}");
    }
}

#[test]
fn a_number_not_in_canonical_decimal_is_missing() {
    let paths: [&[u8]; 6] =
        [b"/projects/03/team", b"/chats/07", b"/chats/+7", b"/chats/-7", b"/chats/7a", b"/chats/123456789012345678901"];
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
    for (path, address) in [
        (b"/chats/0".as_slice(), Address::Chat { task: 0 }),
        (b"/chats/999", Address::Chat { task: 999 }),
        (b"/chats/4294967295", Address::Chat { task: 4_294_967_295 }),
        (b"/chats/4294967296", Address::Chat { task: 4_294_967_296 }),
        (b"/tasks/1", Address::Task { task: 1 }),
        (b"/projects/1/adopt", Address::Adopt { project: 1 }),
        (b"/projects/65536/adopt", Address::Adopt { project: 65_536 }),
    ] {
        assert_eq!(parse(path), Place::Address(address), "{path:?}");
    }
}

#[test]
fn the_welcome_prints_and_parses_each_step() {
    for (step, path) in [
        (Step::Welcome, b"/welcome".as_slice()),
        (Step::WelcomeAsked, b"/welcome?at=welcome"),
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
    // A first that doesn't read is dropped, and the next that reads is the step (client/web/README.md, 2.3).
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
fn chats_filters_and_new_chat_round_trip() {
    use crate::ChatsFilter as Chats;
    let cases: &[(Address, &[u8])] = &[
        (Address::Chats { filter: Chats::All }, b"/chats"),
        (Address::Chats { filter: Chats::Live }, b"/chats?state=live"),
        (Address::Chats { filter: Chats::Closed }, b"/chats?state=closed"),
        (Address::NewChat { draft: false }, b"/chats/new"),
    ];
    for (address, path) in cases {
        assert_eq!(print(address).as_ref(), *path);
        assert_eq!(parse(path), Place::Address(address.clone()));
    }
}

#[test]
fn a_tasks_state_query_reads_as_the_plain_address() {
    let address = Address::Task { task: 9 };
    for path in [
        b"/tasks/9?state=live".as_slice(),
        b"/tasks/9?state=closed&state=live&x=1",
        b"/tasks/9?state=nope",
        b"/tasks/9?state=closed",
    ] {
        assert_eq!(parse(path), Place::Address(address.clone()), "{path:?}");
    }
    assert_eq!(print(&address).as_ref(), b"/tasks/9");
}

#[test]
fn chats_drop_each_unreadable_field_independently() {
    for (path, canonical) in [
        (b"/chats?state=nope".as_slice(), b"/chats".as_slice()),
        (b"/chats?x=1&state=live", b"/chats?state=live"),
        (b"/chats?x=1&state=nope", b"/chats"),
        (b"/chats?at=nope&state=closed", b"/chats?state=closed"),
        (b"/chats?state=nope&state=live&state=closed&at=nope&x=1", b"/chats?state=live"),
        (b"/chats?state=Live&at=Start", b"/chats"),
        (b"/chats?state=live=closed&x=1=nope", b"/chats"),
        (b"/chats?state&at&unknown=live", b"/chats"),
    ] {
        assert_eq!(parse(path), parse(canonical), "{path:?}");
    }
}

#[test]
fn public_paths_print_and_parse_both_ways() {
    use crate::address::{Public, print_public};
    for (page, path) in [
        (Public::Projects, b"/public".as_slice()),
        (Public::Board { project: 1 }, b"/public/projects/0000000000000001"),
        (Public::Task { project: 1, task: 10 }, b"/public/projects/0000000000000001/tasks/000000000000000a"),
        (Public::Landings { project: 1 }, b"/public/projects/0000000000000001/landings"),
        (
            Public::Landing { project: 1, task: u64::MAX },
            b"/public/projects/0000000000000001/landings/ffffffffffffffff",
        ),
    ] {
        assert_eq!(print_public(&page).as_ref(), path);
        assert_eq!(parse(path), Place::Public(page));
    }
}
#[test]
fn a_malformed_public_path_reads_as_public() {
    use crate::address::Public;
    for path in [
        b"/public/projects/123".as_slice(),
        b"/public/projects/ABCDEF0123456789",
        b"/public/projects/123456789abcdef",
        b"/public/projects/0123456789abcdef0",
        b"/public/projects/0123456789abcdef/extra",
        b"/public/projects/0123456789abcdef/",
        b"/public/projects//0123456789abcdef",
        b"/public/projects/0123456789abcdef?x=1",
        b"/public/projects/%610123456789abcd",
        b"/public/",
        b"/public?x=1",
        b"/public/projects/0123456789abcdef/landings/",
        b"/public/projects/0123456789abcdef/tasks/0123456789abcdef/extra",
    ] {
        assert_eq!(parse(path), Place::Public(Public::Projects), "{path:?}");
    }
}
#[test]
fn other_paths_read_as_before() {
    assert_eq!(parse(b"/publicx"), Place::Missing);
    assert_eq!(parse(b"/publicx/projects/0123456789abcdef"), Place::Missing);
    assert_eq!(parse(b"/chats/7"), Place::Address(Address::Chat { task: 7 }));
    assert_eq!(parse(b"/"), Place::Address(Address::Inbox { filter: Filter::ALL }));
}

#[test]
fn a_neighbour_of_repositories_is_missing() {
    assert_eq!(parse(b"/projects/3/settings/other"), Place::Missing);
}

#[test]
fn new_chat_has_its_own_canonical_address() {
    for (path, address, canonical) in [
        (b"/chats/new".as_slice(), Address::NewChat { draft: false }, b"/chats/new".as_slice()),
        (b"/chats/new?x=1", Address::NewChat { draft: false }, b"/chats/new"),
        (b"/chats?at=start", Address::Chats { filter: ChatsFilter::All }, b"/chats"),
        (b"/chats?state=live&at=start", Address::Chats { filter: ChatsFilter::Live }, b"/chats?state=live"),
        (b"/chats/7", Address::Chat { task: 7 }, b"/chats/7"),
    ] {
        assert_eq!(parse(path), Place::Address(address.clone()), "{path:?}");
        assert_eq!(print(&address).as_ref(), canonical);
        assert_eq!(parse(canonical), Place::Address(address));
    }
    assert_eq!(parse(b"/chats/newer"), Place::Missing);
}

#[test]
fn goals_filter_is_its_query() {
    for (filter, path) in [
        (GoalsFilter::All, b"/projects/3/goals".as_slice()),
        (GoalsFilter::Needs, b"/projects/3/goals?state=needs"),
        (GoalsFilter::Live, b"/projects/3/goals?state=live"),
        (GoalsFilter::Ended, b"/projects/3/goals?state=ended"),
    ] {
        let address = Address::Goals { project: 3, filter };
        assert_eq!(print(&address).as_ref(), path);
        assert_eq!(parse(path), Place::Address(address));
    }
}

#[test]
fn goals_drops_unread_state_and_prints_the_canonical_address() {
    for (path, canonical) in [
        (b"/projects/3/goals?state=nope".as_slice(), b"/projects/3/goals".as_slice()),
        (b"/projects/3/goals?state=closed", b"/projects/3/goals"),
        (b"/projects/3/goals?state=Live", b"/projects/3/goals"),
        (b"/projects/3/goals?state=live=ended", b"/projects/3/goals"),
        (b"/projects/3/goals?state&unknown=value", b"/projects/3/goals"),
        (b"/projects/3/goals?state=nope&state=needs&state=ended", b"/projects/3/goals?state=needs"),
        (b"/projects/3/goals/?unknown=x&state=live", b"/projects/3/goals?state=live"),
    ] {
        let Place::Address(address) = parse(path) else { panic!("goals address") };
        assert_eq!(print(&address).as_ref(), canonical, "{path:?}");
    }
}

#[test]
fn the_address_round_trips() {
    for (address, path) in [
        (Address::NewChat { draft: false }, b"/chats/new".as_slice()),
        (Address::NewChat { draft: true }, b"/chats/new?draft=1".as_slice()),
    ] {
        assert_eq!(print(&address).as_ref(), path);
        assert_eq!(parse(path), Place::Address(address));
    }
    for query in [b"/chats/new?other=1".as_slice(), b"/chats/new?draft=0"] {
        assert_eq!(parse(query), Place::Address(Address::NewChat { draft: false }));
    }
    assert_eq!(parse(b"/chats/new?x=1&draft=1&y=2"), Place::Address(Address::NewChat { draft: true }));
}

#[test]
fn the_filter_round_trips() {
    let address = Address::Chats { filter: ChatsFilter::Drafts };
    assert_eq!(print(&address).as_ref(), b"/chats?state=drafts");
    assert_eq!(parse(&print(&address)), Place::Address(address));
}

#[test]
fn the_approval_page_reads_its_code_and_prints_it_shown() {
    let code = jig_client::ApprovalCode::parse(b"WDJBMJHT").unwrap();
    for (path, expected) in [
        (b"/terminal".as_slice(), Place::Approve { code: None }),
        (b"/terminal/", Place::Approve { code: None }),
        (b"/terminal?code=WDJB-MJHT", Place::Approve { code: Some(code) }),
        (b"/terminal?code=wdjbmjht", Place::Approve { code: Some(code) }),
        (b"/terminal?code=WDJB%20MJHT", Place::Approve { code: None }),
        (b"/terminal?code=wdjb mjht", Place::Approve { code: Some(code) }),
        (b"/terminal?code=AEIO", Place::Approve { code: None }),
        (b"/terminal?code=", Place::Approve { code: None }),
        (b"/terminal?x=1&code=WDJB-MJHT", Place::Approve { code: Some(code) }),
        (b"/terminal?code=AEIO&code=WDJB-MJHT", Place::Approve { code: Some(code) }),
        (b"/terminal?code=WDJB-MJHT&code=BCDF-GHJK", Place::Approve { code: Some(code) }),
        (b"/terminal?code=BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB", Place::Approve { code: None }),
        (b"/terminal/x", Place::Missing),
        (b"/terminals", Place::Missing),
    ] {
        assert_eq!(parse(path), expected, "{path:?}");
    }
    for (code, expected) in [(Some(code), b"/terminal?code=WDJB-MJHT".as_slice()), (None, b"/terminal")] {
        let path = crate::address::print_approve(code);
        assert_eq!(path.as_ref(), expected);
        assert_eq!(parse(&path), Place::Approve { code });
    }
}

#[test]
fn work_filter_is_its_query() {
    for (path, filter, canonical) in [
        (b"/projects/3/work?show=drafts".as_slice(), WorkFilter::Drafts, b"/projects/3/work?show=drafts".as_slice()),
        (b"/projects/3/work?show=other", WorkFilter::All, b"/projects/3/work"),
        (b"/projects/3/work?x=1", WorkFilter::All, b"/projects/3/work"),
        (b"/projects/3/work/?show=drafts", WorkFilter::Drafts, b"/projects/3/work?show=drafts"),
        (b"/projects/3/work?show=other&x=1&show=drafts", WorkFilter::Drafts, b"/projects/3/work?show=drafts"),
    ] {
        let address = Address::Work { project: 3, filter };
        assert_eq!(parse(path), Place::Address(address.clone()), "{path:?}");
        assert_eq!(print(&address).as_ref(), canonical);
    }
}
