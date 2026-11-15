//! A registered fixture's kept and binary texts (jig's protocol/store/layouts.md, 8).
pub const FIRST: &str =
    "family fixture-layouts 1\nversioned record Growing { n: u8 }\nversioned record Stable { ok: bool }\n";
pub const KEPT: &str =
    "family fixture-layouts 2\nversioned record Growing { n: u16 }\nversioned record Stable { ok: bool }\n";
pub const CURRENT: &str = "family fixture-layouts 2\nversioned record Growing { n: u16, added: u8 = 7 }\nversioned record Stable { ok: bool }\n";
pub const WRITTEN: &str =
    "family fixture-layouts 2\nversioned record Growing { n: u16, added: u8 }\nversioned record Stable { ok: bool }\n";
pub const HOLDER: &str = "family fixture-holder 1\nrecord Changed { child: bytes 32 holds fixture-child Child }\nenum Value { changed: Changed, stable }\nversioned record Row { value: Value }\n";
pub const CHILD: &str = "family fixture-child 1\nversioned record Child { n: u8 }\n";
pub const CHILD_CURRENT: &str = "family fixture-child 1\nversioned record Child { n: u8, added: u8 = 7 }\n";
pub const CHILD_WRITTEN: &str = "family fixture-child 1\nversioned record Child { n: u8, added: u8 }\n";
pub const SETTLING_KEEPING: &str = "family fixture-layouts 2\nenum Choice { kept }\nversioned record Growing { old: u16, choice: Choice, added: u8 }\nversioned record Stable { ok: bool }\n";
pub const SETTLING_CONVERTING: &str = "family fixture-layouts 2\nenum Choice { kept }\nversioned record Growing { old: u16, choice: Choice }\nversioned record Stable { ok: bool }\n";
pub const SETTLING_DROPPING: &str = "family fixture-layouts 2\nenum Choice { kept, gone }\nversioned record Growing { n was old: u16, choice: Choice, added: u8 = 7 }\nversioned record Stable { ok: bool }\n";
pub const SETTLED: &str = "family fixture-layouts 2\nenum Choice { kept }\nversioned record Growing { n was old: u16, choice: Choice, added: u8 = 7 }\nversioned record Stable { ok: bool }\n";
pub const PATCH: &str = "patch 1\nowner fixture-layouts 1\nfamily fixture-layouts 1 to 2\nwiden Growing.n u8 u16\n";
