//! The host owns secret/clearing/target evidence; Chromium owns navigation and re-entry.
use super::*;
use jig_client::contract as c;
use skein_ui::{EntryKind, Role};
#[path = "host_tests_approve.rs"]
mod approve;
#[path = "host_tests_authorize.rs"]
mod authorize;
#[path = "host_tests_tab.rs"]
mod tab;
const KEY: &[u8] = b"private-host-key";
const ADDRESS: &[u8] = b"http://127.0.0.1:9/authorize?state=test";
fn fixture() -> Host {
    fixture_forges(None)
}
fn fixture_forges(forges: Option<&temper_client_protocol_wire::v1::Forges>) -> Host {
    let mut host =
        Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.root = root::unlinked(&host.env.limits, 11);
    host.event(Event::Start { address: Address::Accounts, kept: List::with_capacity(0), with: None });
    host.client(jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    host.client(jig_client::Event::SignIn { with: jig_client::With::Transport });
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } });
    let me = host
        .out
        .iter()
        .find_map(|request| match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                    },
            } => Some(*watch),
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Push(_)
            | Request::Fact { .. } => None,
        })
        .expect("me watch");
    let link =
        jig_client_protocol_link::Limits::new(host.env.limits.client.domain, limits::EXCHANGES).expect("link bounds");
    let mut frame =
        jig_client_protocol_link::translate::documents::frame(jig_client_test_records::frame_record(), &link)
            .expect("frame");
    frame.providers = List::with_capacity(1);
    frame
        .providers
        .push(c::ProviderOffer {
            provider: 17,
            name: c::DocumentText::complete(
                c::Text::new(Box::from(b"openai".as_slice()), 32).expect("provider name fits"),
            ),
            key_bytes: 4096,
            sign_in: true,
            signed_in_plan: true,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            manage_usage: None,
        })
        .expect("one offered provider");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Watched { watch: me, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(frame))) },
    });
    if let Some(forges) = forges {
        deliver_forges(&mut host, forges);
    }
    host.requests();
    host
}
fn deliver_forges(host: &mut Host, forges: &temper_client_protocol_wire::v1::Forges) {
    let watch = host
        .out
        .iter()
        .find_map(|request| match request {
            Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Family { .. } },
                    },
            } => Some(*watch),
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Push(_)
            | Request::Fact { .. } => None,
        })
        .expect("forges watch");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: temper_client_protocol_wire::FORGE,
                version: temper_client_protocol_wire::CLIENT_VERSION,
                bytes: temper_client_protocol_wire::forges(forges),
            },
        },
    });
}
fn fixture_at(path: &[u8]) -> Host {
    let mut host = fixture();
    host.went(path);
    host.requests();
    host
}
fn build(host: &mut Host) -> Vec<Patch> {
    host.patches()
}
fn node(host: &Host, role: Role, name: &[u8]) -> NodeId {
    host.view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == role && node.name.as_deref() == Some(name))
        .expect("account control")
        .id
}
#[test]
fn the_reload_press_asks_the_browser_to_reload_once() {
    let mut host = fixture();
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } });
    build(&mut host);
    let reload = node(&host, Role::Button, b"Reload");
    let idle = host.report(Report::Start);
    assert!(!idle.reload);
    let duties = host.report(Report::Act { node: reload, act: Act::Press });
    assert!(duties.reload);
    assert!(duties.departs());
    assert!(duties.navigate.is_none());
    assert!(duties.replace.is_none());
    assert!(!host.report(Report::Start).reload);
}

