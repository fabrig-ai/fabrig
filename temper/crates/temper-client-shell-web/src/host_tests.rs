//! The host owns secret/clearing/target evidence; Chromium owns navigation and re-entry.
use super::*;
use jig_client::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Token};
use skein_ui::{EntryKind, Role};
#[path = "host_tests_approve.rs"]
mod approve;
#[path = "host_tests_authorize.rs"]
mod authorize;
#[path = "host_tests_closing.rs"]
mod closing;
#[path = "host_tests_reload.rs"]
mod reload;
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
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: true } });
    let me = Token::new(0);
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
fn batch_admitted(host: &mut Host, exchange: u32, body: &[u8]) {
    let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(body)).expect("batch");
    host.heard(Heard::Begun { exchange, status: 200 });
    let mut complete = true;
    for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
        let reply = match member {
            w::Member::Open(_) => {
                w::OpenAnswer::Admitted.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")
            }
            w::Member::Close(_) => {
                w::CloseAnswer::Closed.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")
            }
            w::Member::Read(_) => {
                complete = false;
                continue; // These fixtures leave lone reads unanswered too.
            }
        };
        host.heard(batch_part(exchange, place, reply));
    }
    if complete {
        host.heard(Heard::Done { exchange });
    }
}
fn batch_part(exchange: u32, place: usize, reply: Box<[u8]>) -> Heard {
    let part = w::BatchReply { member: u16::try_from(place).expect("member"), reply: Some(reply) };
    let bytes = part.encode(&w::CEILINGS).expect("bounded fixture encoding");
    let mut frame = Vec::new();
    wire_piece(&mut frame, &bytes);
    Heard::Part { exchange, bytes: frame.into_boxed_slice() }
}

#[test]
fn the_reload_press_asks_the_browser_to_reload_once() {
    let mut host = fixture();
    host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: false } });
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
    assert_eq!(host.href(link).as_deref(), Some(b"/home".as_slice()));
    host.went(b"/welcome?at=forge");
    build(&mut host);
    let link = node(&host, Role::Link, b"temper");
    host.act(link, Act::Press);
    host.requests();
    assert_eq!(host.show, Some((Box::from(b"/home".as_slice()), false)));
}

#[test]
fn a_filter_chosen_pushes_the_inbox_at_its_query() {
    let mut host = fixture_at(b"/needs");
    host.went(b"/needs?kind=question");
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
    let mut host = fixture_at(b"/needs?kind=question");
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
    let mut host = fixture_at(b"/needs?kind=reply&project=3");
    build(&mut host);
    assert!(
        host.view
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Main && node.styles.as_slice().contains(&temper_client_view::look::NEEDS))
    );
    assert_eq!(host.address, Address::Inbox { filter: Filter { project: Some(3), ..Filter::ALL } });
    assert_eq!(host.show, Some((Box::from(b"/needs?project=3".as_slice()), true)));
}

#[test]
fn the_web_preset_sends_a_whole_suggested_request() {
    assert!(limits::client().client.domain.text >= temper_client_protocol_wire::CUE_BYTES);
}

// Wire fixtures keep the regression on Host's linked root and public transport surface.

