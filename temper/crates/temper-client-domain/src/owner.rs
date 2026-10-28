//! Exhaustive child askers (client/README.md, 3.3).
/// A child owning every handle and request it gives the shared client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    Pages,
    Forge,
    Runner,
}
/// A wire number as an asker's, as jig's `Owner::opaque` widens it: `u32::from` is not `const`.
const fn wide(number: u16) -> u32 {
    let [low, high] = number.to_le_bytes();
    u32::from_le_bytes([low, high, 0, 0])
}
impl Owner {
    #[must_use]
    pub const fn asker(self) -> jig_client::Asker {
        match self {
            Self::Pages => {
                jig_client::Asker(wide(temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Pages)))
            }
            Self::Runner => {
                jig_client::Asker(wide(temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Runner)))
            }
            Self::Forge => {
                jig_client::Asker(wide(temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)))
            }
        }
    }
    #[must_use]
    pub fn of(asker: jig_client::Asker) -> Option<Self> {
        if asker == Self::Pages.asker() {
            Some(Self::Pages)
        } else if asker == Self::Forge.asker() {
            Some(Self::Forge)
        } else if asker == Self::Runner.asker() {
            Some(Self::Runner)
        } else {
            None
        }
    }
}
