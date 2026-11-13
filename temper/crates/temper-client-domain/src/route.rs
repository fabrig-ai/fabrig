//! Route the shared client's tells to temper's owning child.
use crate::{Action, Address, Domain, Event, Fact, Limits, Place, Request, forge, owner::Owner};
use skein_lib::{Env, Queue};
pub(crate) fn forge(root: &mut Domain, env: &Env<Limits>, event: forge::Event, out: &mut Queue<Request>) {
    let mut child = Queue::uncapped();
    forge::step(&mut root.forge, &env.with(env.limits.forge), event, &mut child);
    route_forge(root, env, &mut child, out);
    crate::pages::decided::reconcile(root, env, out);
    crate::pages::home::look(root, env, out);
}
pub(crate) fn event(root: &mut Domain, env: &Env<Limits>, event: Event, out: &mut Queue<Request>) {
    if let Event::Act { .. } = &event {
        crate::pages::acted(&mut root.pages);
    }
    match event {
        Event::Push(report) => pages(root, env, crate::pages::Event::Push(report), out),
        Event::AskApproval => crate::asked::ask(root, env, out),
        Event::Launchpad => {
            let mut requests = Queue::uncapped();
            root.pages.enable_launchpad(&mut requests);
            route_pages(root, env, &mut requests, out);
        }
        Event::ScreenLeft => pages(
            root,
            env,
            crate::pages::Event::Act { act: crate::pages::Act::Terminal(crate::pages::terminal::Act::Dismiss) },
            out,
        ),
        Event::LookKept { look } => root.pages.restore_look(look),
        Event::Copied { token, written } => root.copy.copied(env.now, token, written),
        Event::Start { address, kept, with } => {
            if !root.unlinked {
                client(root, env, jig_client::Event::Start { kept }, out);
            }
            // Start restores the client; sign-in must precede the page's one-shot reads.
            if let Some(with) = with {
                client(root, env, jig_client::Event::SignIn { with }, out);
            }
            told(root, env, jig_client::Told::Link { link: root.client.link().state }, out);
            pages(root, env, crate::pages::Event::Started, out);
            went(root, env, Place::Address(address), out);
        }
        Event::Went { place } => went(root, env, place, out),
        Event::Act { action } => match action {
            Action::Approve(act) => crate::approve::act(root, act),
            Action::Copy { words } => root.copy.press(words, out),
            Action::Pages(crate::pages::Act::SignOut) => {
                root.signed_out = true;
                root.authorization_revision = root.authorization_revision.checked_add(1).expect("revision fits");
                pages(root, env, crate::pages::Event::Act { act: crate::pages::Act::SignOut }, out);
            }
            Action::Pages(crate::pages::Act::RetryPlayground) => {
                if root.forge.playground_failure().is_some() {
                    forge(root, env, forge::Event::Act { act: forge::Act::StartPlayground }, out);
                }
            }
            Action::Pages(crate::pages::Act::Playground) => playground(root, env, out),
            Action::Pages(crate::pages::Act::Suggest { cue }) => suggest(root, env, cue, out),
            Action::Pages(crate::pages::Act::HandArea(act)) => crate::pages::team::hand::act(root, env, act, out),
            Action::Pages(crate::pages::Act::StopAsking(act)) => crate::pages::stop_asking::act(root, env, act, out),
            Action::Pages(crate::pages::Act::Away(act)) => crate::pages::away::act(root, env, act, out),
            Action::Pages(act) => pages(root, env, crate::pages::Event::Act { act }, out),
            Action::Forge(act) => forge(root, env, forge::Event::Act { act }, out),
        },
        Event::Client { event } => client(root, env, event, out),
        Event::Forge(event) => forge(root, env, event, out),
        Event::SignInAddress { value } => pages(root, env, crate::pages::Event::SignInAddress { value }, out),
        Event::AccountKey { value } => pages(root, env, crate::pages::Event::AccountKey { value }, out),
        Event::SignInKept { source, address, refused } => {
            pages(root, env, crate::pages::Event::SignInKept { source, address, refused }, out);
        }
        Event::TabRefused => crate::sign_in_tab::refused(root),
        Event::SignedOut => client(root, env, jig_client::Event::Up { up: jig_client::Up::SignedOut }, out),
    }
    // Findings subscriptions follow the event's foreground reads and acts.
    crate::pages::findings::reconcile(root, env, out);
}
fn went(root: &mut Domain, env: &Env<Limits>, place: Place, out: &mut Queue<Request>) {
    crate::authorize::went(root, &place);
    crate::approve::went(root, &place);
    match place {
        Place::Address(address) => {
            pages(root, env, crate::pages::Event::Went { address: address.clone() }, out);
            if let Address::Adopt { project } = address {
                forge(root, env, forge::Event::Went { address: forge::Address::Adopt { project } }, out);
            } else {
                forge(root, env, forge::Event::Left, out);
            }
        }
        Place::Approve { .. }
        | Place::Authorize { .. }
        | Place::AuthorizeMalformed
        | Place::Missing
        | Place::Public(_) => {
            pages(root, env, crate::pages::Event::Left, out);
            forge(root, env, forge::Event::Left, out);
        }
    }
}
pub(crate) fn pages(root: &mut Domain, env: &Env<Limits>, event: crate::pages::Event, out: &mut Queue<Request>) {
    let mut child = Queue::uncapped();
    playground_facts(root);
    root.pages.push = match root.forge.deployment() {
        Some(answer) => answer.push,
        None => None,
    };
    let task_document = match &event {
        crate::pages::Event::Told {
            told: jig_client::Told::Changed { handle } | jig_client::Told::Page { handle, .. },
        } => match root.pages.task() {
            Some(task) => *handle == task.document,
            None => false,
        },
        crate::pages::Event::Push(_)
        | crate::pages::Event::Started
        | crate::pages::Event::Left
        | crate::pages::Event::SignInKept { .. }
        | crate::pages::Event::SignInAddress { .. }
        | crate::pages::Event::AccountKey { .. }
        | crate::pages::Event::Went { .. }
        | crate::pages::Event::Act { .. }
        | crate::pages::Event::Told { .. }
        | crate::pages::Event::Link { .. } => false,
    };
    crate::pages::chat::sheet::step(root, env, event, &mut child);
    route_pages(root, env, &mut child, out);
    let subject = match root.pages.task() {
        Some(task) => task.panel(&root.client, root.forge_number()),
        None => None,
    };
    let watched = match root.forge.panel() {
        Some(panel) => Some(panel.subject),
        None => None,
    };
    if watched != subject {
        forge(root, env, forge::Event::Panel { subject }, out);
    } else if task_document && let Some(subject) = subject {
        refresh_panel(root, env, subject, false, out);
    }
    crate::pages::decided::reconcile(root, env, out);
    crate::pages::home::look(root, env, out);
}

