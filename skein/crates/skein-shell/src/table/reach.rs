//! Reaching a command and refusing unknown words (runtime/shell.md, 14.4).

use super::{Reach, Reached, Table};

#[cfg(test)]
mod tests;

impl Table {
    /// Reach a command, page or version from the captured invocation.
    #[must_use]
    pub fn reach<'a>(&self, run_as: &[u8], arguments: &'a [Box<[u8]>]) -> Reached<'a> {
        let name = run_as.rsplit(|byte| *byte == b'/').next().unwrap_or_default();
        for command in self.commands {
            match command.reach {
                Reach::RunAs(program) => {
                    if program.as_bytes() == name {
                        return Reached::Command(command, arguments);
                    }
                }
                Reach::Words(_) => {}
            }
        }

        if alone(arguments, b"-h") || alone(arguments, b"--help") || alone(arguments, b"help") {
            return Reached::Help;
        }
        if alone(arguments, b"--version") {
            return Reached::Version;
        }

        let mut longest = None;
        let mut length = 0;
        for command in self.commands {
            match command.reach {
                Reach::Words(words) => {
                    if let Some((prefix, remaining)) = arguments.split_at_checked(words.len())
                        && words_match(words, prefix)
                        && (longest.is_none() || words.len() > length)
                    {
                        longest = Some((command, remaining));
                        length = words.len();
                    }
                }
                Reach::RunAs(_) => {}
            }
        }
        match longest {
            Some((command, remaining)) => {
                if alone(remaining, b"-h") || alone(remaining, b"--help") {
                    Reached::CommandHelp(command)
                } else {
                    Reached::Command(command, remaining)
                }
            }
            None => Reached::Unknown(self.unknown(arguments)),
        }
    }

    /// The no-command refusal, through the first word no command continues with.
    #[must_use]
    pub fn unknown(&self, arguments: &[Box<[u8]>]) -> String {
        let mut out = format!("{}: no command", self.binary);
        for (position, argument) in arguments.iter().enumerate() {
            out.push(' ');
            out.push_str(&String::from_utf8_lossy(argument));
            let prefix = arguments.get(..=position).expect("position comes from these arguments");
            let mut continues = false;
            for command in self.commands {
                match command.reach {
                    Reach::Words(words) => {
                        if prefix.len() <= words.len() && words_match(words, prefix) {
                            continues = true;
                            break;
                        }
                    }
                    Reach::RunAs(_) => {}
                }
            }
            if !continues {
                break;
            }
        }
        out.push_str("; ");
        out.push_str(self.binary);
        out.push_str(" --help lists them\n");
        out
    }
}

fn alone(arguments: &[Box<[u8]>], word: &[u8]) -> bool {
    match arguments.first() {
        Some(argument) => arguments.len() == 1 && argument.as_ref() == word,
        None => false,
    }
}

// Compare the overlapping words as bytes; callers check the required length.
fn words_match(words: &[&str], arguments: &[Box<[u8]>]) -> bool {
    for (word, argument) in words.iter().zip(arguments) {
        if word.as_bytes() != argument.as_ref() {
            return false;
        }
    }
    true
}
