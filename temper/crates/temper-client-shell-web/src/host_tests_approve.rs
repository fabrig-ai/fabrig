//! The host keeps the approval place and prints its code as shown (client/web/README.md, 2.3).
use super::*;
use jig_client_protocol_wire as w;
use skein_lib::Reader;
fn exchange(host: &mut Host, path: &[u8]) -> u32 {
    let requests: Vec<_> = host
        .requests()
        .into_iter()
        .filter_map(|request| {
            let Asked::Exchange { id, call, .. } = request else { return None };
            (call.path.as_ref() == path).then_some(id)
        })
        .collect();
    assert_eq!(requests.len(), 1, "one approval exchange");
    requests[0]
}
fn waiting(host: &mut Host, exchange: u32, kind: w::ApprovalKind) {
    let answer = w::ApprovalState::Waiting(
        w::ApprovalWaiting::new(&w::CEILINGS, w::ApprovalWaitingParts { asked: 0, kind, source: None, reader: None })
            .expect("waiting"),
    );
    host.heard(Heard::Answered {
        exchange,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
        headers: Vec::new(),
    });
    host.requests();
}

#[test]
fn a_typed_address_code_is_replaced_after_sign_in_and_only_typing_offers_approve() {
    let path = b"/terminal?code=WDJB-MJHT";
    let mut signed_out = Host::new(
        crate::limits::client(),
        11,
        Time::ZERO,
        Wall::EPOCH,
        0,
        temper_client_view::sign_in::Ways::parse("google", b"http://127.0.0.1:32123"),
    );
    signed_out.root = root::unlinked(&signed_out.env.limits, 11);
    signed_out.client(jig_client::Event::StartUnlinked { kept: List::with_capacity(0) });
    signed_out.client(jig_client::Event::Up { up: jig_client::Up::SignedOut });
    signed_out.went(path);
    build(&mut signed_out);
    assert_eq!(signed_out.root.approval().expect("page").code(), jig_client::ApprovalCode::parse(b"WDJB-MJHT"));
    let link = signed_out.tree().find(node(&signed_out, Role::Link, b"Continue with Google")).expect("Google link");
    assert_eq!(
        link.external.as_deref(),
        Some(b"http://127.0.0.1:32123/sign-in/google?then=/terminal%3Fcode%3DWDJB-MJHT".as_slice())
    );
    // The sign-in returns by that address; its kind is unknown until the read.
    let mut host = super::tab::linked();
    host.show = None;
    host.went(path);
    let read = exchange(&mut host, b"/jig/v1/approval/read");
    waiting(&mut host, read, w::ApprovalKind::Typed);
    let duties = host.report(Report::Start);
    assert_eq!(duties.show, Some((Box::from(b"/terminal".as_slice()), true)));
    assert!(!duties.departs(), "only replace history; do not reload or navigate");
    assert_eq!(host.root.approval().expect("page").state(), root::approve::State::Typing { unread: false });
    build(&mut host);
    let field = host.tree().find(node(&host, Role::TextField, b"Code")).expect("code field");
    assert!(field.value.as_ref().expect("field value").text.is_empty());
    assert!(
        !host
            .tree()
            .nodes()
            .iter()
            .any(|node| node.role == Role::Alert
                || node.role == Role::Button && node.name.as_deref() == Some(b"Approve"))
    );
    assert!(host.report(Report::Due).show.is_none(), "replacement sent once");
    host.event(Event::Act {
        action: root::Action::Approve(root::approve::Act::Words { words: b"wdjb mjht".as_slice().into() }),
    });
    host.event(Event::Act { action: root::Action::Approve(root::approve::Act::Check) });
    let read = exchange(&mut host, b"/jig/v1/approval/read");
    waiting(&mut host, read, w::ApprovalKind::Typed);
    assert!(host.report(Report::Start).show.is_none(), "a typed code changes no address");
    build(&mut host);
    node(&host, Role::Button, b"Approve");
}

#[test]
fn the_loopback_handoff_uses_the_answers_port_and_replaces_once() {
    let mut host = super::tab::linked();
    host.went(b"/terminal?code=WDJB-MJHT&host=elsewhere.example&port=32123");
    let read = exchange(&mut host, b"/jig/v1/approval/read");
    waiting(&mut host, read, w::ApprovalKind::Returned);
    host.show = None;
    host.event(Event::Act { action: root::Action::Approve(root::approve::Act::Approve) });
    let decide = exchange(&mut host, b"/jig/v1/approval/decide");
    let answer = w::ApprovalDecided::Approved(
        w::ApprovalApproved::new(
            &w::CEILINGS,
            w::ApprovalApprovedParts {
                loopback: Some(
                    w::ApprovalLoopback::new(
                        &w::CEILINGS,
                        w::ApprovalLoopbackParts { port: 43210, grant: skein_lib::Secret::new(Box::new([0xff; 32])) },
                    )
                    .expect("loopback return"),
                ),
            },
        )
        .expect("approved"),
    );
    host.heard(Heard::Answered {
        exchange: decide,
        status: 200,
        body: encoded(answer.measure(), |writer| answer.encode(writer)),
        headers: Vec::new(),
    });
    let duties = host.report(Report::Start);
    assert_eq!(
        duties.replace.as_deref(),
        Some(b"http://127.0.0.1:43210/approved?grant=__________________________________________8".as_slice())
    );
    assert!(duties.navigate.is_none() && duties.show.is_none());
    assert_eq!(host.root.approval().expect("page").state(), root::approve::State::Handing);
    assert!(host.report(Report::Due).replace.is_none());
}

#[test]
fn the_approval_address_prints_its_code_and_reads_once() {
    for (path, printed, reads) in [
        (b"/terminal?code=wdjb-mjht".as_slice(), Some(b"/terminal?code=WDJB-MJHT".as_slice()), 1),
        (b"/terminal?code=AEIO", Some(b"/terminal".as_slice()), 0),
        (b"/terminal?code=WDJB-MJHT", None, 1),
    ] {
        let mut host = super::tab::linked();
        host.show = None;
        let address = host.address.clone();
        host.went(path);
        assert_eq!(host.address, address, "the place keeps the pages' address");
        assert_eq!(host.show, printed.map(|path| (Box::from(path), true)), "{path:?}");
        let mut count = 0;
        for request in host.requests() {
            if let Asked::Exchange { call, .. } = request
                && call.path.as_ref() == b"/jig/v1/approval/read"
            {
                let read = w::ApprovalRead::decode(&w::CEILINGS, &mut Reader::new(&call.body)).unwrap();
                assert_eq!(read.code(), b"WDJBMJHT");
                count += 1;
            }
        }
        assert_eq!(count, reads);
        host.due();
        assert!(!host.requests().iter().any(
            |request| matches!(request, Asked::Exchange { call, .. } if call.path.as_ref() == b"/jig/v1/approval/read")
        ));
    }
}
