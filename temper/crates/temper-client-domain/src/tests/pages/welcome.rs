use crate::ChatsFilter;
use crate::tests::{fixtures as f, playground};
use crate::{
    Action, Address, Domain, Event, Place, Request, Step, forge,
    pages::{self, Act, welcome::FirstRun},
};
use jig_client::{Down, Link, Readout, Told, Up, contract as c};
use skein_lib::{List, Queue, Secret, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn event(root: &mut Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn go(root: &mut Domain, step: Step) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(Address::Welcome { step }) })
}
fn deliver(root: &mut Domain, watch: Token, delivery: c::Delivery) -> Queue<Request> {
    up(root, Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) })
}
fn answer(root: &mut Domain, present: bool) {
    let mut out = Queue::with_capacity(1);
    forge::playground::told(&mut root.forge, &Told::Link { link: Link::Offline }, f::limits().forge, &mut out);
    forge::playground::told(&mut root.forge, &Told::Link { link: Link::Live }, f::limits().forge, &mut out);
    let Some(forge::Request::Client { asked: jig_client::Event::Read { handle, .. } }) = out.pop() else {
        panic!("playground read")
    };
    let page = if present {
        let mut parts = wire::decode_playground_answer(&playground::answer(b"Change the title"), &w::CEILINGS)
            .unwrap()
            .into_parts();
        parts.state = w::PlaygroundState::Starting;
        Readout::Family {
            family: wire::FORGE,
            record: Some(wire::playground_answer(&w::PlaygroundAnswer::new(&w::CEILINGS, parts).unwrap())),
        }
    } else {
        Readout::NotServed { family: Some(wire::FORGE), served: None }
    };
    crate::route::told(root, &f::env(&f::limits()), Told::Page { handle, page }, &mut Queue::uncapped());
}
fn opened(owner: bool, step: Step) -> (Domain, Token, Token, Option<Token>) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let (me, accounts) = f::start_accounts(&mut root, List::with_capacity(0));
    let c::WatchEvent::Delivered(c::Delivery::Me(mut frame)) = f::frame(f::PARTY) else { panic!("me") };
    frame.projects = List::with_capacity(1);
    frame
        .projects
        .push(c::MeProject {
            public: None,
            project: f::PROJECT,
            name: jig_client::contract::DocumentText::complete(
                jig_client::contract::Text::new(b"Project 1".as_slice().into(), 64).expect("project name"),
            ),
            role: c::HeldRole::Member,
            pool: None,
            templates: List::with_capacity(0),
            creating: None,
            models: List::with_capacity(0),
        })
        .unwrap();
    frame.providers = List::with_capacity(1);
    frame
        .providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(c::Text::new(b"openai".as_slice().into(), 32).unwrap()),
            key_bytes: 256,
            sign_in: true,
            signed_in_plan: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .unwrap();
    frame.deployment_role = Some(if owner { c::HeldRole::Owner } else { c::HeldRole::Member });
    let mut out = deliver(&mut root, me, c::Delivery::Me(frame));
    go(&mut root, step);
    let mut forges = None;
    while let Some(request) = out.pop() {
        if let Request::Client {
            request:
                jig_client::Request::Down { down: Down::Open { watch, subject: jig_client::Watchable::Family { .. } } },
        } = request
        {
            forges = Some(watch);
        }
    }
    (root, me, accounts, forges)
}
fn rows(root: &mut Domain, watch: Token, number: u16) -> Queue<Request> {
    forge_rows(root, watch, Some(number), true)
}
fn forge_rows(root: &mut Domain, watch: Token, number: Option<u16>, adopted: bool) -> Queue<Request> {
    let limits = forge::wire_limits(&f::limits().forge).unwrap();
    let mut forges = List::with_capacity(1);
    if let Some(number) = number {
        forges
            .push(
                w::ForgeRow::new(
                    &limits,
                    w::ForgeRowParts {
                        number,
                        kind: w::Kind::Forgejo,
                        mode: if number == wire::FORGE { w::Mode::Fake } else { w::Mode::Connected },
                        connection: w::Connection::Connected(
                            w::Connected::new(&limits, w::ConnectedParts { id: 42, login: b"bot".as_slice().into() })
                                .unwrap(),
                        ),
                        offer: w::Offer::NotOffered,
                        first: Some(
                            w::FirstRepository::new(
                                &limits,
                                w::FirstRepositoryParts {
                                    org: b"work".as_slice().into(),
                                    repository: b"hello".as_slice().into(),
                                    state: w::FirstState::Made(w::Made::new(&limits, w::MadeParts { id: 1 }).unwrap()),
                                    label: Box::new([]),
                                    playground: false,
                                },
                            )
                            .unwrap(),
                        ),
                        installations: List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: {
                            let mut repositories = List::with_capacity(1);
                            if adopted {
                                repositories
                                    .push(
                                        w::Adoption::new(
                                            &limits,
                                            w::AdoptionParts {
                                                owner: b"work".as_slice().into(),
                                                name: b"hello".as_slice().into(),
                                                project: f::PROJECT,
                                            },
                                        )
                                        .unwrap(),
                                    )
                                    .unwrap();
                            }
                            repositories
                        },
                        owned: None,
                        admin: false,
                        read_only: false,
                        ended_stop: None,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges }).unwrap()),
            },
        },
    )
}
fn account(root: &mut Domain, watch: Token, provider: u32, usable: bool) -> Queue<Request> {
    deliver(root, watch, c::Delivery::Accounts(Box::new(playground::accounts(Some(provider), usable, false))))
}
fn moved(mut out: Queue<Request>, replace: bool) -> bool {
    while let Some(request) = out.pop() {
        if request == (Request::Show { address: Address::Chats { filter: ChatsFilter::All, start: false }, replace }) {
            return true;
        }
    }
    false
}
fn decided(root: &Domain) -> bool {
    root.pages().welcome().unwrap().decided
}
#[test]
fn an_empty_store_offers_the_playground_and_set_up() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    account(&mut root, accounts, 18, false);
    let out = forge_rows(&mut root, forges.unwrap(), None, false);
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Empty));
    assert_eq!(pages::welcome::playground_offered(&root), Some(true));
    assert!(!moved(out, true));
    assert!(decided(&root));
}
#[test]
fn a_connected_forge_without_a_project_opens_ready() {
    let (mut root, me, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    account(&mut root, accounts, 18, true);
    let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
    frame.projects = List::with_capacity(0);
    deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    let mut out = forge_rows(&mut root, forges.unwrap(), Some(3), false);
    let step = Step::Ready;
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Unfinished { step }));
    assert_eq!(out.pop(), Some(Request::Show { address: Address::Welcome { step }, replace: true }));
}
#[test]
fn a_whole_store_moves_on_to_chats() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    account(&mut root, accounts, 18, true);
    assert!(moved(rows(&mut root, forges.unwrap(), 3), true));
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Whole));
}
#[test]
fn a_real_account_alone_keeps_the_playground_offered() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    account(&mut root, accounts, 18, true);
    forge_rows(&mut root, forges.unwrap(), None, false);
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Unfinished { step: Step::Forge { forge: None } }));
    assert_eq!(pages::welcome::playground_offered(&root), Some(true));
}
#[test]
fn a_kept_real_forge_never_offers_the_playground() {
    for adopted in [false, true] {
        let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
        answer(&mut root, false);
        account(&mut root, accounts, 18, false);
        forge_rows(&mut root, forges.unwrap(), Some(3), adopted);
        assert_eq!(pages::welcome::playground_offered(&root), Some(false));
        assert!(!moved(event(&mut root, Event::Act { action: Action::Pages(Act::Playground) }), false));
    }
}
#[test]
fn a_connected_forge_without_a_model_opens_model() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    account(&mut root, accounts, 18, false);
    let mut out = rows(&mut root, forges.unwrap(), 3);
    let step = Step::Model { way: None };
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Unfinished { step }));
    assert_eq!(out.pop(), Some(Request::Show { address: Address::Welcome { step }, replace: true }));
}
#[test]
fn the_playgrounds_rows_do_not_make_a_store_whole() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, true);
    account(&mut root, accounts, 17, true);
    rows(&mut root, forges.unwrap(), wire::FORGE);
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Empty));
    assert_eq!(pages::welcome::playground_offered(&root), Some(true));
}
#[test]
fn the_welcome_waits_for_its_facts() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    account(&mut root, accounts, 18, true);
    rows(&mut root, forges.unwrap(), 3);
    assert_eq!(pages::welcome::decision(&root), None);
    assert!(!decided(&root));
    answer(&mut root, false);
    let mut out = Queue::uncapped();
    crate::fire(&mut root, &f::env(&f::limits()), &mut out);
    assert!(decided(&root));
    assert!(moved(out, true));
}
#[test]
fn a_refused_forges_watch_counts_as_no_forge() {
    for end in [c::WatchEnd::NotAllowed, c::WatchEnd::NotServed, c::WatchEnd::SubjectEnded] {
        let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
        answer(&mut root, false);
        account(&mut root, accounts, 18, false);
        up(&mut root, Up::WatchEnded { watch: forges.unwrap(), end });
        assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Empty));
        assert!(decided(&root));
    }
}
#[test]
fn an_ended_accounts_watch_counts_as_no_account() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, false);
    rows(&mut root, forges.unwrap(), 3);
    up(&mut root, Up::WatchEnded { watch: accounts, end: c::WatchEnd::NotAllowed });
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Unfinished { step: Step::Model { way: None } }));
}
#[test]
fn a_later_step_never_moves_on() {
    for step in [Step::Forge { forge: None }, Step::Model { way: None }, Step::Ready] {
        let (mut root, _, accounts, forges) = opened(true, step);
        answer(&mut root, false);
        account(&mut root, accounts, 18, true);
        assert!(!moved(rows(&mut root, forges.unwrap(), 3), true));
        assert!(!decided(&root));
    }
}
#[test]
fn try_the_playground_opens_chats_without_a_playground_row() {
    let (mut root, _, _, forges) = opened(true, Step::Welcome);
    assert!(!moved(event(&mut root, Event::Act { action: Action::Pages(Act::Playground) }), false));
    answer(&mut root, false);
    forge_rows(&mut root, forges.unwrap(), None, false);
    assert!(moved(event(&mut root, Event::Act { action: Action::Pages(Act::Playground) }), false));
    assert!(root.forge().playground_starting());
}
#[test]
fn a_new_playgrounds_project_is_selected_when_its_membership_arrives() {
    let (mut root, me, _, forges) = opened(true, Step::Model { way: None });
    answer(&mut root, false);
    forge_rows(&mut root, forges.unwrap(), None, false);
    let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
    let playground = frame.projects.get(0).unwrap().clone();
    frame.projects.get_mut(0).unwrap().project = f::PROJECT + 1;
    deliver(&mut root, me, c::Delivery::Me(Box::new(frame.clone())));
    assert_eq!(root.pages().project(), Some(f::PROJECT + 1));
    event(&mut root, Event::Act { action: Action::Pages(Act::Playground) });
    playground::deliver_playground_read(
        &mut root,
        Readout::Family { family: wire::FORGE, record: Some(playground::answer(b"Change the title")) },
    );
    assert_eq!(root.pages().project(), Some(f::PROJECT + 1));
    let other = frame.projects.get(0).unwrap().clone();
    frame.projects = List::with_capacity(2);
    frame.projects.push(other).unwrap();
    frame.projects.push(playground).unwrap();
    deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    assert_eq!(root.pages().project(), Some(f::PROJECT));
    event(&mut root, Event::Act { action: Action::Pages(Act::Project { project: f::PROJECT + 1 }) });
    assert_eq!(root.pages().project(), Some(f::PROJECT + 1));
}
#[test]
fn retry_playground_requires_a_failure_and_retries_the_start() {
    let (mut root, _, _, forges) = opened(true, Step::Model { way: None });
    answer(&mut root, false);
    forge_rows(&mut root, forges.unwrap(), None, false);
    event(&mut root, Event::Act { action: Action::Pages(Act::RetryPlayground) });
    assert!(!root.forge().playground_starting());
    event(&mut root, Event::Act { action: Action::Pages(Act::Playground) });
    let first = root.forge().playground_request().unwrap();
    crate::route::told(
        &mut root,
        &f::env(&f::limits()),
        Told::Answered {
            key: jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: first.raw() },
            outcome: jig_client::Outcome::Answered(jig_client::Answered::Family(jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::answer(
                    &w::Answer::new(
                        &w::CEILINGS,
                        w::AnswerParts {
                            outcome: w::Outcome::StartFailed(
                                w::StartFailed::new(
                                    &w::CEILINGS,
                                    w::StartFailedParts { why: w::StartFailure::Account },
                                )
                                .unwrap(),
                            ),
                        },
                    )
                    .unwrap(),
                ),
            })),
        },
        &mut Queue::uncapped(),
    );
    assert!(root.forge().playground_failure().is_some());
    event(&mut root, Event::Act { action: Action::Pages(Act::RetryPlayground) });
    assert!(root.forge().playground_starting());
    assert!(root.forge().playground_failure().is_none());
    assert_ne!(root.forge().playground_request().unwrap(), first);
}
#[test]
fn the_model_step_sends_the_accounts_acts() {
    use pages::connect_model::Act as A;
    let (mut root, _, _, _) = opened(false, Step::Model { way: None });
    go(&mut root, Step::Model { way: Some(crate::ModelWay { source: None, provider: 17, kind: crate::WayKind::Key }) });
    event(&mut root, Event::AccountKey { value: Secret::new(b"private-test-key".as_slice().into()) });
    let mut out =
        event(&mut root, Event::Act { action: Action::Pages(Act::ConnectModel(A::Submit { name: [1; 16] })) });
    let mut sent = false;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { value: Some(value), .. } } } =
            request
        {
            assert_eq!(value.expose(), b"private-test-key");
            sent = true;
        }
    }
    assert!(sent);
    assert!(!root.pages().connect_model().unwrap().request.has_key());
}
#[test]
fn leaving_the_welcome_keeps_the_sessions_forges() {
    let (mut root, _, _, forges) = opened(true, Step::Welcome);
    let watch = forges.unwrap();
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    let mut out = event(
        &mut root,
        Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) },
    );
    let mut closed = false;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Close { watch: closing } } } = request
        {
            closed |= closing == watch;
        }
    }
    assert!(!closed);
    assert!(root.pages().welcome().is_none());
}
#[test]
fn losing_the_owner_role_keeps_the_sessions_watch() {
    let (mut root, me, _, forges) = opened(true, Step::Welcome);
    let watch = forges.unwrap();
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    let c::WatchEvent::Delivered(frame) = f::frame(f::PARTY) else { panic!("frame") };
    let mut out = deliver(&mut root, me, frame);
    let mut closed = false;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Close { watch: closing } } } = request
        {
            closed |= closing == watch;
        }
    }
    assert!(!closed);
}
#[test]
fn an_invalid_forges_document_counts_as_no_forge() {
    let (mut root, _, accounts, forges) = opened(true, Step::Welcome);
    answer(&mut root, true);
    account(&mut root, accounts, 18, true);
    assert!(!moved(
        up(
            &mut root,
            Up::FamilyDelivered {
                watch: forges.unwrap(),
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: Box::new([255])
                }
            }
        ),
        true
    ));
    assert!(decided(&root));
    let forge::forges::State::Ended = root.forges() else { panic!("invalid rows end the watch") };
}

