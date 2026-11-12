//! The forge's row parts and the shared directories argument (temper/docs/design/cli/README.md, 5).

/// Who runs a forge command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Who {
    Documentation,
    Deployment,
    Clients,
    Spawned,
    TestDoubles,
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

pub const DIRECTORIES: Argument = Argument {
    shown: "DIRECTORIES",
    line: "[--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.",
};
