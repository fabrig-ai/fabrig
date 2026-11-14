use hearth_domain_render_vocabulary::errors::{Repository, Tool, Command, Ended};
use super::{Box, Encoder, Lines, field};







pub(super) fn repository_text(item: &Repository, lines: &mut Lines) {
    match item {
        Repository::NoScript => lines.put(b"no script at this path"),
        Repository::NoTemplate => lines.put(b"no template at this path"),
        Repository::Unused => lines.put(b"a template no entry names"),
        Repository::NoTree => lines.put(b"no tree entry at this path"),
        Repository::LinkTarget => lines.put(b"a symbolic link target out of form"),
        Repository::Stray => lines.put(b"a path protocol/format/README.md 2.1 does not allow"),
        Repository::Name => lines.put(b"a name protocol/format/README.md 2.4 does not allow"),
        Repository::Declaration => lines.put(b"a task directory without task.json"),
        Repository::Static => lines.put(b"a template without a destination name"),
        Repository::Destination => lines.put(b"two sources of one destination"),
        Repository::Submodule => lines.put(b"a submodule under a path render reads"),
        Repository::Link => lines.put(b"a document or template listed as a symbolic link"),
        Repository::Environment { env } => {
            lines.put(b"no environment ");
            lines.put(env);
            lines.put(b" under envs/");
        }
    }
}
pub(super) fn repository_json(item: &Repository, json: &mut Encoder) {
    match item {
        Repository::Environment { env } => {
            json.object_start();
            field(json, b"kind", b"environment");
            field(json, b"env", env);
            json.object_end();
        }
        Repository::NoScript => json.string(b"no_script"),
        Repository::NoTemplate => json.string(b"no_template"),
        Repository::Unused => json.string(b"unused"),
        Repository::NoTree => json.string(b"no_tree"),
        Repository::LinkTarget => json.string(b"link_target"),
        Repository::Stray => json.string(b"stray"),
        Repository::Name => json.string(b"name"),
        Repository::Declaration => json.string(b"declaration"),
        Repository::Static => json.string(b"static"),
        Repository::Destination => json.string(b"destination"),
        Repository::Submodule => json.string(b"submodule"),
        Repository::Link => json.string(b"link"),
    }
}


pub(super) fn command_name(item: Command) -> &'static [u8] {
    match item {
        Command::Mmdebstrap => b"mmdebstrap",
        Command::RevParse => b"rev_parse",
        Command::LsTree => b"ls_tree",
        Command::CatFile => b"cat_file",
        Command::Diff => b"diff",
    }
}
pub(super) fn command_text(item: Command) -> &'static [u8] {
    match item {
        Command::Mmdebstrap => b"mmdebstrap",
        Command::RevParse => b"git rev-parse",
        Command::LsTree => b"git ls-tree",
        Command::CatFile => b"git cat-file",
        Command::Diff => b"git diff",
    }
}


pub(super) fn tool_text(item: &Tool, lines: &mut Lines) {
    lines.put(command_text(item.command));
    match &item.ended {
        Ended::Timeout => lines.put(b" timed out"),
        Ended::Start => lines.put(b" could not start"),
        Ended::Exit(code) => {
            lines.put(b" exited ");
            lines.decimal(u64::from(*code));
        }
        Ended::Signal(signal) => {
            lines.put(b" was killed by signal ");
            lines.decimal(u64::from(*signal));
        }
        Ended::Output => lines.put(b" gave invalid output"),
        Ended::Missing { oid } => {
            lines.put(b" is missing object ");
            lines.put(&oid.0);
        }
        Ended::Stream => lines.put(b" failed to read or write a pipe"),
    }
}
pub(super) fn tool_json(item: &Tool, json: &mut Encoder) {
    json.object_start();
    field(json, b"command", command_name(item.command));
    json.key(b"ended");
    json.object_start();
    match &item.ended {
        Ended::Timeout => field(json, b"kind", b"timeout"),
        Ended::Start => field(json, b"kind", b"start"),
        Ended::Exit(code) => {
            json.key(b"code");
            json.unsigned(u64::from(*code));
            field(json, b"kind", b"exit");
        }
        Ended::Signal(signal) => {
            json.key(b"signal");
            json.unsigned(u64::from(*signal));
            field(json, b"kind", b"signal");
        }
        Ended::Output => field(json, b"kind", b"output"),
        Ended::Missing { oid } => {
            field(json, b"kind", b"missing");
            field(json, b"oid", &oid.0);
        }
        Ended::Stream => field(json, b"kind", b"stream"),
    }
    json.object_end();
    json.key(b"tail");
    json.array_start();
    for line in &item.tail {
        json.string(line);
    }
    json.array_end();
    json.object_end();
}

