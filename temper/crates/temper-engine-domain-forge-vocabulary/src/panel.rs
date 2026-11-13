//! The connector's whole change document (domain/forge/change/README.md, 5; client/forge/README.md, 2.1).
use alloc::boxed::Box;
use temper_engine_domain_forge_change as change;

/// Tasks ahead of a queued change (client/forge/records.md).
pub const PANEL_AHEAD: u32 = 64;
/// Gates in a change document (client/forge/records.md).
pub const PANEL_GATES: u32 = 64;
/// Bytes in each repository or branch name (client/forge/records.md).
pub const PANEL_NAME_BYTES: u32 = 1024;
/// Subjects marked together (domain/root.md, section 9).
pub const MARKED: u32 = 32;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PanelStep {
    Ended,
    Ready { landing: Option<u64> },
    Preparing { landing: Option<u64> },
    Producing,
    Opening,
    Recreating,
    Reopening,
    Checking,
    Gating,
    Queued { place: u32, ahead: Box<[u64]> },
    First,
    Updating,
    Resolving,
    Repairing(change::Repair),
    Landing,
    Landed { merge: [u8; 32] },
    Verifying { merge: [u8; 32], tiers: Box<[Box<[u8]>]> },
    Red { merge: [u8; 32], tiers: Box<[Box<[u8]>]> },
    Cleared { merge: [u8; 32], tiers: Box<[Box<[u8]>]>, uncleared: bool },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PanelGate {
    pub number: u64,
    pub kind: change::GateKind,
    pub blocking: bool,
    pub verdict: change::Status,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PanelUsed {
    pub used: u32,
    pub limit: u32,
}
/// A change's questions (domain/forge/change/questions.md, 1; client/forge/records.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PanelQuestions {
    pub route: change::Route,
    pub taken: PanelUsed,
    pub open: Box<[PanelQuestion]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PanelQuestion {
    pub asker: u64,
    pub number: u64,
    pub line: Box<[u8]>,
    pub stands: PanelStands,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PanelStands {
    Taken,
    Asked,
    Held,
}
/// Last judged reviews, unsaved and bound to the merge head (domain/forge/change/gates.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ReviewFacts {
    pub size: Option<skein_forge::api::Size>,
    pub head: Option<[u8; 32]>,
    pub reviews_complete: bool,
    pub reviewers: Box<[crate::Reviewer]>,
    pub unmapped: Box<[UnmappedReview]>,
}
/// A forge account without a verified project person; its display name grants nothing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UnmappedReview {
    pub account: u64,
    pub head: [u8; 32],
    pub verdict: skein_forge::api::Verdict,
}
/// Whether the current head's reviews have been read whole.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReviewRead {
    /// No judged reviews at the current head, including after restart.
    Unread,
    /// Reviews were read only in part.
    Incomplete,
    /// The current head's review set was read whole.
    Complete,
}
/// One enforcing landing rule's approval requirement and its counted reviews.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PanelApproval {
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub freshness: crate::JudgeFreshness,
    pub required: u32,
    pub read: ReviewRead,
    pub counted: Box<[crate::Reviewer]>,
    pub remaining: Option<u32>,
    pub refused: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Panel {
    pub way: change::Way,
    pub in_landing: Option<u64>,
    pub task: u64,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub branch: Box<[u8]>,
    pub landing: Box<[u8]>,
    pub head: Option<[u8; 32]>,
    pub pull: Option<u64>,
    pub step: PanelStep,
    pub hold: Option<change::Hold>,
    pub runs: Option<u64>,
    pub ci: change::Status,
    /// Whether this repository and its effective landing rules ask for CI.
    pub ci_required: bool,
    pub gates: Box<[PanelGate]>,
    pub approvals: Box<[PanelApproval]>,
    pub unmapped_reviews: Box<[UnmappedReview]>,
    pub repairs: PanelUsed,
    pub resolutions: PanelUsed,
    pub updates: PanelUsed,
    pub questions: PanelQuestions,
}
impl Panel {
    /// Heap bytes owned by the approval explanation, charged at document admission.
    #[must_use]
    pub fn approval_bytes(&self) -> usize {
        let mut bytes =
            size_of_val(self.approvals.as_ref()).saturating_add(size_of_val(self.unmapped_reviews.as_ref()));
        for approval in &self.approvals {
            bytes = bytes.saturating_add(size_of_val(approval.counted.as_ref()));
        }
        bytes
    }
    #[must_use]
    pub fn within(&self) -> bool {
        let names = skein_lib::usize_of(PANEL_NAME_BYTES);
        self.owner.len() <= names
            && self.name.len() <= names
            && self.branch.len() <= names
            && self.landing.len() <= names
            && self.gates.len() <= skein_lib::usize_of(PANEL_GATES)
            && approvals_within(&self.approvals)
            && self.unmapped_reviews.len() <= 256
            && self.questions.open.len() <= 8
            && question_lines_within(&self.questions.open)
            && match &self.step {
                PanelStep::Queued { place, ahead } => *place > 0 && ahead.len() <= skein_lib::usize_of(PANEL_AHEAD),
                PanelStep::Verifying { tiers, .. }
                | PanelStep::Red { tiers, .. }
                | PanelStep::Cleared { tiers, .. } => tiers.len() <= 32 && tier_lines_within(tiers),
                PanelStep::Ready { .. }
                | PanelStep::Preparing { .. }
                | PanelStep::Producing
                | PanelStep::Opening
                | PanelStep::Recreating
                | PanelStep::Reopening
                | PanelStep::Checking
                | PanelStep::Gating
                | PanelStep::First
                | PanelStep::Updating
                | PanelStep::Resolving
                | PanelStep::Repairing(_)
                | PanelStep::Landing
                | PanelStep::Landed { .. }
                | PanelStep::Ended => true,
            }
    }
}
fn tier_lines_within(lines: &[Box<[u8]>]) -> bool {
    for line in lines {
        if line.len() > 8192 {
            return false;
        }
    }
    true
}

fn question_lines_within(questions: &[PanelQuestion]) -> bool {
    for question in questions {
        if question.line.len() > change::QUESTION_LINE {
            return false;
        }
    }
    true
}
fn approvals_within(approvals: &[PanelApproval]) -> bool {
    if approvals.len() > 8192 {
        return false;
    }
    for approval in approvals {
        if approval.counted.len() > 256 {
            return false;
        }
    }
    true
}
