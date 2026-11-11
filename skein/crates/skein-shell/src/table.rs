//! A binary's table (runtime/shell.md, 14; skein/docs/foundation/cli.md, 6).

use crate::{Base, Settings};

#[cfg(test)]
mod demo;
mod help;
mod reach;
mod reference;

/// Who an entry is for (skein/docs/foundation/cli.md, 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Set {
    Public,
    Internal,
    Test,
}

/// A binary's table (runtime/shell.md, 14.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Table {
    /// The binary's name.
    pub binary: &'static str,
    /// One line: what the binary is.
    pub line: &'static str,
    /// The tree's build identity, as it words it.
    pub version: &'static str,
    pub commit: &'static str,
    /// In help's order.
    pub commands: &'static [Command],
    /// The groups of settings every command takes.
    pub settings: &'static [&'static Settings],
    /// The variables it reads or sets beside its settings' twins.
    pub variables: &'static [&'static Variable],
    /// Help's last line: where to read more.
    pub more: &'static str,
}

/// How a command is reached.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// Its words after the binary's name; none for the bare binary's form.
    Words(&'static [&'static str]),
    /// The name the binary runs as, for a delegated grammar.
    RunAs(&'static str),
}

/// What a binary's arguments reach (runtime/shell.md, 14.4).
#[derive(PartialEq, Eq, Debug)]
pub enum Reached<'a> {
    /// `-h`, `--help` or `help`, alone: the binary's page.
    Help,
    /// `--version`, alone: the version line.
    Version,
    /// A command's words, then `-h` or `--help` alone: its page.
    CommandHelp(&'static Command),
    /// A command and the arguments after its words.
    Command(&'static Command, &'a [Box<[u8]>]),
    /// No command: the refusal's line, newline included.
    Unknown(String),
}

/// A command's forms, settings and documentation (runtime/shell.md, 14.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Command {
    pub reach: Reach,
    pub set: Set,
    /// Each form's arguments, as usage shows them after the name.
    pub forms: &'static [&'static str],
    pub line: &'static str,
    pub arguments: &'static [Argument],
    /// The groups of settings it takes beside the table's.
    pub settings: &'static [&'static Settings],
    pub switches: &'static [Switch],
    /// The variables it reads or sets beside its settings' twins.
    pub variables: &'static [&'static Variable],
    pub notes: &'static str,
}

/// A positional argument as usage shows it (runtime/shell.md, 14.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Argument {
    pub shown: &'static str,
    pub line: &'static str,
}

/// A flag with no value: `--front`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Switch {
    pub name: &'static str,
    pub set: Set,
    pub line: &'static str,
    pub notes: &'static str,
}

/// Whether a binary reads a variable or sets it for the programs it runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Way {
    Reads,
    Sets,
}

/// A variable's name, use and documentation (runtime/shell.md, 14.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Variable {
    pub name: &'static str,
    pub set: Set,
    /// What its value is: `DIR`, `1`, `none|steps`.
    pub form: &'static str,
    pub line: &'static str,
    /// What holds when it's absent or empty.
    pub absent: &'static str,
    pub notes: &'static str,
    pub way: Way,
}

impl Variable {
    /// Its bytes in the captured environment, the pairs `environment`
    /// returns; an empty value is unset.
    #[must_use]
    #[expect(clippy::type_complexity, reason = "runtime/shell.md, 14.1 takes captured byte environment pairs")]
    pub fn value<'a>(&self, environment: &'a [(Box<[u8]>, Box<[u8]>)]) -> Option<&'a [u8]> {
        lookup(environment, self.name.as_bytes())
    }
}

pub(crate) fn lookup<'a>(environment: &'a [EnvironmentEntry], name: &[u8]) -> Option<&'a [u8]> {
    for (key, value) in environment {
        if key.as_ref() == name {
            return if value.is_empty() { None } else { Some(value) };
        }
    }
    None
}

type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);

/// A tree's test set (skein/docs/foundation/cli.md, 8): its variables, all `Set::Test`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TestTable {
    pub tree: &'static str,
    pub variables: &'static [Variable],
}

/// The host's home directory, read by a setting's `Fallback::Base`.
pub const HOME: Variable = Variable {
    name: "HOME",
    set: Set::Public,
    form: "DIR",
    line: "Base for a setting's directory when its XDG base is unusable.",
    absent: "A directory setting needs an absolute XDG base or an explicit value.",
    notes: "Only absolute paths are used.",
    way: Way::Reads,
};

const XDG_CONFIG_HOME: Variable = Variable {
    name: "XDG_CONFIG_HOME",
    line: "Base for a setting's configuration directory, before the tool's name.",
    absent: "Use $HOME/.config when HOME is absolute.",
    ..HOME
};
const XDG_DATA_HOME: Variable = Variable {
    name: "XDG_DATA_HOME",
    line: "Base for a setting's data directory, before the tool's name.",
    absent: "Use $HOME/.local/share when HOME is absolute.",
    ..HOME
};
const XDG_STATE_HOME: Variable = Variable {
    name: "XDG_STATE_HOME",
    line: "Base for a setting's state directory, before the tool's name.",
    absent: "Use $HOME/.local/state when HOME is absolute.",
    ..HOME
};
const XDG_CACHE_HOME: Variable = Variable {
    name: "XDG_CACHE_HOME",
    line: "Base for a setting's cache directory, before the tool's name.",
    absent: "Use $HOME/.cache when HOME is absolute.",
    ..HOME
};

impl Base {
    /// Its declared XDG variable (runtime/shell.md, 14.1).
    #[must_use]
    pub const fn variable(self) -> &'static Variable {
        match self {
            Self::Config => &XDG_CONFIG_HOME,
            Self::Data => &XDG_DATA_HOME,
            Self::State => &XDG_STATE_HOME,
            Self::Cache => &XDG_CACHE_HOME,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HOME, Variable};

    #[test]
    fn a_variable_reads_the_first_exact_name_and_empty_is_unset() {
        let variable = Variable { name: "DEMO_VALUE", ..HOME };
        for (pairs, expected) in [
            (vec![("DEMO_VALUE", "first"), ("DEMO_VALUE", "second")], Some(b"first".as_slice())),
            (vec![("DEMO_VALUE", ""), ("DEMO_VALUE", "later")], None),
            (vec![], None),
            (vec![("demo_value", "wrong case")], None),
        ] {
            let environment: Box<[_]> = pairs
                .into_iter()
                .map(|(key, value)| (Box::from(key.as_bytes()), Box::from(value.as_bytes())))
                .collect();
            assert_eq!(variable.value(&environment), expected, "first exact name, empty is unset");
        }
    }
}