fn input(host: &mut Host, name: &[u8], value: &[u8]) -> Vec<Patch> {
    let field = node(host, Role::TextField, name);
    host.act(field, Act::Input { value: Box::from(value) });
    build(host)
}
fn send(host: &mut Host, name: &[u8]) -> Token {
    let button = node(host, Role::Button, name);
    host.act(button, Act::Press);
    host.client(jig_client::Event::Stored);
    host.out
        .iter()
        .find_map(|request| match request {
            Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } => Some(*attempt),
            Request::Client { .. }
            | Request::Unlinked { .. }
            | Request::Show { .. }
            | Request::Leave { .. }
            | Request::SignOut
            | Request::Dropped { .. }
            | Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::KeepSignIn { .. }
            | Request::Tab { .. }
            | Request::Copy { .. }
            | Request::Push(_)
            | Request::Fact { .. } => None,
        })
        .expect("account request left the host")
}
#[test]
fn the_key_patch_declares_a_secret_entry() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let way = node(&host, Role::Link, b"Add key");
    host.act(way, Act::Press);
    host.requests();
    let patches = build(&mut host);
    let key = node(&host, Role::TextField, b"Key (pasted once)");
    let made =
        patches
            .iter()
            .find_map(|patch| {
                if let Patch::Insert { nodes, .. } = patch { nodes.iter().find(|made| made.id == key) } else { None }
            })
            .expect("inserted key field");
    assert_eq!(made.attribute.entry, Some(EntryKind::Secret));
    assert!(made.value.as_ref().expect("empty field").text.is_empty());
    let patches = input(&mut host, b"Key (pasted once)", KEY);
    assert!(!format!("{patches:?} {:?}", host.view).contains("private-host-key"));
}
#[test]
fn sending_a_key_emits_the_empty_value_patch() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let way = node(&host, Role::Link, b"Add key");
    host.act(way, Act::Press);
    host.requests();
    build(&mut host);
    input(&mut host, b"Key (pasted once)", KEY);
    let key = node(&host, Role::TextField, b"Key (pasted once)");
    send(&mut host, b"Add key");
    let patches = build(&mut host);
    assert!(
        patches
            .iter()
            .any(|patch| matches!(patch,Patch::Value {node,value:Some(value)} if *node==key && value.text.is_empty()))
    );
}
#[test]
fn a_key_never_reaches_the_small_store_output() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let way = node(&host, Role::Link, b"Add key");
    host.act(way, Act::Press);
    host.requests();
    build(&mut host);
    input(&mut host, b"Key (pasted once)", KEY);
    send(&mut host, b"Add key");
    let requests = host.requests();
    assert!(
        !requests.iter().any(|request| matches!(request, Asked::Save { .. })),
        "carried requests are not small-store writes"
    );
}
#[test]
fn the_external_href_is_the_supplied_sign_in_address() {
    let mut host = fixture();
    host.went(b"/accounts/connect");
    build(&mut host);
    let attempt = send(&mut host, b"Continue with ChatGPT");
    host.client(jig_client::Event::Up {
        up: jig_client::Up::Answer {
            attempt,
            envelope: skein_lib::RequestEnvelope::Final,
            answer: Some(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(
                c::DocumentOutcome::AccountSignInStarted { source: 43, address: Box::from(ADDRESS) },
            )))),
            why: None,
        },
    });
    build(&mut host);
    let link = host
        .view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.external.as_deref() == Some(ADDRESS))
        .expect("sign-in address")
        .id;
    assert_eq!(host.href(link).as_deref(), Some(ADDRESS));
}
#[test]
fn the_brand_href_navigates_inside_the_app() {
    let mut host = fixture_at(b"/welcome");
    build(&mut host);
    let link = node(&host, Role::Link, b"temper");
    assert_eq!(host.href(link).as_deref(), Some(b"/".as_slice()));
    host.went(b"/welcome?at=forge");
    build(&mut host);
    let link = node(&host, Role::Link, b"temper");
    host.act(link, Act::Press);
    host.requests();
    assert_eq!(host.show, Some((Box::from(b"/".as_slice()), false)));
}

