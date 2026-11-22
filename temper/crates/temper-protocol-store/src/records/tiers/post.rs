//! Kept posting, with blame positions in the judged record's failing IDs (domain/after-landing.md, 9, and domain/after-landing-storage.md, 1).
use super::{FITS, List, Undecodable, d, p, w};

pub(super) fn encode(posting: &d::TierPosting, failing: &p::Failures) -> w::TierPosting {
    let context = super::super::Context { posted_failures: failing.clone(), ..super::super::Context::UNBOUNDED };
    let mut landings = List::with_capacity(u32::try_from(posting.landings.len()).expect(FITS));
    for landing in &posting.landings {
        landings
            .push(crate::generated::store::tier_posted_landing_to_record(landing, &context).expect(FITS))
            .expect(FITS);
    }
    let mut comments = List::with_capacity(u32::try_from(posting.comments.len()).expect(FITS));
    for comment in &posting.comments {
        comments
            .push(
                crate::generated::store::tier_comment_to_record(comment, &super::super::Context::UNBOUNDED)
                    .expect(FITS),
            )
            .expect(FITS);
    }
    w::TierPosting {
        number: posting.number,
        tip: posting.tip,
        link: posting.link.clone(),
        status: crate::generated::store::check_to_record(&posting.check, &super::super::Context::UNBOUNDED)
            .expect(FITS),
        description: posting.description.clone(),
        landings,
        comments,
        next: posting.next,
        entry: posting.entry,
        part: posting.part,
        op_bytes: posting.op_bytes,
    }
}
fn encode_blame(blame: &p::Blame, failing: &p::Failures) -> w::TierPostedBlame {
    let (tests, suspect) = match blame {
        p::Blame::Cleared => return w::TierPostedBlame::Cleared,
        p::Blame::Owns(p::Failures::Many) => return w::TierPostedBlame::OwnsMany,
        p::Blame::Suspect(p::Failures::Many) => return w::TierPostedBlame::SuspectMany,
        p::Blame::Owns(p::Failures::Tests(tests)) => (tests, false),
        p::Blame::Suspect(p::Failures::Tests(tests)) => (tests, true),
    };
    let p::Failures::Tests(all) = failing else { unreachable!("enumerated blame has enumerated failures") };
    let mut positions = List::with_capacity(u32::try_from(tests.len()).expect(FITS));
    for test in tests {
        let mut position = None;
        for (at, id) in all.iter().enumerate() {
            if id == test {
                position = Some(u16::try_from(at).expect(FITS));
                break;
            }
        }
        positions.push(position.expect("blame belongs to the judged failures")).expect(FITS);
    }
    let tests = w::TierPostedTests { tests: positions };
    if suspect { w::TierPostedBlame::Suspect(tests) } else { w::TierPostedBlame::Owns(tests) }
}
pub(super) fn decode(posting: w::TierPosting, failing: &p::Failures) -> Result<d::TierPosting, Undecodable> {
    let w::TierPosting { number, tip, link, status, description, landings, comments, next, entry, part, op_bytes } =
        posting;
    let context = super::super::Context { posted_failures: failing.clone(), ..super::super::Context::UNBOUNDED };
    let mut decoded = List::with_capacity(landings.len());
    for landing in landings.into_boxed() {
        decoded
            .push(
                crate::generated::store::tier_posted_landing_to_domain(landing, &context)
                    .or(Err(Undecodable::Malformed))?,
            )
            .expect(FITS);
    }
    let landings = decoded.into_boxed();
    let mut decoded = List::with_capacity(comments.len());
    for comment in comments.into_boxed() {
        let comment =
            crate::generated::store::tier_comment_to_domain(comment, &super::super::Context::UNBOUNDED).expect(FITS);
        match comment.kind {
            d::TierCommentKind::Verdict { at } => {
                if landings.get(usize::from(at)).ok_or(Undecodable::Malformed)?.commit != comment.landing {
                    return Err(Undecodable::Malformed);
                }
            }
            d::TierCommentKind::Repaired { .. } => {}
        }
        decoded.push(comment).expect(FITS);
    }
    let comments = decoded.into_boxed();
    let check = crate::generated::store::check_to_domain(status, &super::super::Context::UNBOUNDED).expect(FITS);
    let count = u32::try_from(comments.len()).or(Err(Undecodable::Malformed))?;
    if next > count.checked_add(1).ok_or(Undecodable::Malformed)?
        || (entry.is_some() && next > count)
        || (part != 0 && (check != skein_forge::api::Check::Pending || !landings.is_empty() || !comments.is_empty()))
        || (part == 0 && check == skein_forge::api::Check::Pending)
        || op_bytes == 0
    {
        return Err(Undecodable::Malformed);
    }
    Ok(d::TierPosting { number, tip, link, check, description, landings, comments, next, entry, part, op_bytes })
}
fn decode_blame(blame: w::TierPostedBlame, failing: &p::Failures) -> Result<p::Blame, Undecodable> {
    let (tests, suspect) = match blame {
        w::TierPostedBlame::Cleared => return Ok(p::Blame::Cleared),
        w::TierPostedBlame::OwnsMany => return Ok(p::Blame::Owns(p::Failures::Many)),
        w::TierPostedBlame::SuspectMany => return Ok(p::Blame::Suspect(p::Failures::Many)),
        w::TierPostedBlame::Owns(tests) => (tests, false),
        w::TierPostedBlame::Suspect(tests) => (tests, true),
    };
    let p::Failures::Tests(all) = failing else { return Err(Undecodable::Malformed) };
    let mut decoded = List::with_capacity(tests.tests.len());
    let mut previous = None;
    for at in tests.tests.into_boxed() {
        if previous >= Some(at) {
            return Err(Undecodable::Malformed);
        }
        previous = Some(at);
        decoded.push(all.get(usize::from(at)).ok_or(Undecodable::Malformed)?.clone()).expect(FITS);
    }
    let tests = p::Failures::Tests(decoded.into_boxed());
    Ok(if suspect { p::Blame::Suspect(tests) } else { p::Blame::Owns(tests) })
}

pub fn tier_posted_blame_to_record(
    value: &p::Blame,
    context: &super::super::Context,
) -> Result<w::TierPostedBlame, w::Problem> {
    Ok(encode_blame(value, &context.posted_failures))
}
pub fn tier_posted_blame_to_domain(
    value: w::TierPostedBlame,
    context: &super::super::Context,
) -> Result<p::Blame, w::Problem> {
    decode_blame(value, &context.posted_failures)
        .or(Err(w::paths::TIER_POSTED_BLAME_TAG.problem(skein_codec::Reason::Malformed)))
}