fn refresh_panel(
    root: &mut Domain,
    env: &Env<Limits>,
    subject: forge::panel::Subject,
    from_ending: bool,
    out: &mut Queue<Request>,
) {
    let forge_number = root.forge_number();
    if let Some(page) = root.pages.task_mut()
        && page.panel(&root.client, forge_number) == Some(subject)
        && let Some(task) = page.document(&root.client)
        && task.chip.task == page.number
        && forge::panel::refreshable(task, forge_number)
        && page.panel_reopened_at != Some(task.activity)
    {
        if from_ending {
            // Only a finished Change can recover without a later document; bound
            // missing-record replies to one immediate reopen per document.
            if task.chip.phase != jig_client::contract::Phase::Ended {
                return;
            }
            page.panel_reopened_at = Some(task.activity);
        }
        forge(root, env, forge::Event::RefreshPanel { subject }, out);
    }
}

pub(crate) fn fire_pages(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::uncapped();
    crate::pages::fire(&mut root.pages, env, &root.client, &mut child);
    route_pages(root, env, &mut child, out);
}

fn route_pages(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<crate::pages::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted pages output") {
            crate::pages::Request::Push(duty) => out.push(Request::Push(duty)),
            crate::pages::Request::Screen { on } => out.push(Request::Screen { on }),
            crate::pages::Request::KeepLook { look } => out.push(Request::KeepLook { look }),
            crate::pages::Request::OpenTab { provider } => {
                root.tab = root.pages.sign_in_key();
                out.push(Request::Tab { tab: crate::Tab::Open { provider } });
            }
            crate::pages::Request::Client(event) => client(root, env, event, out),
            crate::pages::Request::Replace { address } => out.push(Request::Show { address, replace: true }),
            crate::pages::Request::Show { address } => out.push(Request::Show { address, replace: false }),
        }
    }
}