#[test]
fn a_filter_chosen_pushes_the_inbox_at_its_query() {
    let mut host = fixture_at(b"/");
    host.went(b"/?kind=question");
    build(&mut host);
    assert_eq!(
        host.address,
        Address::Inbox { filter: Filter { kind: Some(root::address::Kind::Question), ..Filter::ALL } }
    );
    assert!(
        host.view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Main && node.styles.as_slice().contains(&temper_client_view::look::NEEDS))
    );
    assert!(
        !host
            .view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Button && node.name.as_deref() == Some(b"All kinds"))
    );
}
#[test]
fn a_load_at_a_filtered_address_opens_the_inbox_filtered() {
    let mut host = fixture_at(b"/?kind=question");
    build(&mut host);
    assert_eq!(
        host.address,
        Address::Inbox { filter: Filter { kind: Some(root::address::Kind::Question), ..Filter::ALL } }
    );
    assert_eq!(host.show, None);
    assert!(
        host.view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Main && node.styles.as_slice().contains(&temper_client_view::look::NEEDS))
    );
}
#[test]
fn a_stale_filter_link_opens_less_filtered_at_its_printed_address() {
    let mut host = fixture_at(b"/?kind=reply&project=3");
    build(&mut host);
    assert!(
        host.view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Main && node.styles.as_slice().contains(&temper_client_view::look::NEEDS))
    );
    assert_eq!(host.address, Address::Inbox { filter: Filter { project: Some(3), ..Filter::ALL } });
    assert_eq!(host.show, Some((Box::from(b"/?project=3".as_slice()), true)));
}

#[test]
fn the_web_preset_sends_a_whole_suggested_request() {
    assert!(limits::client().client.domain.text >= temper_client_protocol_wire::CUE_BYTES);
}

// Wire fixtures keep the regression on Host's linked root and public transport surface.
fn encoded(size: u32, write: impl FnOnce(&mut skein_lib::Writer) -> Result<(), skein_lib::Overflow>) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(size).expect("fixture fits"));
    write(&mut writer).expect("measured fixture");
    writer.finish()
}
fn owned(record: Box<[u8]>) -> jig_client_protocol_wire::v20::Owned {
    use jig_client_protocol_wire::{VERSION, v20 as w};
    w::Owned::new(&w::CEILINGS, w::OwnedParts { owner: w::Owner::Jig, version: VERSION, record })
        .expect("fixture envelope")
}
fn streamed(host: &mut Host, event: &jig_client_protocol_wire::v20::StreamEvent) {
    use skein_lib::base64::{self, Alphabet};
    let bytes = encoded(event.measure(), |writer| event.encode(writer));
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture fits"))
        .expect("base64 fits");
    let mut writer = skein_lib::Writer::new(usize::try_from(size).expect("fixture fits"));
    base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("measured base64");
    let mut piece = b"data: ".to_vec();
    piece.extend_from_slice(&writer.finish());
    piece.extend_from_slice(b"\n\n");
    host.heard(Heard::Piece { bytes: piece.into_boxed_slice() });
}
#[test]
#[expect(clippy::too_many_lines, reason = "one wire story delivers sign-in, task activity and me before drawing")]
fn the_first_task_activity_answer_survives_transport_sign_in() {
    use jig_client_protocol_wire::{VERSION, v20 as w};
    use skein_lib::{Duration, Reader};
    // task_record's chip is task 7; the answer carries one committed run marker.
    const TASK_PATH: &[u8] = b"/tasks/7";
    const RUN: u64 = 44;
    const ONE_LINE: u32 = 1;
    let mut host =
        Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.start(TASK_PATH, Vec::new(), None);
    let activity = host
        .requests()
        .into_iter()
        .find_map(|request| match request {
            Asked::Exchange { id, call, .. } if call.path.as_ref() == b"/jig/v1/read" => Some(id),
            Asked::Exchange { .. }
            | Asked::OpenStream { .. }
            | Asked::CloseStream
            | Asked::Save { .. }
            | Asked::Erase { .. } => None,
        })
        .expect("first activity read left the host");
    host.heard(Heard::StreamOpened);
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(&w::CEILINGS, w::ServedParts { owner: w::Owner::Jig, first: VERSION, last: VERSION })
                .expect("served version"),
        )
        .expect("one served family");
    streamed(
        &mut host,
        &w::StreamEvent::Hello(
            w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
                .expect("hello"),
        ),
    );
    let mut watch = None;
    let mut me = None;
    for request in host.requests() {
        let Asked::Exchange { call, .. } = request else { continue };
        if call.path.as_ref() != b"/jig/v1/watch" {
            continue;
        }
        let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("watch envelope");
        let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
        if matches!(subject, w::Subject::Task(_)) {
            watch = Some(open.watch());
        }
        if subject == w::Subject::Me {
            me = Some(open.watch());
        }
    }
    let watch = watch.expect("task watch");
    let mut task = jig_client_test_records::task_record().into_parts();
    task.chip = jig_client_test_records::owner_waiting_chip_record();
    let document = w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).expect("agent task")));
    streamed(
        &mut host,
        &w::StreamEvent::Delivery(
            w::Delivery::new(
                &w::CEILINGS,
                w::DeliveryParts {
                    watch,
                    document: owned(encoded(document.measure(), |writer| document.encode(writer))),
                },
            )
            .expect("task delivery"),
        ),
    );
    let event = w::Activity::RunBegan(
        w::ActivityRunBegan::new(
            &w::CEILINGS,
            w::ActivityRunBeganParts { attempt: RUN, why: w::RunWhy::Made, resumed: true },
        )
        .expect("run marker"),
    );
    let source = w::ActivityLine::new(&w::CEILINGS, w::ActivityLineParts { task: 7, position: 1, at: 1, line: event })
        .expect("source row");
    let line = w::DecodedActivityLine::new(&w::CEILINGS, w::DecodedActivityLineParts { source, decoded: None })
        .expect("activity row");
    let page = jig_client_test_records::activity_rows(&[line]);
    let value = w::ReadValue::Activity(page);
    let answer = w::ReadAnswer::Value(owned(encoded(value.measure(), |writer| value.encode(writer))));
    host.heard(Heard::Answered {
        exchange: activity,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
        headers: Vec::new(),
    });
    let document = w::DeliveryValue::Me(jig_client_test_records::frame_two_projects_record());
    streamed(
        &mut host,
        &w::StreamEvent::Delivery(
            w::Delivery::new(
                &w::CEILINGS,
                w::DeliveryParts {
                    watch: me.expect("me watch"),
                    document: owned(encoded(document.measure(), |writer| document.encode(writer))),
                },
            )
            .expect("me delivery"),
        ),
    );
    build(&mut host);
    let shown = host
        .view
        .tree()
        .nodes()
        .iter()
        .flat_map(|node| &node.text)
        .filter(|run| run.text.as_ref() == b"Run 44 began")
        .count();
    assert_eq!(shown, usize::try_from(ONE_LINE).expect("one line fits"), "first activity answer must reach the page");
}

