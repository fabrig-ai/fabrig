//! Kept posting, with blame positions in the judged record's failing IDs (after-landing.md, 9 and 10).
use super::{FITS, List, Undecodable, d, p, w};

pub(super) fn encode(posting: &d::TierPosting, failing: &p::Failures) -> w::TierPosting {
    let mut landings = List::with_capacity(u32::try_from(posting.landings.len()).expect(FITS));
    for landing in &posting.landings {
        landings
            .push(
                w::TierPostedLanding::new(
                    &w::CEILINGS,
                    w::TierPostedLandingParts {
                        commit: landing.commit,
                        blame: encode_blame(&landing.blame, failing),
                        change: landing.change,
                    },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    let mut comments = List::with_capacity(u32::try_from(posting.comments.len()).expect(FITS));
    for comment in &posting.comments {
        let kind = match comment.kind {
            d::TierCommentKind::Verdict { at } => w::TierCommentKind::Verdict(
                w::TierCommentVerdict::new(&w::CEILINGS, w::TierCommentVerdictParts { at }).expect(FITS),
            ),
            d::TierCommentKind::Repaired { by } => w::TierCommentKind::Repaired(
                w::TierCommentRepair::new(&w::CEILINGS, w::TierCommentRepairParts { by }).expect(FITS),
            ),
        };
        comments
            .push(
                w::TierComment::new(
                    &w::CEILINGS,
                    w::TierCommentParts { pull: comment.pull, landing: comment.landing, kind },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    w::TierPosting::new(
        &w::CEILINGS,
        w::TierPostingParts {
            number: posting.number,
            tip: posting.tip,
            link: posting.link.clone(),
            status: match posting.check {
                skein_forge::api::Check::Pending => w::Check::Pending,
                skein_forge::api::Check::Passed => w::Check::Passed,
                skein_forge::api::Check::Failed => w::Check::Failed,
            },
            description: posting.description.clone(),
            landings,
            comments,
            next: posting.next,
            entry: posting.entry,
            part: posting.part,
            op_bytes: posting.op_bytes,
        },
    )
    .expect(FITS)
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
    let tests = w::TierPostedTests::new(&w::CEILINGS, w::TierPostedTestsParts { tests: positions }).expect(FITS);
    if suspect { w::TierPostedBlame::Suspect(tests) } else { w::TierPostedBlame::Owns(tests) }
}
pub(super) fn decode(posting: w::TierPosting, failing: &p::Failures) -> Result<d::TierPosting, Undecodable> {
    let w::TierPostingParts { number, tip, link, status, description, landings, comments, next, entry, part, op_bytes } =
        posting.into_parts();
    let mut decoded = List::with_capacity(landings.len());
    for landing in landings.into_boxed() {
        let w::TierPostedLandingParts { commit, blame, change } = landing.into_parts();
        decoded.push(d::TierPostedLanding { commit, blame: decode_blame(blame, failing)?, change }).expect(FITS);
    }
    let landings = decoded.into_boxed();
    let mut decoded = List::with_capacity(comments.len());
    for comment in comments.into_boxed() {
        let w::TierCommentParts { pull, landing, kind } = comment.into_parts();
        let kind = match kind {
            w::TierCommentKind::Verdict(value) => {
                let at = value.into_parts().at;
                if landings.get(usize::from(at)).ok_or(Undecodable::Malformed)?.commit != landing {
                    return Err(Undecodable::Malformed);
                }
                d::TierCommentKind::Verdict { at }
            }
            w::TierCommentKind::Repaired(value) => d::TierCommentKind::Repaired { by: value.into_parts().by },
        };
        decoded.push(d::TierComment { pull, landing, kind }).expect(FITS);
    }
    let comments = decoded.into_boxed();
    let check = match status {
        w::Check::Pending => skein_forge::api::Check::Pending,
        w::Check::Passed => skein_forge::api::Check::Passed,
        w::Check::Failed => skein_forge::api::Check::Failed,
    };
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
    let mut decoded = List::with_capacity(tests.tests().len());
    let mut previous = None;
    for at in tests.into_parts().tests.into_boxed() {
        if previous >= Some(at) {
            return Err(Undecodable::Malformed);
        }
        previous = Some(at);
        decoded.push(all.get(usize::from(at)).ok_or(Undecodable::Malformed)?.clone()).expect(FITS);
    }
    let tests = p::Failures::Tests(decoded.into_boxed());
    Ok(if suspect { p::Blame::Suspect(tests) } else { p::Blame::Owns(tests) })
}