pub(crate) fn route_forge(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<forge::Request>,
    out: &mut Queue<Request>,
) {
    for _ in 0..child.len() {
        match child.pop().expect("counted forge output") {
            forge::Request::Repositories { project } => {
                out.push(Request::Show { address: Address::Repositories { project }, replace: false });
            }
            forge::Request::ConnectForge { forge } => {
                let choice = Some(crate::Choice::Forge(forge));
                let address = match root.pages.address() {
                    Address::ConnectForge {
                        forge: Some(crate::Choice::Managed | crate::Choice::Owned | crate::Choice::GitHub),
                    } => Address::ConnectForge { forge: choice },
                    Address::Welcome { step: crate::Step::Forge { forge: Some(crate::Choice::Managed) } } => {
                        Address::Welcome { step: crate::Step::Forge { forge: choice } }
                    }
                    Address::AllHome
                    | Address::Inbox { .. }
                    | Address::Chats { .. }
                    | Address::NewChat { .. }
                    | Address::Told
                    | Address::Remembered
                    | Address::Members
                    | Address::Limits
                    | Address::Accounts
                    | Address::Forges
                    | Address::ConnectForge { .. }
                    | Address::ConnectModel { .. }
                    | Address::Welcome { .. }
                    | Address::Chat { .. }
                    | Address::Talk { .. }
                    | Address::Task { .. }
                    | Address::Home { .. }
                    | Address::Changes { .. }
                    | Address::Team { .. }
                    | Address::Settings { .. }
                    | Address::Models { .. }
                    | Address::Builders { .. }
                    | Address::Role { .. }
                    | Address::Rule { .. }
                    | Address::Repositories { .. }
                    | Address::Work { .. }
                    | Address::Goals { .. }
                    | Address::Ask { .. }
                    | Address::Tell { .. } | Address::Adopt { .. } => continue,
                };
                out.push(Request::Show { address, replace: true });
            }
            forge::Request::PanelEnded { subject } => refresh_panel(root, env, subject, true, out),
            forge::Request::Client { asked } => client(root, env, asked, out),
            forge::Request::Leave { to } => out.push(Request::Leave { to, replace: false }),
            forge::Request::PlaygroundReset => {
                root.pages.playground_choice = crate::pages::PlaygroundChoice::Idle;
                out.push(Request::Show {
                    address: Address::Welcome { step: crate::Step::Forge { forge: None } },
                    replace: false,
                });
            }
            forge::Request::Show { address } => {
                let forge::Address::Adopt { project } = address;
                out.push(Request::Show { address: Address::Adopt { project }, replace: false });
            }
            forge::Request::Send { request, name, ask } => client(
                root,
                env,
                jig_client::Event::Send {
                    key: jig_client::Key { asker: Owner::Forge.asker(), number: request.raw() },
                    name,
                    request: ask,
                },
                out,
            ),
        }
    }
}
pub(crate) fn client(root: &mut Domain, env: &Env<Limits>, event: jig_client::Event, out: &mut Queue<Request>) {
    let mut child = Queue::uncapped();
    if root.unlinked {
        root.client.step_unlinked(env.now, env.wall, event, &mut child);
        route_unlinked(root, env, &mut child, out);
    } else {
        let mut child = Queue::uncapped();
        root.client.step(env.now, env.wall, event, &mut child);
        route_client(root, env, &mut child, out);
    }
}
pub(crate) fn route_client(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client::Request>,
    out: &mut Queue<Request>,
) {
    while let Some(request) = child.pop() {
        match request {
            jig_client::Request::Told { told: value } => told(root, env, value, out),
            jig_client::Request::Approval(answer) => approval(root, env, answer, out),
            jig_client::Request::Authorized(answer) => crate::authorize::answered(root, answer, out),
            request @ (jig_client::Request::Save { .. }
            | jig_client::Request::Erase { .. }
            | jig_client::Request::Exchange { .. }
            | jig_client::Request::OpenStream { .. }
            | jig_client::Request::CloseStream
            | jig_client::Request::ServiceCredential { .. }) => out.push(Request::Client { request }),
        }
    }
    forge_served(root, env, out);
}
pub(crate) fn route_unlinked(
    root: &mut Domain,
    env: &Env<Limits>,
    child: &mut Queue<jig_client::UnlinkedRequest>,
    out: &mut Queue<Request>,
) {
    while let Some(request) = child.pop() {
        match request {
            jig_client::UnlinkedRequest::Told { told: value } => told(root, env, value, out),
            jig_client::UnlinkedRequest::Approval(answer) => approval(root, env, answer, out),
            jig_client::UnlinkedRequest::Authorized(answer) => crate::authorize::answered(root, answer, out),
            request @ (jig_client::UnlinkedRequest::ApprovalAct(_)
            | jig_client::UnlinkedRequest::Down { .. }
            | jig_client::UnlinkedRequest::SignOut) => {
                out.push(Request::Unlinked { request });
            }
        }
    }
    forge_served(root, env, out);
}
pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: jig_client::Told, out: &mut Queue<Request>) {
    tell(root, env, told, out);
    crate::pages::decided::reconcile(root, env, out);
    crate::pages::home::look(root, env, out);
    forge::trials::look(root, env, out);
}
fn tell(root: &mut Domain, env: &Env<Limits>, told: jig_client::Told, out: &mut Queue<Request>) {
    use jig_client::Told;
    crate::sign_in_tab::told(root, &told, out);
    if crate::pages::team::hand::told(root, env, &told, out) {
        return;
    }
    if crate::pages::stop_asking::told(root, env, &told) {
        return;
    }
    let asker = match &told {
        Told::Link { link } => {
            pages(root, env, crate::pages::Event::Link { link: *link }, out);
            forge_served(root, env, out);
            forge_ways(root, env, &told, out);
            forge_playground(root, env, &told, out);
            forge_me(root, env, &told, out);
            return;
        }
        Told::Dropped { count } => {
            out.push(Request::Dropped { count: *count });
            return;
        }
        Told::Kept { key, .. } | Told::Pending { key } | Told::Answered { key, .. } => key.asker,
        Told::Changed { handle } | Told::Text { handle } | Told::Ended { handle, .. } | Told::Page { handle, .. } => {
            handle.asker
        }
    };
    match Owner::of(asker) {
        Some(Owner::Pages) => pages(root, env, crate::pages::Event::Told { told }, out),
        Some(Owner::Forge) => {
            if forge_ways(root, env, &told, out) || forge_playground(root, env, &told, out) {
                return;
            }
            if forge_forges(root, env, &told, out) {
                return;
            }
            if forge_me(root, env, &told, out) {
                return;
            }
            if let Some(event) = forge::told(&root.client, &env.limits, told) {
                forge(root, env, event, out);
            }
        }
        Some(Owner::Runner) | None => out.push(Request::Fact { fact: Fact::UnknownAsker }),
    }
}

