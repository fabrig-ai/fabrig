//! Temper's frame, pages, bindings and checked phrasebook.
use crate::{Limits, forge, words::Words};
use alloc::boxed::Box;
use skein_lib::{Queue, Wall};
use skein_ui::{Act, Builder, NodeId, Patch, Role, Tree};
use temper_client_domain::{Action, Domain};
/// The composed semantic tree and jig's bindings for its last build.
#[derive(Debug)]
pub struct View {
    render: skein_ui::View,
    sign_in: crate::sign_in::Ways,
    forge_sign_in: crate::forge_sign_in::Forges,
    forge_words: Words,
    forge: forge::bindings::Bindings,
    pages: crate::frame::Bindings,
    accounts: crate::accounts::Bindings,
    limits: Limits,
    rows: u32,
    text: u32,
    next_change: Option<Wall>,
    clock_revision: u64,
    revision: Option<u64>,
}
impl View {
    #[must_use]
    pub fn new(limits: &Limits, domain: &temper_client_domain::Limits, forge_words: Words, forge: u16) -> View {
        Self::with_sign_in(limits, domain, forge_words, forge, crate::sign_in::Ways::NONE)
    }
    /// Construct the browser view with the ways advertised by its page.
    #[must_use]
    pub fn with_sign_in(
        limits: &Limits,
        domain: &temper_client_domain::Limits,
        forge_words: Words,
        forge: u16,
        sign_in: crate::sign_in::Ways,
    ) -> View {
        assert!(forge_words.connector == forge, "phrasebook belongs to its forge connector");
        assert!(crate::valid(limits, domain, &forge_words), "view capacities are checked");
        View {
            sign_in,
            forge_sign_in: crate::forge_sign_in::Forges::new(),
            render: skein_ui::View::new(&limits.tree),
            forge: forge::bindings::Bindings::new(limits.tree.bindings, domain.forge.checks, forge),
            pages: crate::frame::Bindings::new(limits.tree.bindings),
            accounts: crate::accounts::Bindings::new(
                domain.client.domain.accounts,
                domain.row_uses,
                domain.client.domain.rows,
            ),
            forge_words,
            limits: *limits,
            rows: domain.client.domain.rows,
            text: domain.client.domain.text,
            next_change: None,
            clock_revision: 0,
            revision: None,
        }
    }
    /// Revision of the last completed view build; absent before its first build.
    #[must_use]
    pub const fn revision(&self) -> Option<u64> {
        self.revision
    }
    /// Supply the page's forge facts before its first build.
    pub fn forge_sign_in(&mut self, forges: crate::forge_sign_in::Forges) {
        self.forge_sign_in = forges;
    }
    fn task_main(&mut self, b: &mut Builder, root: &Domain, times: &mut crate::words::Times) {
        crate::pages::task::main(b, root, &self.forge_words, &mut self.pages, self.rows, times, &self.forge_sign_in);
    }
    /// Earliest change among the time words drawn by the last build.
    #[must_use]
    pub const fn next_change(&self) -> Option<Wall> {
        self.next_change
    }
    #[must_use]
    pub fn tree(&self) -> &Tree {
        self.render.tree()
    }
    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }
}
#[expect(clippy::too_many_lines, reason = "one ordered dispatch draws each address's page")]
pub fn build(view: &mut View, root: &Domain, wall: Wall, out: &mut Queue<Patch>) {
    let shown = temper_client_domain::shown(root).checked_add(view.accounts.revision()).expect("view revisions fit");
    let shown = shown.checked_add(view.clock_revision).expect("view clock revisions fit");
    let due = match view.next_change {
        Some(next) => wall >= next,
        None => false,
    };
    if !view.render.stale(shown) && !due {
        return;
    }
    // skein-ui also gates finish by its revision: a clock wake advances it.
    let shown = if due {
        view.clock_revision = view.clock_revision.checked_add(1).expect("view clock revisions fit");
        shown.checked_add(1).expect("view revisions fit")
    } else {
        shown
    };
    view.forge_words.set_zone_minutes(root.zone_minutes());
    let mut times = crate::words::Times::new(wall);
    view.forge.begin();
    view.pages.begin();
    if root.pages().remembered().is_none() {
        view.pages.remembered_field = None;
    }
    view.pages.model_times = crate::words::Times::new(wall);
    view.accounts.begin(root.pages().accounts().is_none());
    let mut builder = view.render.builder();
    builder.open(Role::Page);
    let rail_project = crate::frame::rail_project(root, &view.forge_words, &view.sign_in);
    let no_project = crate::outside::no_project(root);
    let reading = crate::outside::reading_me(root, &view.forge_words, &view.sign_in);
    let terminal =
        !reading && !no_project && root.approval().is_none() && crate::pages::terminal::shown(root, &view.forge_words);
    if !terminal {
        if let Some(project) = rail_project {
            crate::frame::rail(
                &mut builder,
                root,
                &view.forge_words,
                &mut view.pages,
                &mut view.forge,
                project,
                &mut times,
                &view.forge_sign_in,
            );
        } else {
            crate::outside::top(&mut builder, root, &view.forge_words, &mut view.pages);
        }
        match root.pages().address() {
            temper_client_domain::Address::Welcome { .. } => {}
            temper_client_domain::Address::ConnectModel { .. }
            | temper_client_domain::Address::Remembered
            | temper_client_domain::Address::Members
            | temper_client_domain::Address::Limits
            | temper_client_domain::Address::Accounts
            | temper_client_domain::Address::Chats { .. }
            | temper_client_domain::Address::NewChat { .. }
            | temper_client_domain::Address::Chat { .. }
            | temper_client_domain::Address::Task { .. }
            | temper_client_domain::Address::AllHome
            | temper_client_domain::Address::Inbox { .. }
            | temper_client_domain::Address::Forges
            | temper_client_domain::Address::ConnectForge { .. }
            | temper_client_domain::Address::Home { .. }
            | temper_client_domain::Address::Changes { .. }
            | temper_client_domain::Address::Team { .. }
            | temper_client_domain::Address::Settings { .. }
            | temper_client_domain::Address::Models { .. }
            | temper_client_domain::Address::Builders { .. }
            | temper_client_domain::Address::Role { .. }
            | temper_client_domain::Address::Rule { .. }
            | temper_client_domain::Address::Repositories { .. }
            | temper_client_domain::Address::Work { .. }
            | temper_client_domain::Address::Goals { .. }
            | temper_client_domain::Address::Ask { .. }
            | temper_client_domain::Address::Adopt { .. } => {
                if !reading && !no_project {
                    crate::choice::banner(&mut builder, root, &view.forge_words, &mut view.pages);
                }
            }
        }
        match root.pages().address() {
            temper_client_domain::Address::Chats { .. } | temper_client_domain::Address::NewChat { .. } => {
                if !reading && root.approval().is_none() {
                    crate::frame::playground_banner(&mut builder, root, &view.forge_words, &mut view.forge);
                }
            }
            temper_client_domain::Address::Welcome { .. }
            | temper_client_domain::Address::ConnectModel { .. }
            | temper_client_domain::Address::Remembered
            | temper_client_domain::Address::Members
            | temper_client_domain::Address::Limits
            | temper_client_domain::Address::Accounts
            | temper_client_domain::Address::Chat { .. }
            | temper_client_domain::Address::Task { .. }
            | temper_client_domain::Address::AllHome
            | temper_client_domain::Address::Inbox { .. }
            | temper_client_domain::Address::Forges
            | temper_client_domain::Address::ConnectForge { .. }
            | temper_client_domain::Address::Home { .. }
            | temper_client_domain::Address::Changes { .. }
            | temper_client_domain::Address::Team { .. }
            | temper_client_domain::Address::Settings { .. }
            | temper_client_domain::Address::Models { .. }
            | temper_client_domain::Address::Builders { .. }
            | temper_client_domain::Address::Role { .. }
            | temper_client_domain::Address::Rule { .. }
            | temper_client_domain::Address::Repositories { .. }
            | temper_client_domain::Address::Work { .. }
            | temper_client_domain::Address::Goals { .. }
            | temper_client_domain::Address::Ask { .. }
            | temper_client_domain::Address::Adopt { .. } => {
                // Outside Chats and NewChat the line shows only while the
                // playground is leaving; while it runs, the frame's "Set up
                // temper" is the way out (temper/docs/design/client/view.md, 1).
                if !reading && root.approval().is_none() && root.forge().leaving() {
                    crate::frame::playground_banner(&mut builder, root, &view.forge_words, &mut view.forge);
                }
            }
        }
    }
    if let Some(page) = root.authorization() {
        crate::sign_in::authorize(
            &mut builder,
            root,
            page.state(),
            page.query(),
            &view.forge_words,
            &view.sign_in,
            &view.forge_sign_in,
        );
    } else if let Some(page) = root.approval() {
        crate::approve::main(&mut builder, root, page, &view.forge_words, &view.sign_in, &mut view.pages, &mut times);
    } else if crate::sign_in::offered(root, &view.forge_words, &view.sign_in)
        && let Some(origin) = &view.sign_in.origin
    {
        crate::sign_in::main(&mut builder, root, &view.forge_words, origin, &view.forge_sign_in);
    } else if let Some(heading) = crate::frame::unavailable(root) {
        crate::outside::column(&mut builder, &view.forge_words.say(heading, &[]), &[]);
        crate::frame::unavailable_main(&mut builder, root, &view.forge_words);
        if let Some(page) = root.pages().task()
            && page.document(root.client()).is_none()
        {
            crate::pages::task::reading(&mut builder, page, &view.forge_words);
        }
        builder.close();
    } else if reading {
        builder.open(Role::Main);
        builder.state(skein_ui::State::Busy);
        builder.close();
    } else if no_project
        && root.pages().address() != &temper_client_domain::Address::Remembered
        && root.pages().address() != &temper_client_domain::Address::Members
        && root.pages().address() != &temper_client_domain::Address::Limits
    {
        crate::outside::no_project_main(&mut builder, root, &view.forge_words, &mut view.pages);
    } else if let temper_client_domain::Address::Welcome { step } = root.pages().address() {
        crate::pages::welcome::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.accounts,
            &mut view.forge,
            *step,
            view.limits.tree.text_per_node,
            &view.forge_sign_in,
        );
    } else if let temper_client_domain::Address::Inbox { .. } = root.pages().address() {
        crate::pages::inbox::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            view.rows,
            &mut times,
            &view.forge_sign_in,
        );
    } else if let temper_client_domain::Address::NewChat { .. } = root.pages().address() {
        crate::pages::new_chat::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.accounts,
            &mut view.forge,
            view.limits.tree.text_per_node,
            &mut times,
        );
    } else if let temper_client_domain::Address::Chats { .. } = root.pages().address() {
        crate::pages::chats::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.forge,
            view.limits.tree.text_per_node,
            &mut times,
        );
    } else if let temper_client_domain::Address::Changes { project } = root.pages().address() {
        crate::pages::changes::main(&mut builder, root, &view.forge_words, &mut view.pages, *project, &mut times);
    } else if let temper_client_domain::Address::Team { project, .. } = root.pages().address() {
        crate::pages::team::main(&mut builder, root, &view.forge_words, *project, &mut view.pages, &mut times);
    } else if let temper_client_domain::Address::AllHome = root.pages().address() {
        crate::pages::all_home::main(&mut builder, root, &view.forge_words, &mut view.pages, &mut times, view.rows);
    } else if let temper_client_domain::Address::Home { project } = root.pages().address() {
        crate::pages::home::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.accounts,
            *project,
            &mut times,
            view.rows,
            &view.forge_sign_in,
        );
    } else if let temper_client_domain::Address::Goals { project, .. } = root.pages().address() {
        crate::pages::goals::main(&mut builder, root, &view.forge_words, &mut view.pages, *project, &mut times);
    } else if let temper_client_domain::Address::Work { project, .. } = root.pages().address() {
        crate::pages::work::main(&mut builder, root, &view.forge_words, &mut view.pages, *project, &mut times);
    } else if let temper_client_domain::Address::Ask { project } = root.pages().address() {
        crate::pages::ask_page::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.accounts,
            *project,
            view.limits.tree.text_per_node,
        );
    } else if let temper_client_domain::Address::Settings { project } = root.pages().address() {
        crate::pages::settings::how_work_runs(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.forge,
            *project,
            wall,
        );
    } else if let temper_client_domain::Address::Models { project } = root.pages().address() {
        crate::pages::models::main(&mut builder, root, &view.forge_words, &mut view.pages, *project, &mut times);
    } else if let temper_client_domain::Address::Builders { project } = root.pages().address() {
        crate::pages::builders::main(&mut builder, root, &view.forge_words, &mut view.pages, *project);
    } else if let temper_client_domain::Address::Remembered = root.pages().address() {
        crate::pages::remembered::main(&mut builder, root, &view.forge_words, &mut view.pages, &mut times);
    } else if let temper_client_domain::Address::Members = root.pages().address() {
        crate::pages::deployment::members(&mut builder, root, &view.forge_words, &mut view.pages);
    } else if let temper_client_domain::Address::Limits = root.pages().address() {
        crate::pages::deployment::limits(&mut builder, root, &view.forge_words, &mut view.pages);
    } else if let temper_client_domain::Address::Role { project, .. }
    | temper_client_domain::Address::Rule { project, .. } = root.pages().address()
    {
        crate::pages::roles::main(&mut builder, root, &view.forge_words, &mut view.pages, *project);
    } else if let temper_client_domain::Address::Repositories { project }
    | temper_client_domain::Address::Adopt { project } = root.pages().address()
    {
        crate::pages::settings::repositories(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            *project,
            &view.forge_sign_in,
            &mut view.forge,
        );
    } else if let temper_client_domain::Address::Chat { .. } = root.pages().address() {
        crate::pages::chat::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            view.rows,
            view.text,
            &mut times,
        );
    } else if let temper_client_domain::Address::Task { .. } = root.pages().address() {
        view.task_main(&mut builder, root, &mut times);
    } else if let temper_client_domain::Address::ConnectModel { .. } = root.pages().address() {
        crate::pages::connect_model::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.accounts,
            &mut view.pages,
            rail_project,
            view.limits.tree.text_per_node,
        );
    } else if *root.pages().address() == temper_client_domain::Address::Accounts {
        crate::pages::accounts::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.accounts,
            &mut view.pages,
            rail_project,
            view.limits.tree.text_per_node,
        );
    } else if let temper_client_domain::Address::ConnectForge { forge } = root.pages().address() {
        forge::connect::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.forge,
            &mut view.accounts,
            *forge,
            rail_project,
            &view.forge_sign_in,
        );
    } else if *root.pages().address() == temper_client_domain::Address::Forges {
        forge::forges::main(
            &mut builder,
            root,
            &view.forge_words,
            &mut view.pages,
            &mut view.forge,
            &view.forge_sign_in,
            rail_project,
        );
    }
    if let Some(project) = rail_project
        && !terminal
        && view.forge_words.shell == crate::words::Shell::Browser
    {
        crate::frame::tabbar(&mut builder, root, &view.forge_words, &mut view.pages, project, times.now);
    }
    crate::frame::toast(&mut builder, root, &view.forge_words, &mut view.pages, &mut times);
    if let Some(start) = root.pages().start() {
        crate::pages::start_goal::sheet(&mut builder, root, start, &view.forge_words, &mut view.pages);
    }
    builder.close();
    view.next_change = [times.next, view.pages.model_times.next].into_iter().flatten().min();
    view.render.finish(builder, shown, out);
    view.revision = Some(shown);
}
#[must_use]
pub fn act(view: &mut View, node: NodeId, act: Act) -> Option<Action> {
    if let temper_client_domain::Event::Act { action } = act_event(view, node, act)? { Some(action) } else { None }
}
/// Decode an act being sent and remember its setting row for a refusal (client/pages/settings.md, 2).
#[must_use]
pub fn act_event(view: &mut View, node: NodeId, act: Act) -> Option<temper_client_domain::Event> {
    let event = event(view, node, act)?;
    for shown in view.tree().nodes() {
        if shown.id != node {
            continue;
        }
        if let Some(binding) = shown.binding
            && binding.owner == temper_client_domain::forge::words::owner(view.forge_words.connector)
        {
            view.forge.remember(binding.token);
        }
        break;
    }
    Some(event)
}
/// The line a sign-in's tab holds until its address comes (client/pages/README.md).
#[must_use]
pub fn opening_tab(view: &View, root: &Domain, provider: u32) -> Box<[u8]> {
    let words = &view.forge_words;
    let mut name = b"".as_slice();
    if let Some(me) = temper_client_domain::pages::accounts::current_me(root.client()) {
        for offer in &me.providers {
            if offer.provider == provider {
                name = crate::accounts::words::provider_words(offer.name.bytes()).1;
                break;
            }
        }
    }
    words
        .say(crate::words::Word::WelcomeTabOpening, &[skein_ui::phrases::Hole::Name(crate::choice::shown(name, words))])
}

