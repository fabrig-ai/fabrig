//! Shared command tables (temper/docs/design/protocol/README.md, 4.1).
use std::fmt::Write;

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

pub const DIRECTORIES: Argument = Argument {
    shown: "DIRECTORIES",
    line: "[--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.",
};

pub const HELP: Row = Row {
    words: &["help"],
    who: Who::Documentation,
    forms: &["[NAME | --search WORDS...]"],
    line: "Print the index of temper's pages, one page whole, or the sections that hold every word.",
    arguments: &[
        Argument { shown: "NAME", line: "A page's name, as the index and the pages give it." },
        Argument {
            shown: "WORDS",
            line: "Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.",
        },
    ],
    settings: &[],
    notes: "Reads no directory and no configuration, so it works before temper init. A name that is no page, or a search that matches nothing, exits 1. A search prints at most 20 hits, a line each, NAME: HEADING: LINE, then \"and more; narrow the words\" when more matched.",
};

pub struct Table {
    pub binary: &'static str,
    pub rows: &'static [Row],
}
impl Table {
    /// The row whose words begin `words`, the longest such.
    #[must_use]
    pub fn find(&self, words: &[Box<[u8]>]) -> Option<&'static Row> {
        let mut found: Option<&Row> = None;
        for row in self.rows {
            let eligible = !row.words.is_empty() || words.first().is_none_or(|word| word.starts_with(b"http://"));
            if eligible && begins(words, row.words) && found.is_none_or(|before| before.words.len() < row.words.len()) {
                found = Some(row);
            }
        }
        found
    }

    /// The command words up to the first word no row continues with.
    #[must_use]
    pub fn unknown(&self, words: &[Box<[u8]>]) -> String {
        let mut prefix = Vec::new();
        for word in words {
            prefix.push(word.clone());
            if !self
                .rows
                .iter()
                .any(|row| row.words.len() >= prefix.len() && begins(&prefix, &row.words[..prefix.len()]))
            {
                break;
            }
        }
        let given = String::from_utf8_lossy(&prefix.join(&b' ')).into_owned();
        let given = if given.is_empty() && words.is_empty() { String::new() } else { format!(" {given}") };
        format!("{}: no command{given}; {} --help lists them\n", self.binary, self.binary)
    }

    fn command(&self, row: &Row) -> String {
        if row.words.is_empty() { self.binary.into() } else { format!("{} {}", self.binary, row.words.join(" ")) }
    }

    #[must_use]
    pub fn forms(&self, row: &Row) -> String {
        let mut text = String::new();
        for form in row.forms {
            writeln!(text, "usage: {} {form}", self.command(row)).expect("writing into a string");
        }
        text
    }

    #[must_use]
    pub fn refused(&self, row: &Row, why: &str) -> String {
        format!("{}: {why}\n{}", self.command(row), self.forms(row))
    }

    /// `temper --help`.
    #[must_use]
    pub fn help(&self) -> String {
        let mut text = format!(
            "usage: {binary} COMMAND [ARGUMENT...]\n{binary} help prints the index of temper's pages; {binary} COMMAND --help prints one command.\n",
            binary = self.binary,
        );
        let mut previous = None;
        for row in self.rows {
            if previous != Some(row.who) {
                let label = if row.who == Who::Spawned { "spawned, never run by hand" } else { row.who.word() };
                write!(text, "\n{label}:\n").expect("writing into a string");
                previous = Some(row.who);
            }
            for form in row.forms {
                writeln!(text, "  {} {form}", self.command(row)).expect("writing into a string");
            }
            if row.who != Who::Spawned {
                writeln!(text, "    {}", row.line).expect("writing into a string");
            }
        }
        text
    }

    /// `temper COMMAND --help`.
    #[must_use]
    pub fn row(&self, row: &Row) -> String {
        let mut text = self.forms(row);
        writeln!(text, "{}", row.line).expect("writing into a string");
        for argument in row.arguments {
            writeln!(text, "  {}: {}", argument.shown, argument.line).expect("writing into a string");
        }
        if !row.notes.is_empty() {
            writeln!(text, "{}", row.notes).expect("writing into a string");
        }
        let mut text = text.into_bytes();
        for table in row.settings {
            skein_shell::help(table, &mut text);
        }
        String::from_utf8(text).expect("help is UTF-8")
    }

    /// The reference page `commands`, whole, as `docs/help/reference/commands.md` holds it.
    #[must_use]
    pub fn reference(&self) -> String {
        let mut text = format!(
            "# temper's commands\n\nKind: reference\n\nEvery command of the {} binary, by who runs it: its usage, what it does, its arguments and its notes.\n",
            self.binary,
        );
        for row in self.rows {
            let command = self.command(row);
            let heading = if row.words.is_empty() { format!("{command} [ADDRESS]") } else { command.clone() };
            write!(text, "\n## {heading}\n").expect("writing into a string");
            for form in row.forms {
                write!(text, "\n`{command} {form}`\n").expect("writing into a string");
            }
            write!(text, "\n{}", row.line).expect("writing into a string");
            if row.who == Who::Spawned {
                write!(text, " Spawned by {}; never run by hand.", self.binary).expect("writing into a string");
            }
            text.push('\n');
            if !row.arguments.is_empty() {
                text.push('\n');
                for argument in row.arguments {
                    writeln!(text, "- `{}`: {}", argument.shown, argument.line).expect("writing into a string");
                }
            }
            if !row.notes.is_empty() {
                write!(text, "\n{}\n", row.notes).expect("writing into a string");
            }
        }
        text
    }
}

fn begins(words: &[Box<[u8]>], prefix: &[&str]) -> bool {
    words.len() >= prefix.len() && words.iter().zip(prefix).all(|(word, text)| word.as_ref() == text.as_bytes())
}
