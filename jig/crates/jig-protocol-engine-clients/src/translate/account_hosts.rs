//! Retained configured host identities (domain/accounts-sources.md, 1).
use super::w;
use jig_domain_people as p;
use skein_lib::List;
pub(super) fn encode(rows: &[p::AccountHostFailure]) -> Result<List<w::AccountHostFailure>, w::Problem> {
    let Ok(count) = u32::try_from(rows.len()) else { return Err(count_problem_value()) };
    if count > 128 {
        return Err(count_problem_value());
    }
    let mut out = List::with_capacity(count);
    for row in rows {
        let row = crate::generated::client::account_host_failure_to_record(row);
        if out.push(row).is_err() {
            return Err(count_problem_value());
        }
    }
    Ok(out)
}
fn count_problem_value() -> w::Problem {
    w::Problem { path: w::paths::OUTCOME_ACCOUNT_SIGNED_OUT_FAILED_HOSTS, reason: skein_codec::Reason::Bound }
}
