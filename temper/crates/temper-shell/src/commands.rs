//! A binary's table and its printing (temper/docs/design/protocol/README.md, section 4).
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

/// The row whose words begin `words`, the longest such.
#[must_use]
pub fn find(rows: &'static [Row], words: &[Box<[u8]>]) -> Option<&'static Row> {
    let mut found: Option<&Row> = None;
    for row in rows {
        let eligible = !row.words.is_empty() || words.first().is_none_or(|word| word.starts_with(b"http://"));
        if eligible && begins(words, row.words) && found.is_none_or(|before| before.words.len() < row.words.len()) {
            found = Some(row);
        }
    }
    found
}

fn begins(words: &[Box<[u8]>], prefix: &[&str]) -> bool {
    words.len() >= prefix.len() && words.iter().zip(prefix).all(|(word, text)| word.as_ref() == text.as_bytes())
}

/// The command words up to the first word no row continues with.
#[must_use]
pub fn unknown(name: &str, rows: &[Row], words: &[Box<[u8]>]) -> String {
    let mut prefix = Vec::new();
    for word in words {
        prefix.push(word.clone());
        if !rows.iter().any(|row| row.words.len() >= prefix.len() && begins(&prefix, &row.words[..prefix.len()])) {
            break;
        }
    }
    let given = String::from_utf8_lossy(&prefix.join(&b' ')).into_owned();
    let given = if given.is_empty() && words.is_empty() { String::new() } else { format!(" {given}") };
    format!("{name}: no command{given}; {name} --help lists them\n")
}

#[must_use]
pub fn command(name: &str, row: &Row) -> String {
    if row.words.is_empty() { name.into() } else { format!("{name} {}", row.words.join(" ")) }
}

#[must_use]
pub fn forms(name: &str, row: &Row) -> String {
    let mut text = String::new();
    for form in row.forms {
        writeln!(text, "usage: {} {form}", command(name, row)).expect("writing into a string");
    }
    text
}

#[must_use]
pub fn refused(name: &str, row: &Row, why: &str) -> String {
    format!("{}: {why}\n{}", command(name, row), forms(name, row))
}

/// The binary's `--help`.
#[must_use]
pub fn help(name: &str, rows: &[Row]) -> String {
    let mut text = format!(
        "usage: {name} COMMAND [ARGUMENT...]\n{name} help prints the index of temper's pages; {name} COMMAND --help prints one command.\n",
    );
    let mut previous = None;
    for row in rows {
        if previous != Some(row.who) {
            let label = if row.who == Who::Spawned { "spawned, never run by hand" } else { row.who.word() };
            write!(text, "\n{label}:\n").expect("writing into a string");
            previous = Some(row.who);
        }
        for form in row.forms {
            writeln!(text, "  {} {form}", command(name, row)).expect("writing into a string");
        }
        if row.who != Who::Spawned {
            writeln!(text, "    {}", row.line).expect("writing into a string");
        }
    }
    text
}

/// The binary's `COMMAND --help`.
#[must_use]
pub fn row(name: &str, row: &Row) -> String {
    let mut text = forms(name, row);
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

pub use crate::rows::HELP;

pub const DIRECTORIES: Argument = Argument {
    shown: "DIRECTORIES",
    line: "[--config CONFIG] [--data DATA] [--cache CACHE], anywhere among the arguments: the deployment's configuration, data and cache directories, each from its flag, else its variable, else its default.",
};

pub struct Table {
    pub binary: &'static str,
    pub rows: &'static [Row],
}
impl Table {
    #[must_use]
    pub fn find(&self, words: &[Box<[u8]>]) -> Option<&'static Row> {
        find(self.rows, words)
    }
    #[must_use]
    pub fn unknown(&self, words: &[Box<[u8]>]) -> String {
        unknown(self.binary, self.rows, words)
    }
    fn command(&self, row: &Row) -> String {
        command(self.binary, row)
    }
    #[must_use]
    pub fn forms(&self, row: &Row) -> String {
        forms(self.binary, row)
    }
    #[must_use]
    pub fn refused(&self, row: &Row, why: &str) -> String {
        refused(self.binary, row, why)
    }
    #[must_use]
    pub fn help(&self) -> String {
        help(self.binary, self.rows)
    }
    #[must_use]
    pub fn row(&self, row: &Row) -> String {
        self::row(self.binary, row)
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

#[cfg(test)]
mod tests;
