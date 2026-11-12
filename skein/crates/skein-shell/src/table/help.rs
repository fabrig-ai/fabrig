//! A table's help, command pages and version line (runtime/shell.md, 14.2).

use super::{Command, HOME, Reach, Set, Table, Variable, Way};
use crate::{Fallback, Setting};
use std::collections::BTreeSet;

#[cfg(test)]
mod tests;

const FIRST: usize = 2;
const SECOND: usize = 37;
const WIDTH: usize = 80;

fn one_column(out: &mut String, text: &str, indent: usize) {
    if text.is_empty() {
        return;
    }
    out.push_str(&" ".repeat(indent));
    words(out, text, indent);
}

fn two_columns(out: &mut String, first: &str, second: &str) {
    if first.chars().count() > 33 || second.is_empty() {
        one_column(out, first, FIRST);
        one_column(out, second, SECOND);
    } else {
        out.push_str(&" ".repeat(FIRST));
        out.push_str(first);
        out.push_str(&" ".repeat(SECOND.saturating_sub(FIRST).saturating_sub(first.chars().count())));
        words(out, second, SECOND);
    }
}

fn words(out: &mut String, text: &str, indent: usize) {
    let mut column = indent;
    for word in text.split_whitespace() {
        let space = usize::from(column > indent);
        if space != 0 && column.saturating_add(space).saturating_add(word.chars().count()) > WIDTH {
            out.push('\n');
            out.push_str(&" ".repeat(indent));
            column = indent;
        } else if space != 0 {
            out.push(' ');
            column = column.saturating_add(1);
        }
        out.push_str(word);
        column = column.saturating_add(word.chars().count());
    }
    out.push('\n');
}

impl Table {
    /// The page `-h`, `--help` and `help` print (runtime/shell.md, 14.2).
    #[must_use]
    pub fn help(&self) -> String {
        let mut out = String::new();
        section(&mut out, &format!("{}: {}", self.binary, self.line));

        let mut usage = String::new();
        let mut commands = String::new();
        let mut has_words = false;
        for command in self.commands {
            if command.set != Set::Public {
                continue;
            }
            match command.reach {
                Reach::Words(words) => {
                    if words.is_empty() {
                        command_usage(&mut usage, self.binary, command);
                    } else {
                        has_words = true;
                        let mut line = command.line;
                        for form in command.forms {
                            two_columns(&mut commands, &with_form(&words.join(" "), form), line);
                            line = "";
                        }
                    }
                }
                Reach::RunAs(_) => {}
            }
        }
        if has_words {
            one_column(&mut usage, &format!("{} COMMAND [ARGUMENT...]", self.binary), FIRST);
        }
        named_section(&mut out, "usage", &usage);
        named_section(&mut out, "commands", &commands);

        let mut flags = String::new();
        for group in self.settings {
            let mut names = String::new();
            for setting in group.table {
                if setting.set == Set::Public {
                    join_comma(&mut names, &setting_name(setting));
                }
            }
            if !names.is_empty() {
                two_columns(&mut flags, &names, group.about);
            }
        }
        two_columns(&mut flags, "-h, --help", "Print this.");
        two_columns(&mut flags, "--version", "Print the name, version and commit.");
        named_section(&mut out, "flags", &flags);
        named_section(&mut out, "environment", &self.help_environment(None));
        section(&mut out, self.more);
        out
    }

    /// What `COMMAND -h` and `--help` print for one of its commands.
    #[must_use]
    pub fn command_help(&self, command: &Command) -> String {
        let mut out = String::new();
        if command.set == Set::Internal {
            section(
                &mut out,
                &format!("internal: not part of {}'s public command line; it may change without notice.", self.binary),
            );
        }
        usage_section(&mut out, self.binary, command);
        section(&mut out, command.line);

        let mut arguments = String::new();
        for argument in command.arguments {
            two_columns(&mut arguments, argument.shown, argument.line);
        }
        named_section(&mut out, "arguments", &arguments);

        let mut flags = String::new();
        for group in self.settings.iter().chain(command.settings) {
            for setting in group.table {
                if visible(setting.set, command.set) {
                    let mut line = String::from(setting.about);
                    append_sentence(&mut line, &default_words(group.tool, setting.fallback));
                    if let Some(twin) = setting.twin {
                        append_sentence(&mut line, &format!("Environment: {twin}."));
                    }
                    two_columns(&mut flags, &setting_name(setting), &line);
                }
            }
        }
        for switch in command.switches {
            if visible(switch.set, command.set) {
                two_columns(&mut flags, &format!("--{}", switch.name), switch.line);
            }
        }
        named_section(&mut out, "flags", &flags);
        named_section(&mut out, "environment", &self.help_environment(Some(command)));
        section(&mut out, command.notes);
        out
    }

