//! Team: a project's people and what waits on each
//! (temper/docs/design/client/README.md, 3.5, Team).
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;

/// Other people whose waits are read, one `tasks{query}` watch each.
pub const OTHERS: u32 = 8;

/// What waits on one person in the project, by kind; each count a floor while `cut`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Waits {
    pub questions: u32,
    pub reviews: u32,
    pub proposals: u32,
    /// Escalations: a held task that reached them.
    pub held: u32,
    pub person_tasks: u32,
    pub cut: bool,
}
#[expect(clippy::derivable_impls, reason = "Team’s contract names each zero count and its uncut default explicitly")]
impl Default for Waits {
    fn default() -> Self {
        Self { questions: 0, reviews: 0, proposals: 0, held: 0, person_tasks: 0, cut: false }
    }
}
impl Waits {
    /// The five counts summed, saturating.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.questions
            .saturating_add(self.reviews)
            .saturating_add(self.proposals)
            .saturating_add(self.held)
            .saturating_add(self.person_tasks)
    }
}

/// One person of the project as Team draws them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub party: u64,
    /// The person reading.
    pub you: bool,
    /// The member's name, or its login where the name is empty.
    pub name: Box<[u8]>,
    pub role: c::HeldRole,
    /// What waits on them; `None` while unread or past `OTHERS`: no line is drawn.
    pub waits: Option<Waits>,
}

/// Team's facts: plain values its reads give.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The project's members of kind person, the reader first, then in the document's order, each once.
    pub people: List<Person>,
    /// The project's document was cut: its members may be more.
    pub cut: bool,
    /// The project's triager, as its line names it; no read gives it yet.
    pub triager: Option<Box<[u8]>>,
    /// The project's standing roles; no read gives them yet.
    pub standing_roles: Option<u32>,
}

/// What Team draws (README.md, 3.5, Team, Its parts). A person's line is
/// drawn while their `waits` is `Some`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parts {
    /// People: a person is listed.
    pub people: bool,
    /// The triager's line: a triager is read.
    pub triager: bool,
}

/// What Team's page shows of the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    /// No answer yet: the intro, and "People" with no line under it.
    Reading,
    /// The project's document answered.
    Read(Facts),
    /// A refused or failed read: "This project's people can't be read now."
    Unreadable,
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts {
    Parts { people: !facts.people.is_empty(), triager: facts.triager.is_some() }
}

/// Team's facts from the project's document, the reader's party, what waits on
/// the reader, and what waits on others by party; pure.
#[must_use]
pub fn gather(project: &c::Project, you: u64, yours: Option<Waits>, others: &[(u64, Waits)]) -> Facts {
    let mut people: List<Person> = List::with_capacity(project.members.len());
    for reader in [true, false] {
        for member in &project.members {
            match member.kind {
                c::PartyKind::Person => {}
                c::PartyKind::Service => continue,
            }
            if (member.party == you) != reader {
                continue;
            }
            let mut listed = false;
            for person in &people {
                if person.party == member.party {
                    listed = true;
                    break;
                }
            }
            if listed {
                continue;
            }
            let mut waits = None;
            if reader {
                waits = yours;
            } else {
                for (party, counts) in others {
                    if *party == member.party {
                        waits = Some(*counts);
                        break;
                    }
                }
            }
            let name = if member.name.bytes().is_empty() { member.login.bytes() } else { member.name.bytes() };
            people
                .push(Person { party: member.party, you: reader, name: name.into(), role: member.role, waits })
                .expect("at most one person per document member");
        }
    }
    Facts { people, cut: project.cut, triager: None, standing_roles: None }
}

/// Team's page for the project: no read gives Team its facts yet.
#[must_use]
pub fn shown(_root: &crate::Domain, _project: u32) -> Shown {
    Shown::Reading
}

/// The project's people counted for the rail: no read gives Team its facts yet.
#[must_use]
pub fn people(_root: &crate::Domain, _project: u32) -> Option<u32> {
    None
}
