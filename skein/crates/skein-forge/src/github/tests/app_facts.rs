use super::fixtures::{bytes, decode, limits};
use crate::{
    api,
    github::{
        decode::Malformed,
        document::{Access, App, Document, Kind, Permission},
        plans::failure::malformed,
        server, setup,
    },
};
use skein_lib::{List, Writer, bytes::copy_of};

fn app() -> App {
    App { id: 7, slug: Box::from(*b"example-app"), permissions: Box::from([]), events: Box::from([]) }
}
fn body(app: &App) -> Box<[u8]> {
    let mut out = Writer::new(4096);
    server::app(app, &mut out).unwrap();
    bytes(out)
}
fn wanted() -> App {
    let mut app = app();
    let mut permissions = List::with_capacity(9);
    for (name, access) in setup::PERMISSIONS {
        permissions.push(Permission { name: copy_of(name), access }).unwrap();
    }
    let mut events = List::with_capacity(14);
    for name in setup::EVENTS {
        events.push(copy_of(name)).unwrap();
    }
    app.permissions = permissions.into_boxed();
    app.events = events.into_boxed();
    app
}

#[test]
fn github_app_facts_keep_every_name_and_access() {
    let input = br#"{"id":7,"slug":"example-app","permissions":{"extra\"name":"admin","contents":"write","metadata":"read"},"events":["extra","push","extra\"event"],"ignored":{"nested":[false,{"text":"skipped"}]}}"#;
    let expected = App {
        permissions: Box::from([
            Permission { name: Box::from(*b"extra\"name"), access: Access::Admin },
            Permission { name: Box::from(*b"contents"), access: Access::Write },
            Permission { name: Box::from(*b"metadata"), access: Access::Read },
        ]),
        events: Box::from([copy_of(b"extra"), copy_of(b"push"), copy_of(b"extra\"event")]),
        ..app()
    };
    for input in [input.as_slice(), body(&expected).as_ref()] {
        for chunk in [1, 7, u32::try_from(input.len()).unwrap()] {
            assert_eq!(decode(Kind::App, input, chunk, &limits()).1, Ok(Document::App(expected.clone())));
        }
    }
}

