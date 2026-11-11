//! Shared binary tables (temper/docs/design/cli/README.md, 5).
use skein_shell::{Command, Reach, Set, Table};
use std::fmt::Write;

pub const HELP: Command = Command {
    reach: Reach::Words(&["help"]),
    set: Set::Public,
    forms: &["[NAME | pages | --search WORDS...]"],
    line: "Print this, a page, the pages, or a search.",
    arguments: &[
        skein_shell::Argument { shown: "NAME", line: crate::rows::HELP.arguments[0].line },
        skein_shell::Argument { shown: "WORDS", line: crate::rows::HELP.arguments[1].line },
    ],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: crate::rows::HELP.notes,
};

/// The tests' ctl command, with ctl's own lines (temper/docs/design/cli/README.md, 6).
pub const CTL: Command = Command {
    reach: Reach::Words(crate::rows::CTL.words),
    set: Set::Internal,
    forms: crate::rows::CTL.forms,
    line: crate::rows::CTL.line,
    arguments: &[
        skein_shell::Argument { shown: "DIR", line: crate::rows::CTL.arguments[0].line },
        skein_shell::Argument { shown: "GROUP", line: crate::rows::CTL.arguments[1].line },
        skein_shell::Argument { shown: "VERB", line: crate::rows::CTL.arguments[2].line },
        skein_shell::Argument { shown: "FILE", line: crate::rows::CTL.arguments[3].line },
    ],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: crate::rows::CTL.notes,
};

/// Help's words: stdout on success, or an exit code and stderr.
pub fn help(table: &Table, words: &[Box<[u8]>]) -> Result<String, (u8, String)> {
    if words.is_empty() {
        return Ok(table.help());
    }
    let request = crate::help::request(words).ok_or_else(|| (2, refused(table, &HELP, "invalid arguments")))?;
    if let crate::help::Request::Page(page) = &request
        && page.as_ref() == b"commands"
    {
        return Ok(commands_page(table));
    }
    if let crate::help::Request::Search(words) = &request {
        let found = temper_docs::search(&temper_docs::PAGES, words, 20)
            .map_err(|why| (2, refused(table, &HELP, why.says())))?;
        return crate::help::search_answer(table.binary, &found).map_err(|why| (1, why));
    }
    crate::help::answer(table.binary, &request).map_err(|why| (1, why))
}

/// A command's arguments refused, followed by its usage lines.
#[must_use]
pub fn refused(table: &Table, command: &Command, why: &str) -> String {
    let name = name(table, command);
    let mut text = format!("{name}: {why}\n");
    for form in command.forms {
        writeln!(text, "usage: {}", with_form(&name, form)).expect("writing into a string");
    }
    text
}

/// The binary's own `commands` reference page.
#[must_use]
pub fn commands_page(table: &Table) -> String {
    let mut text = format!(
        "# temper's commands\n\nKind: reference\n\nEvery command of the {} binary, by who runs it: its usage, what it does, its arguments and its notes.\n",
        table.binary,
    );
    for set in [Set::Public, Set::Internal] {
        for command in table.commands.iter().filter(|command| command.set == set) {
            command_section(&mut text, table, command, "##", None);
        }
    }
    text
}

/// The compiled reference for several binaries (protocol/documentation.md, 8.1).
#[must_use]
pub fn commands_reference(tables: &[&Table]) -> String {
    let mut text = String::from(
        "# temper's commands\n\nKind: reference\n\nEvery command of temper's binaries, a section each: its usage, what it does, its arguments and its notes. A command two binaries share is written under the first and named under the others.\n",
    );
    let mut earlier: Vec<&Table> = Vec::new();
    for table in tables {
        write!(text, "\n## {}\n\n{}\n", table.binary, table.line).expect("writing into a string");
        for set in [Set::Public, Set::Internal] {
            for command in table.commands.iter().filter(|command| command.set == set) {
                let first = earlier.iter().find(|table| table.commands.contains(command)).copied();
                command_section(&mut text, table, command, "###", first);
            }
        }
        earlier.push(table);
    }
    text
}

fn command_section(text: &mut String, table: &Table, command: &Command, level: &str, first: Option<&Table>) {
    let name = name(table, command);
    let heading = match command.reach {
        Reach::Words(_) => name.clone(),
        Reach::RunAs(program) => format!("run as {program}"),
    };
    write!(text, "\n{level} {heading}\n").expect("writing into a string");
    for form in command.forms {
        write!(text, "\n`{}`\n", with_form(&name, form)).expect("writing into a string");
    }
    if let Some(first) = first {
        let original = match command.reach {
            Reach::Words(_) => self::name(first, command),
            Reach::RunAs(program) => format!("{}'s run as {program}", first.binary),
        };
        write!(text, "\nAs {original}, above.\n").expect("writing into a string");
        return;
    }
    write!(text, "\n{}", command.line).expect("writing into a string");
    if command.set == Set::Internal {
        text.push_str(" Internal: it may change without notice.");
    }
    text.push('\n');
    if !command.arguments.is_empty() {
        text.push('\n');
        for argument in command.arguments {
            writeln!(text, "- `{}`: {}", argument.shown, argument.line).expect("writing into a string");
        }
    }
    if !command.notes.is_empty() {
        write!(text, "\n{}\n", command.notes).expect("writing into a string");
    }
}

fn name(table: &Table, command: &Command) -> String {
    match command.reach {
        Reach::Words(words) => with_form(table.binary, &words.join(" ")),
        Reach::RunAs(program) => program.into(),
    }
}

fn with_form(name: &str, form: &str) -> String {
    if form.is_empty() { name.into() } else { format!("{name} {form}") }
}

#[cfg(test)]
mod tests;
