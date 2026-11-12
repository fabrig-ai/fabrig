//! Malformed document fields (skein/docs/design/forge/README.md, 4.4).
use skein_json::tokenizer;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Malformed {
    Json(tokenizer::Error),
    /// A JSON document past the component's body bound (forge/forgejo.md, 1.3).
    DocumentTooLong {
        limit: u32,
        size: u64,
    },
    Shape {
        field: &'static [u8],
    },
    Missing {
        field: &'static [u8],
    },
    Range {
        field: &'static [u8],
    },
    TooLong {
        field: &'static [u8],
    },
    Id {
        field: &'static [u8],
    },
    Time {
        field: &'static [u8],
    },
    Value {
        field: &'static [u8],
    },
    Bound {
        field: &'static [u8],
    },
}
impl Malformed {
    /// Carries the tokenizer's length bound and bytes delivered at its refusal.
    #[must_use]
    pub fn with_document_bound(self, limit: u32, size: u64) -> Self {
        if self == Self::Json(tokenizer::Error::TooLong) { Self::DocumentTooLong { limit, size } } else { self }
    }
}
