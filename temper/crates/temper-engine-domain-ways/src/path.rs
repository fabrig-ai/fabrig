//! A branch's path (temper/docs/design/domain/forge/change/branch-path.md).
use alloc::boxed::Box;
mod approvals;
pub(crate) mod cards;
mod derive;
mod rows;
mod rules;
mod scopes;
pub use approvals::approval_words;
pub use rows::{PathRule, PathRules, rules_by_path};
pub use scopes::{by_path, set_names};
/// Who acts at a stop (domain/forge/change/branch-path.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Actor {
    Person,
    Agent,
    Code,
    Bar,
}

/// What a stop is, one per kind of step (temper/docs/design/domain/forge/change/branch-path.md).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Step {
    Ask,
    Draft,
    Plan,
    Produce,
    ProduceMany,
    Check,
    Review,
    Repair,
    Approve,
    Conflict,
    Resolve,
    Merge,
    CatchUp,
    Prepare,
    Prove,
    Turn,
    Verify,
    Branch,
    Design,
    CloseOut,
}

/// What a stop opens.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Opens {
    Nothing,
    Charter(u32),
    /// kind 1 for a rule of scope 0, else kind 2.
    Rule {
        kind: u32,
        scope: u32,
        number: u32,
    },
    Gate {
        number: u32,
        charter: u32,
    },
    Tier(Box<[u8]>),
    Queue,
    Turn,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fact {
    pub label: Box<[u8]>,
    pub value: Box<[u8]>,
}

/// A stop's card (8.13, *A stop's card*): at most `FACTS` facts.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Card {
    pub who: Box<[u8]>,
    pub name: Box<[u8]>,
    pub facts: Box<[Fact]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Stop {
    pub actor: Actor,
    pub step: Step,
    pub words: Box<[u8]>,
    pub opens: Opens,
    pub card: Card,
    pub needed: bool,
    pub over: Box<[u8]>,
}

/// `repository` is `OWNER/NAME`, none for a project with no repository.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Path {
    pub repository: Option<Box<[u8]>>,
    pub branch: Box<[u8]>,
    pub stops: Box<[Stop]>,
}

pub const STOPS: usize = 24; // a path's most stops
pub const FACTS: usize = 4; // a card's most facts
pub const OVER: usize = 16;
pub const WORDS: usize = 64; // a stop's words' most bytes
pub const WHO: usize = 128; // a card's `who`
pub const NAME: usize = 64; // a card's `name`
pub const LABEL: usize = 32; // a fact's label
pub const VALUE: usize = 128; // a fact's value, but an agent's "Asked to"
pub const ASKED: usize = 184; // an agent's "Asked to", its quotes included

impl Stop {
    /// Compare the drawn stop, ignoring its card.
    #[must_use]
    pub fn same(&self, other: &Stop) -> bool {
        self.actor == other.actor
            && self.step == other.step
            && self.words == other.words
            && self.opens == other.opens
            && self.needed == other.needed
            && self.over == other.over
    }
}
impl Path {
    /// Compare every drawn stop in order, ignoring repository, branch and cards.
    #[must_use]
    pub fn same(&self, other: &Path) -> bool {
        if self.stops.len() != other.stops.len() {
            return false;
        }
        for (own, other) in self.stops.iter().zip(other.stops.iter()) {
            if !own.same(other) {
                return false;
            }
        }
        true
    }
}

/// How work starts on the project (kind 7, its way's template and that template's work).
/// See temper/docs/design/domain/forge/change/branch-path.md.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Start {
    Chat,
    /// A goal: its template's charter, and the template's work, kind 11.
    Goal {
        charter: u32,
        design: bool,
        close_out: bool,
    },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tier {
    pub name: Box<[u8]>,
    pub follows: temper_engine_domain_forge::Follows,
    pub description: Box<[u8]>,
}

/// The repository the path is over, as the forge holds it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub host: Box<[u8]>, // the forge's host, as `covers_branch` takes it
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub ci: bool,                    // adopted with CI
    pub checks: Box<[u32]>,          // its checks, for a repository adopted without CI
    pub contexts: Box<[Box<[u8]>]>,  // the landing branch's required checks, as the forge's protection holds them
    pub workflows: Box<[Box<[u8]>]>, // the landing branch's CI workflows by file, as the forge holds them, the dispatched left out
    pub gates: Box<[Box<[u8]>]>,     // its test gates' names (landing by turn)
    pub tiers: Box<[Tier]>,
}

/// A role of the project's policy: its name, the reader's holding and its people count.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Role {
    pub number: u32,
    pub name: Box<[u8]>,
    pub yours: bool,
    pub holders: u32,
}

/// What an agent's card says of a charter, gathered by the root.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Agent {
    pub charter: u32,
    pub model: Option<Box<[u8]>>, // the model's shown name, else its name
    pub effort: Option<Box<[u8]>>,
    pub usable: bool,     // the reader's offer holds an account for it
    pub can: Box<[u8]>,   // `can(...)`'s words
    pub asked: Box<[u8]>, // `first_sentence(...)` of its instructions
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Inputs {
    pub branch: Box<[u8]>, // the landing branch, as `main`
    pub repository: Option<Repository>,
    pub rules: Box<[temper_engine_domain_forge::settings::Held]>, // every rule held for the project, items laid over
    pub sets: Box<[temper_protocol_reaches::PathSet]>,
    pub start: Start,
    pub producer: u32, // kind 3 number 1
    pub reviewer: u32, // kind 3 number 2
    pub by: temper_engine_domain_forge_change::Way,
    pub merge_job: bool,
    pub roles: Box<[Role]>,
    pub agents: Box<[Agent]>, // one per charter the path names
}

/// Derive the branch's path (temper/docs/design/domain/forge/change/branch-path.md).
#[must_use]
pub fn derive(inputs: &Inputs) -> Path {
    let repository = match &inputs.repository {
        Some(repository) => {
            let mut name = skein_lib::List::uncapped();
            for part in [repository.owner.as_ref(), b"/", repository.name.as_ref()] {
                for &byte in part {
                    name.push(byte).expect("repository name fits memory");
                }
            }
            Some(name.into_boxed())
        }
        None => None,
    };
    Path { repository, branch: inputs.branch.clone(), stops: derive::stops(inputs) }
}
