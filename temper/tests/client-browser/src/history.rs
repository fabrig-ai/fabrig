//! Reading histories made before the page opens (temper/docs/design/client/web/testing.md, 3).
use crate::{Tier, WITHIN, temper};
use skein_fake_person::Step as PersonStep;
use skein_fake_person_browser::{Command, Opens, Script, Sent, Setup, Stdin, Step, harness::Output};
use skein_ui::Role;
use std::path::Path;

impl Tier {
    /// Make the ordinary stories' project as its owner before opening the browser's fresh link.
    /// The deployment seeds none (temper/docs/design/protocol/configuration/moved.md, 2).
    #[must_use]
    pub fn project_face(&self) -> Script {
        self.ctl_face(self.project_calls().into())
    }

    /// Make a playground before the reader opens its page, without starting a chat.
    #[must_use]
    pub fn playground_face(&self) -> Script {
        self.ctl_face(self.ctl(&["playground", "start", "--key", "history-playground"]).into())
    }

    pub(crate) fn project_calls(&self) -> [Step; 2] {
        self.ctl(&["project", "make", "Project 1", "--key", "browser-project"])
    }

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
    pub fn ctl_face(&self, mut calls: Vec<Step>) -> Script {
        calls.extend([open_printed(), exited()]);
        self.ctl_only(calls)
    }

    /// Use the printed link in ctl only, without opening it in the browser.
    pub(crate) fn ctl_only(&self, calls: Vec<Step>) -> Script {
        let program = temper();
        let directory = self.scratch.path();
        crate::configuration::install(directory, &program);
        Script::new(seed_calls(&program, directory, calls).into_boxed_slice())
    }

    /// Hold the printed link for ctl instead of opening it in the first page.
    #[must_use]
    pub fn held_setup(&self) -> Setup<'_> {
        let mut setup = self.setup();
        setup.printed.sent = Sent::Command;
        setup
    }
}

/// Open the browser's fresh printed link after the owner's ctl face finishes.
#[must_use]
pub fn page_opens() -> Vec<Step> {
    vec![open_printed(), exited()]
}

/// Open the reader's fresh printed link after ctl finishes, then navigate to `address`.
#[must_use]
pub fn reader_opens(address: &[u8]) -> Vec<Step> {
    let mut steps = page_opens();
    steps.extend([
        Step::Person(PersonStep::See {
            find: skein_fake_person::Find::exactly_named(Role::Button, b"Try the playground"),
            within: WITHIN,
        }),
        Step::Person(PersonStep::Press {
            find: skein_fake_person::Find::exactly_named(Role::Button, b"Try the playground"),
        }),
        Step::Person(PersonStep::See { find: crate::live(), within: WITHIN }),
        Step::Person(PersonStep::Go { address: crate::page_address(address) }),
    ]);
    steps
}

fn seed_calls(program: &Path, directory: &Path, calls: Vec<Step>) -> Vec<Step> {
    use crate::scripted;
    let client = directory.join("client");
    let mut steps = vec![
        Step::Run(Command {
            program: program.into(),
            args: vec!["ctl".into(), client.as_os_str().into(), "sign-in".into(), "-".into()],
            env: vec![],
            stdin: Stdin::Printed,
            opens: None,
            within: WITHIN,
        }),
        exited(),
    ];
    if directory.join(scripted::SCRIPTS).is_file() {
        // The provider takes effect at this start. Use the owner's seed session,
        // so the key is usable before seeding any history or opening a page.
        let key = directory.join(scripted::KEY);
        steps.extend(ctl_call(
            program,
            &client,
            &[
                "account",
                "add-key",
                &scripted::PROVIDER.to_string(),
                key.to_str().expect("model key path"),
                "--key",
                "browser-model-key",
            ],
        ));
        steps.extend(ctl_call(program, &client, &["account", "list", "--until", "usable"]));
    }
    steps.extend(calls);
    steps
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
    fn the_scripted_account_is_usable_before_playground_setup() {
        let scratch = skein_scratch::Scratch::new("browser-account-seed");
        let directory = scratch.path();
        crate::scripted::write(directory, &serde_json::json!({}));
        let program = Path::new("/test/temper");
        let client = directory.join("client");
        let start = ctl_call(program, &client, &["playground", "start"]);
        let steps = seed_calls(program, directory, start.into());
        let key = directory.join(crate::scripted::KEY);
        let provider = crate::scripted::PROVIDER.to_string();
        let expected: &[&[&OsStr]] = &[
            &[OsStr::new("sign-in"), OsStr::new("-")],
            &[
                OsStr::new("account"),
                OsStr::new("add-key"),
                OsStr::new(&provider),
                key.as_os_str(),
                OsStr::new("--key"),
                OsStr::new("browser-model-key"),
            ],
            &[OsStr::new("account"), OsStr::new("list"), OsStr::new("--until"), OsStr::new("usable")],
            &[OsStr::new("playground"), OsStr::new("start")],
        ];
        assert_eq!(steps.len(), 2 * expected.len(), "one exit check per call");
        for (pair, words) in steps.as_chunks::<2>().0.iter().zip(expected) {
            let [Step::Run(command), Step::Exited { code: 0, .. }] = pair else {
                panic!("ctl call followed by its successful exit");
            };
            assert_eq!(&command.args[2..], *words, "the owner waits for usability before seeding the story");
        }
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
