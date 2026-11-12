//! Root obligations pass unchanged to the native client shell (client/line.md, 2).
use jig_client::contract::VersionRange;
use jig_client_protocol_link as link;
use skein_lib::Secret;
use temper_client_domain as client;
/// Values shown outside the domain, including one-time credentials.
#[derive(Debug)]
pub enum Notice {
    SignedIn,
    SignedOut,
    Credential { party: u64, credential: Secret },
    Dropped { count: u32 },
    Unread,
    Served { family: u16, versions: Option<VersionRange> },
    Unasked,
    Leave { to: Box<[u8]> },
    Fact(link::Fact),
}
/// Each root obligation has exactly one destination.
#[derive(Debug)]
pub enum Down {
    Shell(jig_client::Request),
    Unlinked(jig_client::UnlinkedRequest),
    Duty(Duty),
    Notice(Notice),
}
/// Navigation, authentication and facts stay in the host.
#[derive(Debug)]
pub enum Duty {
    Copy { token: u32 },
    Show { address: client::Address, replace: bool },
    SignOut,
    Fact(client::Fact),
}
#[must_use]
pub fn down(request: client::Request) -> Down {
    match request {
        client::Request::Copy { token, words: _ } => Down::Duty(Duty::Copy { token }),
        client::Request::Client { request } => Down::Shell(request),
        client::Request::Unlinked { request } => Down::Unlinked(request),
        client::Request::Show { address, replace } => Down::Duty(Duty::Show { address, replace }),
        client::Request::Push(_)
        | client::Request::Screen { .. }
        | client::Request::KeepLook { .. }
        | client::Request::KeepSignIn { .. } => Down::Notice(Notice::Unasked),
        client::Request::Tab { tab } => match tab {
            client::Tab::Point { to } => Down::Notice(Notice::Leave { to }),
            client::Tab::Open { .. } | client::Tab::Close => Down::Notice(Notice::Unasked),
        },
        client::Request::Leave { to, .. } => Down::Notice(Notice::Leave { to }),
        temper_client_domain::Request::SignOut => Down::Duty(Duty::SignOut),
        client::Request::Dropped { count } => Down::Notice(Notice::Dropped { count }),
        client::Request::Fact { fact } => Down::Duty(Duty::Fact(fact)),
    }
}
