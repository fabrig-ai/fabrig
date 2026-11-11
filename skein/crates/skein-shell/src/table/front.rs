//! The table's own words before a binary runs its commands (runtime/shell.md, 14.5).

use super::{Command, Reach, Table, help::usage_section};

#[cfg(test)]
mod tests;

/// What a binary's arguments ask of its table before its own commands
/// (runtime/shell.md, 14.5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Front<'a> {
    /// Written to stdout; the binary exits 0.
    Print(String),
    /// One of the table's commands, with the arguments after its words.
    Run { command: &'a Command, rest: &'a [Box<[u8]>] },
    /// Written to stderr; the binary exits 2.
    Refuse(String),
}

impl Table {
    /// Answer the table's own arguments before a command reads its rest.
    #[must_use]
    pub fn front<'a>(&'a self, arguments: &'a [Box<[u8]>]) -> Front<'a> {
        let first = arguments.first().map(AsRef::as_ref);
        if is_help(first) {
            return Front::Print(self.help());
        }
        if first == Some(b"--version".as_slice()) {
            return Front::Print(self.version_line());
        }
        if arguments.len() == 1
            && first == Some(b"help".as_slice())
            && !self.commands.iter().any(|command| command.reach == Reach::Words(&["help"]))
        {
            return Front::Print(self.help());
        }
        let longest = self
            .commands
            .iter()
            .filter_map(|command| match command.reach {
                Reach::Words(words) => {
                    if words.is_empty() {
                        None
                    } else {
                        after_words(words, arguments).map(|rest| (words.len(), command, rest))
                    }
                }
                Reach::RunAs(_) => None,
            })
            .max_by_key(|(length, _, _)| *length);
        if let Some((_, command, rest)) = longest {
            return if is_help(rest.first().map(AsRef::as_ref)) {
                Front::Print(self.command_help(command))
            } else {
                Front::Run { command, rest }
            };
        }
        if let Some(command) = self.commands.iter().find(|command| command.reach == Reach::Words(&[])) {
            return Front::Run { command, rest: arguments };
        }
        let word = first.map_or_else(String::new, |word| format!(" {}", String::from_utf8_lossy(word)));
        Front::Refuse(format!("{}: no command{word}; {} --help says more\n", self.binary, self.binary))
    }

    /// Refuse a command's arguments with the reason and its usage part.
    #[must_use]
    pub fn refusal(&self, command: &Command, what: &str) -> String {
        let mut name = String::from(self.binary);
        match command.reach {
            Reach::Words(words) => {
                if !words.is_empty() {
                    name.push(' ');
                    name.push_str(&words.join(" "));
                }
            }
            Reach::RunAs(_) => {}
        }
        let mut usage = String::new();
        usage_section(&mut usage, self.binary, command);
        format!("{name}: {what}\n{usage}")
    }
}

fn is_help(argument: Option<&[u8]>) -> bool {
    argument == Some(b"-h".as_slice()) || argument == Some(b"--help".as_slice())
}

fn after_words<'a>(words: &[&str], mut arguments: &'a [Box<[u8]>]) -> Option<&'a [Box<[u8]>]> {
    for word in words {
        let (first, rest) = arguments.split_first()?;
        if first.as_ref() != word.as_bytes() {
            return None;
        }
        arguments = rest;
    }
    Some(arguments)
}
