//! A table's reference documents (skein/docs/design/shell.md, 15.3).

use std::collections::BTreeSet;

use super::{Command, HOME, Reach, Set, Switch, Table, TestTable, Variable, Way};
use crate::{Fallback, Setting, Settings};

impl Table {
    /// `<binary>-args.md`: its commands and flags, public then internal.
    #[must_use]
    pub fn arguments_document(&self) -> String {
        let mut out = document(self.binary, "arguments", &format!("{}'s table", self.binary));
        let flags = flags(self);
        for set in [Set::Public, Set::Internal, Set::Test] {
            if set == Set::Test
                && !self.commands.iter().any(|command| command.set == set)
                && !flags.iter().any(|flag| flag.set() == set)
            {
                continue;
            }
            section(&mut out, set);
            for command in self.commands.iter().filter(|command| command.set == set) {
                command_entry(&mut out, self, command);
            }
            for flag in flags.iter().filter(|flag| flag.set() == set) {
                flag_entry(&mut out, *flag);
            }
        }
        out
    }

    /// `<binary>-env.md`: the variables this binary alone reads, or sets,
    /// of the tree whose tables are `tree` (this one among them).
    #[must_use]
    pub fn environment_document(&self, tree: &[&Table]) -> String {
        let mut out = document(self.binary, "environment", &format!("{}'s table", self.binary));
        let tree = tree_variables(tree);
        let entries = variables(self);
        for set in [Set::Public, Set::Internal, Set::Test] {
            if set == Set::Test && !entries.iter().any(|entry| entry.variable.set == set) {
                continue;
            }
            section(&mut out, set);
            for entry in entries.iter().filter(|entry| entry.variable.set == set) {
                if entry.variable.way == Way::Sets || readers(entry.variable.name, &tree).len() < 2 {
                    variable_entry(&mut out, entry, &[]);
                }
            }
        }
        out
    }

    /// `<tree>/docs/design/cli/common-env.md`: the variables two or more of `tables` read.
    #[must_use]
    pub fn common_document(tree: &str, tables: &[&Table]) -> String {
        let mut out = document(tree, "common environment", "the tree's tables");
        let tree = tree_variables(tables);
        for set in [Set::Public, Set::Internal, Set::Test] {
            if set == Set::Test
                && !tree.iter().any(|(_, entries)| entries.iter().any(|entry| entry.variable.set == set))
            {
                continue;
            }
            section(&mut out, set);
            let mut seen = BTreeSet::new();
            for (_, entries) in &tree {
                for entry in entries {
                    let variable = entry.variable;
                    if variable.set != set || variable.way != Way::Reads || !seen.insert(variable.name) {
                        continue;
                    }
                    let readers = readers(variable.name, &tree);
                    if readers.len() >= 2 {
                        variable_entry(&mut out, entry, &readers);
                    }
                }
            }
        }
        out
    }
}

impl TestTable {
    /// `<tree>/docs/design/cli/test-env.md`: the tree's test table.
    #[must_use]
    pub fn document(&self) -> String {
        let mut out = document(self.tree, "test environment", "the tree's test table");
        section(&mut out, Set::Test);
        let mut seen = BTreeSet::new();
        for variable in self.variables {
            if seen.insert(variable.name) {
                variable_entry(&mut out, &VariableEntry::declared(*variable), &[]);
            }
        }
        out
    }
}

fn document(name: &str, kind: &str, source: &str) -> String {
    format!(
        "# {name}: {kind}\n\nGenerated from {source} (`skein/docs/design/shell.md`, 15.3): \
         a test checks it byte for byte, and the tree's rewrite switch rewrites it.\n"
    )
}

fn set_name(set: Set) -> &'static str {
    match set {
        Set::Public => "public",
        Set::Internal => "internal",
        Set::Test => "test",
    }
}

