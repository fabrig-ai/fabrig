//! Durable refusal facts and unsaved retry kinds (client/domain/contract/answers.md,
//! section 1). A judged settings refusal also carries the connector's bounded words.
pub use jig_domain_contract::ops::party::{
    Accepter, Again, CeilingDimension, CeilingHolder, DependencyStatus, Ending, HoldOwner, Lack, LimitKind, Malformed,
    Party, QuestionAnswerer, Refusal,
};

/// Maximum named tasks in circle, settings-full and live-work refusal records.
pub const NAMED_TASKS: u32 = 32;
