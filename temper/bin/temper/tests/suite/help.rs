use temper_client_shell::CLIENT;
use temper_shell::table;

fn help(text: &str) -> bool {
    let Some((_, commands)) = text.split_once("\ncommands:\n") else { return false };
    let names: Vec<_> = commands
        .split("\n\n")
        .next()
        .expect("command lines")
        .lines()
        .filter_map(|line| line.strip_prefix("  "))
        .filter(|line| !line.starts_with(' '))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    text == CLIENT.help() && names == ["help"] && text.contains("  temper [ADDRESS]\n")
}

fn internal(text: &str) -> bool {
    text == CLIENT.command_help(&table::CTL) && text.starts_with("internal:")
}

fn version(text: &str) -> bool {
    let prefix = format!("temper {} (", env!("CARGO_PKG_VERSION"));
    let Some(commit) = text.strip_prefix(&prefix).and_then(|text| text.strip_suffix(")\n")) else {
        return false;
    };
    let hash = commit.strip_suffix("-dirty").unwrap_or(commit);
    commit == "unknown" || (hash.len() == 12 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn commands(text: &str) -> bool {
    text == table::commands_page(&CLIENT)
}

fn pages(text: &str) -> bool {
    text == temper_shell::help::answer(CLIENT.binary, &temper_shell::help::Request::Index).expect("index")
}

fn empty(text: &str) -> bool {
    text.is_empty()
}

type Check = fn(&str) -> bool;
type Case = (&'static [&'static str], i32, Check, &'static str);
const CASES: &[Case] = &[
    (&["-h"], 0, help, ""),
    (&["--help"], 0, help, ""),
    (&["help"], 0, help, ""),
    (&["ctl", "--help"], 0, internal, ""),
    (&["--version"], 0, version, ""),
    (&["help", "commands"], 0, commands, ""),
    (&["help", "pages"], 0, pages, ""),
    (&["frob"], 2, empty, "temper: not an address: frob\nusage: temper [ADDRESS]\n"),
    (&["http://127.0.0.1:8367", "extra"], 2, empty, "temper: invalid arguments\nusage: temper [ADDRESS]\n"),
    (
        &["help", "-x"],
        2,
        empty,
        "temper help: invalid arguments\nusage: temper help [NAME | pages | --search WORDS...]\n",
    ),
    (&[], 1, empty, "temper: no command; temper --help lists them\n"),
    (&["http://127.0.0.1:8367"], 1, empty, "temper: the TUI needs a terminal on stdin and stdout\n"),
];

#[test]
fn the_binary_answers_its_help_and_refuses_what_it_hasnt() {
    let mut failures = Vec::new();
    for (arguments, code, stdout, stderr) in CASES {
        let output = super::output(arguments);
        let out = String::from_utf8(output.stdout).expect("UTF-8 stdout");
        let err = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        if output.status.code() != Some(*code) || !stdout(&out) || err != *stderr {
            failures.push(format!(
                "{arguments:?}: exit {:?}, stdout {out:?}, stderr {err:?}; expected exit {code}, stderr {stderr:?}",
                output.status.code()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