fn forge_served(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let served = if root.client.link().not_served.contains(&root.forge_number()) {
        forge::Served::OutOfDate
    } else {
        forge::Served::Yes
    };
    if root.forge.served() != served {
        forge(root, env, forge::Event::Served { served }, out);
    }
}

fn forge_me(root: &mut Domain, env: &Env<Limits>, told: &jig_client::Told, out: &mut Queue<Request>) -> bool {
    let mut child = Queue::uncapped();
    let consumed = forge::me::told(&mut root.forge, &root.client, told, &mut child);
    if consumed {
        forge::ways::configure(&mut root.forge, &mut child);
        forge::forges::configure(&mut root.forge, env.limits.forge, &mut child);
        forge::away::configure(&mut root.forge, env.limits.forge, &mut child);
        forge::dismissed::configure(&mut root.forge, env.limits.forge, &mut child);
        forge::trials::configure(&mut root.forge, env.limits.forge, &mut child);
        forge::approvals::configure(&mut root.forge, env.limits.forge, &mut child);
    }
    route_forge(root, env, &mut child, out);
    consumed
}

fn forge_playground(root: &mut Domain, env: &Env<Limits>, told: &jig_client::Told, out: &mut Queue<Request>) -> bool {
    let mut child = Queue::uncapped();
    let unknown = if let jig_client::Told::Pending { key } = told {
        match root.client.pending(*key) {
            Some(pending) => pending.progress == jig_client::Progress::Unknown,
            None => false,
        }
    } else {
        false
    };
    let value;
    let told = if unknown && let jig_client::Told::Pending { key } = told {
        value = jig_client::Told::Answered { key: *key, outcome: jig_client::Outcome::Unknown };
        &value
    } else {
        told
    };
    let consumed = forge::deployment::told(&mut root.forge, told, env.limits.forge, &mut child)
        || forge::playground::told(&mut root.forge, told, env.limits.forge, &mut child);
    route_forge(root, env, &mut child, out);
    // A playground answer can arrive after the page and account watch. Way resolution
    // must exclude its provider on that same rebuild.
    playground_facts(root);
    consumed
}
fn suggest(root: &mut Domain, env: &Env<Limits>, cue: u8, out: &mut Queue<Request>) {
    if root.forge.playground_starting() || root.forge.playground_failure().is_some() {
        return;
    }
    let Some(playground) = root.forge.playground() else { return };
    if root.pages.project() != playground.project {
        return;
    }
    let Some(words) = playground.cues.get(usize::from(cue)) else { return };
    let words = words.clone();
    let mut child = Queue::uncapped();
    root.pages.suggest(env, &root.client, &words, &mut child);
    while let Some(request) = child.pop() {
        match request {
            crate::pages::Request::Push(duty) => out.push(Request::Push(duty)),
            crate::pages::Request::Screen { on } => out.push(Request::Screen { on }),
            crate::pages::Request::KeepLook { look } => out.push(Request::KeepLook { look }),
            crate::pages::Request::OpenTab { .. } => unreachable!("a suggestion asks no sign-in"),
            crate::pages::Request::Client(event) => client(root, env, event, out),
            crate::pages::Request::Replace { address } => out.push(Request::Show { address, replace: true }),
            crate::pages::Request::Show { address } => out.push(Request::Show { address, replace: false }),
        }
    }
}

