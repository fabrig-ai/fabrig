//! Reading histories made before the page opens (temper/docs/design/client/web/testing.md, 3).
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

    /// Start chat `task`, the next number the box gives, waiting for its answer and each following message's.
    #[must_use]
    pub fn ctl_next_chat(&self, task: u32, first: &str, then: &[&str]) -> Vec<Step> {
        next_chat_calls(&temper(), &self.scratch.path().join("client"), task, first, then)
    }

    /// Send chat `task` each of `words`, each answered before the next; `from` numbers
    /// the first message's key, past any this chat was sent.
    #[must_use]
    pub fn ctl_messages(&self, task: u32, from: u32, words: &[&str]) -> Vec<Step> {
        message_calls(&temper(), &self.scratch.path().join("client"), task, from, words)
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
        Step::Person(PersonStep::Go { address: crate::page_address(address) }),
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
    calls.extend(next_chat_calls(program, directory, 1, first, then));
    calls
}

fn next_chat_calls(program: &Path, directory: &Path, task: u32, first: &str, then: &[&str]) -> Vec<Step> {
    let key = format!("history-chat-{task}");
    let mut calls = Vec::from(ctl_call(
        program,
        directory,
        &["chat", "start", "--project", "1", "--model", "demo", "--key", &key, first],
    ));
    calls.extend(ctl_call(program, directory, &["task", "show", &task.to_string(), "--until", "replied"]));
    calls.extend(message_calls(program, directory, task, 0, then));
    calls
}

fn message_calls(program: &Path, directory: &Path, task: u32, from: u32, words: &[&str]) -> Vec<Step> {
    let mut calls = Vec::new();
    let task = task.to_string();
    for (offset, words) in words.iter().enumerate() {
        let number = from.checked_add(u32::try_from(offset).expect("message count fits")).expect("message key fits");
        let key = format!("history-message-{task}-{number}");
        calls.extend(ctl_call(program, directory, &["task", "message", &task, words, "--key", &key]));
        calls.extend(ctl_call(program, directory, &["task", "show", &task, "--until", "replied"]));
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

    #[test]
    fn two_chats_name_their_own_task_and_keep_distinct_keys() {
        const WORDS: &str = "Repeated words";
        let program = Path::new("/test/temper");
        let directory = Path::new("/test/client");
        let mut keys = BTreeSet::new();
        let mut starts = 0;
        let mut messages = 0;
        let mut waits = 0;
        for (task, from, calls) in [
            (1, 0, next_chat_calls(program, directory, 1, WORDS, &[WORDS])),
            (2, 0, next_chat_calls(program, directory, 2, WORDS, &[WORDS])),
            (1, 1, message_calls(program, directory, 1, 1, &[WORDS, WORDS])),
        ] {
            let task = task.to_string();
            let mut number = from;
            let mut waiting = false;
            for step in calls {
                if let Step::Run(command) = step {
                    let words: Vec<_> =
                        command.args.iter().skip(2).map(|arg| arg.to_str().expect("ctl word")).collect();
                    match words.as_slice() {
                        ["chat", "start", "--project", "1", "--model", "demo", "--key", key, WORDS] => {
                            assert!(!waiting, "the previous answer must finish before another message");
                            assert_eq!(*key, format!("history-chat-{task}"));
                            assert!(keys.insert((*key).to_owned()), "a key cannot name two acts");
                            starts += 1;
                            waiting = true;
                        }
                        ["task", "message", called_task, WORDS, "--key", key] => {
                            assert!(!waiting, "the previous answer must finish before another message");
                            assert_eq!(*called_task, task, "a message belongs to its chat");
                            assert_eq!(*key, format!("history-message-{task}-{number}"));
                            assert!(keys.insert((*key).to_owned()), "a key cannot name two acts");
                            number += 1;
                            messages += 1;
                            waiting = true;
                        }
                        ["task", "show", called_task, "--until", "replied"] => {
                            assert!(waiting, "a wait belongs to a message");
                            assert_eq!(*called_task, task, "a wait belongs to its chat");
                            waits += 1;
                            waiting = false;
                        }
                        _ => panic!("unexpected history call"),
                    }
                }
            }
            assert!(!waiting, "each batch finishes after its final answer");
        }
        assert_eq!(starts, 2);
        assert_eq!(messages, 4);
        assert_eq!(waits, 6);
        assert_eq!(keys.len(), 6);
    }
}
