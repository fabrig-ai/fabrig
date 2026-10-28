//! CI on a commit, folded over the adoption's contexts (domain/forge.md, 5.3).
use alloc::boxed::Box;
use skein_forge::api::{Check, Ci, Status};

use crate::Protection;

pub(crate) fn required(protection: &Protection) -> &[Box<[u8]>] {
    match protection {
        Protection::Rule(rule) => &rule.contexts,
        Protection::Unknown | Protection::Absent => &[],
    }
}

pub(crate) fn fold(checks: &[Status], required: &[Box<[u8]>]) -> Ci {
    let mut reported = false;
    let mut pending = false;
    for row in checks {
        if !required.is_empty() && !required.contains(&row.context) {
            continue;
        }
        reported = true;
        match row.check {
            Check::Failed => return Ci::Failed,
            Check::Pending => pending = true,
            Check::Passed => {}
        }
    }
    for context in required {
        let mut found = false;
        for row in checks {
            if row.context == *context {
                found = true;
                break;
            }
        }
        if !found {
            pending = true;
        }
    }
    if pending {
        Ci::Pending
    } else if reported {
        Ci::Passed
    } else {
        Ci::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{List, Time};

    #[test]
    fn commit_checks_fold_over_required_contexts() {
        struct Case {
            rows: &'static [(&'static [u8], Check)],
            required: &'static [&'static [u8]],
            want: Ci,
        }
        let cases = [
            Case { rows: &[], required: &[], want: Ci::None },
            Case { rows: &[(b"ci", Check::Passed)], required: &[], want: Ci::Passed },
            Case { rows: &[(b"ci", Check::Passed), (b"lint", Check::Pending)], required: &[], want: Ci::Pending },
            Case { rows: &[(b"ci", Check::Failed), (b"lint", Check::Pending)], required: &[], want: Ci::Failed },
            Case { rows: &[(b"lint", Check::Failed), (b"ci", Check::Passed)], required: &[b"ci"], want: Ci::Passed },
            Case { rows: &[(b"lint", Check::Passed)], required: &[b"ci"], want: Ci::Pending },
            Case { rows: &[(b"ci", Check::Passed)], required: &[b"ci", b"lint"], want: Ci::Pending },
            Case { rows: &[(b"ci", Check::Failed)], required: &[b"ci"], want: Ci::Failed },
        ];
        for case in cases {
            let mut rows = List::with_capacity(u32::try_from(case.rows.len()).expect("table row count"));
            for (context, check) in case.rows {
                rows.push(Status {
                    at: Time::ZERO,
                    context: Box::from(*context),
                    check: *check,
                    description: Box::new([]),
                    url: Box::new([]),
                    job: None,
                })
                .expect("sized from the table");
            }
            let mut contexts = List::with_capacity(u32::try_from(case.required.len()).expect("table context count"));
            for context in case.required {
                contexts.push(Box::from(*context)).expect("sized from the table");
            }
            assert_eq!(
                fold(rows.as_slice(), contexts.as_slice()),
                case.want,
                "rows={:?}, required={:?}",
                rows.as_slice(),
                contexts.as_slice()
            );
        }
    }

    #[test]
    fn protection_names_required_contexts() {
        assert!(required(&Protection::Unknown).is_empty());
        assert!(required(&Protection::Absent).is_empty());
        let rule = Protection::Rule(skein_forge::api::Protection {
            queue: false,
            signatures: false,
            linear: false,
            code_owners: false,
            bypass: skein_forge::api::Bypass::Never,
            branch: Box::from(b"main".as_slice()),
            contexts: Box::new([Box::from(b"ci".as_slice())]),
            approvals: 0,
            dismiss_stale: false,
        });
        assert_eq!(required(&rule), &[Box::<[u8]>::from(b"ci".as_slice())]);
    }
}
