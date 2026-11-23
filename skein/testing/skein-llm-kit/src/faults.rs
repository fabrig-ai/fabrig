//! Kinds copied from fake outputs into a world's ledger (testing/fakes/llm.md, 5).
use skein_fake_llm_domain::api::{Cut, Error, Strike};
use skein_world::faults::Kind;

pub const RATE_WINDOW: Kind = Kind { family: "provider", name: "rate window" };
pub const SPENT_CREDITS: Kind = Kind { family: "provider", name: "spent credits" };
pub const EXPIRED_SIGN_IN: Kind = Kind { family: "provider", name: "expired sign-in" };
pub const UNSERVED_MODEL: Kind = Kind { family: "provider", name: "unserved model" };
pub const CUT_BEFORE_OUTPUT: Kind = Kind { family: "provider", name: "cut before output" };
pub const CUT_MID_TEXT: Kind = Kind { family: "provider", name: "cut mid-text" };
pub const CUT_MID_CALL: Kind = Kind { family: "provider", name: "cut mid-call" };
pub const HANG_AFTER_HEAD: Kind = Kind { family: "provider", name: "hang after head" };
pub const IDLE_MID_STREAM: Kind = Kind { family: "provider", name: "idle mid-stream" };

/// The class answered by the domain, without keeping another log.
#[must_use]
pub fn failed(error: &Error) -> Kind {
    let name = match error {
        Error::Spent => return SPENT_CREDITS,
        Error::SignedOut => return EXPIRED_SIGN_IN,
        Error::Unserved => return UNSERVED_MODEL,
        Error::Window { .. } => return RATE_WINDOW,
        Error::Cut(Cut::BeforeOutput) => return CUT_BEFORE_OUTPUT,
        Error::Cut(Cut::MidText) => return CUT_MID_TEXT,
        Error::Cut(Cut::MidCall) => return CUT_MID_CALL,
        Error::Overloaded => "overloaded",
        Error::Unavailable => "unavailable",
        Error::RateLimited { .. } => "rate limited",
        Error::Exhausted { .. } => "exhausted",
        Error::Refused => "refused",
        Error::Unsupported => "unsupported",
        Error::ContextTooLong => "context too long",
        Error::InvalidRequest => "invalid",
        Error::Unauthorized => "unauthorized",
    };
    Kind { family: "provider", name }
}

/// The account fault the world installed, recorded until its clear.
#[must_use]
pub fn struck(strike: &Strike) -> Kind {
    match strike {
        Strike::Spent => SPENT_CREDITS,
        Strike::SignedOut => EXPIRED_SIGN_IN,
        Strike::Unserved { .. } => UNSERVED_MODEL,
        Strike::Window { .. } => RATE_WINDOW,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Duration;

    #[test]
    fn each_fault_names_its_provider_ledger_kind() {
        for (kind, name) in [
            (RATE_WINDOW, "rate window"),
            (SPENT_CREDITS, "spent credits"),
            (EXPIRED_SIGN_IN, "expired sign-in"),
            (UNSERVED_MODEL, "unserved model"),
            (CUT_BEFORE_OUTPUT, "cut before output"),
            (CUT_MID_TEXT, "cut mid-text"),
            (CUT_MID_CALL, "cut mid-call"),
            (HANG_AFTER_HEAD, "hang after head"),
            (IDLE_MID_STREAM, "idle mid-stream"),
        ] {
            assert_eq!(kind, Kind { family: "provider", name });
        }
        for (error, name) in [
            (Error::Spent, "spent credits"),
            (Error::SignedOut, "expired sign-in"),
            (Error::Unserved, "unserved model"),
            (Error::Window { reset: Duration::ZERO, limit: 1, remaining: 0 }, "rate window"),
            (Error::Cut(Cut::BeforeOutput), "cut before output"),
            (Error::Cut(Cut::MidText), "cut mid-text"),
            (Error::Cut(Cut::MidCall), "cut mid-call"),
            (Error::Overloaded, "overloaded"),
            (Error::Unavailable, "unavailable"),
            (Error::RateLimited { retry_after: Duration::ZERO }, "rate limited"),
            (Error::Exhausted { retry_after: None }, "exhausted"),
            (Error::Refused, "refused"),
            (Error::Unsupported, "unsupported"),
            (Error::ContextTooLong, "context too long"),
            (Error::InvalidRequest, "invalid"),
            (Error::Unauthorized, "unauthorized"),
        ] {
            assert_eq!(failed(&error), Kind { family: "provider", name });
        }
        for (strike, name) in [
            (Strike::Spent, "spent credits"),
            (Strike::SignedOut, "expired sign-in"),
            (Strike::Unserved { model: b"model".as_slice().into() }, "unserved model"),
            (Strike::Window { tokens: 1, length: Duration::from_secs(1) }, "rate window"),
        ] {
            assert_eq!(struck(&strike), Kind { family: "provider", name });
        }
    }
}
