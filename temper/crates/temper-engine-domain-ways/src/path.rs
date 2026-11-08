//! A branch's path (temper/docs/design/domain/forge.md, 8.13).
use alloc::boxed::Box;
/// Who acts at a stop (domain/forge.md, 8.13).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Actor {
    Person,
    Agent,
    Code,
    Bar,
}

/// What a stop is, one per kind of step of 8.13's order.
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
    Update,
    Resolve,
    Merge,
    Prepare,
    Prove,
    Turn,
    Verify,
    Branch,
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
    Gate(u32),
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
}

/// `repository` is `OWNER/NAME`, none for a project with no repository.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Path {
    pub repository: Option<Box<[u8]>>,
    pub branch: Box<[u8]>,
    pub stops: Box<[Stop]>,
}

pub const STOPS: usize = 32; // a path's most stops
pub const FACTS: usize = 4; // a card's most facts
pub const WORDS: usize = 64; // a stop's words' most bytes
pub const WHO: usize = 128; // a card's `who`
pub const NAME: usize = 64; // a card's `name`
pub const LABEL: usize = 32; // a fact's label
pub const VALUE: usize = 384; // a fact's value

impl Stop {
    /// Compare the drawn stop, ignoring its card.
    #[must_use]
    pub fn same(&self, other: &Stop) -> bool {
        self.actor == other.actor && self.step == other.step && self.words == other.words && self.opens == other.opens
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

/// How work starts on the project (kind 7 and its way's template).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Start {
    Chat,
    Goal { charter: u32 },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tier {
    pub name: Box<[u8]>,
    pub follows: temper_engine_domain_forge::Follows,
}

/// The repository the path is over, as the forge holds it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub host: Box<[u8]>, // the forge's host, as `covers_branch` takes it
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub ci: bool,                // adopted with CI
    pub checks: Box<[u32]>,      // its checks, for a repository adopted without CI
    pub gates: Box<[Box<[u8]>]>, // its test gates' names (landing by turn)
    pub tiers: Box<[Tier]>,
}

/// A role of the project's policy: its name and whether the reader holds it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Role {
    pub number: u32,
    pub name: Box<[u8]>,
    pub yours: bool,
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
    pub start: Start,
    pub producer: u32, // kind 3 number 1
    pub reviewer: u32, // kind 3 number 2
    pub by: temper_engine_domain_forge_change::Way,
    pub roles: Box<[Role]>,
    pub agents: Box<[Agent]>, // one per charter the path names
}

/// The inputs' branch and repository, with no stops; task `ways-derive` writes the derivation.
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
    Path { repository, branch: inputs.branch.clone(), stops: Box::default() }
}
