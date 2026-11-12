//! Seeded approval people and the same rows' client context (client/forge/README.md, 4.1).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary seeded tests construct projects and inspect their client trees"
)]
use crate::route::approvals::tests::{approval, counted, members_fixture, row_for, wait};
use alloc::boxed::Box;
use alloc::vec::Vec;
use jig_client::contract as c;
use jig_domain_people::Role;
use skein_lib::{Duration, Env, List, Queue, Rng, Time, Wall};
use temper_client_domain::{self as client, Event};
use temper_client_view::{
    self as view,
    words::{Shell, Words},
};

#[test]
fn random_projects_keep_signers_disjoint_eligible_bounded_and_say_you() {
    let words = Words::english(3, 256, Shell::Browser).unwrap();
    let mut saw = [false; 4];
    for seed in 0..96 {
        let mut rng = Rng::new(seed);
        let n = rng.below(13);
        let role = u32::try_from(rng.below(4)).unwrap();
        let required = u32::try_from(rng.below(16)).unwrap() + 1;
        let same_role = rng.below(2) == 0;
        let unsigned = rng.below(3) == 0;
        let mut members = Vec::new();
        let mut names = Vec::new();
        for person in 1..=n {
            names.push(format!("Person-{person}-{}", "é".repeat(35)));
        }
        for (index, name) in names.iter().enumerate() {
            let held = if same_role { role } else { u32::try_from(rng.below(4)).unwrap() };
            members.push((
                u64::try_from(index).unwrap() + 1,
                Role::from_number(held),
                rng.below(8) != 0,
                name.as_str(),
                "login",
            ));
        }
        let (domain, _) = members_fixture(&members);
        let reviewers: Vec<_> = (1..=16).filter(|_| !unsigned && rng.below(4) == 0).collect();
        let mut short = approval(role, required, false);
        short.counted = counted(&reviewers, role);
        let waiting = wait(Box::new([short]));
        let eligible: Vec<_> = members
            .iter()
            .filter(|(_, held, account, _, _)| held.number() == role && *account)
            .map(|member| member.0)
            .collect();
        for &(watcher, held, account, _, _) in &members {
            let Some(row) = row_for(&domain, &waiting, watcher) else { continue };
            let can: Vec<_> = row.can_sign.iter().map(|p| p.person).collect();
            let signed: Vec<_> = row.signed.iter().map(|p| p.person).collect();
            assert!(signed.iter().all(|person| !can.contains(person)), "seed {seed}");
            let expected: Vec<_> = eligible.iter().copied().filter(|p| !reviewers.contains(p)).collect();
            assert_eq!(can.len() + usize::try_from(row.can_sign_more).unwrap(), expected.len(), "seed {seed}");
            assert_eq!(can.first() == Some(&watcher), held.number() == role && account, "seed {seed}");
            assert!(can.iter().all(|p| expected.contains(p)), "seed {seed}");
            assert_eq!(signed, reviewers.iter().copied().take(8).collect::<Vec<_>>());
            assert!(row.signed.len() <= 8 && can.len() <= 8);
            for person in row.signed.iter().chain(row.can_sign.iter()) {
                assert!(person.name.len() <= 64 && core::str::from_utf8(&person.name).is_ok());
            }
            saw[0] |= row.can_sign_more > 0;
            saw[1] |= row.counted == 0;
            saw[2] |= row.yours == crate::YourReview::NoAccount;
            saw[3] |= row.yours == crate::YourReview::TooFew;
            check_context(&row, watcher, &words);
        }
    }
    assert_eq!(saw, [true; 4], "the sweep includes overflow, unsigned waits and both eligibility deficits");
}

fn client_people(people: &[crate::ApprovalPerson]) -> Box<[client::forge::approvals::ApprovalPerson]> {
    people.iter().map(|p| client::forge::approvals::ApprovalPerson { person: p.person, name: p.name.clone() }).collect()
}

fn check_context(row: &crate::ApprovalRow, me: u64, words: &Words) {
    let row = client::forge::approvals::Row {
        task: row.task,
        project: row.project,
        goal: row.goal,
        forge: row.forge,
        kind: client::forge::forges::Kind::Forgejo,
        owner: row.owner.clone(),
        name: row.name.clone(),
        pull: row.pull,
        address: Box::default(),
        title: row.title.clone(),
        since: row.since,
        scope: row.scope,
        rule: row.rule,
        role: row.role,
        required: row.required,
        counted: row.counted,
        sets: Box::new([]),
        whole: row.whole,
        yours: match row.yours {
            crate::YourReview::Counts => client::forge::approvals::Yours::Counts,
            crate::YourReview::NoAccount => client::forge::approvals::Yours::NoAccount,
            crate::YourReview::TooFew => client::forge::approvals::Yours::TooFew,
            crate::YourReview::OtherRole => client::forge::approvals::Yours::OtherRole,
        },
        signed: client_people(&row.signed),
        can_sign: client_people(&row.can_sign),
        can_sign_more: row.can_sign_more,
        size: row.size.map(|size| client::forge::approvals::Size {
            files: size.files,
            additions: size.additions,
            deletions: size.deletions,
            cut: size.cut,
        }),
    };
    let env = client_env();
    let mut root = client_fixture(me, &env);
    client::step(
        &mut root,
        &env,
        Event::Forge(client::forge::Event::Approvals {
            event: client::forge::approvals::Event::Rows { rows: Box::new([row.clone()]), more: 0 },
        }),
        &mut Queue::uncapped(),
    );
    let limits = view::Limits::of(&env.limits, words).unwrap();
    let mut shown = view::View::new(&limits, &env.limits, words.clone(), 3);
    view::build(&mut shown, &root, Wall::EPOCH, &mut Queue::uncapped());
    let who: Vec<_> = shown
        .tree()
        .nodes()
        .iter()
        .filter(|node| {
            node.role == skein_ui::Role::Paragraph && node.styles.as_slice().contains(&view::look::ASK_CONTEXT)
        })
        .filter_map(|node| node.name.as_deref())
        .filter(|line| line.windows(7).any(|w| w == b"signed:") || line.windows(10).any(|w| w == b" can give "))
        .collect();
    let expected = row.counted > 0 || row.yours != client::forge::approvals::Yours::Counts && !row.can_sign.is_empty();
    assert_eq!(who.len(), usize::from(expected), "{row:?}");
    if let Some(line) = who.first() {
        assert!(line.ends_with(b"."));
        let name = format!("Person-{me}-");
        assert!(!line.windows(name.len()).any(|w| w == name.as_bytes()), "viewer is only you: {line:?}");
        if row.can_sign.iter().any(|p| p.person == me) {
            assert!(line.windows(3).any(|w| w == b"you" || w == b"You"));
        }
    }
}