#[test]
fn the_deadline_takes_the_earlier_domain_or_view_time_from_the_same_clocks() {
    let now = Time::from_nanos(100);
    let wall = Wall::from_nanos(1000);
    for (domain, next, expected) in [
        (Some(105), Some(1020), Some(105)),
        (Some(130), Some(1020), Some(120)),
        (None, Some(1020), Some(120)),
        (Some(130), None, Some(130)),
        (None, None, None),
        (None, Some(990), Some(100)),
    ] {
        assert_eq!(
            deadline(domain.map(Time::from_nanos), next.map(Wall::from_nanos), now, wall),
            expected.map(Time::from_nanos)
        );
    }
}

#[test]
fn the_patch_queue_is_reserved_once_at_start() {
    let mut host = fixture();
    assert!(!host.patches().is_empty());
    assert!(host.patches.is_empty());
    assert_eq!(host.patches.room(), host.bounds.tree.patches);
    host.went(b"/chats");
    assert!(!host.patches().is_empty());
    assert!(host.patches.is_empty());
    assert_eq!(host.patches.room(), host.bounds.tree.patches);
}

#[test]
fn a_flow_or_path_draws_on_temper_page_and_keeps_its_fallback() {
    use temper_client_view::paths::{self, Mark, Stop};
    const CASE: &[u8] = include_bytes!("../../../../skein/docs/design/draft/diagrams/how-hello-works.md");
    const SVG: &str = include_str!("../../../../skein/docs/design/draft/diagrams/how-hello-works.640.svg");
    const FALLBACK: &str = "Design draft, you write it; then Goal, you start it; then Checked changes, CI and a review; then main, the merge queue.";
    let stops = [
        Stop { mark: Mark::Person, words: b"you ask", needed: false, over: b"" },
        Stop { mark: Mark::Agent, words: b"produce", needed: false, over: b"" },
        Stop { mark: Mark::Code, words: b"check", needed: false, over: b"" },
        Stop { mark: Mark::Agent, words: b"repair", needed: false, over: b"" },
        Stop { mark: Mark::Code, words: b"check", needed: false, over: b"" },
        Stop { mark: Mark::Person, words: b"you approve", needed: false, over: b"" },
        Stop { mark: Mark::Code, words: b"update", needed: false, over: b"" },
        Stop { mark: Mark::Agent, words: b"resolve", needed: false, over: b"" },
        Stop { mark: Mark::Code, words: b"merge", needed: false, over: b"" },
        Stop { mark: Mark::Bar, words: b"main", needed: false, over: b"" },
    ];
    let bounds = fixture().bounds.tree;
    for flow in [true, false] {
        let mut view = skein_ui::View::new(&bounds);
        let mut builder = view.builder();
        builder.open(Role::Page);
        builder.open(Role::Main);
        if flow {
            skein_ui::markdown::read(CASE, 1, &mut builder);
        } else {
            paths::path(&mut builder, b"The way a change goes", &stops, None).expect("bounded path");
        }
        builder.close();
        builder.close();
        let mut out = Queue::with_capacity(bounds.patches);
        view.finish(builder, 0, &mut out);
        let mut patches = Vec::new();
        while let Some(patch) = out.pop() {
            patches.push(patch);
        }
        let drawing = view.tree().nodes().iter().find(|node| node.role == Role::Drawing).expect("drawing").id;
        let mut page = page(&bounds, Primary::Control);
        let mut ops = Vec::new();
        page.apply(&patches, &mut ops);
        assert!(!page.html().contains("<svg"), "fallback before measurement");
        for width in [640, 240, 640] {
            page.resized(drawing, width, 0, &mut ops);
            let html = page.html();
            assert_eq!(html.contains("<svg"), !flow || width == 640, "flow {flow}, width {width}");
            assert_eq!(html.contains("data-drawn="), !flow || width == 640);
            if flow {
                if width == 640 {
                    assert!(html.contains(SVG.strip_suffix('\n').expect("case's final newline")), "{html}");
                }
                assert!(html.contains(FALLBACK), "flow fallback at width {width}: {html}");
            } else {
                let list =
                    html.split("<ul ").nth(1).expect("fallback list").split('>').next().expect("list attributes");
                assert!(list.contains("aria-label=\"The way a change goes\""), "{html}");
                for words in ["you ask, a person", "produce, an agent", "check, code", "main, a branch"] {
                    assert!(html.contains(words), "path fallback at width {width}: {html}");
                }
                if width == 640 {
                    for name in ["main", "rise-person", "rise-agent"] {
                        assert!(html.contains(&format!("class=\"temper-ways-{name}\"")), "{html}");
                    }
                }
            }
        }
    }
}

