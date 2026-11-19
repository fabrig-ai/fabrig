use crate::{
    limits,
    route::{self, Down},
};
use jig_client_protocol_link as link;
use skein_lib::{List, Queue, Time};
use temper_client_domain as client;
pub use temper_fixtures::client_documents as documents;
mod family;
pub(crate) mod fixtures;
mod render;

#[test]
fn a_jig_read_reaches_the_link_with_its_own_token() {
    let down = route::down(client::Request::Unlinked {
        request: jig_client::UnlinkedRequest::Down {
            down: jig_client::Down::Read {
                read: fixtures::READ,
                what: jig_client::Readable::Jig(fixtures::inbox_read()),
            },
        },
    });
    let Down::Unlinked(jig_client::UnlinkedRequest::Down {
        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(what) },
    }) = down
    else {
        panic!("jig read reaches its link");
    };
    assert_eq!(read, fixtures::READ);
    assert_eq!(what, fixtures::inbox_read());
}
#[test]
fn a_table_record_saved_then_kept_comes_back_the_same() {
    let limits = fixtures::link_limits();
    {
        let (name, record) = ("table", fixtures::table_record());
        let mut link = link::Link::new(&limits).expect("bounded store codec");
        let mut out = Queue::uncapped();
        link.step(
            Time::ZERO,
            link::Event::Save { record: link::Saved { version: 2, record: record.clone() } },
            &mut out,
        );
        let Some(link::Request::Save { record: link::Record { key, bytes } }) = out.pop() else {
            panic!("{name}: save writes");
        };
        let mut kept = List::with_capacity(1);
        kept.push(link::Record { key, bytes }).expect("one record fits");
        let mut restarted = link::Link::new(&limits).expect("bounded restore");
        restarted.step(Time::ZERO, link::Event::Start { kept }, &mut out);
        let Some(link::Request::Kept { record: saved }) = out.pop() else {
            panic!("{name}: record restored");
        };
        assert_eq!(Some(saved.record), Some(record), "{name}: unchanged record");
        assert!(out.is_empty(), "{name}: only one restored record");
    }
}
#[test]
fn the_native_limits_cover_wire_and_transport() {
    let limits = limits::native();
    // The stock's 21 executors and one deployment charter fit whole.
    assert_eq!(limits.client.client.domain.authority, 22);
    assert_eq!(limits.client.client.domain.rows, 16);
    assert_eq!(limits.client.client.domain.delegates, 80);
    let wire = client::forge::wire_limits(&limits.client.forge).expect("client wire bounds");
    let demo = temper_client_protocol_wire::v1::PlaygroundAnswer::encoded_bound(&wire).expect("demo bound");
    assert!(demo <= limits.client.client.domain.document_bytes, "family read answer fits");
    assert!(demo <= u64::from(limits.connections.body_bytes), "whole HTTP answer fits");
    assert!(limits.client.client.domain.text >= temper_client_protocol_wire::CUE_BYTES, "whole cue sends");
}

#[test]
fn an_equal_line_waits_once_in_the_order_it_came() {
    let mut lines = crate::terminal::waiting::Waiting::new();
    for line in [b"sign-in again".as_slice(), b"store failed", b"sign-in again", b"credential one", b"credential two"] {
        lines.push(Box::from(line));
    }
    let mut said = Vec::<Box<[u8]>>::new();
    lines.say(1, |line| said.push(Box::from(line)));
    assert_eq!(said, [Box::<[u8]>::from(b"sign-in again".as_slice())]);
    said.clear();
    lines.say(8, |line| said.push(Box::from(line)));
    assert_eq!(said, [b"store failed".as_slice(), b"credential one", b"credential two"].map(Box::<[u8]>::from));
    assert!(lines.is_empty());
}

#[test]
fn the_native_limits_make_the_terminal() {
    let (view, _) = limits::view(&limits::native()).expect("native view limits");
    let bounds = limits::terminal();
    let settings = skein_ui_text::Settings {
        name: Box::from(*b"temper"),
        longest: bounds.line,
        interval: bounds.interval,
        status: Some(temper_client_view::frame::link_region()),
        lines: bounds.lines,
        line_bytes: 166,
    };
    assert!(skein_ui_text::Terminal::operations(&settings, &view.tree).is_some(), "native terminal operations fit");
    assert!(skein_ui_text::Terminal::new(settings, &view.tree).is_some(), "native terminal settings fit");
}

#[test]
fn a_kept_credential_signs_in_held_with_no_exchange() {
    let mut link = link::Link::new(&fixtures::link_limits()).expect("bounded link");
    let mut out = Queue::uncapped();
    link.step(Time::ZERO, link::Event::SignIn { with: link::With::Held(fixtures::credential()) }, &mut out);
    assert_eq!(link.credential(), Some(&fixtures::credential()));
    assert!(out.is_empty(), "a held credential sends no sign-in exchange");
}

#[test]
fn validity_accepts_stock_limits_and_refuses_invalid_edges() {
    let mut limits = limits::native();
    assert!(limits::valid(&limits));
    limits.connections.body_bytes = u32::MAX; // Stream record + 512 overflows.
    assert!(!limits::valid(&limits));
}

#[test]
fn validity_refuses_unusable_io_and_overflowing_operation_counts() {
    let mut limits = limits::native();
    limits.io.refusals = 0;
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.io.retry = skein_lib::Duration::ZERO;
    assert!(!limits::valid(&limits));
    limits = limits::native();
    limits.io.sockets = u32::MAX;
    assert!(!limits::valid(&limits));
}

#[test]
fn the_native_preset_receives_every_supported_membership() {
    use jig_client_protocol_wire as w;
    use skein_lib::{Reader, Writer};
    // Small and Standard serve up to nine memberships (client/ctl/README.md, 9).
    for (count, accepted) in [(3, true), (9, true), (10, false)] {
        let mut parts = jig_client_test_records::frame_two_projects_record().into_parts();
        let mut reader = Reader::new(&parts.projects);
        let size = reader.u32().expect("membership size");
        let project = w::MeProject::decode(&w::CEILINGS, &mut Reader::new(reader.bytes(size).expect("membership")))
            .expect("membership record");
        let mut projects = Vec::new();
        for number in 1..=count {
            let mut row = project.clone().into_parts();
            row.project = number;
            let row = w::MeProject::new(&w::CEILINGS, row).expect("membership");
            let mut bytes = Writer::new(usize::try_from(row.measure()).expect("membership fits"));
            row.encode(&mut bytes).expect("measured membership");
            let bytes = bytes.finish();
            projects.extend_from_slice(&u32::try_from(bytes.len()).expect("membership size").to_be_bytes());
            projects.extend_from_slice(&bytes);
        }
        parts.projects = projects.into();
        let frame = w::Me::new(&w::CEILINGS, parts).expect("me frame");
        let received = link::translate::documents::frame(frame, &fixtures::link_limits());
        assert_eq!(received.is_ok(), accepted, "{count} memberships through native preset: {received:?}");
        if let Ok(frame) = received {
            assert_eq!(frame.projects.len(), count);
        }
    }
}
