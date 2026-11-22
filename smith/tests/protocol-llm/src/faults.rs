//! The fake's faults copied at their boundaries (smith/docs/design/faults.md, 3).
use skein_fake_llm_domain::{
    Request,
    api::{Error, Strike},
};
use skein_lib::{Time, Token};
use skein_llm_kit::faults::{HANG_AFTER_HEAD, IDLE_MID_STREAM, failed, struck};
use skein_world::faults::{FaultLedger, Injection, Struck};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub(crate) struct Faults {
    pub(crate) ledger: FaultLedger,
    strikes: Vec<(u32, Strike, Injection)>,
    calls: BTreeMap<Token, (Struck, bool, Option<Injection>)>,
}

impl Faults {
    pub(crate) fn strike(&mut self, credential: u32, strike: Strike, now: Time) {
        if self.strikes.iter().any(|(account, active, _)| *account == credential && struck(active) == struck(&strike)) {
            return;
        }
        let entry = self.ledger.inject(struck(&strike), now, identity(Some(credential), None));
        self.strikes.push((credential, strike, entry));
    }

    pub(crate) fn clear(&mut self, credential: u32, strike: &Strike, now: Time) {
        if let Some(index) = self
            .strikes
            .iter()
            .position(|(account, active, _)| *account == credential && struck(active) == struck(strike))
        {
            let (_, _, entry) = self.strikes.remove(index);
            self.ledger.stop(entry, now).expect("a strike clears once");
        }
    }

    pub(crate) fn call(&mut self, token: Token, credential: Option<u32>, name: String) {
        self.calls.insert(token, (identity(credential, Some(name)), false, None));
    }

    pub(crate) fn report(&mut self, now: Time, report: Request) -> Request {
        let (token, report) = match report {
            report @ (Request::Piece { call, .. } | Request::Hold { call, .. }) => (call, report),
            Request::Reply { to, result } => {
                let token = to.into_token();
                (token, Request::Reply { to: skein_lib::ReplyTo::new(token), result })
            }
        };
        let Some((identity, output, held)) = self.calls.get_mut(&token) else { return report };
        match &report {
            Request::Piece { .. } => {
                *output = true;
                if let Some(entry) = held.take() {
                    self.ledger.stop(entry, now).expect("released hold");
                }
            }
            Request::Hold { .. } => {
                let kind = if *output { IDLE_MID_STREAM } else { HANG_AFTER_HEAD };
                *held = Some(self.ledger.inject(kind, now, identity.clone()));
            }
            Request::Reply { result: Err(error), .. } => {
                let kind = failed(&error.error);
                let covered =
                    self.ledger.striking(now, identity).iter().any(|entry| self.ledger.get(*entry).kind == kind);
                if !covered || matches!(error.error, Error::Window { .. }) {
                    let entry = self.ledger.inject(kind, now, identity.clone());
                    let until =
                        if let Error::Window { reset, .. } = error.error { now.saturating_add(reset) } else { now };
                    self.ledger.stop(entry, until).expect("failure interval");
                }
            }
            Request::Reply { result: Ok(_), .. } => {}
        }
        report
    }
}

fn identity(credential: Option<u32>, request: Option<String>) -> Struck {
    Struck {
        account: credential.map(|index| if index == 0 { "a".into() } else { format!("account {index}") }),
        request,
        ..Struck::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_fake_llm_domain::api::Usage;
    use skein_lib::{Duration, ReplyTo};
    use skein_llm_kit::faults::{RATE_WINDOW, SPENT_CREDITS};

    const CALL: Token = Token::new(1);

    fn at(milliseconds: u64) -> Time {
        Time::ZERO.saturating_add(Duration::from_millis(milliseconds))
    }

    #[test]
    fn an_account_strike_is_one_interval_until_its_clear() {
        let mut faults = Faults::default();
        faults.strike(0, Strike::Spent, at(1));
        faults.strike(0, Strike::Spent, at(2));
        faults.clear(1, &Strike::Spent, at(3));
        faults.clear(0, &Strike::Spent, at(4));
        faults.clear(0, &Strike::Spent, at(5));
        let entries = faults.ledger.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, SPENT_CREDITS);
        assert_eq!((entries[0].from, entries[0].until), (at(1), Some(at(4))));
        assert_eq!(entries[0].struck.account.as_deref(), Some("a"));
        assert_eq!(entries[0].struck.request, None);
    }

    #[test]
    fn a_window_refusal_keeps_its_call_and_reset_beside_the_strike() {
        let mut faults = Faults::default();
        faults.strike(0, Strike::Window { tokens: 1, length: Duration::from_millis(20) }, at(0));
        faults.call(CALL, Some(0), "run 1 call 2".into());
        faults.report(
            at(5),
            Request::Reply {
                to: ReplyTo::new(CALL),
                result: Err(Error::Window { reset: Duration::from_millis(15), limit: 1, remaining: 0 }.into()),
            },
        );
        let entries = faults.ledger.entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].until, None);
        assert_eq!(entries[1].kind, RATE_WINDOW);
        assert_eq!((entries[1].from, entries[1].until), (at(5), Some(at(20))));
        assert_eq!(entries[1].struck.account.as_deref(), Some("a"));
        assert_eq!(entries[1].struck.request.as_deref(), Some("run 1 call 2"));
    }

    #[test]
    fn holds_record_their_place_and_stay_open() {
        for (output, expected) in [(false, HANG_AFTER_HEAD), (true, IDLE_MID_STREAM)] {
            let mut faults = Faults::default();
            faults.call(CALL, Some(0), "run 1 call 2".into());
            if output {
                faults.report(
                    at(1),
                    Request::Piece { call: CALL, usage: Usage::NONE, text: b"before".as_slice().into() },
                );
            }
            faults.report(at(2), Request::Hold { call: CALL, usage: Usage::NONE });
            let entry = &faults.ledger.entries()[0];
            assert_eq!(entry.kind, expected);
            assert_eq!((entry.from, entry.until), (at(2), None));
            assert_eq!(entry.struck.request.as_deref(), Some("run 1 call 2"));
        }
    }
}
