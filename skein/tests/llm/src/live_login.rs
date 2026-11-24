//! Admission of a lent live credential over facts supplied by the reader
//! (skein/docs/design/llm/README.md, 5).

use std::io;
use std::path::{Path, PathBuf};

use skein_json::{Document, Kind, document, tokenizer};

/// The longest login file the harness reads: one is a few kilobytes.
const LOGIN_BYTES: u32 = 1 << 16;
/// A login's grammar: no string or number is longer than its file, and a
/// login nests a few containers deep. The chunk only paces `parse`.
const LOGIN_JSON: tokenizer::Limits =
    tokenizer::Limits { depth: 8, string: LOGIN_BYTES, number: LOGIN_BYTES, chunk: 4096, length: LOGIN_BYTES };
/// Each record reads at least a byte of the file, and no text decodes longer than it is.
const LOGIN_RETAINED: document::Limits = document::Limits { tokens: LOGIN_BYTES, text: LOGIN_BYTES };

#[derive(Clone, Copy)]
pub enum Login {
    Codex,
    Anthropic,
}

impl Login {
    /// The caller names a file, or for Codex, a directory holding `auth.json`.
    #[must_use]
    pub fn file(self, named: &Path, named_is_dir: bool) -> PathBuf {
        if matches!(self, Self::Codex) && named_is_dir { named.join("auth.json") } else { named.to_path_buf() }
    }

    /// Admit the opened entry by kind and permissions before asking for its
    /// bytes. The reader gets a cap one byte past the bound so this policy can
    /// distinguish a full file from an oversized one. No refusal holds bytes read.
    pub fn admit(
        self,
        path: &Path,
        regular: bool,
        mode: u32,
        read: impl FnOnce(u64) -> io::Result<Vec<u8>>,
    ) -> Result<Box<[u8]>, String> {
        let noun = match self {
            Self::Codex => "login",
            Self::Anthropic => "key",
        };
        let path = path.display();
        if !regular || mode & 0o7777 != 0o600 {
            return Err(format!("{path}: {noun} file must have mode 0600"));
        }
        let text = read(u64::from(LOGIN_BYTES) + 1).map_err(|error| format!("{path}: {error}"))?;
        if text.len() > usize::try_from(LOGIN_BYTES).expect("u32 fits usize") {
            return Err(format!("{path}: {noun} file is too large"));
        }
        match self {
            Self::Codex => openai_bearer(&text).map_err(|refusal| format!("{path}: {refusal}")),
            Self::Anthropic => {
                let value = text.trim_ascii();
                if value.is_empty() { Err(format!("{path}: key file is empty")) } else { Ok(value.into()) }
            }
        }
    }
}

/// The public route's bearer: an API key's `OPENAI_API_KEY` where a string,
/// refused empty, else a sign-in's access token.
fn openai_bearer(text: &[u8]) -> Result<Box<[u8]>, String> {
    let document = skein_json::parse(text, &LOGIN_JSON, &LOGIN_RETAINED)
        .map_err(|error| format!("not a login's JSON: {error:?}"))?;
    let key = document.member(0, b"OPENAI_API_KEY").and_then(|at| document.token(at));
    let field: &[&str] = if key.is_some_and(|record| record.kind == Kind::String) {
        &["OPENAI_API_KEY"]
    } else {
        &["tokens", "access_token"]
    };
    login_string(&document, field).ok_or_else(|| format!("no non-empty string at .{}", field.join(".")))
}

/// The decoded bytes of the string at `field` below the root object, or
/// nothing when a member is absent, is not a string, or is empty.
fn login_string(document: &Document, field: &[&str]) -> Option<Box<[u8]>> {
    let mut at = 0;
    for name in field {
        at = document.member(at, name.as_bytes())?;
    }
    let record = document.token(at).filter(|record| record.kind == Kind::String)?;
    document.text(record).filter(|text| !text.is_empty()).map(Box::from)
}
