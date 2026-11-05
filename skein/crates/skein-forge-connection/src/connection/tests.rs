//! The complete GitHub handoff, held at simultaneous peak and released at its terminals.
use super::*;
use crate::tests::fixtures::Exchange as Pending;
use crate::tests::fixtures::{self as f, ConnectionHarness};
use crate::{connection::Failure, settings::Transport};
use github::plans::plan::{Comparison, Exchange as GitHubExchange, Retention};

fn peak(h: &mut ConnectionHarness) {
    let size = usize::try_from(h.env.limits.string_bytes).expect("string bound");
    let request = github::request::Request::Raw {
        owner: f::filled(size, b'o'),
        name: f::filled(size, b'n'),
        path: f::filled(size, b'p'),
        commit: [0; 32],
    };
    h.start(Pending::GitHub {
        exchange: GitHubExchange {
            request,
            retention: Some(Retention::Whole { bytes: f::shared().answer_bytes }),
            comparison: Comparison::MergeBase,
        },
        authorization: Some(Secret::new(Box::from(b"Bearer test-token".as_slice()))),
    })
    .expect("maximum exchange encodes");
}
fn retained(h: &ConnectionHarness) -> &GitHubExchange {
    h.github.request.as_ref().expect("retained request")
}
#[test]
#[expect(clippy::print_stdout, clippy::disallowed_macros, reason = "report the handoff fixture peak")]
fn github_exchange_handoff_stays_under_the_ceiling_on_every_connection() {
    // All configured connections retaining their full exchange handoffs.
    const CEILING: i64 = 13_000;
    let span = skein_heap::Span::start();
    let count = f::limits().connections + f::github_settings().forges.len();
    let mut connections = List::with_capacity(count);
    for _ in 0..count {
        let mut h = ConnectionHarness::github();
        peak(&mut h);
        connections.push(h).expect("all call and own connections");
    }
    for h in &connections {
        let exchange = retained(h);
        assert_eq!(exchange.retention, Some(Retention::Whole { bytes: f::shared().answer_bytes }));
        assert_eq!(exchange.comparison, Comparison::MergeBase);
        let github::request::Request::Raw { owner, name, path, .. } = &exchange.request else { unreachable!("raw") };
        assert_eq!(owner.len(), usize::try_from(h.env.limits.string_bytes).expect("bound"));
        assert_eq!((name.len(), path.len()), (owner.len(), owner.len()));
    }
    let grown = span.end();
    assert!(grown.peak <= CEILING, "{} <= {CEILING}", grown.peak);
    println!("GitHub exchange handoffs: peak={} ceiling={CEILING}", grown.peak);
}
#[test]
fn github_exchange_handoff_is_dropped_at_its_answer_and_replaced() {
    let mut h = ConnectionHarness::github();
    peak(&mut h);
    h.github_answer(&f::ok(b""));
    assert!(h.github.request.is_none(), "answer releases the handoff");
    h.start(Pending::GitHub {
        exchange: GitHubExchange {
            request: github::request::Request::Raw {
                owner: Box::from(b"a".as_slice()),
                name: Box::from(b"b".as_slice()),
                path: Box::from(b"c".as_slice()),
                commit: [0; 32],
            },
            retention: Some(Retention::Head { bytes: 1 }),
            comparison: Comparison::Full,
        },
        authorization: Some(Secret::new(Box::from(b"Bearer test-token".as_slice()))),
    })
    .expect("replacement request");
    let exchange = retained(&h);
    assert_eq!(exchange.retention, Some(Retention::Head { bytes: 1 }));
    assert_eq!(exchange.comparison, Comparison::Full);
}

#[test]
fn github_followed_request_sends_exact_bytes_without_authorization_or_range() {
    let redirect = github::redirect::parse(b"http://logs.example.test/job%2Fpart?signature=invented", 128, 1024)
        .expect("invented log origin");
    let mut h = ConnectionHarness::github();
    h.forge.transport = Transport::Plaintext { name: Some(redirect.origin.host.clone()) };
    h.forge.address = skein_io::Addr::Ip(([127, 0, 0, 1], redirect.origin.port).into());
    h.start(Pending::GitHub {
        exchange: GitHubExchange {
            request: github::redirect::request(&redirect),
            retention: Some(Retention::Tail { bytes: 8 }),
            comparison: Comparison::Full,
        },
        authorization: None,
    })
    .expect("followed request encodes");
    h.github_answer(&f::ok(b"log"));
    assert_eq!(h.sent.as_ref(), b"GET /job%2Fpart?signature=invented HTTP/1.1\r\nHost: logs.example.test\r\nUser-Agent: kit/0.0.0\r\nAccept: application/vnd.github+json\r\nX-GitHub-Api-Version: 2026-03-10\r\n\r\n");
}

#[test]
fn queued_answers_precede_transport_endings() {
    let mut h = ConnectionHarness::github();
    h.github.heard.push(Heard::Done(Body::Empty));
    h.connection.failed(Failure::Seen);
    assert!(h.github.ready(&h.connection));
    assert_eq!(h.github.heard(&mut h.connection), Some(Heard::Done(Body::Empty)));
    assert_eq!(h.github.heard(&mut h.connection), Some(Heard::Failed(Failure::Seen)));
    assert_eq!(h.github.heard(&mut h.connection), None, "one terminal");
    assert!(!h.github.ready(&h.connection));
}
