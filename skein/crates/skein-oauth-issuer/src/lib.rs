//! Stateless `OpenID` Connect issuer (oauth/issuer.md): the caller owns
//! people, code spending, access tokens, clocks and randomness.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod documents;
mod form;
mod requests;
mod settings;
#[cfg(test)]
mod tests;

use alloc::boxed::Box;
use skein_lib::{List, Secret};

pub use settings::{Limits, valid};

#[derive(Debug)]
pub struct Settings {
    pub issuer: Box<[u8]>,
    pub authorization_endpoint: Box<[u8]>,
    pub token_endpoint: Box<[u8]>,
    pub userinfo_endpoint: Box<[u8]>,
    pub jwks_uri: Box<[u8]>,
    pub clients: List<Client>,
    /// PKCS#8 `PrivateKeyInfo` DER, RSA of 2048 to 4096 bits.
    pub key: Secret,
}
/// A client registration (skein/docs/design/oauth/issuer.md, 2).
#[derive(Debug)]
pub struct Client {
    pub id: Box<[u8]>,
    /// None for a public client.
    pub secret: Option<Secret>,
    pub redirect_uri: Box<[u8]>,
}
#[derive(Debug)]
pub struct Issuer {
    settings: Settings,
    clients: Box<[Client]>,
    limits: Limits,
    modulus: Box<[u8]>,
    exponent: Box<[u8]>,
    kid: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    Limits,
    Address,
    Client,
    Key,
    Claims,
    Sign,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Authorization {
    pub client: Box<[u8]>,
    pub redirect_uri: Box<[u8]>,
    pub state: Option<Box<[u8]>>,
    pub nonce: Option<Box<[u8]>>,
    pub challenge: [u8; 43],
}
// skein/docs/design/oauth/README.md, 4: state and nonce bytes stay out of Debug.
impl core::fmt::Debug for Authorization {
    #[expect(clippy::manual_map, reason = "explicit matches keep closures out of step code")]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let state_len = match &self.state {
            Some(state) => Some(state.len()),
            None => None,
        };
        let nonce_len = match &self.nonce {
            Some(nonce) => Some(nonce.len()),
            None => None,
        };
        f.debug_struct("Authorization")
            .field("client", &self.client)
            .field("redirect_uri", &self.redirect_uri)
            .field("state_len", &state_len)
            .field("nonce_len", &nonce_len)
            .field("challenge", &self.challenge)
            .finish()
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    Malformed,
    UnknownClient,
    UnknownRedirect,
    Back { location: Box<[u8]> },
}
#[expect(clippy::partial_pub_fields, reason = "oauth/issuer.md 5 exposes the authenticated client and code alone")]
pub struct TokenRequest {
    pub client: Box<[u8]>,
    pub code: Secret,
    redirect_uri: Box<[u8]>,
    verifier: Box<[u8]>,
}
// skein/docs/design/oauth/README.md, 4: a verifier's bytes stay out of Debug.
impl core::fmt::Debug for TokenRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TokenRequest")
            .field("client", &self.client)
            .field("code", &self.code)
            .field("redirect_uri", &self.redirect_uri)
            .field("verifier_len", &self.verifier.len())
            .finish()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TokenError {
    InvalidRequest,
    InvalidClient { header: bool },
    InvalidGrant,
    UnsupportedGrantType,
}
#[derive(Debug)]
pub struct Answer {
    pub status: u16,
    pub challenge: Option<Box<[u8]>>,
    pub body: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Person {
    pub subject: Box<[u8]>,
    pub username: Option<Box<[u8]>>,
    pub email: Option<Email>,
    pub name: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Email {
    pub address: Box<[u8]>,
    pub verified: bool,
}
