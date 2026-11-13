//! Shared commands items (temper/docs/design/client/view-crates.md, 3.3).
use alloc::boxed::Box;
use jig_client::contract as c;
use temper_client_domain::{
    Address, Domain,
    pages::{
        chat, chats, home,
        terminal::{self, Command, Note},
    },
};

pub(crate) fn description(command: Command) -> &'static [u8] {
    match command {
        Command::Model => b"Change the model, from the next turn",
        Command::Effort => b"Change the effort",
        Command::Draft => b"Show or hide the draft",
        Command::Pause => b"Pause this chat; a message resumes it",
        Command::Close => b"Close this chat; it stays in Chats",
        Command::StartOver => b"Start a new Ask temper chat",
        Command::Goal => b"Start a goal from the draft",
        Command::New => b"Start a new chat",
        Command::Chats => b"Open another chat",
        Command::Theme => b"Dark or light",
        Command::Font => b"DejaVu Sans Mono or IBM Plex Mono, to compare",
        Command::Help => b"Commands and keys",
        Command::Exit => b"Leave the terminal",
    }
}

fn efforts(root: &Domain) -> bool {
    let Some(me) = root.pages().me(root.client()) else { return false };
    let draft = root.pages().choice();
    let Some(project) = draft.project() else { return false };
    let Some(models) = temper_client_domain::pages::choice::offer(me, project) else { return false };
    let Some(index) = draft.selected(me) else { return false };
    let Some(model) = models.get(index) else { return false };
    !model.efforts.is_empty()
}

#[expect(
    clippy::too_many_lines,
    reason = "one command match keeps chat permissions and the talk's exclusions together"
)]
pub(crate) fn offered(root: &Domain, task: Option<&c::Task>, command: Command) -> bool {
    if let Some(chat) = root.pages().chat()
        && chat.talk().is_some()
    {
        return match command {
            Command::Model
            | Command::Effort
            | Command::Draft
            | Command::Pause
            | Command::Close
            | Command::StartOver
            | Command::Goal => false,
            Command::New | Command::Chats | Command::Theme | Command::Font | Command::Help | Command::Exit => true,
        };
    }
    match command {
        Command::Model => crate::choice::changeable(root, task),
        Command::Effort => crate::choice::changeable(root, task) && efforts(root),
        Command::Draft => {
            if let Some(chat) = root.pages().chat() {
                chat.draft.is_some()
                    || chat.writing
                    || match task {
                        Some(task) => {
                            task.result.is_none()
                                && !chats::ended(&task.chip)
                                && temper_client_domain::pages::chat_offers_draft(root, task)
                        }
                        None => false,
                    }
            } else {
                match root.pages().address() {
                    Address::NewChat { draft } => {
                        *draft
                            || match root.pages().project() {
                                Some(project) => temper_client_domain::pages::drafts_offered(root, project),
                                None => false,
                            }
                    }
                    Address::Chat { .. }
                    | Address::Talk { .. }
                    | Address::Ask { .. } | Address::Tell { .. }
                    | Address::AllHome
                    | Address::Inbox { .. }
                    | Address::Chats { .. }
                    // page-view: draw Remembered.
                    | Address::Told
                    | Address::Remembered
                    | Address::Members
                    | Address::Limits
                    | Address::Accounts
                    | Address::Forges
                    | Address::ConnectForge { .. }
                    | Address::ConnectModel { .. }
                    | Address::Welcome { .. }
                    | Address::Task { .. }
                    | Address::Work { .. }
                    | Address::Goals { .. }
                    | Address::Home { .. }
                    | Address::Changes { .. }
                    | Address::Team { .. }
                    | Address::Settings { .. }
                    | Address::Models { .. }
                    | Address::Builders { .. }
                    | Address::Role { .. }
                    | Address::Rule { .. }
                    | Address::Repositories { .. }
                    | Address::Adopt { .. } => false,
                }
            }
        }
        Command::Pause => match (task, root.pages().me(root.client())) {
            (Some(task), Some(me)) => !chat::paused(&task.chip, me) && chat::allowed(task, c::ActKind::Stop),
            (Some(task), None) => chat::allowed(task, c::ActKind::Stop),
            (None, _) => false,
        },
        Command::Close => match task {
            Some(task) => chat::allowed(task, c::ActKind::Cancel),
            None => false,
        },
        Command::StartOver => match (root.pages().chat(), task) {
            (Some(chat), Some(task)) => {
                crate::common::chat::asking(root)
                    && chat.draft.is_some()
                    && task.result.is_none()
                    && !chats::ended(&task.chip)
            }
            (Some(_), None) | (None, _) => false,
        },
        Command::Goal => {
            if crate::common::chat::asking(root) {
                return false;
            }
            let Some(chat) = root.pages().chat() else { return false };
            let Some(task) = task else { return false };
            let Some(draft) = &chat.draft else { return false };
            let repository = match home::repository(root, task.project) {
                home::Repository::In { .. } => true,
                home::Repository::Unknown
                | home::Repository::Copying { .. }
                | home::Repository::CopiesIn { .. }
                | home::Repository::CopyOffer { .. }
                | home::Repository::Add { .. }
                | home::Repository::Adopting(_)
                | home::Repository::Offer { .. }
                | home::Repository::GitHub { .. }
                | home::Repository::None => false,
            };
            chat.goal_from(root).is_none()
                && !draft.body.trim_ascii().is_empty()
                && temper_client_domain::pages::drafts_offered(root, task.project)
                && repository
        }
        Command::New | Command::Chats | Command::Theme | Command::Font | Command::Help | Command::Exit => true,
    }
}