fn section(out: &mut String, set: Set) {
    let title = match set {
        Set::Public => "Public",
        Set::Internal => "Internal",
        Set::Test => "Test",
    };
    out.push_str("\n## ");
    out.push_str(title);
    out.push('\n');
}

fn item(out: &mut String, label: &str, value: &str) {
    if !value.is_empty() {
        out.push_str("- ");
        if !label.is_empty() {
            out.push_str("**");
            out.push_str(label);
            out.push_str(":** ");
        }
        out.push_str(value);
        out.push('\n');
    }
}

fn heading(out: &mut String, heading: &str) {
    out.push_str("\n### ");
    out.push_str(heading);
    out.push_str("\n\n");
}

fn code(text: &str) -> String {
    format!("`{text}`")
}

fn command_name(binary: &str, reach: Reach) -> String {
    match reach {
        Reach::Words(words) => {
            let mut name = binary.to_owned();
            for word in words {
                name.push(' ');
                name.push_str(word);
            }
            code(&name)
        }
        Reach::RunAs(name) => format!("run as `{name}`"),
    }
}

fn command_entry(out: &mut String, table: &Table, command: &Command) {
    heading(out, &command_name(table.binary, command.reach));
    item(out, "Set", set_name(command.set));
    item(out, "Forms", &command.forms.iter().map(|form| code(form)).collect::<Vec<_>>().join(", "));
    item(out, "", command.line);
    let arguments = command
        .arguments
        .iter()
        .map(|argument| format!("`{}`: {}", argument.shown, argument.line))
        .collect::<Vec<_>>()
        .join("; ");
    item(out, "Arguments", &arguments);
    let mut seen = BTreeSet::new();
    let flags = command_flags(table, command)
        .into_iter()
        .filter(|flag| flag.set() == command.set && seen.insert(flag.name()))
        .map(|flag| code(&flag.shown()))
        .collect::<Vec<_>>()
        .join(", ");
    item(out, "Flags", &flags);
    item(out, "Notes", command.notes);
}

#[derive(Clone, Copy)]
enum Flag {
    Setting(&'static Settings, &'static Setting),
    Switch(&'static Switch),
}

impl Flag {
    fn name(self) -> &'static str {
        match self {
            Self::Setting(_, setting) => setting.name,
            Self::Switch(switch) => switch.name,
        }
    }

    fn set(self) -> Set {
        match self {
            Self::Setting(_, setting) => setting.set,
            Self::Switch(switch) => switch.set,
        }
    }

    fn shown(self) -> String {
        match self {
            Self::Setting(_, setting) => format!("--{} {}", setting.name, setting.value),
            Self::Switch(switch) => format!("--{}", switch.name),
        }
    }
}

