use super::{fixtures as f, read_support as r};
use skein_fake_forge_domain::{self as fake, api as a};
use skein_forge::{
    api,
    github::{request::Method, request::Request},
};

#[test]
fn github_fake_reviews_drive_the_plans_across_pages_and_skip_pending() {
    let mut h = r::harness();
    let (number, commit) = r::pull(&mut h);
    fake::grant(&mut h.domain, b"acme/widgets", 31, a::Permission::Write);
    let mut ids = skein_lib::List::with_capacity(3);
    for verdict in [a::Verdict::Approve, a::Verdict::RequestChanges, a::Verdict::Comment] {
        let a::Answer::Reviewed(id) = r::call_as(
            &mut h,
            31,
            a::Op::Write(a::Write::Review { number, verdict: Some(verdict), body: f::boxed(b"review body") }),
        ) else {
            panic!("review created")
        };
        ids.push(id).unwrap();
    }
    r::call(&mut h, a::Op::Write(a::Write::Review { number, verdict: None, body: f::boxed(b"pending") }));
    let token = r::token(&mut h);
    for (page, count, more) in [(1, 2, true), (2, 1, false)] {
        let api::Answer::Reviews { reviews, more: actual, .. } =
            r::read(&mut h, &token, api::Read::Reviews { number, page, skip: 0 }).unwrap()
        else {
            panic!("reviews")
        };
        assert_eq!(reviews.len(), count);
        assert_eq!(actual, more);
        for review in reviews {
            assert!(review.official);
            assert_eq!(review.author, 31);
            assert_eq!(review.commit, r::object(commit));
            assert_eq!(review.body.as_ref(), b"review body");
            assert_eq!(review.provenance, api::Provenance::Original);
        }
    }
    // A review fills the answer: resume the cut on the same wire page, then pass pending.
    let bounds = skein_forge::github::plans::plan::Bounds { answer_bytes: 139, ..r::bounds() };
    for (page, skip, index, cut, more) in [(1, 0, 0, true, true), (1, 1, 1, false, true), (2, 0, 2, false, false)] {
        let answer = r::read_bounded(&mut h, &token, api::Read::Reviews { number, page, skip }, bounds).unwrap();
        assert_eq!(skein_forge::measure::answer(&answer), Some(u64::from(bounds.answer_bytes)));
        let api::Answer::Reviews { reviews, cut: actual_cut, more: actual_more } = answer else { panic!("reviews") };
        assert_eq!((actual_cut, actual_more), (cut, more));
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews.first().unwrap().id, *ids.get(index).unwrap());
    }
    let api::Answer::Remarks { remarks, more, .. } =
        r::read(&mut h, &token, api::Read::Remarks { number, review: *ids.get(0).unwrap(), page: 1, skip: 0 }).unwrap()
    else {
        panic!("review comments")
    };
    assert!(remarks.is_empty());
    assert!(!more);
    assert_eq!(r::read(&mut h, &token, api::Read::Reviews { number: 999, page: 1, skip: 0 }), Err(api::Error::Missing));
    assert_eq!(
        r::read(&mut h, &token, api::Read::Remarks { number, review: 999, page: 1, skip: 0 }),
        Err(api::Error::Missing)
    );
    assert_eq!(h.hooks, 0);
}
#[test]
fn github_fake_review_routes_refuse_outside_installation_and_zero_pages() {
    let mut h = r::harness();
    let token = r::token(&mut h);
    for request in [
        Request::Reviews { owner: f::boxed(b"acme"), name: f::boxed(b"absent"), number: 1, page: 1, per_page: 100 },
        Request::ReviewComments {
            owner: f::boxed(b"acme"),
            name: f::boxed(b"absent"),
            number: 1,
            review: 1,
            page: 1,
            per_page: 100,
        },
    ] {
        let encoded = skein_forge::github::request::encode(&request, &h.limits.github).unwrap();
        let response = h.ask(encoded.method, &encoded.target, Some(token.expose()), b"");
        f::refusal(&h, &response, 404, b"Not Found");
    }
    assert_eq!(h.calls, 0);
    for target in [
        b"/repos/acme/widgets/pulls/1/reviews?per_page=1&page=0".as_slice(),
        b"/repos/acme/widgets/pulls/1/reviews/1/comments?per_page=0&page=1",
    ] {
        let response = h.ask(Method::Get, target, Some(token.expose()), b"");
        f::refusal(&h, &response, 404, b"Not Found");
    }
}
