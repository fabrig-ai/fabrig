//! A named model call's failure (jig/docs/design/domain/hosts/hub.md, 3).
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelFailure {
    pub endpoint: Box<[u8]>,
    pub model: Box<[u8]>,
    pub kind: ModelFailureKind,
    pub evidence: ModelEvidence,
    pub status: Option<u16>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ModelFailureKind {
    Unavailable,
    Overloaded,
    RateLimited,
    Exhausted,
    TimedOut,
    ContextTooLong,
    Invalid,
    Refused,
    Unsupported,
    Unauthorized,
    Protocol,
    Cancelled,
    Limit,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ModelEvidence {
    Unsent,
    Unknown,
    Response,
}

impl ModelFailure {
    pub const NAME_BYTES: u32 = 40;

    #[must_use]
    pub fn outage(&self) -> bool {
        match self.evidence {
            ModelEvidence::Unsent => match self.kind {
                ModelFailureKind::Unavailable | ModelFailureKind::Overloaded => true,
                ModelFailureKind::RateLimited
                | ModelFailureKind::Exhausted
                | ModelFailureKind::TimedOut
                | ModelFailureKind::ContextTooLong
                | ModelFailureKind::Invalid
                | ModelFailureKind::Refused
                | ModelFailureKind::Unsupported
                | ModelFailureKind::Unauthorized
                | ModelFailureKind::Protocol
                | ModelFailureKind::Cancelled
                | ModelFailureKind::Limit => false,
            },
            ModelEvidence::Unknown | ModelEvidence::Response => false,
        }
    }

    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        let bound = skein_lib::length::usize_of(Self::NAME_BYTES);
        let endpoint = self.endpoint.get(..self.endpoint.len().min(bound)).expect("bounded endpoint");
        let model = self.model.get(..self.model.len().min(bound)).expect("bounded model");
        let status = Decimal::of(u64::from(self.status.unwrap_or(0)));
        let (label, value): (&[u8], &[u8]) = match self.status {
            Some(_) => (b", status ", status.as_bytes()),
            None => (b"", b""),
        };
        let parts: [&[u8]; 6] = [b": ", self.kind.words(), label, value, b", ", self.evidence.words()];
        let mut length =
            endpoint.len().checked_add(1).expect("bounded endpoint").checked_add(model.len()).expect("bounded names");
        for part in parts {
            length = length.checked_add(part.len()).expect("bounded model failure words");
        }
        let mut writer = Writer::new(length);
        name(&mut writer, endpoint);
        writer.put(b" ").expect("measured separator");
        name(&mut writer, model);
        for part in parts {
            writer.put(part).expect("measured model failure words");
        }
        writer.finish()
    }
}

fn name(writer: &mut Writer, bytes: &[u8]) {
    for byte in bytes {
        let printable = if (b' '..=b'~').contains(byte) { *byte } else { b'?' };
        writer.put(&[printable]).expect("measured name");
    }
}

impl ModelFailureKind {
    const fn words(self) -> &'static [u8] {
        match self {
            Self::Unavailable => b"unavailable",
            Self::Overloaded => b"overloaded",
            Self::RateLimited => b"rate limited",
            Self::Exhausted => b"exhausted",
            Self::TimedOut => b"timed out",
            Self::ContextTooLong => b"context too long",
            Self::Invalid => b"invalid",
            Self::Refused => b"refused",
            Self::Unsupported => b"unsupported",
            Self::Unauthorized => b"unauthorized",
            Self::Protocol => b"protocol",
            Self::Cancelled => b"cancelled",
            Self::Limit => b"limit",
        }
    }
}

impl ModelEvidence {
    const fn words(self) -> &'static [u8] {
        match self {
            Self::Unsent => b"unsent",
            Self::Unknown => b"unknown",
            Self::Response => b"response",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(kind: ModelFailureKind, evidence: ModelEvidence, status: Option<u16>) -> ModelFailure {
        ModelFailure {
            endpoint: b"anthropic".as_slice().into(),
            model: b"claude-opus-5-5".as_slice().into(),
            kind,
            evidence,
            status,
        }
    }

    #[test]
    fn words_name_the_kind_evidence_and_optional_status() {
        use ModelEvidence::{Response, Unknown, Unsent};
        use ModelFailureKind::*;
        assert_eq!(
            failure(Unavailable, Unsent, Some(403)).words().as_ref(),
            b"anthropic claude-opus-5-5: unavailable, status 403, unsent"
        );
        assert_eq!(
            failure(Overloaded, Unsent, None).words().as_ref(),
            b"anthropic claude-opus-5-5: overloaded, unsent"
        );
        for (kind, word) in [
            (Unavailable, "unavailable"),
            (Overloaded, "overloaded"),
            (RateLimited, "rate limited"),
            (Exhausted, "exhausted"),
            (TimedOut, "timed out"),
            (ContextTooLong, "context too long"),
            (Invalid, "invalid"),
            (Refused, "refused"),
            (Unsupported, "unsupported"),
            (Unauthorized, "unauthorized"),
            (Protocol, "protocol"),
            (Cancelled, "cancelled"),
            (Limit, "limit"),
        ] {
            for (evidence, word_evidence) in [(Unsent, "unsent"), (Unknown, "unknown"), (Response, "response")] {
                let words = failure(kind, evidence, None).words();
                let rest = words.strip_prefix(b"anthropic claude-opus-5-5: ").expect("call names");
                let rest = rest.strip_prefix(word.as_bytes()).expect("kind word");
                let rest = rest.strip_prefix(b", ").expect("evidence separator");
                assert_eq!(rest, word_evidence.as_bytes());
            }
        }
    }

    #[test]
    #[expect(clippy::disallowed_methods, reason = "the words contract promises valid UTF-8")]
    fn names_are_printable_and_the_longest_words_fit_the_reason() {
        let mut value = failure(ModelFailureKind::ContextTooLong, ModelEvidence::Response, Some(u16::MAX));
        value.endpoint = b"a\0\n\x7f\x80\xff ~".as_slice().into();
        value.model = b"m\x1b\xc3\xa9".as_slice().into();
        assert_eq!(value.words().as_ref(), b"a????? ~ m???: context too long, status 65535, response");
        value.endpoint = Box::new([b'e'; 40]);
        value.model = Box::new([b'm'; 40]);
        let words = value.words();
        assert_eq!(words.len(), 123);
        core::str::from_utf8(&words).expect("model failure words are UTF-8");
    }

    #[test]
    fn only_an_unsent_unavailable_or_overloaded_call_is_an_outage() {
        use ModelEvidence::{Response, Unknown, Unsent};
        use ModelFailureKind::{Invalid, Overloaded, Unavailable};
        for (kind, evidence, outage) in [
            (Unavailable, Unsent, true),
            (Overloaded, Unsent, true),
            (Unavailable, Response, false),
            (Unavailable, Unknown, false),
            (Invalid, Unsent, false),
        ] {
            assert_eq!(failure(kind, evidence, None).outage(), outage, "{kind:?} {evidence:?}");
        }
    }
}