fn playground(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if crate::pages::welcome::playground_offered(root) != Some(true) {
        return;
    }
    root.pages.playground_choice = crate::pages::PlaygroundChoice::Waiting;
    forge(root, env, forge::Event::Act { act: forge::Act::StartPlayground }, out);
    playground_facts(root);
    out.push(Request::Show { address: Address::NewChat { draft: false }, replace: false });
}

fn forge_forges(root: &mut Domain, env: &Env<Limits>, told: &jig_client::Told, out: &mut Queue<Request>) -> bool {
    let mut child = Queue::uncapped();
    let consumed = forge::sheet::told(&mut root.forge, &root.client, told, env.limits.forge, &mut child)
        || forge::away::told(&mut root.forge, &root.client, told, env.limits.forge, &mut child)
        || forge::dismissed::told(&mut root.forge, &root.client, told, env.limits.forge, &mut child)
        || forge::admin::told(&mut root.forge, &root.client, told, env.limits.forge, &mut child)
        || forge::github::told(&mut root.forge, &root.client, told, env.limits.forge, &mut child)
        || forge::forges::told(&mut root.forge, &root.client, told, env.limits.forge, &mut child);
    if consumed
        && let jig_client::Told::Answered { .. } = told
        && let Some(last) = root.forge.set_up()
        && last.answer == Some(forge::forges::SetUpAnswer::Outcome(forge::forges::SetUpOutcome::SettingUp))
        && let Address::Welcome { step: crate::Step::Forge { .. } } = root.pages.address()
    {
        out.push(Request::Show {
            address: Address::Welcome { step: crate::Step::Model { way: None } },
            replace: false,
        });
    }
    forge::sheet::progress(&mut root.forge, env.limits.forge, &mut child);
    route_forge(root, env, &mut child, out);
    if consumed
        && let jig_client::Told::Changed { handle } | jig_client::Told::Ended { handle, .. } = told
        && *handle == forge::dismissed::handle()
    {
        pages(root, env, crate::pages::Event::Told { told: told.clone() }, out);
    }
    consumed
}

