//! Forge records from temper-secrets (protocol/README.md, 9; protocol/credentials.md, 2.1).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[path = "generated/v1.rs"]
pub mod v1;
mod codec;
// Browser clients need only the schema and byte codecs; startup access pulls
// in the native forge/TLS stack and is enabled explicitly by its consumers.
#[cfg(feature = "forge")]
mod forge;
pub use codec::{PASTED_BYTES, PASTED_PAYLOAD, PastedError, decode, decode_pasted, encode, encode_pasted};
#[cfg(feature = "forge")]
pub use forge::{ForgeRows, NotConnected, Origin, Rows, Start, access, origin};

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
}
/// Every forge record, in wire-code order (temper's contracts T5).
pub const ITEMS: [Item; 10] = [
    Item::AdminToken,
    Item::ApiToken,
    Item::GitToken,
    Item::User,
    Item::WebhookSecret,
    Item::AppId,
    Item::AppSlug,
    Item::AppKey,
    Item::Accepted,
    Item::Origin,
];
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
        _ => return None,
    };
    Some((u16::try_from(number >> 48_u32).ok()?, u16::try_from((number >> 16_u32) & 0xffff_ffff).ok()?, item))
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Unreadable;

#[cfg(test)]
mod tests {
    #[test]
    fn items_hold_each_item_once_in_code_order() {
        for (index, item) in super::ITEMS.iter().enumerate() {
            assert_eq!(usize::from(item.code()), index.checked_add(1).expect("ten items"));
            assert_eq!(super::parse(super::number(7, *item)), Some((super::FAMILY_FORGE, 7, *item)));
        }
    }
}
