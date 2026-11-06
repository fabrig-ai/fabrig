//! Reading histories made before the page opens (temper/docs/design/client/browser.md, 4.3).
use crate::{Tier, WITHIN, temper};
use skein_fake_person::Step as PersonStep;
use skein_fake_person_browser::{Command, Opens, Script, Sent, Setup, Stdin, Step, harness::Output};
use skein_ui::Role;

impl Tier {
    /// One `temper ctl` call on this deployment's client directory, with closed stdin and exit 0.
    #[must_use]
    pub fn ctl(&self, words: &[&str]) -> [Step; 2] {
        let mut args = vec!["ctl".into(), self.scratch.path().join("client").into_os_string()];
        args.extend(words.iter().map(std::ffi::OsString::from));
        [
            Step::Run(Command {
                program: temper(),
                args,
                env: vec![],
                stdin: Stdin::Closed,
                opens: None,
                within: WITHIN,
            }),
            exited(),
        ]
    }

    /// Start the playground and chat T1, waiting for its answer and each following message's answer.
    #[must_use]
    pub fn ctl_chat(&self, first: &str, then: &[&str]) -> Vec<Step> {
        let mut calls = Vec::from(self.ctl(&["playground", "start", "--key", "history-playground"]));
        calls.extend(self.ctl(&["chat", "start", "--project", "1", "--model", "demo", "--key", "history-chat", first]));
        calls.extend(self.ctl(&["task", "show", "1", "--until", "replied"]));
        for (number, words) in then.iter().enumerate() {
            let key = format!("history-message-{number}");
            calls.extend(self.ctl(&["task", "message", "1", words, "--key", &key]));
            calls.extend(self.ctl(&["task", "show", "1", "--until", "replied"]));
        }
        calls
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
