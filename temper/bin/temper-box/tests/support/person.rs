//! A real native client driven by jig's finite semantic-tree script.
use jig_client_protocol_link::Signing;
use skein_fake_person::{Doing, Face, Find, Found, Next, Step, Target, TreeFace};
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::Host;
use temper_client_domain::Address;
use temper_client_domain::ChatsFilter;
use temper_client_shell_terminal::client::{NativeClient, Settings, Start};
/// Resolve the process story's Reply and task title together on one Item.
/// Other semantic queries and interaction checks retain `TreeFace`'s contract.
pub struct ReplyFace<'a> {
    tree: &'a skein_ui::Tree,
}
impl<'a> ReplyFace<'a> {
    #[must_use]
    pub fn new(tree: &'a skein_ui::Tree) -> Self {
        Self { tree }
    }
}
impl Face for ReplyFace<'_> {
    fn find(&mut self, find: &Find) -> Found {
        if find.target == Find::named(skein_ui::Role::Heading { level: 3 }, b"replied").target
            && let [scope @ Target::Role { role: skein_ui::Role::Item, .. }] = find.within.as_ref()
        {
            let item = Find { within: Box::new([]), target: scope.clone(), nth: 0 };
            for node in self.tree.nodes().iter().filter(|node| node.role == skein_ui::Role::Item) {
                let Some(mut face) = TreeFace::subtree(self.tree, node.id) else { continue };
                // A descendant Item matching the title does not qualify this root.
                if face.find(&item) == Found::Node(node.id)
                    && let reply @ Found::Node(_) = face.find(find)
                {
                    return reply;
                }
            }
            return Found::Absent;
        }
        TreeFace::new(self.tree).find(find)
    }
    fn role(&self, node: skein_ui::NodeId) -> Option<skein_ui::Role> {
        TreeFace::new(self.tree).role(node)
    }
    fn allows(&self, node: skein_ui::NodeId, act: &skein_ui::Act) -> bool {
        TreeFace::new(self.tree).allows(node, act)
    }
}
const START_WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(40);
// The demo uses its configuration's parking and forge-polling intervals.
const DEMO_LANDS_WITHIN: skein_lib::Duration = skein_lib::Duration::from_secs(122);
const SIGN_IN_FIRST: skein_lib::Duration = skein_lib::Duration::from_millis(100);
const SIGN_IN_MOST: skein_lib::Duration = skein_lib::Duration::from_secs(2);
const PIPE: Token = Token::new(u64::MAX);
const CLOSE: Token = Token::new(u64::MAX - 1);
#[derive(Clone, Copy, PartialEq, Eq)]
enum Begin {
    LinkWithKey,
    LinkWithOpenAiKey,
    Token,
    Link,
    Playground,
    RunningPlayground,
}
/// The script is separate from the client's production composition.
pub struct Script {
    steps: Box<[Step<Address>]>,
    begin: Begin,
}
#[must_use]
pub fn answered_chat(words: &[u8], answer: &[u8]) -> Script {
    Script {
        begin: Begin::LinkWithKey,
        steps: Box::new([
            Step::Go { address: Address::NewChat { draft: false } },
            Step::See { find: Find::named(skein_ui::Role::TextArea, b"What do you want done?"), within: START_WITHIN },
            Step::Type { find: Find::named(skein_ui::Role::TextArea, b"What do you want done?"), words: words.into() },
            Step::Gone { find: Find::text(b"No model yet"), within: START_WITHIN },
            Step::Press { find: Find::named(skein_ui::Role::Button, b"Send") },
            Step::See { find: Find::text(answer), within: START_WITHIN },
        ]),
    }
}
/// The configured first owner adds a deployment key through the accounts page.
#[must_use]
pub fn deployment_account() -> Script {
    use skein_ui::Role;
    Script {
        begin: Begin::LinkWithOpenAiKey,
        steps: Box::new([
            Step::Go { address: Address::Accounts },
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"What agents run on."),
                within: START_WITHIN,
            },
            Step::See {
                find: Find::exactly_named(Role::Disclosure, b"OpenAI API key \xc2\xb7 held by the deployment"),
                within: START_WITHIN,
            },
        ]),
    }
}
/// Every fresh deployment offers both subscriptions before either is chosen.
#[must_use]
pub fn welcome_model_offers() -> Script {
    use skein_ui::Role;
    Script {
        begin: Begin::Link,
        steps: Box::new([
            Step::Go { address: Address::Welcome { step: temper_client_domain::Step::Model { way: None } } },
            Step::See { find: Find::exactly_named(Role::Button, b"Continue with ChatGPT"), within: START_WITHIN },
            Step::See { find: Find::exactly_named(Role::Link, b"Use your Claude subscription"), within: START_WITHIN },
        ]),
    }
}
/// The same account's source is still signed in after a healthy service restart,
/// its card found by words of its heading.
#[must_use]
pub fn kept_chatgpt_account(heading: &[u8]) -> Script {
    use skein_ui::Role;
    Script {
        begin: Begin::Token,
        steps: Box::new([
            Step::Go { address: Address::Accounts },
            Step::See { find: Find::named(Role::Disclosure, heading), within: START_WITHIN },
            chatgpt_source_ready(heading),
        ]),
    }
}
/// Observe a ctl-started sign-in through the same person's served accounts page.
#[must_use]
pub fn signed_in_account(signing: &[u8], name: &[u8]) -> Script {
    use skein_ui::Role;
    // jig's translate.rs:124 leaves the signed-in display empty; follow it when jig carries one.
    let card = format!("ChatGPT · OpenAI · held by {}", String::from_utf8_lossy(name));
    Script {
        begin: Begin::Token,
        steps: Box::new([
            // A sign-in started from another client is drawn on Accounts (README 11).
            Step::Go { address: Address::Accounts },
            Step::See { find: Find::exactly_named(Role::Status, signing), within: START_WITHIN },
            Step::See { find: Find::exactly_named(Role::Disclosure, card.as_bytes()), within: START_WITHIN },
            chatgpt_source_ready(card.as_bytes()),
            Step::See { find: Find::exactly_named(Role::Status, b"Signed in to ChatGPT"), within: START_WITHIN },
        ]),
    }
}
/// Read the state in the sign-in source's own Item, inside the account card
/// whose heading holds these words.
/// The source kind and state are separate nodes (client/pages/accounts.md, 2).
#[must_use]
pub fn chatgpt_source_ready(heading: &[u8]) -> Step<Address> {
    use skein_ui::Role;
    // TreeFace reads disclosure contents even while the native shell folds them.
    Step::See {
        find: Find::exactly_named(Role::Status, b"Ready").within(Box::new([
            Target::Role { role: Role::Disclosure, name: heading.into() },
            Target::Role { role: Role::Item, name: b"ChatGPT sign-in".as_slice().into() },
        ])),
        within: START_WITHIN,
    }
}
/// Signs in by the printed link; the demo's running start keeps the deployment key.
/// Asks `cue` in a new chat, task `task`, then sees its committed reply.
/// Chats are ended by their requester; the scripted text replies and waits.
#[must_use]
pub fn demo_chat(task: u64, cue: &[u8], text: &[u8]) -> Script {
    use skein_ui::Role;
    Script {
        begin: Begin::RunningPlayground,
        steps: Box::new([
            Step::See { find: Find::exactly_named(Role::Button, b"Demo"), within: START_WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Demo") },
            Step::Go { address: Address::Chats { filter: ChatsFilter::All } },
            Step::See { find: Find::exactly_named(Role::Status, b"Ready"), within: START_WITHIN },
            Step::Gone { find: Find::exactly_named(Role::Status, b"Starting"), within: START_WITHIN },
            Step::Go { address: Address::NewChat { draft: false } },
            Step::See { find: Find::named(Role::TextArea, b"What do you want done?"), within: START_WITHIN },
            Step::Press { find: Find::exactly_named(Role::Button, b"Scripted model") },
            Step::Type { find: Find::named(Role::TextArea, b"What do you want done?"), words: cue.into() },
            Step::Press { find: Find::named(Role::Button, b"Send") },
            Step::See {
                find: Find::exactly_named(Role::Paragraph, b"You")
                    .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }])),
                within: START_WITHIN,
            },
            Step::See {
                find: Find::exactly_named(Role::Paragraph, cue)
                    .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }])),
                within: START_WITHIN,
            },
            Step::See {
                find: Find::text(text)
                    .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }])),
                within: START_WITHIN,
            },
            Step::See { find: Find::named(Role::TextArea, b"Message"), within: START_WITHIN },
            Step::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
            // Reading the reply in its chat marks it seen before the inbox opens.
            Step::Gone { find: Find::named(Role::Heading { level: 3 }, b"replied"), within: START_WITHIN },
            Step::Go { address: Address::Chat { task } },
            Step::See {
                find: Find::text(text)
                    .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }])),
                within: START_WITHIN,
            },
            Step::Gone {
                find: Find::text(b"Invalid call: unknown tool")
                    .within(Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }])),
                within: START_WITHIN,
            },
        ]),
    }
}
/// The first run chooses the scripted model, then delegates and lands without answering its wait.
#[must_use]
pub fn demo_change() -> Script {
    use skein_ui::Role;
    // The empty first run's chat is task 1; its change is task 2.
    const CHANGE: u64 = 2;
    let conversation = || Box::new([Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() }]);
    let details = || Box::new([Target::Role { role: Role::Region, name: b"Task details".as_slice().into() }]);
    let card = || {
        Box::new([
            Target::Role { role: Role::Log, name: b"Conversation".as_slice().into() },
            Target::Role { role: Role::Region, name: b"Task 2".as_slice().into() },
        ])
    };
    Script {
        begin: Begin::Playground,
        steps: playground_opening(conversation).into_iter().chain([
            Step::See {
                find: Find::text("CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change.".as_bytes()).within(conversation()),
                within: DEMO_LANDS_WITHIN,
            },
            // client/view.md, 4: the committed opening turn fits whole.
            Step::Gone { find: Find::exactly_named(Role::Paragraph, b"delegate").within(conversation()), within: START_WITHIN },
            // client/view.md, 5: the delegated change's card says it landed.
            Step::See { find: Find::text("T2 · CHANGE".as_bytes()).within(card()), within: START_WITHIN },
            Step::See { find: Find::exactly_named(Role::Status, b"Landed").within(card()), within: START_WITHIN },
            Step::See { find: Find::text("CI passed. Landed · merge commit ".as_bytes()).within(card()), within: START_WITHIN },
            Step::Gone { find: Find::exactly_named(Role::Status, b"Applied"), within: START_WITHIN },
            // The owner's chat log holds the result's words alone, not its ending.
            Step::Go { address: Address::Task { task: CHANGE } },
            Step::See {
                find: Find::exactly_named(Role::Heading { level: 1 }, b"Export widget's table as CSV").within(details()),
                within: START_WITHIN,
            },
            // Completed, not held: a change that fails to land is held; the clone check reads the merge.
            // client/pages/goal.md, 2: the header words an ended task by its owner's ending.
            Step::See { find: Find::exactly_named(Role::Status, b"Completed").within(details()), within: START_WITHIN },
        ]).collect(),
    }
}