#[test]
fn the_github_rows_external_web_link_has_its_address_and_noopener() {
    use temper_client_protocol_wire::v1 as w;
    for web in [None, Some(b"https://github.test/base/?a=1&b=2".as_slice())] {
        let mut forges = List::with_capacity(1);
        forges
            .push(
                w::ForgeRow::new(
                    &w::CEILINGS,
                    w::ForgeRowParts {
                        starts_with: w::StartsWith::Repository,
                        makings: skein_lib::List::with_capacity(0),
                        host: b"git.example:3000".as_slice().into(),
                        web: web.map(Into::into),
                        repositories: List::with_capacity(0),
                        app: Some(w::AppWay::Pasted),
                        number: 2,
                        kind: w::Kind::Github,
                        mode: w::Mode::Connected,
                        connection: w::Connection::Connected(
                            w::Connected::new(
                                &w::CEILINGS,
                                w::ConnectedParts { id: 9, login: b"my-app[bot]".as_slice().into() },
                            )
                            .expect("connected App"),
                        ),
                        offer: w::Offer::NotOffered,
                        first: None,
                        installations: List::with_capacity(0),
                        ended: None,
                        owned: None,
                        admin: false,
                        read_only: false,
                        ended_stop: None,
                    },
                )
                .expect("forge row"),
            )
            .expect("one forge");
        let mut host = fixture_forges(Some(
            &w::Forges::new(&w::CEILINGS, w::ForgesParts { forges, linked: false }).expect("forges"),
        ));
        host.went(b"/forges");
        let mut page = skein_ui_web::page::Page::new(
            skein_ui_web::Look {
                names: &[temper_client_view::look::NAMES],
                pictures: &[],
                components: &[],
                elements: temper_client_view::look::ELEMENTS,
                draw: |_, _, _, _, _| false,
            },
            &host.bounds.tree,
            b"temper",
            skein_ui_web::mapping::Primary::Control,
        );
        let mut ops = Vec::new();
        page.apply(&build(&mut host), &mut ops);
        for link in page.links().to_vec() {
            if let Some(address) = host.href(link) {
                page.set_href(link, &address, &mut ops);
            }
        }
        let html = page.html();
        let label = "Open on GitHub ↗";
        assert_eq!(html.contains(label), web.is_some());
        if web.is_some() {
            let at = html.find(label).expect("GitHub web link");
            let start = html[..at].rfind("<a ").expect("external anchor");
            let end = start + html[start..].find('>').expect("anchor attributes");
            let anchor = &html[start..end];
            assert!(anchor.contains("href=\"https://github.test/base/?a=1&amp;b=2\""), "{anchor}");
            let (_, attributes) = anchor.split_once("rel=\"").expect("external link rel");
            let (rel, _) = attributes.split_once('"').expect("closed rel attribute");
            assert!(rel.split_ascii_whitespace().any(|word| word == "noopener"), "{anchor}");
            assert!(anchor.contains("target=\"_blank\""), "{anchor}");
        }
    }
}

