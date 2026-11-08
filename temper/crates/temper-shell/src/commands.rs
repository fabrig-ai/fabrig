//! Shared command tables (temper/docs/design/protocol/README.md, 4.1).
/// Who runs a command: `temper --help`'s groups, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    Documentation,
    Deployment,
    Clients,
    Spawned,
    TestDoubles,
}
impl Who {
    /// "documentation", "deployment", "clients", "spawned", "test doubles".
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            Self::Documentation => "documentation",
            Self::Deployment => "deployment",
            Self::Clients => "clients",
            Self::Spawned => "spawned",
            Self::TestDoubles => "test doubles",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Row {
    /// After `temper`: `["forge", "run"]`.
    pub words: &'static [&'static str],
    pub who: Who,
    /// Each form's arguments, as its usage shows them after the words.
    pub forms: &'static [&'static str],
    pub line: &'static str,
    pub arguments: &'static [Argument],
    /// The skein settings tables whose flags it takes, printed by its `--help`.
    pub settings: &'static [&'static skein_shell::Settings],
    pub notes: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Argument {
    pub shown: &'static str,
    pub line: &'static str,
}

pub struct Table {
    pub binary: &'static str,
    pub rows: &'static [Row],
}
impl Table {
    pub fn unknown(&self, _words: &[Box<[u8]>]) -> String {
        String::new()
    }
    pub fn forms(&self, _row: &Row) -> String {
        String::new()
    }
}