fn strip_forges(root: &mut Domain, watch: Token, connection: Option<w::Connection>) {
    let mut forges = List::with_capacity(1);
    if let Some(connection) = connection {
        forges
            .push(
                w::ForgeRow::new(
                    &w::CEILINGS,
                    w::ForgeRowParts {
                        number: 3,
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Owned,
                        connection,
                        offer: w::Offer::Offered,
                        first: None,
                        installations: List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: List::with_capacity(0),
                        owned: None,
                        admin: false,
                        read_only: false,
                        ended_stop: None,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::forges(&w::Forges::new(&w::CEILINGS, w::ForgesParts { forges }).unwrap()),
            },
        },
    );
}
#[test]
fn the_setup_strip_shows_while_there_is_no_demo_and_the_first_run_is_not_over() {
    use forge::listed;
    let (mut root, _, accounts, forges) = opened(true, Step::Model { way: None });
    event(&mut root, Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) });
    assert!(listed::setup(&root).is_none());
    account(&mut root, accounts, 18, false);
    strip_forges(&mut root, forges.unwrap(), None);
    assert!(listed::setup(&root).is_none());
    answer(&mut root, false);
    assert!(listed::setup(&root).is_some());
    answer(&mut root, true);
    assert!(listed::setup(&root).is_some());
}
#[test]
fn the_strip_names_what_is_missing_and_a_non_owner_misses_the_model_alone() {
    use forge::listed;
    for owner in [false, true] {
        let (mut root, _, accounts, forges) = opened(owner, Step::Model { way: None });
        answer(&mut root, false);
        account(&mut root, accounts, 18, false);
        strip_forges(&mut root, forges.unwrap(), None);
        let setup = listed::setup(&root).unwrap();
        assert_eq!(setup.forge, owner);
        assert!(setup.model);
        account(&mut root, accounts, 18, true);
        if owner {
            let setup = listed::setup(&root).unwrap();
            assert!(setup.forge);
            assert!(!setup.model);
        } else {
            assert!(!listed::setup(&root).unwrap().model);
        }
    }
}
#[test]
fn continue_setting_up_goes_to_the_first_step_not_done() {
    use forge::listed;
    let (mut root, _, accounts, forges) = opened(true, Step::Model { way: None });
    let watch = forges.unwrap();
    answer(&mut root, false);
    account(&mut root, accounts, 18, false);
    strip_forges(&mut root, watch, None);
    assert_eq!(listed::setup(&root).unwrap().step, Step::Forge { forge: None });
    strip_forges(&mut root, watch, Some(w::Connection::Connecting));
    assert_eq!(listed::setup(&root).unwrap().step, Step::Model { way: None });
    account(&mut root, accounts, 18, true);
    assert_eq!(listed::setup(&root).unwrap().step, Step::Ready);
    strip_forges(
        &mut root,
        watch,
        Some(w::Connection::Connected(
            w::Connected::new(&w::CEILINGS, w::ConnectedParts { id: 42, login: b"temper".as_slice().into() }).unwrap(),
        )),
    );
    assert_eq!(pages::welcome::decision(&root), Some(FirstRun::Whole));
    rows(&mut root, watch, 3);
    assert!(listed::setup(&root).is_none());
    account(&mut root, accounts, 18, false);
    let setup = listed::setup(&root).unwrap();
    assert!(!setup.forge);
    assert!(setup.model);
    assert_eq!(setup.step, Step::Model { way: None });
}

