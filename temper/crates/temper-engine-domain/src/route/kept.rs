//! Kept-act routing (temper/docs/design/draft/playground-reset.md, 4).
//! The kept-acts body supplies sending and saved-answer handling.
use super::{Decision, Domain, Env, Limits, ReplyTo, forge, people};

pub(crate) fn send(_domain: &mut Domain, _row: forge::Kept) {}

#[expect(dead_code, reason = "kept act replies are unreachable until kept-acts sends acts")]
pub(crate) fn acted(
    _domain: &mut Domain,
    _env: &Env<Limits>,
    _decision: &mut Decision,
    _to: ReplyTo,
    _outcome: people::Outcome,
) {
    unreachable!("kept-acts does not send acts yet");
}
