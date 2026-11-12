//! Forge records from temper-secrets (protocol/README.md, 7; protocol/credentials.md, 2.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v1.rs"]
pub mod v1;
mod codec;
pub use codec::{
    PASTED_BYTES, PASTED_PAYLOAD, PastedError, decode, decode_pasted, decoded_origin, encode, encode_pasted,
};

pub const FAMILY_FORGE: u16 = 1;
pub const VERSION: u16 = 1;
pub const CARRIED: u64 = 1 << 63;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u16)]
pub enum Item {
    AdminToken = 1,
    ApiToken,
    GitToken,
    User,
    WebhookSecret,
    AppId,
    AppSlug,
    AppKey,
    Accepted,
    Origin,
    AppMade = 12,
}
impl Item {
    #[must_use]
    pub const fn code(self) -> u16 {
        match self {
            Self::AdminToken => 1,
            Self::ApiToken => 2,
            Self::GitToken => 3,
            Self::User => 4,
            Self::WebhookSecret => 5,
            Self::AppId => 6,
            Self::AppSlug => 7,
            Self::AppKey => 8,
            Self::Accepted => 9,
            Self::Origin => 10,
            Self::AppMade => 12,
        }
    }
}
#[must_use]
pub fn number(forge: u16, item: Item) -> u64 {
    (u64::from(FAMILY_FORGE) << 48) | (u64::from(forge) << 16) | u64::from(item.code())
}
#[must_use]
pub fn parse(number: u64) -> Option<(u16, u16, Item)> {
    if number >= CARRIED {
        return None;
    }
    let item = match number & 0xffff {
        1 => Item::AdminToken,
        2 => Item::ApiToken,
        3 => Item::GitToken,
        4 => Item::User,
        5 => Item::WebhookSecret,
        6 => Item::AppId,
        7 => Item::AppSlug,
        8 => Item::AppKey,
        9 => Item::Accepted,
        10 => Item::Origin,
        12 => Item::AppMade,
        _ => return None,
    };
    Some((u16::try_from(number >> 48_u32).ok()?, u16::try_from((number >> 16_u32) & 0xffff_ffff).ok()?, item))
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unreadable;