#[test]
fn the_ordinary_setting_keeps_the_whole_document_bound() {
    let client = limits::client_for(false);
    assert_eq!(client.window_bytes, None);
    assert_eq!(client.client.domain.long_text, 131_072);
    assert_eq!(client.client.domain.turn_text, 131_072);
}

#[test]
fn the_small_window_setting_names_a_valid_narrower_window() {
    let client = limits::client_for(true);
    assert_eq!(client.window_bytes, Some(32_768));
    assert_eq!(client.client.domain.long_text, 32_768);
    assert_eq!(client.client.domain.turn_text, 32_768);
    assert!(limits::valid(&client));
}

#[test]
fn validity_accepts_stock_limits_and_refuses_invalid_edges() {
    let mut client = limits::client();
    assert!(limits::valid(&client));
    client.forge.field_bytes = 6; // The client refuses a field too small for its form.
    assert!(!limits::valid(&client));
}

#[test]
fn a_stream_refused_full_reaches_the_client_link() {
    use jig_client_protocol_wire as w;
    for bound in [w::StreamBound::Component, w::StreamBound::Transport] {
        let mut host =
            Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
        host.start(b"/", Vec::new(), None);
        host.requests();
        host.heard(Heard::StreamRefused { status: 409 });
        let record = w::StreamsFull::new(&w::CEILINGS, w::StreamsFullParts { bound, size: 2 }).expect("full record");
        let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("fixed refusal"));
        record.encode(&mut writer).expect("refusal bytes");
        host.heard(Heard::Piece { bytes: writer.finish() });
        host.heard(Heard::StreamEnded);
        assert_eq!(host.root.client().link().streams_full.as_ref().map(|full| full.size), Some(2));
        assert!(host.requests().iter().any(|request| matches!(request, Asked::CloseStream)));
    }
}

#[path = "host_tests_copy.rs"]
mod copy;

