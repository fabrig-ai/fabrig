//! A row refused before reaching the root (temper/docs/design/protocol/store.md, 5).
use alloc::boxed::Box;

/// The relative application key of a row this build cannot read.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct UnreadableRow {
    pub key: Box<[u8]>,
}
impl UnreadableRow {
    /// The family's durable number and variant, independent of the value's codec.
    #[must_use]
    pub fn family(&self) -> Option<(u8, &'static str)> {
        use temper_protocol_store::keys;
        let number = *self.key.first()?;
        let name = match number {
            0 => "Call",
            1 => "ProposedEffect",
            2 => "Repository",
            3 => "Hold",
            4 => "Names",
            5 => "Subscription",
            6 => "BranchHead",
            7 => "PullState",
            8 => "Ci",
            9 => "Landed",
            10 => "Entry",
            11 => "Client",
            12 => "Change",
            13 => "Issue",
            14 => "Release",
            15 => "Application",
            16 => "Writer",
            keys::BINDING => "Binding",
            keys::SETTING => "Setting",
            keys::FLIGHT => "Flight",
            22 => "Delivery",
            23 => "Landing",
            24 => "Receipt",
            keys::FORGE => "Forge",
            keys::FORGE_NUMBER => "ForgeNumber",
            keys::PLAYGROUND => "Playground",
            keys::TIER => "Tier",
            keys::VERIFICATION => "Verification",
            keys::LANDING_LOG => "LandingLog",
            31 => "Finding",
            32 => "Mark",
            33 => "QaPass",
            keys::DISMISSED => "Dismissed",
            keys::MAKING => "Making",
            keys::PROJECT_FACTS => "ProjectFacts",
            keys::LOOKED => "Looked",
            keys::AWAY => "Away",
            keys::DECIDED => "Decided",
            keys::CLOSER => "Closer",
            keys::LANDED_DRAFT => "LandedDraft",
            _ => return None,
        };
        Some((number, name))
    }
}