#[test]
fn the_playground_forge_shows_while_the_forge_half_holds() {
    let (mut root, _, accounts, forges) = opened(true, Step::Model { way: None });
    let watch = forges.unwrap();
    playground::deliver_playground_read(
        &mut root,
        Readout::Family { family: wire::FORGE, record: Some(playground::answer(b"Change the title")) },
    );
    event(&mut root, Event::Act { action: Action::Pages(Act::Playground) });
    account(&mut root, accounts, 18, true);
    rows(&mut root, watch, wire::FORGE);
    assert_eq!(forge::listed::playground(&root).unwrap().1.number, wire::FORGE);
    rows(&mut root, watch, 3);
    assert!(forge::listed::playground(&root).is_none());
}

#[test]
fn without_a_playground_model_usable_reads_the_documents_usable() {
    for (document_usable, line_usable) in [(true, false), (false, true), (false, false), (true, true)] {
        let (mut root, _, accounts, _) = opened(false, Step::Model { way: None });
        answer(&mut root, false);
        let mut document = playground::accounts(Some(18), line_usable, false);
        document.usable = document_usable;
        deliver(&mut root, accounts, c::Delivery::Accounts(Box::new(document)));
        assert_eq!(pages::welcome::model_usable(&root), Some(document_usable));
    }
}

