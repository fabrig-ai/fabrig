//! The command table's answers before any effect (temper/docs/design/testing/qa/contracts.md, 3).
use skein_shell::{Command, Reach};
use temper_qa_tool::{Admission, TABLE, admit};

const VERSION: &str = "1.2.3";
const COMMIT: &str = "0123456789ab-dirty";
const BUILD_COMMIT: &[u8] = b"0123456789abcdef0123456789abcdef01234567";

fn words(command: &Command) -> &'static [&'static str] {
    let Reach::Words(words) = command.reach else { panic!("expected a QA command: {command:?}") };
    words
}

fn captured(arguments: impl IntoIterator<Item = impl AsRef<[u8]>>) -> Vec<Box<[u8]>> {
    arguments.into_iter().map(|argument| argument.as_ref().into()).collect()
}

fn answer(arguments: &[Box<[u8]>]) -> (u8, String, String) {
    let Admission::Answer { code, stdout, stderr } = admit(VERSION, COMMIT, arguments) else {
        panic!("expected an answer for {arguments:?}");
    };
    (code, stdout, stderr)
}

fn paragraph(text: &str) -> String {
    text.split("\n\n").next().expect("paragraph").lines().collect::<Vec<_>>().join(" ")
}

#[test]
fn the_table_answers_help_version_and_unknown_commands() {
    for flag in ["--help", "-h", "help"] {
        let (code, stdout, stderr) = answer(&captured([flag]));
        assert_eq!(code, 0, "{flag}");
        assert_eq!(stderr, "", "{flag}");
        assert_eq!(
            paragraph(&stdout),
            "temper-qa: QA after landing: main's landings triaged into cases, and passes run on builds kept for them."
        );
        let body = stdout.split_once("\n\n").expect("help body").1;
        for command in TABLE.commands {
            let name = words(command).join(" ");
            assert!(!body.contains(&name), "internal entry: {name}");
        }
        for name in ["TEMPER_INTERNAL_QA_HOME", "TEMPER_INTERNAL_QA_SESSION", "TEMPER_INTERNAL_QA_AREA"] {
            assert!(!body.contains(name), "internal entry: {name}");
        }
        assert_eq!(stdout, TABLE.help());
    }
    assert_eq!(answer(&captured(["--version"])), (0, format!("temper-qa {VERSION} ({COMMIT})\n"), String::new()));
    assert_eq!(
        answer(&captured(["nope"])),
        (2, String::new(), "temper-qa: no command nope; temper-qa --help says more\n".into())
    );
    for command in TABLE.commands {
        let words = words(command);
        let arguments = captured(words.iter().copied().chain(["--help"]));
        let (code, stdout, stderr) = answer(&arguments);
        assert_eq!(code, 0, "{words:?}");
        assert_eq!(stderr, "", "{words:?}");
        assert_eq!(
            paragraph(&stdout),
            "internal: not part of temper-qa's public command line; it may change without notice.",
            "{words:?}"
        );
        assert!(stdout.contains(&format!("temper-qa {}", words.join(" "))), "{stdout}");
    }
}