#[test]
fn github_app_facts_default_only_optional_fields() {
    for input in [
        br#"{"id":7,"slug":"example-app"}"#.as_slice(),
        br#"{"id":7,"slug":"example-app","permissions":{}}"#,
        br#"{"id":7,"slug":"example-app","events":[]}"#,
    ] {
        assert_eq!(decode(Kind::App, input, 1, &limits()).1, Ok(Document::App(app())));
    }
    for (input, field) in [
        (br#"{"slug":"example-app","permissions":{},"events":[]}"#.as_slice(), b"id".as_slice()),
        (br#"{"id":7,"permissions":{},"events":[]}"#, b"slug"),
    ] {
        let error = Malformed::Missing { field };
        assert_eq!(decode(Kind::App, input, 1, &limits()).1, Err(error));
        assert_eq!(malformed(error), api::Error::InvalidAnswer);
    }
}

#[test]
fn github_app_facts_refuse_unknown_access_and_wrong_shapes() {
    for input in [
        br#"{"id":7,"slug":"example-app","permissions":{"contents":"owner"}}"#.as_slice(),
        br#"{"id":7,"slug":"example-app","permissions":{"extra":"none"}}"#,
        br#"{"id":7,"slug":"example-app","permissions":{"extra":"unknown-access"}}"#,
        br#"{"id":7,"slug":"example-app","permissions":{"extra":true}}"#,
        br#"{"id":7,"slug":"example-app","permissions":{"extra":null}}"#,
        br#"{"id":7,"slug":"example-app","permissions":[]}"#,
        br#"{"id":7,"slug":"example-app","events":{}}"#,
        br#"{"id":7,"slug":"example-app","events":[false]}"#,
        br#"{"id":7,"slug":"example-app","events":[{}]}"#,
    ] {
        for chunk in [1, 7] {
            let error = decode(Kind::App, input, chunk, &limits()).1.unwrap_err();
            assert_eq!(malformed(error), api::Error::InvalidAnswer);
        }
    }
}

struct AppBound {
    limits: crate::github::limits::Limits,
    app: App,
    expected: Result<Document, Malformed>,
    classified: Option<api::Error>,
}
#[test]
#[expect(clippy::too_many_lines, reason = "the labelled cases keep every input and expectation of one rule together")]
fn app_fact_bounds_keep_fitting_values_and_classify_excess() {
    skein_rows::check(
        [
            ("github_app_facts_name_bounds_are_too_large: permissions 128", {
                let name = copy_of(&[b'x'; 256][..128]);
                let app =
                    App { permissions: Box::from([Permission { name: name.clone(), access: Access::Admin }]), ..app() };
                AppBound { limits: limits(), expected: Ok(Document::App(app.clone())), app, classified: None }
            }),
            ("github_app_facts_name_bounds_are_too_large: events 128", {
                let name = copy_of(&[b'x'; 256][..128]);
                let app = App { events: Box::from([name.clone()]), ..app() };
                AppBound { limits: limits(), expected: Ok(Document::App(app.clone())), app, classified: None }
            }),
            ("github_app_facts_name_bounds_are_too_large: permissions 129", {
                let name = copy_of(&[b'x'; 256][..129]);
                let app =
                    App { permissions: Box::from([Permission { name: name.clone(), access: Access::Admin }]), ..app() };
                AppBound {
                    limits: limits(),
                    expected: Err(Malformed::Bound { field: b"permissions" }),
                    app,
                    classified: Some(api::Error::TooLarge),
                }
            }),
            ("github_app_facts_name_bounds_are_too_large: events 129", {
                let name = copy_of(&[b'x'; 256][..129]);
                let app = App { events: Box::from([name.clone()]), ..app() };
                AppBound {
                    limits: limits(),
                    expected: Err(Malformed::Bound { field: b"events" }),
                    app,
                    classified: Some(api::Error::TooLarge),
                }
            }),
            ("github_app_facts_name_bounds_are_too_large: permissions 256", {
                let name = copy_of(&[b'x'; 256][..256]);
                let app =
                    App { permissions: Box::from([Permission { name: name.clone(), access: Access::Admin }]), ..app() };
                AppBound {
                    limits: limits(),
                    expected: Err(Malformed::Bound { field: b"permissions" }),
                    app,
                    classified: Some(api::Error::TooLarge),
                }
            }),
            ("github_app_facts_name_bounds_are_too_large: events 256", {
                let name = copy_of(&[b'x'; 256][..256]);
                let app = App { events: Box::from([name.clone()]), ..app() };
                AppBound {
                    limits: limits(),
                    expected: Err(Malformed::Bound { field: b"events" }),
                    app,
                    classified: Some(api::Error::TooLarge),
                }
            }),
            ("github_app_facts_list_bounds_are_too_large: permissions 2", {
                let mut limits = limits();
                limits.inner = 2;
                let app = App {
                    permissions: Box::from([
                        Permission { name: copy_of(b"one"), access: Access::Read },
                        Permission { name: copy_of(b"two"), access: Access::Read },
                    ]),
                    ..app()
                };
                AppBound { limits, expected: Ok(Document::App(app.clone())), app, classified: None }
            }),
            ("github_app_facts_list_bounds_are_too_large: events 2", {
                let mut limits = limits();
                limits.inner = 2;
                let app = App { events: Box::from([copy_of(b"one"), copy_of(b"two")]), ..app() };
                AppBound { limits, expected: Ok(Document::App(app.clone())), app, classified: None }
            }),
            ("github_app_facts_list_bounds_are_too_large: permissions 3", {
                let mut limits = limits();
                limits.inner = 2;
                let app = App {
                    permissions: Box::from([
                        Permission { name: copy_of(b"one"), access: Access::Read },
                        Permission { name: copy_of(b"two"), access: Access::Read },
                        Permission { name: copy_of(b"three"), access: Access::Read },
                    ]),
                    ..app()
                };
                AppBound {
                    limits,
                    expected: Err(Malformed::Bound { field: b"permissions" }),
                    app,
                    classified: Some(api::Error::TooLarge),
                }
            }),
            ("github_app_facts_list_bounds_are_too_large: events 3", {
                let mut limits = limits();
                limits.inner = 2;
                let app = App { events: Box::from([copy_of(b"one"), copy_of(b"two"), copy_of(b"three")]), ..app() };
                AppBound {
                    limits,
                    expected: Err(Malformed::Bound { field: b"events" }),
                    app,
                    classified: Some(api::Error::TooLarge),
                }
            }),
        ],
        |case| {
            let input = body(&case.app);
            for chunk in [1, 7, u32::try_from(input.len()).unwrap()] {
                let result = decode(Kind::App, &input, chunk, &case.limits).1;
                assert_eq!(result, case.expected);
                let classified = match result {
                    Ok(_) => None,
                    Err(error) => Some(malformed(error)),
                };
                assert_eq!(classified, case.classified);
            }
        },
    );
}

#[test]
fn github_setup_lacks_names_missing_and_weaker_grants() {
    let full = wanted();
    let missing = setup::lacks(&app());
    assert_eq!(missing.permissions, full.permissions);
    assert_eq!(missing.events, full.events);
    let mut held = wanted();
    for permission in &mut held.permissions {
        permission.access = Access::Admin;
    }
    assert!(setup::lacks(&held).permissions.is_empty());
    assert!(setup::lacks(&held).events.is_empty());
    held.permissions.first_mut().unwrap().access = Access::Read;
    held.events = Box::from([copy_of(b"push"), copy_of(b"extra")]);
    let missing = setup::lacks(&held);
    assert_eq!(missing.permissions, Box::from([Permission { name: copy_of(b"contents"), access: Access::Write }]));
    assert_eq!(missing.events.len(), 13);
    assert!(!missing.events.contains(&copy_of(b"push")));
    assert_eq!(missing.events.first().unwrap().as_ref(), b"issues");
    assert_eq!(missing.events.last().unwrap().as_ref(), b"merge_group");
    let complete = setup::lacks(&full);
    assert!(complete.permissions.is_empty());
    assert!(complete.events.is_empty());
}

#[test]
fn github_app_access_covers_only_equal_or_weaker_grants() {
    for (held, read, write, admin) in
        [(Access::Read, true, false, false), (Access::Write, true, true, false), (Access::Admin, true, true, true)]
    {
        assert_eq!(held.satisfies(Access::Read), read);
        assert_eq!(held.satisfies(Access::Write), write);
        assert_eq!(held.satisfies(Access::Admin), admin);
        assert_eq!(Access::read(held.text()), Some(held));
    }
}

#[test]
fn github_app_access_is_independent_of_name_bound() {
    let mut good = limits();
    good.string_bytes = 3;
    let input = br#"{"id":7,"slug":"app","permissions":{"x":"admin"},"events":["e"]}"#;
    let expected = App {
        id: 7,
        slug: copy_of(b"app"),
        permissions: Box::from([Permission { name: copy_of(b"x"), access: Access::Admin }]),
        events: Box::from([copy_of(b"e")]),
    };
    assert_eq!(decode(Kind::App, input, 1, &good).1, Ok(Document::App(expected)));
}