/// The address a link node opens, which the browser shell gives as its href
/// (client/web/README.md, 2.3, and client/web/gallery.md, 1): its external address, or the page its press goes to.
#[must_use]
pub fn href(view: &mut View, node: NodeId) -> Option<Box<[u8]>> {
    for shown in view.tree().nodes() {
        if shown.id == node
            && let Some(address) = &shown.external
        {
            return Some(address.clone());
        }
    }
    match act(view, node, Act::Press)? {
        Action::Pages(temper_client_domain::pages::Act::OpenProject { .. }) => {
            Some(temper_client_domain::address::print(&temper_client_domain::Address::Chats {
                filter: temper_client_domain::ChatsFilter::All,
            }))
        }
        Action::Pages(temper_client_domain::pages::Act::Go { address }) => {
            Some(temper_client_domain::address::print(&address))
        }
        Action::Forge(temper_client_domain::forge::Act::Go {
            address: temper_client_domain::forge::Address::Adopt { project },
        }) => Some(temper_client_domain::address::print(&temper_client_domain::Address::Adopt { project })),
        Action::Approve(_) | Action::Pages(_) | Action::Forge(_) | Action::Copy { .. } => None,
    }
}
/// Read the event an act gives, without changing the view (client/web/journeys.md, 3.3).
/// Shell inputs can move a secret directly to the domain without a clonable action.
#[must_use]
pub fn event(view: &View, node: NodeId, act: Act) -> Option<temper_client_domain::Event> {
    let bound = view.render.act(node, act)?;
    if bound.binding.owner == crate::accounts::owner() {
        return view.accounts.decode(bound.binding.token, bound.act);
    }
    if bound.binding.owner == crate::frame::owner() {
        return Some(temper_client_domain::Event::Act { action: view.pages.decode(bound.binding.token, bound.act)? });
    }
    if bound.binding.owner == temper_client_domain::forge::words::owner(view.forge_words.connector) {
        forge::bindings::event(&view.forge, bound.binding.token, bound.act)
    } else {
        None
    }
}