#[test]
fn the_look_is_kept_and_full_screen_carried() {
    use root::pages::terminal::{Font, Look, Theme};
    let terminal = Look { terminal: true, theme: Theme::Dark, font: Font::DejaVu, night: false };
    let light = Look { terminal: false, theme: Theme::Light, font: Font::Plex, night: false };
    for (value, decoded) in [
        (Some(b"100".as_slice()), Some(terminal)),
        (Some(b"011".as_slice()), Some(light)),
        (Some(b"000".as_slice()), Some(Look::default())),
        (Some(b"111".as_slice()), Some(Look { terminal: true, ..light })),
        (Some(b"1".as_slice()), None),
        (Some(b"abc".as_slice()), None),
        (Some(b"1000".as_slice()), Some(terminal)),
        (Some(b"0110".as_slice()), Some(light)),
        (Some(b"1001".as_slice()), Some(Look { night: true, ..terminal })),
        (Some(b"0111".as_slice()), Some(Look { night: true, ..light })),
        (Some(b"0000".as_slice()), Some(Look::default())),
        (Some(b"1111".as_slice()), Some(Look { terminal: true, night: true, ..light })),
        (Some(b"10000".as_slice()), None),
        (Some(b"1002".as_slice()), None),
        (Some(b"000x".as_slice()), None),
        (Some(b"2000".as_slice()), None),
        (Some(b"0200".as_slice()), None),
        (Some(b"0020".as_slice()), None),
        (Some(b"200".as_slice()), None),
        (Some(b"020".as_slice()), None),
        (Some(b"002".as_slice()), None),
        (None, None),
    ] {
        assert_eq!(value.and_then(records::decode_look), decoded, "{value:?}");
        let mut host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        let stored = value
            .map(|value| skein_ui_web::store::Record { key: records::LOOK_KEY.into(), value: value.into() })
            .into_iter()
            .collect();
        host.start(b"/chats/new", stored, None);
        assert_eq!(host.root.pages().look(), decoded.unwrap_or_default(), "{value:?}");
        assert_eq!(host.night(), decoded.unwrap_or_default().night, "night restored before any report: {value:?}");
        let duties = host.report(Report::Start);
        assert_eq!(duties.keep_look, None, "start never writes a look");
        assert_eq!(duties.screen, None, "a reload never asks for full screen");
    }
    for (before, look, bytes) in [
        (terminal, Look { night: true, ..terminal }, *b"1001"),
        (Look { night: true, ..terminal }, terminal, *b"1000"),
        (light, Look { night: true, ..light }, *b"0111"),
        (Look { night: true, ..light }, light, *b"0110"),
    ] {
        assert_eq!(records::encode_look(look), bytes);
        let mut host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        host.event(Event::LookKept { look: before });
        host.event(Event::Act {
            action: root::Action::Pages(root::pages::Act::Terminal(root::pages::terminal::Act::Night)),
        });
        let duties = host.report(Report::Start);
        assert_eq!(host.night(), look.night);
        assert_eq!(duties.keep_look, Some(bytes));
        assert_eq!(duties.screen, None, "night asks no full screen");
        assert!(duties.asked.is_empty(), "the transport carries no kept write");
        assert_eq!(duties.writes, 0);
        assert_eq!(host.report(Report::Start).keep_look, None, "a duty is taken once");
    }
    for on in [true, false] {
        let mut host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        host.out.push(Request::Screen { on });
        let duties = host.report(Report::Start);
        assert_eq!(duties.screen, Some(on));
        assert!(duties.asked.is_empty());
        assert_eq!(host.report(Report::Start).screen, None);
    }
    for (report, listing) in [
        (skein_ui_web::screen::Report::Entered, Some(root::pages::terminal::Listing::Model)),
        (skein_ui_web::screen::Report::Left, None),
        (skein_ui_web::screen::Report::Refused, Some(root::pages::terminal::Listing::Model)),
    ] {
        let mut host = Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        host.event(Event::LookKept { look: terminal });
        host.event(Event::Act {
            action: root::Action::Pages(root::pages::Act::Terminal(root::pages::terminal::Act::Run {
                command: root::pages::terminal::Command::Model,
            })),
        });
        let duties = host.report(Report::Screen(report));
        assert_eq!(host.root.pages().listing(), listing, "{report:?}");
        assert_eq!(host.root.pages().look(), terminal, "leaving full screen keeps the terminal");
        assert_eq!(duties.screen, None);
        assert_eq!(duties.keep_look, None);
        assert!(duties.asked.is_empty());
    }
}