    /// `BINARY VERSION (COMMIT)` and a newline.
    #[must_use]
    pub fn version_line(&self) -> String {
        format!("{} {} ({})\n", self.binary, self.version, self.commit)
    }

    fn help_environment(&self, command: Option<&Command>) -> String {
        let audience = command.map_or(Set::Public, |command| command.set);
        let commands: Box<[_]> = match command {
            Some(command) => Box::from([command]),
            None => self.commands.iter().filter(|command| command.set == Set::Public).collect(),
        };
        let groups: Box<[_]> = self
            .settings
            .iter()
            .copied()
            .chain(commands.iter().flat_map(|command| command.settings.iter().copied()))
            .collect();
        let mut out = String::new();
        let mut seen = BTreeSet::new();
        for group in &groups {
            let mut twins = String::new();
            let mut flags = String::new();
            let settings: Box<[_]> = group
                .table
                .iter()
                .filter(|setting| visible(setting.set, audience) && setting.twin.is_some_and(|twin| seen.insert(twin)))
                .collect();
            let mut settings = settings.iter().peekable();
            while let Some(setting) = settings.next() {
                if let Some(twin) = setting.twin {
                    join_comma(&mut twins, twin);
                    if !flags.is_empty() {
                        flags.push_str(if settings.peek().is_some() { ", " } else { " and " });
                    }
                    flags.push_str("--");
                    flags.push_str(setting.name);
                }
            }
            if !twins.is_empty() {
                two_columns(&mut out, &twins, &format!("As {flags}."));
            }
        }
        for variable in self.variables.iter().chain(commands.iter().flat_map(|command| command.variables)) {
            environment_variable(&mut out, &mut seen, variable, audience);
        }
        for group in &groups {
            for setting in group.table {
                if visible(setting.set, audience) {
                    match setting.fallback {
                        Fallback::Base(base) => {
                            environment_variable(&mut out, &mut seen, base.variable(), audience);
                            environment_variable(&mut out, &mut seen, &HOME, audience);
                        }
                        Fallback::Refuse | Fallback::Text(_) => {}
                    }
                }
            }
        }
        out
    }
}

fn visible(entry: Set, audience: Set) -> bool {
    entry == Set::Public || audience == Set::Internal
}

fn environment_variable(out: &mut String, seen: &mut BTreeSet<&'static str>, variable: &Variable, audience: Set) {
    let reads = match variable.way {
        Way::Reads => true,
        Way::Sets => false,
    };
    if visible(variable.set, audience) && reads && seen.insert(variable.name) {
        two_columns(out, variable.name, variable.line);
    }
}

pub(super) fn usage_section(out: &mut String, binary: &str, command: &Command) {
    let mut usage = String::new();
    command_usage(&mut usage, binary, command);
    named_section(out, "usage", &usage);
}

fn command_usage(out: &mut String, binary: &str, command: &Command) {
    let name = match command.reach {
        Reach::Words(words) => with_form(binary, &words.join(" ")),
        Reach::RunAs(name) => String::from(name),
    };
    for form in command.forms {
        one_column(out, &with_form(&name, form), FIRST);
    }
}

fn with_form(name: &str, form: &str) -> String {
    if form.is_empty() { String::from(name) } else { format!("{name} {form}") }
}

fn setting_name(setting: &Setting) -> String {
    with_form(&format!("--{}", setting.name), setting.value)
}

fn default_words(tool: &str, fallback: Fallback) -> String {
    match fallback {
        Fallback::Refuse => String::from("Required."),
        Fallback::Text(text) => format!("Default: {text}."),
        Fallback::Base(base) => format!("Default: ${}/{tool}.", base.variable().name),
    }
}

fn append_sentence(out: &mut String, sentence: &str) {
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(sentence);
}

fn join_comma(out: &mut String, text: &str) {
    if !out.is_empty() {
        out.push_str(", ");
    }
    out.push_str(text);
}

fn section(out: &mut String, text: &str) {
    if !text.is_empty() {
        if !out.is_empty() {
            out.push('\n');
        }
        one_column(out, text, 0);
    }
}

fn named_section(out: &mut String, name: &str, body: &str) {
    if !body.is_empty() {
        section(out, &format!("{name}:"));
        out.push_str(body);
    }
}