fn client_env() -> Env<client::Limits> {
    let requests = skein_lib::RequestLimits {
        requests: 4,
        bytes: 262_144,
        out: 16,
        first: Duration::from_millis(10),
        most: Duration::from_millis(80),
        margin: Duration::from_secs(1),
    };
    Env {
        now: Time::ZERO,
        wall: Wall::EPOCH,
        limits: client::Limits {
            client: jig_client::Limits {
                domain: jig_client::DomainLimits {
                    accounts: jig_client::AccountCounts { accounts: 1, sources: 1, sign_ins: 1 },
                    owners: &[jig_client::Spoken {
                        family: temper_client_protocol_wire::FORGE,
                        version: temper_client_protocol_wire::CLIENT_VERSION,
                    }],
                    requests,
                    ask_bytes: 65536,
                    handles: 16,
                    watches: 16,
                    turn_bytes: 65536,
                    backoff: jig_client::Backoff { first: Duration::from_millis(10), most: Duration::from_millis(80) },
                    authority: 12,
                    heartbeat: Duration::from_millis(100),
                    reads: 8,
                    parts: 8,
                    delegates: 2,
                    document_bytes: 65536,
                    records: 2,
                    kept: 4,
                    projects: 2,
                    text: 256,
                    long_text: 256,
                    turn_text: 256,
                    rows: 2,
                    fields: 4,
                    segments: 9,
                },
                exchanges: 8,
            },
            forge: client::forge::Limits { field_bytes: 256, checks: 8 },
            row_uses: 8,
            window_bytes: None,
        },
    }
}

fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(c::Text::new(bytes.into(), 256).unwrap())
}

fn client_fixture(me: u64, env: &Env<client::Limits>) -> client::Domain {
    let mut root = client::unlinked(&env.limits, 11);
    let mut out = Queue::uncapped();
    for event in [
        Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } },
        Event::Start {
            address: client::Address::Inbox { filter: client::address::Filter::ALL },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    ] {
        client::step(&mut root, env, event, &mut out);
    }
    while let Some(request) = out.pop() {
        let client::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(subject) },
                },
        } = request
        else {
            continue;
        };
        let delivery = match subject {
            c::Subject::Me => c::Delivery::Me(Box::new(c::Me {
                providers: List::with_capacity(0),
                providers_cut: false,
                key_bytes: 256,
                redirect_bytes: 256,
                words_bytes: 8192,
                party: c::MeParty {
                    number: me,
                    kind: c::PartyKind::Person,
                    login: text(b"viewer"),
                    name: text(b"Viewer"),
                },
                projects: List::from_boxed(Box::new([c::MeProject {
                    project: 1,
                    name: text(b"Project"),
                    role: c::HeldRole::Member,
                    pool: None,
                    public: None,
                    defaults: List::with_capacity(0),
                    templates: List::with_capacity(0),
                    creating: None,
                    models: List::with_capacity(0),
                    left_out: 0,
                }])),
                needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
                needs_total: 0,
                updates: c::Count { value: 0, limited: false },
                results_seen: 0,
                cut: false,
                retention: Duration::from_secs(1000),
                deployment_role: None,
            })),
            c::Subject::Inbox { .. } => c::Delivery::Inbox(Box::new(c::Inbox {
                party: me,
                needs: List::with_capacity(0),
                updates: List::with_capacity(0),
                next: None,
                cached_needs: 0,
                cached_updates: c::Count { value: 0, limited: false },
                results_seen: 0,
                updates_time_complete: true,
                selection: None,
            })),
            c::Subject::Settings
            | c::Subject::Task { .. }
            | c::Subject::Tree { .. }
            | c::Subject::Chips { .. }
            | c::Subject::Tasks { .. }
            | c::Subject::Run { .. }
            | c::Subject::Deployment
            | c::Subject::Accounts => continue,
        };
        client::step(
            &mut root,
            env,
            Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
                },
            },
            &mut out,
        );
    }
    root
}