#[test]
fn each_command_refuses_bad_arguments_before_opening_a_home() {
    type Cases<'a> = &'a [(&'a [&'a [u8]], &'a str)];
    let commands: &[(&[&str], Cases<'_>)] = &[
        (&["run"], &[(&[b"extra"], "no arguments are taken")]),
        (&["wake"], &[(&[b"extra"], "no arguments are taken")]),
        (
            &["build", "keep"],
            &[
                (&[], "COMMIT and DIR are not given"),
                (&[BUILD_COMMIT], "DIR is not given"),
                (&[BUILD_COMMIT, b"stage", b"extra"], "expected COMMIT and DIR"),
                (&[b"--unknown", b"stage"], "expected COMMIT and DIR"),
            ],
        ),
        (&["triage", "none"], &[(&[], "WHY is not given"), (&[b"--unknown"], "expected one argument")]),
        (&["triage", "case"], &[(&[], "FILE is not given"), (&[b"--unknown"], "expected one argument")]),
        (&["triage", "gap"], &[(&[b"qa", b"\xff"], "expected KIND WHY as UTF-8 arguments")]),
        (
            &["pass", "gap"],
            &[
                (&[], "expected CASE KIND WHY"),
                (&[b"no", b"qa", b"missing"], "expected a case number"),
                (&[b"1", b"\xff", b"missing"], "expected KIND WHY as UTF-8 arguments"),
            ],
        ),
        (&["gaps"], &[(&[b"--unknown"], "expected [--open]")]),
        (&["pass", "setup"], &[(&[], "NAME is not given"), (&[b"--unknown"], "expected NAME")]),
        (
            &["pass", "recipe"],
            &[
                (&[], "NAME and FILE are not given"),
                (&[b"fresh"], "FILE is not given"),
                (&[b"fresh", b"--unknown"], "expected NAME and FILE"),
            ],
        ),
        (
            &["pass", "outcome"],
            &[
                (&[], "expected CASE RESULT NOTES"),
                (&[b"no", b"failed", b""], "expected a case number"),
                (&[b"1", b"\xff", b""], "RESULT and NOTES must be UTF-8"),
                (&[b"1", b"failed", b"", b"--finding"], "expected --finding N"),
                (&[b"1", b"failed", b"", b"--finding", b"no"], "expected --finding N"),
                (&[b"1", b"failed", b"", b"--finding", b"1", b"--finding"], "expected --finding N"),
                (&[b"1", b"failed", b"", b"--unknown", b"1"], "expected --finding N"),
            ],
        ),
        (&["pass", "finding"], &[(&[], "FILE is not given"), (&[b"--unknown"], "expected one argument")]),
        (
            &["pass", "check"],
            &[
                (&[], "expected FINDING RESULT NOTES"),
                (&[b"no", b"failed", b""], "expected a finding number"),
                (&[b"1", b"failed", b"\xff"], "RESULT and NOTES must be UTF-8"),
                (&[b"1", b"failed", b"", b"--finding", b"1"], "expected FINDING RESULT NOTES"),
            ],
        ),
        (&["pass", "note"], &[(&[], "WORDS is not given"), (&[b"--unknown"], "expected one argument")]),
        (&["show"], &[(&[b"extra"], "no arguments are taken")]),
        (&["lines"], &[(&[b"extra"], "no arguments are taken")]),
        (&["report"], &[(&[], "PASS is not given"), (&[b"0"], "expected a pass number")]),
        (&["pass", "open"], &[(&[b"--unknown"], "expected [--ctl]")]),
        (&["pass", "hold"], &[(&[b"--unknown"], "expected [--ctl]")]),
        (&["pass", "line"], &[(&[], "LINE is not given"), (&[b"--unknown"], "expected one argument")]),
        (&["pass", "close"], &[(&[b"extra"], "no arguments are taken")]),
        (&["tune"], &[(&[b"extra"], "no arguments are taken")]),
    ];
    assert_eq!(commands.len(), TABLE.commands.len());
    for &(name, cases) in commands {
        let command = TABLE.commands.iter().find(|command| words(command) == name).expect("command in table");
        for &(rest, why) in cases {
            let arguments = captured(name.iter().map(|word| word.as_bytes()).chain(rest.iter().copied()));
            assert_eq!(answer(&arguments), (2, String::new(), TABLE.refusal(command, why)), "{name:?} {rest:?}");
        }
    }
    let arguments = captured(["pass", "outcome", "3", "passed", "ok", "--finding"]);
    let (_, _, stderr) = answer(&arguments);
    assert!(stderr.contains("pass outcome CASE RESULT NOTES [--finding N]..."), "{stderr}");
}

#[test]
fn a_command_is_admitted_with_its_argument_bytes() {
    let path = b"stage-\xff";
    let arguments = captured([b"build".as_slice(), b"keep", BUILD_COMMIT, path]);
    let Admission::Run { command, arguments } = admit(VERSION, COMMIT, &arguments) else {
        panic!("expected build keep to run");
    };
    assert_eq!(words(&command), ["build", "keep"]);
    assert_eq!(arguments, captured([BUILD_COMMIT, path]));
}
