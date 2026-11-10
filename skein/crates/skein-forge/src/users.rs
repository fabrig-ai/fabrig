use alloc::boxed::Box;
use skein_lib::Lru;

/// Logins by forge and id (forge/README.md, 4.4).
pub type Users = Lru<(u16, u64), Box<[u8]>>;
