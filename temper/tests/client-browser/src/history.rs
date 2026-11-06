//! Reading histories made before the page opens (temper/docs/design/client/browser.md, 4.3).
use crate::{Tier, WITHIN, temper};
use skein_fake_person::Step as PersonStep;
use skein_fake_person_browser::{Command, Opens, Script, Sent, Setup, Stdin, Step, harness::Output};
use skein_ui::Role;
use std::path::Path;

impl Tier {
    /// One `temper ctl` call on this deployment's client directory, with closed stdin and exit 0.
    #[must_use]
    pub fn ctl(&self, words: &[&str]) -> [Step; 2] {
        ctl_call(&temper(), &self.scratch.path().join("client"), words)
    }

    /// Start the playground and chat T1, waiting for its answer and each following message's answer.
    #[must_use]
    pub fn ctl_chat(&self, first: &str, then: &[&str]) -> Vec<Step> {
        chat_calls(&temper(), &self.scratch.path().join("client"), first, then)
    }

    /// Sign ctl in, finish its calls, then open the spent link to print the reader's fresh link.
    #[must_use]
    pub fn ctl_face(&self, calls: Vec<Step>) -> Script {
        let mut steps = vec![
            Step::Run(Command {
                program: temper(),
                args: vec![
                    "ctl".into(),
                    self.scratch.path().join("client").into_os_string(),
                    "sign-in".into(),
                    "-".into(),
                ],
                env: vec![],
                stdin: Stdin::Printed,
                opens: None,
                within: WITHIN,
            }),
            exited(),
        ];
        steps.extend(calls);
        steps.extend([open_printed(), exited()]);
        Script::new(steps.into_boxed_slice())
    }

    /// Hold the printed link for ctl instead of opening it in the first page.
    #[must_use]
    pub fn held_setup(&self) -> Setup<'_> {
        let mut setup = self.setup();
        setup.printed.sent = Sent::Command;
        setup
    }
}

/// Open the reader's fresh printed link after ctl finishes, then navigate to `address`.
#[must_use]
pub fn reader_opens(address: &[u8]) -> Vec<Step> {
    vec![
        open_printed(),
        exited(),
        Step::Person(PersonStep::See {
            find: skein_fake_person::Find::exactly_named(Role::Button, b"Try the playground"),
            within: WITHIN,
        }),
        Step::Person(PersonStep::Press {
            find: skein_fake_person::Find::exactly_named(Role::Button, b"Try the playground"),
        }),
        Step::Person(PersonStep::See { find: crate::live(), within: WITHIN }),
        Step::Person(PersonStep::Go { address: address.into() }),
    ]
}

fn ctl_call(program: &Path, directory: &Path, words: &[&str]) -> [Step; 2] {
    let mut args = vec!["ctl".into(), directory.into()];
    args.extend(words.iter().map(std::ffi::OsString::from));
    [
        Step::Run(Command {
            program: program.into(),
            args,
            env: vec![],
            stdin: Stdin::Closed,
            opens: None,
            within: WITHIN,
        }),
        exited(),
    ]
}

fn chat_calls(program: &Path, directory: &Path, first: &str, then: &[&str]) -> Vec<Step> {
    let mut calls = Vec::from(ctl_call(program, directory, &["playground", "start", "--key", "history-playground"]));
    calls.extend(ctl_call(
        program,
        directory,
        &["chat", "start", "--project", "1", "--model", "demo", "--key", "history-chat", first],
    ));
    calls.extend(ctl_call(program, directory, &["task", "show", "1", "--until", "replied"]));
    for (number, words) in then.iter().enumerate() {
        let key = format!("history-message-{number}");
        calls.extend(ctl_call(program, directory, &["task", "message", "1", words, "--key", &key]));
        calls.extend(ctl_call(program, directory, &["task", "show", "1", "--until", "replied"]));
    }
    calls
}

fn exited() -> Step {
    Step::Exited { code: 0, within: WITHIN }
}

fn open_printed() -> Step {
    Step::Run(Command {
        program: "/bin/cat".into(),
        args: vec![],
        env: vec![],
        stdin: Stdin::Printed,
        opens: Some(Opens {
            prefix: b"http://127.0.0.1:".as_slice().into(),
            on: Output::Stdout,
            keep: b"owner-link".as_slice().into(),
        }),
        within: WITHIN,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, ffi::OsStr};

    fn calls() -> Vec<Step> {
        chat_calls(
            Path::new("/test/temper"),
            Path::new("/test/client"),
            "Repeated words",
            &["Repeated words", "Repeated words"],
        )
    }

    #[test]
    fn repeated_words_still_get_distinct_act_keys() {
        // Playground, chat and the two following messages are four distinct acts.
        const ACTS: usize = 4;
        let mut keys = BTreeSet::new();
        for step in calls() {
            if let Step::Run(command) = step {
                for pair in command.args.windows(2) {
                    if pair[0] == OsStr::new("--key") {
                        assert!(keys.insert(pair[1].clone()), "a key cannot name two acts");
                    }
                }
            }
        }
        assert_eq!(keys.len(), ACTS);
    }

    #[test]
    fn each_message_waits_for_its_answer_before_the_next() {
        // The first words and the two following messages each need an answer.
        const MESSAGES: u32 = 3;
        let mut waiting = false;
        let mut messages = 0;
        for step in calls() {
            if let Step::Run(command) = step {
                let words: Vec<_> = command.args.iter().skip(2).map(|arg| arg.to_str().expect("ctl word")).collect();
                match words.as_slice() {
                    ["chat", "start", ..] | ["task", "message", ..] => {
                        assert!(!waiting, "the previous answer must finish before another message");
                        waiting = true;
                        messages += 1;
                    }
                    ["task", "show", "1", "--until", "replied"] => {
                        assert!(waiting, "a wait belongs to a message");
                        waiting = false;
                    }
                    ["playground", "start", ..] => {}
                    _ => panic!("unexpected history call"),
                }
            }
        }
        assert!(!waiting, "the reader opens only after the final answer");
        assert_eq!(messages, MESSAGES, "the first words and both following messages");
    }
}
