//! A page's parts (temper/docs/design/client/pages/README.md, A page's parts).
use crate::Domain;
use skein_lib::Wall;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HomeFacts {
    pub across: bool,
    pub projects: u32,
    pub brief: bool,
    pub open: u32,
    pub showing: u32,
    pub to_begin: bool,
    pub follows: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "each page part follows its independent rule")]
pub struct HomeParts {
    pub brief: bool,
    pub needs: bool,
    pub decided: bool,
    pub projects: bool,
    pub ask: bool,
    pub follows: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NeedsFacts {
    pub known: bool,
    pub open: u32,
    pub showing: u32,
    /// Only you's items, separate from open cards.
    pub only: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "each page part follows its independent rule")]
pub struct NeedsParts {
    pub list: bool,
    pub done: bool,
    pub decided: bool,
    /// The inbox has answered and Only you has an item.
    pub only: bool,
}
#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for HomeFacts {
    fn default() -> Self {
        Self { across: false, projects: 0, brief: false, open: 0, showing: 0, to_begin: false, follows: 0 }
    }
}
#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for HomeParts {
    fn default() -> Self {
        Self { brief: false, needs: false, decided: false, projects: false, ask: false, follows: false }
    }
}
#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for NeedsFacts {
    fn default() -> Self {
        Self { known: false, open: 0, showing: 0, only: 0 }
    }
}
#[expect(clippy::derivable_impls, reason = "the programming model uses explicit Default implementations")]
impl Default for NeedsParts {
    fn default() -> Self {
        Self { list: false, done: false, decided: false, only: false }
    }
}
#[must_use]
pub const fn home(facts: &HomeFacts) -> HomeParts {
    HomeParts {
        brief: facts.brief && !facts.to_begin,
        needs: facts.open > 0 && !facts.to_begin,
        decided: facts.showing > 0 && !facts.to_begin,
        projects: facts.across && facts.projects >= 2,
        ask: !facts.to_begin,
        follows: facts.across && facts.follows > 0,
    }
}
#[must_use]
pub const fn needs(facts: &NeedsFacts) -> NeedsParts {
    NeedsParts {
        list: facts.known && facts.open > 0,
        done: facts.known && facts.open == 0,
        decided: facts.showing > 0,
        only: facts.known && facts.only > 0,
    }
}
#[must_use]
pub fn needs_facts(root: &Domain, wall: Wall) -> NeedsFacts {
    let (known, open) = match root.pages().inbox() {
        Some(inbox) => (
            inbox.known(),
            u32::try_from(root.pages().needs().needs(inbox, root.forge().approvals()).len()).expect("inbox cards fit"),
        ),
        None => (false, 0),
    };
    NeedsFacts {
        known,
        open,
        showing: super::all_home::showing(root, wall),
        only: super::only_you::read(root).items.len(),
    }
}