fn playground_opening(conversation: impl Fn() -> Box<[Target; 1]>) -> [Step<Address>; 14] {
    use skein_ui::Role;
    [
        Step::Go { address: Address::Welcome { step: temper_client_domain::Step::Welcome } },
        Step::See { find: Find::exactly_named(Role::Button, b"Try the playground"), within: START_WITHIN },
        Step::Press { find: Find::exactly_named(Role::Button, b"Try the playground") },
        Step::Go { address: Address::Chats { filter: ChatsFilter::All } },
        Step::See { find: Find::exactly_named(Role::Status, b"Starting"), within: START_WITHIN },
        Step::See { find: Find::exactly_named(Role::Status, b"Ready"), within: START_WITHIN },
        Step::Go { address: Address::NewChat { draft: false } },
        Step::Press { find: Find::exactly_named(Role::Button, b"Scripted model") },
        Step::Go { address: Address::Chats { filter: ChatsFilter::All } },
        // client/forge/family.md, 7: a suggested request starts and opens its chat.
        Step::See { find: Find::named(Role::Button, b"Add CSV export to widget"), within: START_WITHIN },
        Step::Press { find: Find::named(Role::Button, b"Add CSV export to widget") },
        Step::See { find: Find::exactly_named(Role::Paragraph, b"You").within(conversation()), within: START_WITHIN },
        Step::See { find: Find::exactly_named(Role::Paragraph, b"Add CSV export to widget").within(conversation()), within: START_WITHIN },
        Step::See {
            find: Find::text("I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge.".as_bytes()).within(conversation()),
            within: START_WITHIN,
        },
    ]
}
/// Signs in to the first run and sees the welcome without starting the playground.
#[must_use]
pub fn welcome_open() -> Script {
    Script {
        begin: Begin::Playground,
        steps: Box::new([
            Step::Go { address: Address::Welcome { step: temper_client_domain::Step::Welcome } },
            Step::See {
                find: Find::exactly_named(skein_ui::Role::Button, b"Try the playground"),
                within: START_WITHIN,
            },
        ]),
    }
}
/// Starts the first run's playground and sees Ready and its chat's first answer.
#[must_use]
pub fn demo_start_answered() -> Script {
    let conversation =
        || Box::new([Target::Role { role: skein_ui::Role::Log, name: b"Conversation".as_slice().into() }]);
    Script { begin: Begin::Playground, steps: Box::new(playground_opening(conversation)) }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectSelection {
    Pending,
    Selected,
}
/// Stdout and the client share one host; the link comes only from the binary.
pub struct Person {
    serve_data: Option<std::path::PathBuf>,
    pub client: Option<NativeClient>,
    root: Option<Fd>,
    actor: skein_fake_person::Person<Address>,
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    closing: u32,
    bytes: Vec<u8>,
    wake: Option<Time>,
    signing_until: Option<Time>,
    signing_retry: Option<Time>,
    signing_backoff: skein_lib::Duration,
    pub finished: bool,
    project: ProjectSelection,
    pub failed: Option<String>,
    adopting: bool,
    begin: Begin,
    key_send: Option<u32>,
    directory: Box<[u8]>,
    seed: u64,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl Person {
    /// Start the native sign-in's bound once the box's serve has listened.
    #[must_use]
    pub fn with_serve_data(mut self, data: &std::path::Path) -> Self {
        self.serve_data = Some(data.into());
        self
    }
    /// Every byte read from the binary's stdout, including the printed link.
    #[must_use]
    pub fn stdout(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn with_link(root: Fd, input: Fd, output: Fd, script: Script, link: &[u8]) -> Self {
        let mut person = Self::new(root, input, output, script);
        person.bytes.extend_from_slice(link);
        person.bytes.push(b'\n');
        person
    }
    #[must_use]
    pub fn new(root: Fd, input: Fd, output: Fd, script: Script) -> Self {
        Self::build(root, Some(input), Some(output), script)
    }
    /// Wait for ctl to finish a sign-in in its own directory; load it through the native shell.
    #[must_use]
    pub fn waiting_for_ctl(root: Fd, script: Script) -> Self {
        let mut person = Self::build(root, None, None, Script { begin: Begin::Token, ..script });
        person.directory = b"client".as_slice().into();
        person
    }
    /// Keep the script's key-entry steps, then load ctl's completed sign-in.
    #[must_use]
    pub fn waiting_for_ctl_with_key(root: Fd, script: Script) -> Self {
        let mut person = Self::build(root, None, None, script);
        person.begin = Begin::Token;
        person
    }
    /// Hold the native observer until ctl supplies its identity and the expected page words.
    #[must_use]
    pub fn waiting_for_observation(root: Fd) -> Self {
        let mut person = Self::waiting_for_ctl(root, Script { begin: Begin::Token, steps: Box::new([]) });
        // Accounts belongs to the person and needs no project selection.
        person.project = ProjectSelection::Selected;
        person
    }
    pub fn observe(&mut self, link: &[u8], script: Script) {
        assert!(self.client.is_none(), "observation starts once");
        self.actor = skein_fake_person::Person::new(script.steps);
        self.ctl_ready(link);
    }
    #[must_use]
    pub fn steps_seen(&self) -> u32 {
        self.actor.at()
    }
    pub fn ctl_ready(&mut self, link: &[u8]) {
        assert!(self.bytes.is_empty(), "one handover from ctl");
        self.bytes.extend_from_slice(link);
        self.bytes.push(b'\n');
    }
    /// The welcome has answered and the playground button has not yet been pressed.
    #[must_use]
    pub fn choosing_playground(&self) -> bool {
        self.begin == Begin::Playground && self.actor.at() == 2
    }
    fn build(root: Fd, input: Option<Fd>, output: Option<Fd>, script: Script) -> Self {
        let steps = if matches!(script.begin, Begin::LinkWithKey | Begin::LinkWithOpenAiKey) {
            use skein_ui::Role;
            let mut key_way = Find::exactly_named(Role::Link, b"Add key");
            if script.begin == Begin::LinkWithOpenAiKey {
                key_way.within = Box::new([skein_fake_person::Target::ExactRole {
                    role: Role::Slot,
                    name: b"OpenAI API key".as_slice().into(),
                }]);
            }
            let mut setup = vec![
                Step::Go { address: Address::ConnectModel { way: None } },
                Step::See {
                    find: Find::exactly_named(Role::Heading { level: 1 }, b"Connect a model"),
                    within: START_WITHIN,
                },
                Step::See { find: Find::named(Role::Link, b"Use deployment accounts"), within: START_WITHIN },
                Step::Press { find: Find::named(Role::Link, b"Use deployment accounts") },
                Step::Press { find: key_way },
                Step::Type {
                    find: Find::named(Role::TextField, b"Key (pasted once)"),
                    words: jig_channel_kit::fixtures::KEY.into(),
                },
                Step::Press { find: Find::named(Role::Button, b"Add key") },
                Step::See { find: Find::text(b"Added your"), within: START_WITHIN },
            ];
            setup.extend(script.steps);
            setup.into_boxed_slice()
        } else {
            script.steps
        };
        Self {
            serve_data: None,
            client: None,
            root: Some(root),
            actor: skein_fake_person::Person::new(steps),
            input,
            output,
            reading: false,
            closing: 0,
            bytes: Vec::new(),
            wake: None,
            signing_until: None,
            signing_retry: None,
            signing_backoff: SIGN_IN_FIRST,
            finished: false,
            project: if matches!(script.begin, Begin::Link | Begin::Playground | Begin::RunningPlayground) {
                ProjectSelection::Selected
            } else {
                ProjectSelection::Pending
            },
            failed: None,
            adopting: false,
            begin: script.begin,
            key_send: if matches!(script.begin, Begin::LinkWithKey | Begin::LinkWithOpenAiKey) {
                Some(6)
            } else {
                None
            },
            directory: b"private".as_slice().into(),
            // Fresh demo clients must not reuse the same request names when
            // signing the same owner in again on the same deployment.
            seed: if matches!(script.begin, Begin::Link | Begin::Playground | Begin::RunningPlayground) {
                let seed = skein_shell::crypto_seed().expect("demo client's fresh seed");
                u64::from_le_bytes(seed.expose()[..8].try_into().expect("eight seed bytes"))
            } else {
                7
            },
            completions: Queue::with_capacity(512),
            submissions: Queue::with_capacity(512),
        }
    }
    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submissions.push(Submit { op: CLOSE, kind: Op::Close { fd } });
    }
    fn completed(&mut self, complete: Complete) {
        if complete.op == CLOSE {
            self.closing -= 1;
            complete.result.expect("pipe closed");
        } else if complete.op == PIPE {
            self.reading = false;
            let Op::PipeRead { buf, .. } = complete.kind else {
                panic!("stdout read");
            };
            let Done::Count(count) = complete.result.expect("stdout read completes") else {
                panic!("read count");
            };
            self.bytes.extend_from_slice(&buf[..usize::try_from(count).expect("bounded read")]);
            assert!(self.bytes.len() <= 65_536, "bounded printed links");
            if count == 0 {
                let fd = self.output.take().expect("stdout");
                self.close(fd);
            }
        } else {
            self.client.as_mut().expect("native client").completions().push(complete);
        }
    }
    fn start(&mut self, now: Time) {
        if self.client.is_none()
            && let Some(link) = super::printed_link(&self.bytes)
        {
            if self.serve_data.as_deref().is_some_and(|data| !super::boxed::serve_listening(data)) {
                self.wake = Some(now.saturating_add(skein_lib::Duration::from_millis(100)));
                return;
            }
            self.wake = None;
            let address = std::str::from_utf8(link)
                .expect("printed URL")
                .strip_prefix("http://")
                .expect("loopback URL")
                .split('/')
                .next()
                .expect("URL address")
                .parse()
                .expect("socket address");
            self.signing_until = Some(now.saturating_add(START_WITHIN));
            self.client = Some(
                NativeClient::new(
                    Settings {
                        engine: skein_io::Addr::Ip(address),
                        directory: self.directory.clone(),
                        limits: {
                            let mut limits = temper_client_shell_terminal::limits::native();
                            // One standing inbox open beside the eight native logical exchanges.
                            limits.client.client.exchanges = 9;
                            limits
                        },
                    },
                    self.root.take().expect("client root"),
                    skein_shell::effective_user(),
                    self.seed,
                    if self.begin == Begin::Token { Start::Token } else { Start::Link(link.into()) },
                )
                .expect("printed sign-in link"),
            );
        }
    }
    fn notices(&mut self) {
        let client = self.client.as_mut().expect("native client");
        if self.adopting
            && let Some(refusal) = adoption_refusal(client.tree())
        {
            self.failed = Some(format!("native adoption refused: {refusal}"));
        }
        while let Some(notice) = client.notices() {
            if let temper_client_shell_terminal::client::Shown::Notice(
                temper_client_shell_terminal::route::Notice::Leave { .. },
            ) = notice
            {
                continue;
            }
            assert!(
                !matches!(
                    notice,
                    temper_client_shell_terminal::client::Shown::Notice(
                        temper_client_shell_terminal::route::Notice::Fact(_)
                    )
                ),
                "native person step {} protocol fact: {notice:?}",
                self.actor.at()
            );
            assert!(
                !matches!(
                    notice,
                    temper_client_shell_terminal::client::Shown::Terminal(
                        temper_client_shell_terminal::terminal::Notice::DirectoryRefused
                    )
                ),
                "private client directory refused"
            );
        }
    }
    fn drive(&mut self, now: Time, wall: Wall) {
        let Some(client) = &mut self.client else {
            return;
        };
        client.iterate(now, wall);
        while let Some(signing) = client.signing() {
            match signing {
                Signing::SignedIn { .. } => {
                    self.signing_retry = None;
                    self.signing_until = None;
                }
                Signing::Again | Signing::Busy => {
                    let until = self.signing_until.expect("sign-in deadline");
                    assert!(now < until, "native signing failed: {signing:?}");
                    self.signing_retry = Some(now.saturating_add(self.signing_backoff).min(until));
                    self.signing_backoff = self.signing_backoff.saturating_mul(2).min(SIGN_IN_MOST);
                }
                other @ (Signing::LinkUsed
                | Signing::LinkExpired
                | Signing::Approval(_)
                | Signing::LinkUnknown
                | Signing::Refused
                | Signing::Lost
                | Signing::SignedOut
                | Signing::Full { .. }) => {
                    panic!("native signing failed: {other:?}");
                }
            }
        }
        self.notices();
        let client = self.client.as_mut().expect("native client");
        self.wake = None;
        if self.signing_backoff != SIGN_IN_FIRST
            && let Some(until) = self.signing_until
        {
            assert!(now < until, "native signing failed: Again");
            self.wake = Some(until);
        }
        if let Some(retry) = self.signing_retry {
            if now >= retry {
                self.signing_retry = None;
                // Start::Token installs a held credential locally; only link sign-ins have replies.
                client
                    .sign_in(super::printed_link(&self.bytes).expect("printed sign-in link"))
                    .expect("printed sign-in link");
            } else {
                self.wake = Some(self.wake.map_or(retry, |at| at.min(retry)));
            }
        }
        if client.started() && self.project == ProjectSelection::Pending && select_project(client) {
            self.project = ProjectSelection::Selected;
            self.wake = Some(now);
        }
        let signing_wake = self.wake;
        // The goal fact's Watch and Read finish before the setup carries its key.
        let key_waits = self.key_send == Some(self.actor.at()) && {
            let pending = client.pending();
            pending.shell.active > 1
                || pending.shell.queued != 0
                || pending.shell.held != 0
                || pending.admission.attempts != 0
                || pending.admission.deferred_reads != 0
        };
        if client.started()
            && self.project == ProjectSelection::Selected
            && !self.finished
            && self.failed.is_none()
            // Finish opening exchanges before a scripted one-shot account paste.
            && (client.pending().admission.attempts == 0
                || !client.tree().nodes().iter().any(|node| {
                    node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Add key")
                }))
            && !key_waits
        {
            let before = self.actor.at();
            match self.actor.next(&mut ReplyFace::new(client.tree()), now) {
                Next::Do(Doing::Act { node, act }) => {
                    self.adopting |= client.tree().nodes().iter().any(|item| {
                        item.id == node && item.role == skein_ui::Role::Button && item.name.as_deref() == Some(b"Adopt")
                    });
                    assert!(client.act(node, act), "offered person act");
                }
                Next::Do(Doing::Go { address }) => {
                    self.adopting = false;
                    client.go(address);
                }
                Next::Do(Doing::Reload) => panic!("chat script has no reload"),
                Next::Wait { until } => self.wake = until,
                Next::Done => {
                    self.finished = true;
                    client.close();
                }
                Next::Failed { step, why } => {
                    self.failed = Some(person_failure(step, &why, client));
                }
            }
            if self.actor.at() != before {
                self.wake = Some(now);
            }
        }
        self.wake = [self.wake, signing_wake].into_iter().flatten().min();
        while let Some(submit) = client.submissions().pop() {
            self.submissions.push(submit);
        }
    }
}

fn person_failure(step: u32, why: &skein_fake_person::Why, client: &NativeClient) -> String {
    let nodes: Vec<_> = client
        .tree()
        .nodes()
        .iter()
        .map(|node| (node.role, node.name.as_deref().map(String::from_utf8_lossy), &node.states))
        .collect();
    format!("native person step {step}: {why:?}; client: {:?}; tree: {nodes:?}", client.pending())
}

/// A key's acknowledgement can precede the model watch that enables Send.
pub fn a_chat_waits_for_the_model_watch_before_pressing_start() {
    let mut actor = skein_fake_person::Person::new(answered_chat(b"Hello", b"answer").steps);
    let waiting = chat_start_view(false);
    let mut face = ReplyFace::new(waiting.tree());
    assert!(matches!(actor.next(&mut face, Time::ZERO), Next::Do(Doing::Go { .. })));
    assert_eq!(actor.next(&mut face, Time::ZERO), Next::Wait { until: None });
    assert!(matches!(actor.next(&mut face, Time::ZERO), Next::Do(Doing::Act { act: skein_ui::Act::Input { .. }, .. })));
    assert!(matches!(actor.next(&mut face, Time::ZERO), Next::Wait { .. }), "Send waits for the model watch");
    assert_eq!(actor.at(), 3, "the delayed watch keeps the script before Send");
    let ready = chat_start_view(true);
    let mut face = ReplyFace::new(ready.tree());
    assert_eq!(actor.next(&mut face, Time::ZERO), Next::Wait { until: None });
    assert!(matches!(actor.next(&mut face, Time::ZERO), Next::Do(Doing::Act { act: skein_ui::Act::Press, .. })));
}

fn chat_start_view(usable: bool) -> skein_ui::View {
    use skein_ui::{Role, Run, State};
    let (limits, _) = temper_client_shell_terminal::limits::view(&temper_client_shell_terminal::limits::native())
        .expect("native view bounds");
    let mut view = skein_ui::View::new(&limits.tree);
    let mut builder = view.builder();
    builder.open(Role::Page);
    builder.open(Role::TextArea);
    builder.name(b"What do you want done?");
    builder.bind(skein_ui::Binding { owner: skein_ui::Owner { number: 1 }, token: Token::new(1) });
    builder.close();
    builder.open(Role::Button);
    builder.name(b"Send");
    builder.bind(skein_ui::Binding { owner: skein_ui::Owner { number: 1 }, token: Token::new(2) });
    if !usable {
        builder.state(State::Disabled);
    }
    builder.close();
    if !usable {
        builder.open(Role::Paragraph);
        builder.text(Run::Plain, b"No model yet \xc2\xb7 ");
        builder.close();
    }
    builder.close();
    let mut patches = Queue::with_capacity(limits.tree.patches);
    view.finish(builder, 1, &mut patches);
    view
}
fn adoption_refusal(tree: &skein_ui::Tree) -> Option<String> {
    let nodes = tree.nodes();
    let (at, alert) = nodes.iter().enumerate().find(|(_, node)| node.role == skein_ui::Role::Alert)?;
    let end = at.checked_add(usize::try_from(alert.size).expect("bounded alert subtree")).expect("alert end");
    let mut reason = String::new();
    for node in &nodes[at..end] {
        for run in &node.text {
            reason.push_str(&String::from_utf8_lossy(&run.text));
        }
    }
    Some(reason)
}

pub fn a_native_adoption_refusal_names_the_alert_without_a_wait() {
    let (view, _) = temper_client_shell_terminal::limits::view(&temper_client_shell_terminal::limits::native())
        .expect("native view bounds");
    let mut builder = skein_ui::Builder::new(&view.tree);
    builder.open(skein_ui::Role::Page);
    builder.open(skein_ui::Role::Alert);
    builder.text(skein_ui::Run::Plain, b"Forge 1 did not answer; nothing was adopted.");
    builder.close();
    builder.close();
    assert_eq!(adoption_refusal(&builder.finish()).as_deref(), Some("Forge 1 did not answer; nothing was adopted."));
}
/// The switcher's button for a project `me` leaves unnamed, "Project N"
/// (client/view.md, 3).
fn an_unnamed_project(name: Option<&[u8]>) -> bool {
    let Some(number) = name.and_then(|name| name.strip_prefix(b"Project ".as_slice())) else { return false };
    !number.is_empty() && number.iter().all(u8::is_ascii_digit)
}
// Project zero is absent from me; the sole configured project may already be selected.
fn select_project(client: &mut NativeClient) -> bool {
    let node = client
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == skein_ui::Role::Button && an_unnamed_project(node.name.as_deref()))
        .map(|node| node.id);
    let Some(node) = node else { return false };
    assert!(client.act(node, skein_ui::Act::Press), "offered project selection");
    true
}
impl Host for Person {
    fn iterate(&mut self, now: Time, wall: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.completed(complete);
        }
        if let Some(fd) = self.input.take() {
            self.close(fd);
        }
        self.start(now);
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submissions
                .push(Submit { op: PIPE, kind: Op::PipeRead { fd, buf: vec![0; 1024].into_boxed_slice() } });
        }
        self.drive(now, wall);
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.input.is_some()
            || (!self.reading && self.output.is_some())
            || self.wake.is_some_and(|at| at <= now)
            || self.client.as_ref().is_some_and(|client| client.work_pending(now))
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_mechanism_deadline), self.wake].into_iter().flatten().min()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        [self.client.as_ref().and_then(Host::next_policy_deadline), self.wake].into_iter().flatten().min()
    }
    fn is_empty(&self) -> bool {
        self.output.is_none() && self.closing == 0 && self.client.as_ref().is_some_and(Host::is_empty)
    }
    fn operations(&self) -> u32 {
        512
    }
}