#[expect(clippy::manual_map, reason = "step code reads the first repository without closures")]
fn playground_facts(root: &mut Domain) {
    root.pages.welcome_first = match forge::listed::unadopted_first(root.forges()) {
        Some((forge, org, name)) => Some(crate::pages::welcome::First { forge, org: org.into(), name: name.into() }),
        None => None,
    };
    let playground = match root.forge.playground() {
        Some(playground) => Some(playground.forge),
        None => None,
    };
    root.pages.welcome_copies = forge::making::copies(root.forges(), playground);
    root.pages.welcome_decision = crate::pages::welcome::decision(root);
    root.pages.playground_provider = root.forge.playground_provider();
    crate::pages::project_changed(&mut root.pages, &root.client);
    let project = match root.forge.playground() {
        Some(playground) => playground.project,
        None => None,
    };
    crate::pages::playground_project(&mut root.pages, &root.client, project);
    root.pages.playground_model = match root.forge.playground() {
        Some(playground) => playground.state == forge::PlaygroundState::Running,
        None => false,
    };
}

/// Reconcile playground lifecycle after the shared client's account and forge tells.
pub(crate) fn playground_progress(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::uncapped();
    forge::model::progress(&mut root.forge, &root.client, &mut child);
    route_forge(root, env, &mut child, out);
    playground_facts(root);
}

/// Advance Settings to its row and a named welcome forge to Model (temper/docs/design/client/pages/README.md).
pub(crate) fn welcome_progress(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    crate::pages::forge_progress(root, env, out);
    if let Address::Welcome { step: crate::Step::Forge { forge: Some(crate::Choice::Forge(number)) } } =
        root.pages.address()
        && let forge::forges::State::Rows(rows) = root.forges()
    {
        for row in rows {
            if row.number == *number && crate::pages::welcome::forge_connected(row) {
                out.push(Request::Show {
                    address: Address::Welcome { step: crate::Step::Model { way: None } },
                    replace: true,
                });
                return;
            }
        }
    }
}

