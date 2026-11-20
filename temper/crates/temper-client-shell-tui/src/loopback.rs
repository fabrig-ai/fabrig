//! The browser's one return (temper/docs/design/client/tui/pages/sign-in.md, 4).
use skein_http::{Header, server};
use skein_lib::{
    Secret,
    base64::{self, Alphabet},
};

#[derive(Clone, PartialEq, Eq)]
pub struct Returned {
    pub get: bool,
    pub target: Box<[u8]>,
    pub host: Option<Box<[u8]>>,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Answer {
    Redirect { to: Box<[u8]> },
    NotFound,
}

/// Read only the exact origin, path and unpadded URL-base64 grant.
#[must_use]
pub fn grant(returned: &Returned, port: u16) -> Option<Secret> {
    if !returned.get || returned.host.as_deref()? != format!("127.0.0.1:{port}").as_bytes() {
        return None;
    }
    target_grant(&returned.target)
}

/// Read the grant in a target even if the return wasn't accepted.
pub(crate) fn target_grant(target: &[u8]) -> Option<Secret> {
    let text = target.strip_prefix(b"/approved?grant=")?;
    if text.len() != 43 {
        return None;
    }
    let bytes = base64::decode(Alphabet::Url, text, 32).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    Some(Secret::new(bytes))
}

impl Answer {
    /// The server supplies the length, connection and date fields.
    #[must_use]
    pub fn response(self) -> server::Response {
        let mut headers = vec![
            Header { name: b"Referrer-Policy".as_slice().into(), value: b"no-referrer".as_slice().into() },
            Header { name: b"Cache-Control".as_slice().into(), value: b"no-store".as_slice().into() },
        ];
        let status = match self {
            Self::Redirect { to } => {
                headers.push(Header { name: b"Location".as_slice().into(), value: to });
                303
            }
            Self::NotFound => 404,
        };
        server::Response { status, headers: headers.into(), body: server::Body::None, close: true }
    }
}

#[cfg(test)]
mod tests;