#[must_use]
pub fn adopt_and_change(_forgejo: &super::forgejo::Throwaway) -> Script {
    use skein_ui::Role;
    let mut steps = answered_chat(b"Hello", super::ANSWER).steps.into_vec();
    steps.extend([
        Step::Go { address: Address::Forges },
        // The Account slot of this row is rendered only for Connected.
        Step::See {
            find: Find::named(Role::Description, b"temper \xc2\xb7 user ").within(Box::new([Target::ExactRole {
                role: Role::Disclosure,
                name: b"Forgejo on this machine".as_slice().into(),
            }])),
            within: super::WITHIN,
        },
        Step::Go {
            address: Address::Inbox {
                filter: temper_client_domain::address::Filter {
                    project: Some(1),
                    ..temper_client_domain::address::Filter::ALL
                },
            },
        },
        Step::See { find: Find::exactly_named(Role::Link, b"How work runs"), within: super::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Link, b"How work runs") },
        Step::See { find: Find::exactly_named(Role::Link, b"Repositories"), within: super::WITHIN },
        Step::Press { find: Find::exactly_named(Role::Link, b"Repositories") },
        Step::See { find: Find::named(Role::Link, b"Add repositories"), within: super::WITHIN },
        Step::Press { find: Find::named(Role::Link, b"Add repositories") },
        Step::See { find: Find::named(Role::TextField, b"Owner"), within: super::WITHIN },
        Step::Type { find: Find::named(Role::TextField, b"Owner"), words: b"acme".as_slice().into() },
        Step::Type { find: Find::exactly_named(Role::TextField, b"Name"), words: b"widget".as_slice().into() },
        Step::Type { find: Find::named(Role::TextField, b"Landing branch"), words: b"main".as_slice().into() },
        Step::Press { find: Find::named(Role::Button, b"Adopt") },
        Step::See { find: Find::text(b"acme/widget"), within: super::WITHIN },
    ]);
    steps.extend([
        Step::Go { address: Address::NewChat { draft: false } },
        Step::See { find: Find::named(Role::TextArea, b"What do you want done?"), within: super::WITHIN },
        Step::Type {
            find: Find::named(Role::TextArea, b"What do you want done?"),
            words: b"@forge_chat Land a small change".as_slice().into(),
        },
        Step::Press { find: Find::named(Role::Button, b"Send") },
        Step::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
        inbox_reply(FORGE_CHAT_REPLY),
    ]);
    Script { steps: steps.into_boxed_slice(), begin: Begin::LinkWithKey }
}
/// Adopt and start the landing chat, ending the sitting before its reply.
#[must_use]
pub fn adopt_and_start_change(forge: &super::forgejo::Throwaway) -> Script {
    let mut script = adopt_and_change(forge);
    // The last two steps visit the inbox and await the landing's reply.
    let mut steps = script.steps.into_vec();
    steps.truncate(steps.len() - 2);
    steps.push(Step::See {
        find: Find::text(temper_real_loop_world::landing::REQUESTED.as_bytes()),
        within: super::WITHIN,
    });
    script.steps = steps.into_boxed_slice();
    script
}

