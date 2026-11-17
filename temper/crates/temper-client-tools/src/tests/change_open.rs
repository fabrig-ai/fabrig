//! A person's open-change act, with engine answers supplied by the client fixture.
use super::fixtures::{self as f, OPEN_CHANGE as OPEN};
use jig_client::{self as client, contract as c};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn open(extra: &[&str]) -> (f::Harness, u32) {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(f::extended(OPEN, extra).as_slice());
    (root, call)
}
fn sent(root: &f::Harness, call: u32) -> w::OpenChangeParts {
    let client::Ask::Family { scope, record } = root.adoption_bytes(call) else { panic!("forge request") };
    assert_eq!(*scope, c::Scope::Project(f::PROJECT));
    assert_eq!((record.family, record.version), (wire::FORGE, wire::CLIENT_VERSION));
    let bounds = temper_client_domain_forge::forge::wire_limits(&root.limits.domain.forge).unwrap();
    let w::Request::OpenChange(request) = wire::decode_requesting(&record.bytes, &bounds).unwrap().into_parts().request
    else {
        panic!("OpenChange")
    };
    request.into_parts()
}
fn request_bytes(ask: &client::Ask) -> &[u8] {
    let client::Ask::Family { record, .. } = ask else { panic!("forge request") };
    &record.bytes
}
fn opened(task: u64) -> w::Outcome {
    w::Outcome::Opened(w::Opened::new(&w::CEILINGS, w::OpenedParts { task }).unwrap())
}
fn refused(reason: w::OpenRefusal) -> w::Outcome {
    w::Outcome::OpenRefused(w::OpenRefused::new(&w::CEILINGS, w::OpenRefusedParts { reason }).unwrap())
}
fn repository(owner: &[u8], name: &[u8]) -> w::DraftRepository {
    w::DraftRepository::new(&w::CEILINGS, w::DraftRepositoryParts { owner: owner.into(), name: name.into() }).unwrap()
}
fn usage(words: &[&str], expected: &str) {
    let mut root = f::signed_in(f::PARTY);
    let call = root.submit(words);
    let printed = root.answer(call).expect("local usage");
    assert_eq!((printed.status, printed.stderr.as_ref()), (1, expected.as_bytes()));
    assert!(printed.stdout.is_empty());
    assert!(root.nothing_sent());
}
#[test]
fn missing_controls_are_collected_and_only_absent_controls_are_named() {
    for (extra, expected) in [
        (&[][..], "change open needs: --branch BRANCH; --title TEXT; --budget AMOUNT or --as-allowed\n"),
        (&["--branch", "docs/x", "--budget", "500"][..], "change open needs: --title TEXT\n"),
        (&["--title", "Fix", "--as-allowed"][..], "change open needs: --branch BRANCH\n"),
        (&["--branch", "docs/x", "--title", "Fix"][..], "change open needs: --budget AMOUNT or --as-allowed\n"),
    ] {
        let words = f::extended(&["change", "open", "--project", "shop", "--key", "open-change"], extra);
        usage(words.as_slice(), expected);
    }
}
#[test]
fn malformed_controls_are_usage_before_any_send_in_control_order() {
    for bad in ["o", "o/", "/n", "o/n/extra"] {
        usage(f::extended(OPEN, &["--repository", bad]).as_slice(), "change open: --repository must be OWNER/NAME\n");
    }
    for at in ["", "abc", "0123456789abcdef0123456789abcdef0123456g", "0123456789abcdef0123456789abcdef0123456789abc"] {
        usage(f::extended(OPEN, &["--at", at]).as_slice(), "change open: --at must be 40 or 64 hexadecimal digits\n");
    }
    usage(f::extended(OPEN, &["--as-allowed"]).as_slice(), "change open: --budget and --as-allowed are alternatives\n");
    usage(
        f::extended(OPEN, &["--repository", "bad", "--at", "bad", "--as-allowed"]).as_slice(),
        "change open: --repository must be OWNER/NAME\n",
    );
}
#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "ordinary test argv replaces one named control to exercise each byte bound"
)]
fn byte_bounds_accept_the_bound_and_refuse_the_next_byte_naming_it() {
    let bytes = "x".repeat(16_385);
    for (flag, bound, expected) in [
        ("--branch", 1024, "change open: --branch must be 1 to 1024 bytes\n"),
        ("--into", 1024, "change open: --into must be 1 to 1024 bytes\n"),
        ("--title", 256, "change open: --title must be 1 to 256 bytes\n"),
        ("--body", 16_384, "change open: --body must be 1 to 16384 bytes\n"),
    ] {
        for length in [0, bound, bound + 1] {
            let text = &bytes[..length];
            let mut words = f::extended(OPEN, &[]);
            if let Some(at) = words.iter().position(|word| *word == flag) {
                *words.get_mut(u32::try_from(at + 1).unwrap()).unwrap() = text;
            } else {
                words.push(flag).unwrap();
                words.push(text).unwrap();
            }
            if length == 0 || length > bound {
                usage(words.as_slice(), expected);
            } else {
                let mut root = f::signed_in(f::PARTY);
                let call = root.submit(words.as_slice());
                let request = sent(&root, call);
                let field = match flag {
                    "--branch" => request.branch,
                    "--into" => request.into.unwrap(),
                    "--title" => request.title,
                    "--body" => request.body.unwrap(),
                    _ => panic!("bounded field"),
                };
                assert_eq!(field.as_ref(), text.as_bytes());
            }
        }
    }
    usage(
        f::extended(
            &["change", "open", "--project", "shop", "--branch", "docs/x", "--budget", "500", "--key", "open-change"],
            &["--title", &"é".repeat(129)],
        )
        .as_slice(),
        "change open: --title must be 1 to 256 bytes\n",
    );
}
#[test]
fn keyed_requests_keep_defaults_and_all_supplied_controls_and_print_the_task() {
    let (mut root, call) = open(&[]);
    let request = sent(&root, call);
    assert_eq!(
        (request.project, request.repository, request.branch.as_ref()),
        (f::PROJECT, None, b"docs/x".as_slice())
    );
    assert_eq!(
        (request.at, request.into, request.title.as_ref(), request.body),
        (None, None, b"Fix the docs".as_slice(), None)
    );
    let w::OpenBudget::Amount(amount) = request.budget else { panic!("named budget") };
    assert_eq!(amount.amount(), 500);
    assert_eq!(root.sends.len(), 1);
    assert_eq!(root.sent_name(call), Some(f::named("open-change")));
    root.adoption_answer(call, opened(7));
    let printed = root.answer(call).unwrap();
    assert_eq!(
        (printed.status, printed.stdout.as_ref(), printed.stderr.as_ref()),
        (0, b"task 7\n".as_slice(), b"".as_slice())
    );
    for at in
        ["ABCDEF0123456789ABCDEF0123456789ABCDEF01", "ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789"]
    {
        let mut root = f::signed_in(f::PARTY);
        let call = root.submit(&[
            "change",
            "open",
            "--project",
            "shop",
            "--repository",
            "o/n",
            "--branch",
            "docs/x",
            "--at",
            at,
            "--into",
            "release",
            "--title",
            "Fix the docs",
            "--body",
            "More words",
            "--as-allowed",
            "--key",
            "open-change",
            "--json",
        ]);
        let request = sent(&root, call);
        assert_eq!(request.repository, Some(repository(b"o", b"n")));
        assert_eq!(request.at.as_deref(), Some(at.to_ascii_lowercase().as_bytes()));
        assert_eq!(request.into.as_deref(), Some(b"release".as_slice()));
        assert_eq!(request.body.as_deref(), Some(b"More words".as_slice()));
        assert_eq!(request.budget, w::OpenBudget::AsAllowed);
        root.adoption_answer(call, opened(7));
        let printed = root.answer(call).unwrap();
        assert_eq!((printed.status, printed.stdout.as_ref()), (0, b"{\"task\":7}\n".as_slice()));
    }
}
#[test]
fn forge_refusals_name_projects_repositories_and_the_refused_branch() {
    let mut repositories = List::with_capacity(2);
    repositories.push(repository(b"o", b"a")).unwrap();
    repositories.push(repository(b"o", b"b")).unwrap();
    for (why, extra, expected) in [
        (
            w::OpenRefusal::Several(
                w::SeveralRepositories::new(&w::CEILINGS, w::SeveralRepositoriesParts { repositories }).unwrap(),
            ),
            &[][..],
            "temper ctl: change open: project shop has several repositories, o/a, o/b: name one with --repository\n",
        ),
        (
            w::OpenRefusal::NoRepository,
            &[],
            "temper ctl: change open: project shop has no repository a change lands into\n",
        ),
        (
            w::OpenRefusal::NotARepository,
            &["--repository", "o/n"],
            "temper ctl: change open: o/n is not a repository of project shop\n",
        ),
        (
            w::OpenRefusal::UnderPrefix(
                w::UnderPrefix::new(
                    &w::CEILINGS,
                    w::UnderPrefixParts {
                        owner: b"o".as_slice().into(),
                        name: b"n".as_slice().into(),
                        prefix: b"temper/".as_slice().into(),
                    },
                )
                .unwrap(),
            ),
            &["--into", "temper/x"],
            "temper ctl: change open: temper/x is under o/n's prefix temper/: a change lands into one of its landing branches\n",
        ),
        (
            w::OpenRefusal::NamePast(w::NamePast::new(&w::CEILINGS, w::NamePastParts { bound: 4 }).unwrap()),
            &[],
            "temper ctl: change open: docs/x is past the forge's 4-byte name bound\n",
        ),
        (
            w::OpenRefusal::NamePast(w::NamePast::new(&w::CEILINGS, w::NamePastParts { bound: 6 }).unwrap()),
            &["--into", "release"],
            "temper ctl: change open: release is past the forge's 6-byte name bound\n",
        ),
    ] {
        let (mut root, call) = open(extra);
        root.adoption_answer(call, refused(why));
        let printed = root.answer(call).unwrap();
        assert_eq!((printed.status, printed.stderr.as_ref()), (2, expected.as_bytes()));
        assert!(printed.stdout.is_empty());
        assert_eq!(root.sends.len(), 1, "a refusal never sends a follow-on act");
    }
    let mut root = f::signed_in(f::PARTY);
    let mut call = f::call(OPEN);
    *call.values.get_mut(0).unwrap() = Some(crate::jig::call::Values::One(crate::jig::call::Value::Project(
        crate::jig::call::Project::Number(f::PROJECT),
    )));
    let call = root.submit_prepared(call, false, None);
    root.adoption_answer(call, refused(w::OpenRefusal::NoRepository));
    assert_eq!(
        root.answer(call).unwrap().stderr.as_ref(),
        b"temper ctl: change open: project shop has no repository a change lands into\n"
    );
}
fn core(why: jig_client_protocol_wire::v27::Refusal) -> w::Outcome {
    let mut bytes = skein_lib::Writer::new(usize::try_from(why.measure()).unwrap());
    why.encode(&mut bytes).unwrap();
    w::Outcome::CoreRefused(w::CoreRefused::new(&w::CEILINGS, w::CoreRefusedParts { refusal: bytes.finish() }).unwrap())
}
#[test]
fn core_refusals_use_jigs_words_on_one_prefixed_line() {
    use jig_client_protocol_wire::v27 as j;
    for why in [
        j::Refusal::Authority(
            j::RefusalAuthority::new(
                &j::CEILINGS,
                j::RefusalAuthorityParts {
                    lacking: j::Lacking::Budget(
                        j::LackingBudget::new(&j::CEILINGS, j::LackingBudgetParts { needed: 500, left: 0 }).unwrap(),
                    ),
                    proposable: None,
                },
            )
            .unwrap(),
        ),
        j::Refusal::Authority(
            j::RefusalAuthority::new(
                &j::CEILINGS,
                j::RefusalAuthorityParts {
                    lacking: j::Lacking::Tasks(
                        j::LackingTasks::new(&j::CEILINGS, j::LackingTasksParts { needed: Some(1), left: 0 }).unwrap(),
                    ),
                    proposable: None,
                },
            )
            .unwrap(),
        ),
        j::Refusal::Role(j::RefusalRole::new(&j::CEILINGS, j::RefusalRoleParts { held: None }).unwrap()),
    ] {
        let expected = if let j::Refusal::Role(_) = &why {
            b"temper ctl: change open: The held role does not permit this action\n".as_slice()
        } else {
            b"temper ctl: change open: Authority missing\n".as_slice()
        };
        let (mut root, call) = open(&[]);
        root.adoption_answer(call, core(why));
        let printed = root.answer(call).unwrap();
        assert_eq!(printed.status, 2);
        assert_eq!(printed.stderr.as_ref(), expected);
        assert!(printed.stdout.is_empty());
        assert_eq!(root.sends.len(), 1);
    }
}
#[test]
fn waits_for_me_and_replays_and_restores_the_same_keyed_request() {
    let mut root = f::started(List::with_capacity(0));
    let first = root.submit(OPEN);
    assert!(root.nothing_sent());
    root.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    assert_eq!(sent(&root, first).project, f::PROJECT);
    let key = root.sent_name(first).unwrap();
    let request = root.adoption_bytes(first).clone();
    let mut records = List::with_capacity(1);
    records
        .push(skein_lib::RequestRecord {
            key,
            scope: f::PARTY,
            first_sent: None,
            request: client::Held { key: client::Key { asker: f::forge_asker(), number: 1 }, request: request.clone() },
        })
        .unwrap();
    let first_answer = opened(7);
    root.adoption_answer(first, first_answer.clone());
    let expected = root.answer(first).unwrap();
    let second = root.submit(OPEN);
    assert_eq!(root.sent_name(second), Some(key));
    assert_eq!(request_bytes(root.adoption_bytes(second)), request_bytes(&request));
    root.adoption_answer(second, first_answer.clone());
    let actual = root.answer(second).unwrap();
    assert_eq!((actual.status, actual.stdout), (expected.status, expected.stdout.clone()));
    let mut restored = f::started(records);
    restored.deliver(c::Delivery::Me(Box::new(f::me(f::PARTY))));
    restored.adoption_answer(0, first_answer);
    assert_eq!(restored.kept_lines.as_slice(), &[expected.stdout]);
}
#[test]
fn unknown_and_malformed_answers_are_not_reported_as_opened() {
    let (mut root, call) = open(&[]);
    root.adoption_answer(call, w::Outcome::Unknown);
    let printed = root.answer(call).unwrap();
    assert_eq!((printed.status, printed.stderr.as_ref()), (1, b"unknown past retention\n".as_slice()));
    assert!(printed.stdout.is_empty());
    let (mut root, call) = open(&[]);
    let client::Outcome::Answered(answer) = f::malformed_adoption_answer() else { panic!("family answer") };
    let attempt = root.sends.get(&call).unwrap().0;
    root.client(client::Event::Up {
        up: client::Up::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(answer),
            why: None,
        },
    });
    let printed = root.answer(call).unwrap();
    assert_eq!((printed.status, printed.stderr.as_ref()), (1, b"change open answer decoding failed\n".as_slice()));
    assert!(printed.stdout.is_empty());
}
#[test]
fn help_change_lists_the_open_synopsis() {
    let help = f::call(&["help", "change"]);
    let selected = crate::help::selection(&help).unwrap();
    let printed = f::selected_help(selected);
    assert!(skein_lib::bytes::find(&printed, b"change open --project PROJECT [--repository OWNER/NAME] --branch BRANCH [--at COMMIT] [--into BRANCH] --title TEXT [--body TEXT] (--budget AMOUNT | --as-allowed) --key KEY\n").is_some());
}
