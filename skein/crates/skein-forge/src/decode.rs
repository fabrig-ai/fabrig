//! Malformed document fields (skein/docs/design/forge/README.md, 4.4).
use skein_json::tokenizer;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Malformed {
    Json(tokenizer::Error),
    Shape { field: &'static [u8] },
    Missing { field: &'static [u8] },
    Range { field: &'static [u8] },
    TooLong { field: &'static [u8] },
    Id { field: &'static [u8] },
    Time { field: &'static [u8] },
    Value { field: &'static [u8] },
    Bound { field: &'static [u8] },
}