/// A person's live sitting keeps the native stories, allowing real completions their budget.
#[must_use]
pub fn live_adopt_and_change(forge: &super::forgejo::Throwaway) -> Script {
    let mut script = adopt_and_change(forge);
    for step in &mut script.steps {
        if let Step::See { within, .. } | Step::Gone { within, .. } = step {
            *within = skein_lib::Duration::from_secs(1_740);
        }
    }
    script
}

#[must_use]
pub fn found() -> Script {
    Script {
        begin: Begin::Token,
        steps: Box::new([
            Step::Go { address: Address::Inbox { filter: temper_client_domain::address::Filter::ALL } },
            inbox_reply(FORGE_CHAT_REPLY),
        ]),
    }
}
/// The forge chat's reply in the inbox: the chat is task 2, after "Hello", and
/// the view names each inbox entry by its kind and its source task.
const FORGE_CHAT_REPLY: &[u8] = b"Reply from T2";
#[must_use]
pub fn inbox_reply(entry: &[u8]) -> Step<Address> {
    Step::See {
        find: Find::named(skein_ui::Role::Heading { level: 3 }, b"replied")
            .within(Box::new([Target::Role { role: skein_ui::Role::Item, name: entry.into() }])),
        within: super::WITHIN,
    }
}

#[cfg(test)]
pub mod serve_gate_tests {
    use super::*;

    pub fn native_sign_in_keeps_its_start_bound_until_serve_listens() {
        let scratch = super::super::Scratch::new("person-serve-gate");
        let mut person = Person::with_link(
            Fd::new(0),
            Fd::new(1),
            Fd::new(2),
            found(),
            b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=fixture",
        )
        .with_serve_data(scratch.path());
        person.start(Time::ZERO);
        assert!(person.client.is_none(), "an early link starts no native sign-in");
        assert!(person.signing_until.is_none(), "serve startup spends none of START_WITHIN");
        assert!(person.next_policy_deadline().is_some(), "the log is checked again without pipe output");
        std::fs::create_dir_all(scratch.path().join("box/logs")).expect("logs");
        std::fs::write(scratch.path().join("box/logs/serve.log"), b"listening clients 127.0.0.1:8367\n")
            .expect("listening");
        let now = Time::from_nanos(START_WITHIN.as_nanos());
        person.start(now);
        assert!(person.client.is_some(), "listening admits native sign-in");
        assert_eq!(person.signing_until, Some(now.saturating_add(START_WITHIN)));
        scratch.remove();
    }
}