/// A simple chat confirmation is a list; delegate decisions keep their sheet.
pub(crate) fn confirms(root: &Domain) -> bool {
    let Some(open) = root.pages().confirmation() else { return false };
    let Some(chat) = root.pages().chat() else { return false };
    let Some(task) = chat.task(root.client()) else { return false };
    open.act == c::ActKind::Cancel
        && open.object == c::ObjectKey::Task(chat.task)
        && task.delegates.is_empty()
        && chat.closing.is_some()
}

fn help() -> Box<[u8]> {
    let mut width = 0;
    for command in Command::ALL {
        width = core::cmp::max(width, command.name().len());
    }
    let width = width.checked_add(2).expect("command width fits");
    let keys = b"\nEnter sends\nShift+Enter makes a new line\nCtrl+\\ shows or hides the draft\nPgUp and PgDn read back\nEsc closes a list";
    let mut length = keys.len();
    for command in Command::ALL {
        length = length
            .checked_add(width)
            .expect("help fits")
            .checked_add(description(command).len())
            .expect("help fits")
            .checked_add(1)
            .expect("help fits");
    }
    let mut out = skein_lib::Writer::new(length);
    for command in Command::ALL {
        out.put(command.name()).expect("help fits");
        for _ in command.name().len()..width {
            out.put(b" ").expect("help fits");
        }
        out.put(description(command)).expect("help fits");
        out.put(b"\n").expect("help fits");
    }
    out.put(keys).expect("help fits");
    out.finish()
}

/// A terminal note (temper/docs/design/client/pages/README.md, The terminal's notes).
pub(crate) fn note(note: &Note) -> Box<[u8]> {
    match note {
        Note::Help => help(),
        Note::Unknown { words } => crate::accounts::words::join(&[b"Unknown command ", words, b". /help lists them."]),
        Note::Font { font } => skein_lib::bytes::copy_of(match font {
            terminal::Font::DejaVu => b"Font: DejaVu Sans Mono.",
            terminal::Font::Plex => b"Font: IBM Plex Mono.",
        }),
        Note::Closed => skein_lib::bytes::copy_of(b"This chat is closed. /new starts another."),
    }
}
