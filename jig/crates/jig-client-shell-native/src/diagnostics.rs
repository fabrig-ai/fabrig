//! Payload-free transport state, copied only for an explicit diagnostic.
use jig_client::Message;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Authorize,
    Request,
    Read,
    Watch,
    Unwatch,
    StreamGone,
    SignInLink,
    SignInService,
    SignOut,
    Stream,
}
impl Kind {
    pub(crate) fn message(message: &Message) -> Self {
        match message {
            Message::Authorize(_) => Self::Authorize,
            Message::Request(_) => Self::Request,
            Message::Read(_) => Self::Read,
            Message::Watch(_) => Self::Watch,
            Message::Unwatch(_) => Self::Unwatch,
            Message::StreamGone(_) => Self::StreamGone,
            Message::SignInLink(_) => Self::SignInLink,
            Message::SignInService => Self::SignInService,
            Message::SignOut => Self::SignOut,
        }
    }
}
/// Lists are bounded by the shell's held-request and HTTP exchange limits.
/// No record, credential, token, path or address enters this snapshot.
#[derive(Debug)]
pub struct Pending {
    pub held: u32,
    pub held_kinds: List<Kind>,
    pub active: u32,
    pub active_kinds: List<Kind>,
    pub queued: u32,
    pub queued_kinds: List<Kind>,
    pub files_ready: bool,
}