#[cfg(test)]
mod tab_tests {
    use super::*;
    use crate::tests::fixtures as f;
    use jig_client::contract as c;
    use skein_lib::List;

    #[test]
    fn opening_tab_names_the_given_providers_plan() {
        let (mut root, watch) = f::root_with_watch(None);
        let mut me = temper_client_domain::pages::accounts::current_me(root.client()).expect("fixture me").clone();
        let mut providers = List::with_capacity(2);
        for (provider, name) in [(31, b"anthropic".as_slice()), (17, b"openai".as_slice())] {
            providers
                .push(c::ProviderOffer {
                    provider,
                    name: c::DocumentText::complete(c::Text::new(name.into(), 32).expect("provider name")),
                    key_bytes: 256,
                    sign_in: true,
                    signed_in_plan: true,
                    key: true,
                    programs: List::with_capacity(0),
                    programs_cut: false,
                    none: false,
                    manage_usage: None,
                })
                .expect("two providers");
        }
        me.providers = providers;
        f::root_event(
            &mut root,
            temper_client_domain::Event::Client {
                event: jig_client::Event::Up {
                    up: jig_client::Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                    },
                },
            },
        );
        let view = f::view();
        assert_eq!(opening_tab(&view, &root, 17).as_ref(), "Opening ChatGPT’s sign-in…".as_bytes());
        assert_eq!(opening_tab(&view, &root, 31).as_ref(), "Opening Claude’s sign-in…".as_bytes());
        assert_eq!(opening_tab(&view, &root, 99).as_ref(), "Opening ’s sign-in…".as_bytes());
        let root = temper_client_domain::new(&f::limits(), f::SEED);
        assert_eq!(opening_tab(&view, &root, 17).as_ref(), "Opening ’s sign-in…".as_bytes());
    }
}