#[test]
fn with_a_playground_model_usable_reads_other_providers_lines() {
    for (provider, document_usable, line_usable, expected) in
        [(18, true, false, false), (18, false, true, true), (17, true, true, false)]
    {
        let (mut root, _, accounts, _) = opened(false, Step::Model { way: None });
        answer(&mut root, true);
        let mut document = playground::accounts(Some(provider), line_usable, false);
        document.usable = document_usable;
        deliver(&mut root, accounts, c::Delivery::Accounts(Box::new(document)));
        assert_eq!(pages::welcome::model_usable(&root), Some(expected));
    }
}

#[test]
fn ready_and_the_strip_count_an_owners_deployment_model() {
    let (mut root, _, accounts, _) = opened(true, Step::Model { way: None });
    account(&mut root, accounts, 18, false);
    let mut out = playground::deliver_playground_read(
        &mut root,
        Readout::Family { family: wire::FORGE, record: Some(playground::answer(b"Change the title")) },
    );
    let watch = playground::deployment_watch(&mut out).expect("owner start opens the deployment watch");
    let mut out = playground::deliver_deployment(
        &mut root,
        watch,
        playground::deployment(&[playground::PLAYGROUND_ACCOUNT, playground::OTHER_ACCOUNT]),
    );
    assert!(playground::removed(&mut out).is_empty(), "removal waits for the running read");
    assert_eq!(pages::welcome::model_usable(&root), Some(true));

    let mut out = playground::deliver_playground_read(
        &mut root,
        Readout::Family { family: wire::FORGE, record: Some(playground::answer(b"Change the title")) },
    );
    assert!(playground::removed(&mut out).is_empty());
    assert_eq!(pages::welcome::model_usable(&root), Some(true), "the watch keeps the model after removal");
    playground::deliver_deployment(&mut root, watch, playground::deployment(&[playground::OTHER_ACCOUNT]));
    assert_eq!(pages::welcome::model_usable(&root), Some(true), "removed playground account changes nothing");

    let (mut member, _, accounts, _) = opened(false, Step::Model { way: None });
    answer(&mut member, true);
    account(&mut member, accounts, 18, false);
    assert_eq!(pages::welcome::model_usable(&member), Some(false));
}