/// Reconcile the session's reads with the open project and How work runs tab.
pub(crate) fn ways_progress(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut child = Queue::uncapped();
    let on_tab = match root.pages.address() {
        Address::Settings { .. } => true,
        Address::Work { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Home { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::Tell { .. } | Address::Adopt { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Ask { .. }
        | Address::Welcome { .. } => false,
    };
    let shown = if let Address::Inbox { .. } = root.pages.address() {
        match root.pages.inbox() {
            Some(inbox) => match root.pages.needs().shown(inbox, root.forge.approvals()) {
                Some(crate::pages::needs::Need::Card(card)) => card.project,
                Some(crate::pages::needs::Need::Approval(row)) => Some(row.project),
                None => None,
            },
            None => None,
        }
    } else {
        None
    };
    forge::ways::progress(&mut root.forge, env.now, root.pages.project(), shown, on_tab, env.limits.forge, &mut child);
    forge::ways::proposals(&mut root.forge, root.pages.cards(), env.limits.forge, &mut child);
    route_forge(root, env, &mut child, out);
}
fn forge_ways(root: &mut Domain, env: &Env<Limits>, told: &jig_client::Told, out: &mut Queue<Request>) -> bool {
    let mut child = Queue::uncapped();
    let consumed = forge::ways::told(&mut root.forge, env.now, &root.client, told, env.limits.forge, &mut child);
    crate::pages::stop_asking::applied(root, env, told);
    route_forge(root, env, &mut child, out);
    if consumed
        && let jig_client::Told::Changed { handle } | jig_client::Told::Ended { handle, .. } = told
        && *handle == root.forge.settings_handle()
    {
        pages(root, env, crate::pages::Event::Told { told: told.clone() }, out);
    }
    consumed
}

/// Settle the host check and advance a held sign-in or added key once usable (client/pages/README.md).
pub(crate) fn welcome_model_progress(root: &mut Domain, out: &mut Queue<Request>) {
    use crate::pages::{accounts::SignInState, connect_model::ProgramCheck};
    if let Address::ConnectModel { .. } = root.pages.address() {
        return;
    }
    let usable = crate::pages::welcome::model_usable(root) == Some(true);
    let address = root.pages.address().clone();
    let Some(page) = &mut root.pages.connect_model else { return };
    if let Address::Welcome { step: crate::Step::Model { way: Some(way) } } = &address
        && way.kind == crate::WayKind::SignIn
    {
        let held = match page.request.sign_in_state(&root.client) {
            SignInState::Signing { .. } | SignInState::Ended { .. } => true,
            SignInState::Unknown | SignInState::Started { .. } => false,
        };
        if usable && held {
            out.push(Request::Show { address: Address::Welcome { step: crate::Step::Ready }, replace: true });
        }
        return;
    }
    if let Address::Welcome { step: crate::Step::Model { way: Some(way) } } = address
        && way.kind == crate::WayKind::Key
        && let Some(pending) = &page.request.pending
        && let jig_client::contract::Ask::AccountChange {
            change: jig_client::contract::AccountChange::AddKey { .. },
            ..
        } = &pending.ask
        && usable
    {
        if let Some(outcome) = &pending.outcome
            && !crate::pages::seen_answered(outcome)
        {
            return;
        }
        page.request.pending = None;
        out.push(Request::Show { address: Address::Welcome { step: crate::Step::Ready }, replace: true });
    }
    if page.check == ProgramCheck::Idle {
        return;
    }
    if usable {
        page.check = ProgramCheck::Idle;
        return;
    }
    if let ProgramCheck::Checking { source } = page.check
        && let Some(pending) = &page.request.pending
        && pending.outcome.is_some()
        && let Some(accounts) = crate::pages::accounts::document(&root.client)
        && accounts.settled
        && !accounts.cut
        && program_settled(accounts, source)
    {
        page.check = ProgramCheck::NotYet;
    }
}

/// A connected host's pending login observation keeps the check out (temper/docs/design/domain/trial.md, 6.1).
fn program_settled(accounts: &jig_client::contract::Accounts, source: u32) -> bool {
    if accounts.program_hosts.is_empty() {
        return true;
    }
    for account in &accounts.accounts {
        for row in &account.sources {
            if row.number != source {
                continue;
            }
            let Some(name) = &row.name else { return false };
            for host in &accounts.program_hosts {
                if host.connected && host.awaiting_logins {
                    return false;
                }
                for login in &host.programs {
                    if host.connected && login.name == *name && login.state.is_none() {
                        return false;
                    }
                }
            }
            return true;
        }
    }
    false
}

fn approval(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval, out: &mut Queue<Request>) {
    match answer {
        answer @ (jig_client::Approval::Asked { .. } | jig_client::Approval::NotAsked(_)) => {
            crate::asked::answered(root, env, answer, out);
        }
        answer @ (jig_client::Approval::Read { .. } | jig_client::Approval::Decided { .. }) => {
            crate::approve::answered(root, env, answer);
        }
    }
}