fn owned(record: Box<[u8]>) -> jig_client_protocol_wire::Owned {
    use jig_client_protocol_wire as w;
    use jig_client_protocol_wire::VERSION;
    w::Owned { owner: w::Owner::Jig, version: VERSION, record }
}
fn streamed(host: &mut Host, event: &w::StreamEvent) {
    host.heard(Heard::Piece { bytes: stream_bytes(event) });
}
fn stream_bytes(event: &w::StreamEvent) -> Box<[u8]> {
    use skein_lib::base64::{self, Alphabet};
    let bytes = event.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding");
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(bytes.len()).expect("fixture fits"))
        .expect("base64 fits");
    let mut writer = skein_lib::Writer::new(usize::try_from(size).expect("fixture fits"));
    base64::encode(Alphabet::Standard, &bytes, &mut writer).expect("measured base64");
    let mut piece = b"data: ".to_vec();
    piece.extend_from_slice(&writer.finish());
    piece.extend_from_slice(b"\n\n");
    piece.into_boxed_slice()
}
#[test]
fn the_first_task_activity_answer_survives_transport_sign_in() {
    use jig_client_protocol_wire as w;
    use jig_client_protocol_wire::VERSION;
    use skein_lib::{Duration, Reader};
    use temper_client_protocol_wire as wire;
    // task_record's chip is task 7; the answer carries one committed run marker.
    const TASK_PATH: &[u8] = b"/tasks/7";
    const RUN: u64 = 44;
    const ONE_LINE: u32 = 1;
    let mut host =
        Host::new(crate::limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, temper_client_view::sign_in::Ways::NONE);
    host.start(TASK_PATH, Vec::new(), None);
    assert!(host.requests().iter().any(|request| matches!(request, Asked::OpenStream { .. })));
    host.heard(Heard::StreamOpened);
    let served = List::from_boxed(Box::new([
        w::Served { owner: w::Owner::Jig, first: VERSION, last: VERSION },
        w::Served {
            owner: w::Owner::Family(w::Family { number: wire::FORGE }),
            first: wire::CLIENT_VERSION,
            last: wire::CLIENT_VERSION,
        },
    ]));
    streamed(
        &mut host,
        &jig_client_test_records::opened_with_layouts(
            w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
            jig_client_test_records::frame_record(),
            &[jig_client_test_records::family_layout(wire::FORGE, wire::CLIENT_LAYOUT)],
        ),
    );
    let mut activity = None;
    let mut watch = None;
    let me = 0;
    for request in host.requests() {
        let Asked::Exchange { id, call, .. } = request else { continue };
        if call.path.as_ref() != b"/jig/v1/batch" {
            continue;
        }
        let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("startup batch");
        for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
            match member {
                w::Member::Read(read) if read.owner == w::Owner::Jig => {
                    let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(&read.record)).expect("read");
                    if matches!(read, w::Read::Activity(_)) {
                        activity = Some((id, place));
                    }
                }
                w::Member::Open(open) if open.subject.owner == w::Owner::Jig => {
                    let subject =
                        w::Subject::decode(&w::CEILINGS, &mut Reader::new(&open.subject.record)).expect("subject");
                    if matches!(subject, w::Subject::Task(_)) {
                        watch = Some(open.watch);
                    }
                }
                w::Member::Read(_) | w::Member::Open(_) | w::Member::Close(_) => {}
            }
        }
    }
    let (exchange, place) = activity.expect("first activity read left the host");
    let watch = watch.expect("task watch");
    let mut task = jig_client_test_records::task_record();
    task.chip = jig_client_test_records::owner_waiting_chip_record();
    let document = w::DeliveryValue::Task(Box::new(task));
    streamed(
        &mut host,
        &w::StreamEvent::Delivery(w::Delivery {
            watch,
            document: owned(document.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")),
        }),
    );
    let event =
        w::Activity::RunBegan(w::ActivityRunBegan { attempt: RUN, why: w::RunWhy::Made, resumed: true, fresh: None });
    let source = w::ActivityLine { task: 7, position: 1, at: 1, line: event };
    let line = w::DecodedActivityLine { source, decoded: None };
    let page = jig_client_test_records::activity_rows(&[line]);
    let value = w::ReadValue::Activity(page);
    let answer = w::ReadAnswer::Value(owned(
        value.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding"),
    ));
    host.heard(Heard::Begun { exchange, status: 200 });
    host.heard(batch_part(exchange, place, answer.encode(&w::CEILINGS).expect("bounded fixture encoding")));
    let document = w::DeliveryValue::Me(jig_client_test_records::frame_two_projects_record());
    streamed(
        &mut host,
        &w::StreamEvent::Delivery(w::Delivery {
            watch: me,
            document: owned(document.encode(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture encoding")),
        }),
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
    assert_ne!(host.patches().len(), 0);
    assert!(host.patches.is_empty());
    assert_eq!(host.patches.room(), host.bounds.tree.patches);
    host.went(b"/chats");
    assert_ne!(host.patches().len(), 0);
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
            .push(w::ForgeRow {
                catching_up: None,
                starts_with: w::StartsWith::Repository,
                makings: skein_lib::List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
                web: web.map(Into::into),
                repositories: List::with_capacity(0),
                app: Some(w::AppWay::Pasted),
                number: 2,
                kind: w::Kind::Github,
                mode: w::Mode::Connected,
                connection: w::Connection::Connected(w::Connected { id: 9, login: b"my-app[bot]".as_slice().into() }),
                offer: w::Offer::NotOffered,
                first: None,
                installations: List::with_capacity(0),
                ended: None,
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
            })
            .expect("one forge");
        let mut host = fixture_forges(Some(&w::Forges { forges, full: List::with_capacity(0), linked: false }));
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
        let record = w::StreamsFull { bound, size: 2 };
        let mut writer = skein_lib::Writer::new(
            usize::try_from(record.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                .expect("fixed refusal"),
        );
        record.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("refusal bytes");
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

#[test]
fn root_and_missing_starts_wait_for_me_and_replace_even_the_provisional_home() {
    for path in [b"/".as_slice(), b"/nowhere"] {
        for projects in [0, 1, 2] {
            let mut host = Host::new(limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
            host.root = root::unlinked(&host.env.limits, 11);
            host.start(path, Vec::new(), None);
            host.client(jig_client::Event::StartUnlinked { kept: List::uncapped() });
            host.client(jig_client::Event::SignIn { with: jig_client::With::Transport });
            host.client(jig_client::Event::Up { up: jig_client::Up::Served { served: true } });
            assert_eq!(host.report(Report::Start).show, None);
            build(&mut host);
            assert_eq!(host.root.pages().address(), &Address::AllHome);
            assert!(!host.view.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&view::look::NEEDS)));
            let link = jig_client_protocol_link::Limits::new(host.env.limits.client.domain, limits::EXCHANGES).unwrap();
            let mut me = jig_client_protocol_link::translate::documents::frame(
                jig_client_test_records::frame_two_projects_record(),
                &link,
            )
            .unwrap();
            let project = me.projects.get(0).unwrap().clone();
            me.projects = List::with_capacity(2);
            for n in 0..projects {
                let mut project = project.clone();
                project.project = 1 + n;
                me.projects.push(project).unwrap();
            }
            host.client(jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: Token::new(0),
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            });
            let expected = if projects == 1 { b"/projects/1".as_slice() } else { b"/home".as_slice() };
            assert_eq!(host.report(Report::Start).show, Some((Box::from(expected), true)), "{path:?}, {projects}");
            assert_eq!(host.report(Report::Start).show, None);
        }
    }
}

const PLAYGROUND_PROJECT: u32 = 7;
const PLAYGROUND_NAME: &[u8] = b"playground/widget";

fn wire_piece(pieces: &mut Vec<u8>, bytes: &[u8]) {
    pieces.extend_from_slice(&u32::try_from(bytes.len()).expect("fixture piece length").to_be_bytes());
    pieces.extend_from_slice(bytes);
}

fn playground_me(client: &jig_client::DomainLimits) -> w::Me {
    // The box can fill the client's short template list and its document text (client/view.md, 4).
    let mut templates = Vec::new();
    let template_name = vec![b'x'; usize::try_from(client.text).expect("document text bound")];
    for (offset, name) in (0..client.rows).zip(std::iter::repeat(template_name.as_slice())) {
        let number = jig_client::shapes::CHAT + offset;
        let template = w::Template {
            number,
            name: jig_client_test_records::wire_text(name),
            executor: w::TemplateExecutor::Charter(w::TemplateCharter { value: number }),
            contract: w::TaskContractDocument::Report(w::TaskContractDocumentReport { words: client.text }),
            shallower: 1,
            budget: w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap: None }),
            deadline: w::TemplateDeadline::None,
            tracked: None,
            flags: w::TaskFlags {
                conversation: true,
                listed: false,
                retention: w::Retention::Kept,
                ended_by: w::EndedBy::Requester,
            },
            label: number,
            grace: None,
            llm: None,
            resources: List::with_capacity(0),
        };
        wire_piece(&mut templates, &template.encode(&w::CEILINGS).expect("box template"));
    }
    let project = w::MeProject {
        left_out: 0,
        project: PLAYGROUND_PROJECT,
        name: jig_client_test_records::wire_text(PLAYGROUND_NAME),
        role: w::HeldRole::Owner,
        pool: None,
        templates: templates.into_boxed_slice(),
        creating: None,
        models: List::with_capacity(0),
        defaults: List::with_capacity(0),
        public: None,
    };
    let mut projects = Vec::new();
    wire_piece(&mut projects, &project.encode(&w::CEILINGS).expect("box project"));
    let mut me = jig_client_test_records::frame_record();
    me.projects = projects.into_boxed_slice();
    me.deployment_role = Some(w::HeldRole::Owner);
    me.party.name = jig_client_test_records::wire_text(b"Owner");
    me
}

fn fresh_chats_report(host: &mut Host, report: Report, linked: bool) -> Duties {
    let duties = host.report(report);
    build(host);
    let nodes = host.tree().nodes();
    assert!(!nodes.iter().any(|node| node.styles.as_slice().contains(&view::look::BEHIND)), "never Behind");
    if linked {
        let foot =
            nodes.iter().position(|node| node.styles.as_slice().contains(&view::look::RAIL_FOOT)).expect("rail foot");
        let end = foot + usize::try_from(nodes[foot].size).expect("foot subtree size");
        let at = (foot + 1..end)
            .find(|at| nodes[*at].region == Some(view::frame::link_region()))
            .expect("link region in foot");
        let status = &nodes[at + 1];
        assert_eq!(nodes[at].size, 1 + status.size, "the link region holds only its own status");
        assert_eq!(status.role, Role::Status);
        assert_eq!(status.name.as_deref(), Some(b"Live".as_slice()));
        assert!(status.styles.as_slice().contains(&view::look::LIVE));
    }
    duties
}

#[test]
fn a_fresh_load_of_a_playgrounds_chats_stays_live() {
    use skein_lib::Duration;
    use temper_client_protocol_wire as wire;
    let bounds = limits::client_for(true);
    let frame = playground_me(&bounds.client.domain);
    let mut host = Host::new(bounds, 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
    host.start(b"/chats", Vec::new(), None);
    fresh_chats_report(&mut host, Report::Start, false);
    fresh_chats_report(&mut host, Report::Heard(Heard::StreamOpened), false);
    let served = List::from_boxed(Box::new([
        w::Served { owner: w::Owner::Jig, first: w::VERSION, last: w::VERSION },
        w::Served {
            owner: w::Owner::Family(w::Family { number: wire::FORGE }),
            first: wire::CLIENT_VERSION,
            last: wire::CLIENT_VERSION,
        },
    ]));
    let opened = jig_client_test_records::opened_with_layouts(
        w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
        frame,
        &[jig_client_test_records::family_layout(wire::FORGE, wire::CLIENT_LAYOUT)],
    );
    let bytes = stream_bytes(&opened);
    let mut pieces = bytes.chunks(usize::try_from(limits::PIECE).expect("stream chunk size"));
    let last = pieces.next_back().expect("opened frame has bytes");
    for bytes in pieces {
        fresh_chats_report(&mut host, Report::Heard(Heard::Piece { bytes: bytes.into() }), false);
    }
    let duties = fresh_chats_report(&mut host, Report::Heard(Heard::Piece { bytes: last.into() }), true);
    let watch = admit_chats(&mut host, duties);

    let mut row = jig_client_test_records::task_line_record();
    row.project = PLAYGROUND_PROJECT;
    row.requester = w::Requester::Person(w::RequesterPerson { value: 7 }); // frame_record's person.
    row.label = jig_client::shapes::CHAT;
    row.flags.conversation = true;
    // The live list requests the forge family's chat part, even for a chat with no draft.
    row.parts = List::from_boxed(Box::new([w::RowPart::Unknown]));
    let mut rows = Vec::new();
    wire_piece(&mut rows, &row.encode(&w::CEILINGS).expect("chat row"));
    let document = w::DeliveryValue::Tasks(w::Tasks { ended_mark: 0, rows: rows.into_boxed_slice(), more: false });
    let delivered = w::StreamEvent::Delivery(w::Delivery {
        watch,
        document: owned(document.encode(&w::CEILINGS).expect("chat list")),
    });
    fresh_chats_report(&mut host, Report::Heard(Heard::Piece { bytes: stream_bytes(&delivered) }), true);
    assert!(
        host.tree().nodes().iter().any(|n| n.role == Role::List && n.name.as_deref() == Some(b"Your chats")),
        "the live list is populated: address={:?}, live={:?}",
        host.root.pages().address(),
        host.root.pages().chats().and_then(|chats| chats.live(host.root.client()))
    );
    assert!(
        host.tree().nodes().iter().any(|n| n.role == Role::Link && n.name.as_deref() == Some(b"T7")),
        "the list shows the fixture chat's link"
    );
}

fn admit_chats(host: &mut Host, duties: Duties) -> u64 {
    let (exchange, batch) = duties
        .asked
        .into_iter()
        .find_map(|asked| {
            let Asked::Exchange { id, call, .. } = asked else { return None };
            (call.path.as_ref() == b"/jig/v1/batch")
                .then(|| (id, w::Batch::decode(&w::CEILINGS, &mut Reader::new(&call.body)).expect("startup batch")))
        })
        .expect("first page's watches and reads leave in a batch");
    fresh_chats_report(host, Report::Heard(Heard::Begun { exchange, status: 200 }), true);
    let mut watch = None;
    for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
        let reply = match member {
            w::Member::Open(open) => {
                if open.subject.owner == w::Owner::Jig
                    && let w::Subject::Tasks(tasks) =
                        w::Subject::decode(&w::CEILINGS, &mut Reader::new(&open.subject.record)).expect("watch subject")
                    && tasks.query.project == Some(PLAYGROUND_PROJECT)
                    && tasks.query.parts.iter().any(|part| part.kind == temper_client_protocol_wire::CHAT_PART)
                {
                    assert!(watch.replace(open.watch).is_none(), "one watch for the live chats");
                }
                w::OpenAnswer::Admitted.encode(&w::CEILINGS)
            }
            w::Member::Read(read) => playground_read(read).encode(&w::CEILINGS),
            w::Member::Close(_) => panic!("fresh load opens its watches"),
        }
        .expect("startup batch member reply");
        fresh_chats_report(host, Report::Heard(batch_part(exchange, place, reply)), true);
    }
    fresh_chats_report(host, Report::Heard(Heard::Done { exchange }), true);
    assert!(host.root.forge().playground_running(), "the host has the running playground's fact");
    watch.expect("the fresh page watches the playground chats")
}

fn playground_read(read: w::Owned) -> w::ReadAnswer {
    use temper_client_protocol_wire::{self as wire, v1::Reading};
    if read.owner == w::Owner::Family(w::Family { number: wire::FORGE }) {
        let request = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(&read.record)).expect("family scope");
        let reading = Reading::decode(&wire::v1::CEILINGS, &mut Reader::new(&request.record)).expect("forge read");
        if reading.read == wire::v1::Read::Playground {
            let answer = wire::v1::PlaygroundAnswer {
                forge: 1,
                project: Some(PLAYGROUND_PROJECT),
                provider: 1,
                owner: b"playground".as_slice().into(),
                name: b"widget".as_slice().into(),
                state: wire::v1::PlaygroundState::Running,
                cues: List::with_capacity(0),
            };
            return w::ReadAnswer::Value(w::Owned { record: wire::playground_answer(&answer), ..read });
        }
    }
    // The other reads don't supply the live chats watch or its frame.
    w::ReadAnswer::None(w::ReadAnswerNone { value: w::ReadNone::Unknown })
}

#[test]
fn an_old_root_filter_replaces_at_once_but_a_needs_filter_stays() {
    for (path, replacement) in [
        (b"/?kind=question&goal=9".as_slice(), Some((Box::from(b"/needs?kind=question&goal=9".as_slice()), true))),
        (b"/needs?kind=question", None),
    ] {
        let mut host = Host::new(limits::client(), 11, Time::ZERO, Wall::EPOCH, 0, view::sign_in::Ways::NONE);
        host.start(path, Vec::new(), None);
        assert_eq!(host.report(Report::Start).show, replacement);
        assert_eq!(host.root.pages().inbox().unwrap().filter.kind, Some(root::address::Kind::Question));
        assert_eq!(
            host.root.pages().inbox().unwrap().filter.goal,
            if path.starts_with(b"/?") { Some(9) } else { None }
        );
    }
}