fn ready_default() -> (Domain, Token) {
    let (mut root, me, accounts, forges) = opened(true, Step::Ready);
    answer(&mut root, false);
    account(&mut root, accounts, 18, true);
    forge_rows(&mut root, forges.unwrap(), Some(3), false);
    (root, me)
}
fn press_default(root: &mut Domain) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(Act::MakeDefault) })
}
fn default_answer(root: &mut Domain, outcome: jig_client::Outcome) -> Queue<Request> {
    let key = root.pages().default_project().making.expect("pending Default");
    let mut out = Queue::uncapped();
    crate::route::told(root, &f::env(&f::limits()), Told::Answered { key, outcome }, &mut out);
    out
}
#[test]
fn ready_makes_default_once_per_press_with_its_own_key() {
    let (mut root, _) = ready_default();
    let mut out = press_default(&mut root);
    let key = root.pages().default_project().making.unwrap();
    assert_eq!(key.asker, crate::owner::Owner::Pages.asker());
    let mut sends = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Down { down: Down::Send { request, key: name, .. } } } =
            request
        {
            assert_eq!(
                request,
                jig_client::Ask::Jig(Box::new(c::Ask::MakeProject {
                    name: c::Text::new(b"Default".as_slice().into(), 64).unwrap(),
                    policy: 1,
                }))
            );
            assert_eq!(
                name,
                crate::key::named(crate::key::TOOLS, &crate::key::Name::new(b"welcome-default-1").unwrap())
            );
            sends += 1;
        }
    }
    assert_eq!(sends, 1);
    assert!(press_default(&mut root).is_empty());
    go(&mut root, Step::Model { way: None });
    go(&mut root, Step::Ready);
    assert!(press_default(&mut root).is_empty());
    assert_eq!(root.pages().default_project().making, Some(key));
}
#[test]
fn default_made_opens_chats_and_waits_for_its_membership() {
    let (mut root, me) = ready_default();
    press_default(&mut root);
    assert!(moved(
        default_answer(
            &mut root,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::ProjectMade { project: 77 }
            )),))
        ),
        false
    ));
    assert_eq!(root.pages().default_project().project, Some(77));
    let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
    frame.projects.get_mut(0).unwrap().project = 77;
    event(&mut root, Event::Went { place: Place::Address(Address::Chats { filter: ChatsFilter::All, start: false }) });
    deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
    assert_eq!(root.pages().project(), Some(77));
    assert_eq!(root.pages().default_project().project, None);
}
#[test]
fn default_name_taken_opens_only_the_persons_default() {
    for own_default in [false, true] {
        let (mut root, me) = ready_default();
        if own_default {
            let mut frame = pages::accounts::current_me(root.client()).unwrap().clone();
            frame.projects.get_mut(0).unwrap().name =
                c::DocumentText::complete(c::Text::new(b"Default".as_slice().into(), 64).unwrap());
            deliver(&mut root, me, c::Delivery::Me(Box::new(frame)));
        }
        press_default(&mut root);
        let out = default_answer(
            &mut root,
            jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::NameTaken,
                at: None,
            })),
        );
        assert_eq!(moved(out, false), own_default);
        assert_eq!(
            root.pages().default_project().ended,
            if own_default { None } else { Some(pages::welcome::Ended::Refused(c::Refusal::NameTaken)) }
        );
    }
}
#[test]
fn default_refused_or_unknown_offers_a_new_attempt_without_sending_anything() {
    for outcome in [
        jig_client::Outcome::Unknown,
        jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Provider,
            at: None,
        })),
    ] {
        let (mut root, _) = ready_default();
        press_default(&mut root);
        let key = root.pages().default_project().making.unwrap();
        assert!(default_answer(&mut root, outcome).is_empty());
        assert!(root.pages().default_project().ended.is_some());
        assert!(root.pages().default_project().making.is_none());
        press_default(&mut root);
        assert_eq!(root.pages().default_project().attempt, 2);
        assert_ne!(root.pages().default_project().making, Some(key));
        assert!(root.pages().default_project().ended.is_none());
    }
}
#[test]
fn default_is_refused_until_both_forge_and_model_are_done() {
    let (mut root, _, accounts, forges) = opened(true, Step::Ready);
    answer(&mut root, false);
    account(&mut root, accounts, 18, true);
    forge_rows(&mut root, forges.unwrap(), None, false);
    assert!(press_default(&mut root).is_empty());
    account(&mut root, accounts, 18, false);
    rows(&mut root, forges.unwrap(), 3);
    assert!(press_default(&mut root).is_empty());
    assert_eq!(root.pages().default_project().attempt, 0);
}
