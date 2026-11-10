use super::{fixtures as f, read_support as r};
use skein_fake_forge_domain::api as a;
use skein_forge::{
    api,
    github::{
        document::{Document, Kind, Row},
        request::{Method, Request},
    },
};
use skein_lib::Time;

#[test]
fn github_fake_issues_and_comments_drive_the_dialect_plans() {
    let mut h = r::harness();
    let first = r::issue(&mut h, b"first");
    let second = r::issue(&mut h, b"second");
    let third = r::issue(&mut h, b"third");
    for text in [b"one".as_slice(), b"two", b"three"] {
        r::call(&mut h, a::Op::Write(a::Write::Comment { number: first, body: f::boxed(text) }));
    }
    let token = r::token(&mut h);
    for (page, expected, more) in [(1, &[first, second][..], true), (2, &[third][..], false)] {
        let api::Answer::Items { items, more: actual, .. } =
            r::read(&mut h, &token, api::Read::Items { since: Time::ZERO, page, kind: None }).expect("items plan")
        else {
            panic!("items answer")
        };
        assert_eq!(actual, more);
        assert_eq!(items.len(), expected.len());
        for (item, expected) in items.iter().zip(expected) {
            assert_eq!(item.number, *expected);
        }
    }
    let api::Answer::Item { item, comments, more } =
        r::read(&mut h, &token, api::Read::Item { number: first, after: 0, since: Time::ZERO })
            .expect("item and paged comments plan")
    else {
        panic!("item answer")
    };
    assert_eq!(item.number, first);
    assert_eq!(item.title.as_ref(), b"first");
    assert_eq!(comments.len(), 3);
    for (comment, body) in comments.iter().zip([b"one".as_slice(), b"two", b"three"]) {
        assert_eq!(comment.body.as_ref(), body);
        assert_eq!(comment.provenance, api::Provenance::Original);
    }
    assert!(!more);
    let api::Answer::Item { comments: after, .. } = r::read(
        &mut h,
        &token,
        api::Read::Item { number: first, after: comments.first().expect("first comment").id, since: Time::ZERO },
    )
    .expect("after filter") else {
        panic!("item answer")
    };
    assert_eq!(after.len(), 2);
    assert_eq!(
        r::read(&mut h, &token, api::Read::Item { number: 999, after: 0, since: Time::ZERO }),
        Err(api::Error::Missing)
    );
    assert_eq!(h.hooks, 0);
}
#[test]
fn github_fake_merged_pull_reads_as_merged_in_the_mixed_listing() {
    let mut h = r::harness();
    let issue = r::issue(&mut h, b"issue");
    let (pull, head) = r::pull(&mut h);
    h.env.now = Time::from_nanos(2_000_000_000);
    r::call(&mut h, a::Op::Write(a::Write::Merge { number: pull, head, how: a::MergeHow::Style }));
    let token = r::token(&mut h);
    let api::Answer::Items { items, .. } =
        r::read(&mut h, &token, api::Read::Items { since: Time::ZERO, page: 1, kind: None }).expect("mixed listing")
    else {
        panic!("items")
    };
    assert_eq!(items.len(), 2);
    for (item, (number, kind)) in items.iter().zip([(issue, api::Kind::Issue), (pull, api::Kind::Pull)]) {
        assert_eq!((item.number, item.kind), (number, kind));
    }
    let api::Answer::Items { items, .. } =
        r::read(&mut h, &token, api::Read::Items { since: Time::ZERO, page: 1, kind: Some(api::Kind::Issue) })
            .expect("kind filtering")
    else {
        panic!("items")
    };
    assert_eq!(items.len(), 1);
    let response = h.ask(Method::Get, b"/repos/acme/widgets/issues/2", Some(token.expose()), b"");
    let (_, Document::Issue(row)) = h.decode(Kind::Issue, &response) else { panic!("pull issue") };
    assert!(row.pull);
    assert_eq!(row.state, api::State::Closed);
    assert!(row.merged);
    assert_closed_pull_listing(&mut h, pull, true);
}
#[test]
fn github_fake_closed_unmerged_pull_reads_as_unmerged_through_the_listing() {
    let mut h = r::harness();
    let (pull, _) = r::pull(&mut h);
    r::call(&mut h, a::Op::Write(a::Write::Close { number: pull }));
    assert_closed_pull_listing(&mut h, pull, false);
}
fn assert_closed_pull_listing(h: &mut f::Harness, number: u64, merged: bool) {
    let token = r::token(h);
    let request = Request::Issues {
        owner: f::boxed(b"acme"),
        name: f::boxed(b"widgets"),
        since: super::wall(0),
        page: 1,
        per_page: 100,
    };
    let encoded = skein_forge::github::request::encode(&request, &h.limits.github).expect("issues listing");
    let response = h.ask(encoded.method, &encoded.target, Some(token.expose()), b"");
    assert_eq!(response.status, 200);
    let (rows, Document::Rows) = h.decode(Kind::IssueRows, &response) else { panic!("issues listing") };
    let mut found = None;
    for row in &rows {
        let Row::Issue(row) = row else { panic!("issue listing row") };
        if row.number == number {
            found = Some(row);
        }
    }
    let row = found.expect("pull in listing");
    assert!(row.pull);
    assert_eq!(row.state, api::State::Closed);
    assert_eq!(row.merged, merged);
    let merged_at = if merged { br#""merged_at":"1970-01-01T00:00:02Z""#.as_slice() } else { br#""merged_at":null"# };
    assert!(skein_lib::bytes::find(&response.body, merged_at).is_some());
}
#[test]
fn github_fake_issue_routes_refuse_outside_installation_and_invalid_pages() {
    let mut h = r::harness();
    let token = r::token(&mut h);
    for request in [
        Request::Issues {
            owner: f::boxed(b"acme"),
            name: f::boxed(b"absent"),
            since: super::wall(0),
            page: 1,
            per_page: 100,
        },
        Request::Issue { owner: f::boxed(b"acme"), name: f::boxed(b"absent"), number: 1 },
        Request::Comments {
            owner: f::boxed(b"acme"),
            name: f::boxed(b"absent"),
            number: 1,
            page: 1,
            per_page: 100,
            since: None,
        },
    ] {
        let encoded = skein_forge::github::request::encode(&request, &h.limits.github).expect("encoded read");
        let response =
            h.ask(encoded.method, &encoded.target, Some(token.expose()), encoded.body.as_deref().unwrap_or(b""));
        f::refusal(&h, &response, 404, b"Not Found");
    }
    assert_eq!(h.calls, 0, "scope refusal precedes domain call");
    r::issue(&mut h, b"one");
    r::issue(&mut h, b"two");
    r::issue(&mut h, b"three");
    let request = Request::Issues {
        owner: f::boxed(b"acme"),
        name: f::boxed(b"widgets"),
        since: super::wall(0),
        page: 1,
        per_page: 100,
    };
    let events = r::events(&mut h, &request, &token);
    let mut rows = 0_u32;
    for event in events.into_boxed() {
        if let skein_forge::github::plans::Heard::Row(Row::Issue(_)) = event {
            rows = rows.checked_add(1).unwrap();
        }
    }
    assert_eq!(rows, 2, "scenario page cap");
    for target in [
        b"/repos/acme/widgets/issues/1/comments?per_page=0&page=1".as_slice(),
        b"/repos/acme/widgets/issues/1/comments?per_page=1&page=0",
    ] {
        let response = h.ask(Method::Get, target, Some(token.expose()), b"");
        f::refusal(&h, &response, 404, b"Not Found");
    }
}

#[test]
fn github_fake_issues_since_observed_cutoff_and_inclusive_timestamp() {
    let mut h = r::harness();
    h.env.now = Time::from_nanos(31_536_000_000_000_000);
    let number = r::issue(&mut h, b"at the observed bound");
    let token = r::token(&mut h);
    for (since, present) in [
        (b"".as_slice(), true),
        (b"&since=1970-01-01T00%3A00%3A00Z", false),
        (b"&since=1970-01-01T00%3A00%3A01Z", false),
        (b"&since=1970-12-31T23%3A59%3A59Z", false),
        (b"&since=1971-01-01T00%3A00%3A00Z", true),
        (b"&since=1971-01-01T00%3A00%3A01Z", false),
    ] {
        let target = f::parts(&[
            b"/repos/acme/widgets/issues?state=all",
            since,
            b"&sort=updated&direction=asc&per_page=100&page=1",
        ]);
        let response = h.ask(Method::Get, &target, Some(token.expose()), b"");
        assert_eq!(response.status, 200);
        let (rows, Document::Rows) = h.decode(Kind::IssueRows, &response) else { panic!("listing") };
        assert_eq!(rows.len(), u32::from(present), "{since:?}");
        if present {
            let Row::Issue(row) = rows.as_slice().first().unwrap() else { panic!("issue") };
            assert_eq!(row.number, number);
        }
    }
}

#[test]
#[expect(clippy::manual_map, reason = "step-style tests spell Option transformations with exhaustive matches")]
fn github_fake_comments_since_filters_updates_before_pages_and_links() {
    // Whole seconds after GitHub's observed issues-since cutoff.
    const T1: u64 = 1_700_000_000_000_000_000;
    const T2: u64 = 1_700_000_010_000_000_000;
    const T3: u64 = 1_700_000_020_000_000_000;
    for (label, since, edited, page, expected, more) in [
        ("absent", None, false, 1, &["one", "two"][..], true),
        ("inclusive", Some(T2), false, 1, &["two", "three"][..], false),
        ("filtered last page", Some(T2), false, 2, &[][..], false),
        ("edited old comment", Some(T2), true, 1, &["edited", "two"][..], true),
        ("edited last page", Some(T2), true, 2, &["three"][..], false),
    ] {
        let mut h = r::harness();
        let number = r::issue(&mut h, b"thread");
        let mut first = None;
        for (now, body) in [(T1, b"one".as_slice()), (T2, b"two"), (T3, b"three")] {
            h.env.now = Time::from_nanos(now);
            let a::Answer::Commented(id) =
                r::call(&mut h, a::Op::Write(a::Write::Comment { number, body: f::boxed(body) }))
            else {
                panic!("comment")
            };
            first = first.or(Some(id));
        }
        if edited {
            r::call(
                &mut h,
                a::Op::Write(a::Write::EditComment { id: first.expect("first"), body: f::boxed(b"edited") }),
            );
        }
        let token = r::token(&mut h);
        let request = Request::Comments {
            owner: f::boxed(b"acme"),
            name: f::boxed(b"widgets"),
            number,
            since: match since {
                Some(time) => Some(skein_lib::Wall::from_nanos(time)),
                None => None,
            },
            page,
            per_page: 2,
        };
        let encoded = skein_forge::github::request::encode(&request, &h.limits.github).expect("comments");
        let response = h.ask(encoded.method, &encoded.target, Some(token.expose()), b"");
        assert_eq!(response.status, 200, "{label}");
        let (rows, Document::Rows) = h.decode(Kind::CommentRows, &response) else { panic!("comments") };
        assert_eq!(usize::try_from(rows.len()).unwrap(), expected.len(), "{label}");
        for (row, expected) in rows.iter().zip(expected) {
            let Row::Comment(row) = row else { panic!("comment row") };
            let skein_forge::text::Body::Kept(body) = &row.body else { panic!("kept body") };
            assert_eq!(body.as_ref(), expected.as_bytes(), "{label}");
        }
        assert_eq!(response.link.is_some(), more, "{label}: next Link");
        if page == 1 {
            let since = match since {
                Some(time) => Time::from_nanos(time),
                None => Time::ZERO,
            };
            let api::Answer::Item { comments, more, .. } =
                r::read(&mut h, &token, api::Read::Item { number, after: 0, since }).expect("filtered comment plan")
            else {
                panic!("item")
            };
            let wanted: &[&str] = if edited {
                &["edited", "two", "three"]
            } else if since == Time::ZERO {
                &["one", "two", "three"]
            } else {
                &["two", "three"]
            };
            assert_eq!(comments.len(), wanted.len(), "{label}: plan follows filtered pages");
            for (comment, body) in comments.iter().zip(wanted) {
                assert_eq!(comment.body.as_ref(), body.as_bytes(), "{label}: plan follows filtered pages");
            }
            assert!(!more);
        }
    }
}