fn group_flags(groups: &[&'static Settings]) -> Vec<Flag> {
    groups.iter().flat_map(|group| group.table.iter().map(|setting| Flag::Setting(group, setting))).collect()
}

fn command_flags(table: &Table, command: &Command) -> Vec<Flag> {
    let mut flags = group_flags(table.settings);
    flags.extend(group_flags(command.settings));
    flags.extend(command.switches.iter().map(Flag::Switch));
    flags
}

fn flags(table: &Table) -> Vec<Flag> {
    let mut flags = group_flags(table.settings);
    for command in table.commands {
        flags.extend(group_flags(command.settings));
        flags.extend(command.switches.iter().map(Flag::Switch));
    }
    let mut seen = BTreeSet::new();
    flags.retain(|flag| seen.insert(flag.name()));
    flags
}

fn default_words(group: &Settings, setting: &Setting) -> String {
    match setting.fallback {
        Fallback::Refuse => String::from("required"),
        Fallback::Text(text) => format!("default {text}"),
        Fallback::Base(base) => format!("default ${}/{}", base.variable().name, group.tool),
    }
}

fn flag_entry(out: &mut String, flag: Flag) {
    heading(out, &code(&flag.shown()));
    item(out, "Set", set_name(flag.set()));
    match flag {
        Flag::Setting(group, setting) => {
            let default = default_words(group, setting);
            match setting.fallback {
                Fallback::Refuse => out.push_str("- **Required.**\n"),
                Fallback::Text(_) | Fallback::Base(_) => {
                    item(out, "Default", default.strip_prefix("default ").expect("a default starts with its label"));
                }
            }
            if let Some(twin) = setting.twin {
                item(out, "Twin", &code(twin));
            }
            item(out, "", setting.about);
            item(out, "Notes", setting.notes);
        }
        Flag::Switch(switch) => {
            item(out, "", switch.line);
            item(out, "Notes", switch.notes);
        }
    }
}

struct VariableEntry {
    variable: Variable,
    absent: String,
    twin: Option<&'static str>,
}

impl VariableEntry {
    fn declared(variable: Variable) -> Self {
        Self { variable, absent: variable.absent.to_owned(), twin: None }
    }
}

fn push_variable(entries: &mut Vec<VariableEntry>, entry: VariableEntry) {
    if !entries.iter().any(|present| present.variable.name == entry.variable.name) {
        entries.push(entry);
    }
}

fn twins(entries: &mut Vec<VariableEntry>, groups: &[&Settings]) {
    for group in groups {
        for setting in group.table {
            if let Some(name) = setting.twin {
                push_variable(
                    entries,
                    VariableEntry {
                        variable: Variable {
                            name,
                            set: setting.set,
                            form: setting.value,
                            line: setting.about,
                            absent: "",
                            notes: setting.notes,
                            way: Way::Reads,
                        },
                        absent: default_words(group, setting),
                        twin: Some(setting.name),
                    },
                );
            }
        }
    }
}

fn variables(table: &Table) -> Vec<VariableEntry> {
    let mut entries = Vec::new();
    twins(&mut entries, table.settings);
    for variable in table.variables {
        push_variable(&mut entries, VariableEntry::declared(**variable));
    }
    for command in table.commands {
        twins(&mut entries, command.settings);
        for variable in command.variables {
            push_variable(&mut entries, VariableEntry::declared(**variable));
        }
    }
    for group in table.settings.iter().chain(table.commands.iter().flat_map(|command| command.settings)) {
        for setting in group.table {
            match setting.fallback {
                Fallback::Base(base) => {
                    push_variable(&mut entries, VariableEntry::declared(*base.variable()));
                    push_variable(&mut entries, VariableEntry::declared(HOME));
                }
                Fallback::Refuse | Fallback::Text(_) => {}
            }
        }
    }
    entries
}

type TreeVariables = Vec<(&'static str, Vec<VariableEntry>)>;

fn tree_variables(tables: &[&Table]) -> TreeVariables {
    tables.iter().map(|table| (table.binary, variables(table))).collect()
}

fn readers(name: &str, tree: &TreeVariables) -> Vec<&'static str> {
    let mut readers = Vec::new();
    for (binary, entries) in tree {
        if !readers.contains(binary)
            && entries.iter().any(|entry| entry.variable.name == name && entry.variable.way == Way::Reads)
        {
            readers.push(*binary);
        }
    }
    readers
}

fn variable_entry(out: &mut String, entry: &VariableEntry, readers: &[&str]) {
    let variable = entry.variable;
    heading(out, &code(variable.name));
    item(out, "Set", set_name(variable.set));
    out.push_str(match variable.way {
        Way::Reads => "- **Reads**\n",
        Way::Sets => "- **Sets**\n",
    });
    item(out, "Form", variable.form);
    item(out, "Absent", &entry.absent);
    if let Some(twin) = entry.twin {
        item(out, "Twin of", &format!("`--{twin}`"));
    }
    item(out, "", variable.line);
    item(out, "Notes", variable.notes);
    item(out, "Read by", &readers.join(", "));
}

#[cfg(test)]
mod tests;
