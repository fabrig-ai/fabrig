//! What a call waited for when the link's deadline passed (client/tools.md, 6.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Waited {
    Link,
    Stream,
    Read,
    Outcome,
    Again(Again),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Again {
    Busy,
    NotReady,
    Lost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Act {
    None,
    NotSent,
    Kept { named: bool },
    Once { named: bool },
}
pub(crate) fn waited(
    link: jig_client::Link,
    connected: bool,
    heard: bool,
    acting: bool,
    retry: Option<jig_client::Retry>,
) -> Waited {
    match link {
        jig_client::Link::Offline | jig_client::Link::SignedOut | jig_client::Link::OutOfDate => Waited::Link,
        jig_client::Link::Starting => {
            if !connected {
                Waited::Link
            } else if !heard {
                Waited::Stream
            } else {
                Waited::Read
            }
        }
        jig_client::Link::Live => {
            if acting {
                match retry {
                    Some(jig_client::Retry::Busy) => Waited::Again(Again::Busy),
                    Some(jig_client::Retry::NotReady) => Waited::Again(Again::NotReady),
                    Some(jig_client::Retry::Lost | jig_client::Retry::Undecodable) => Waited::Again(Again::Lost),
                    Some(jig_client::Retry::Full { .. }) | None => Waited::Outcome,
                }
            } else {
                Waited::Read
            }
        }
    }
}
pub(crate) fn line(address: &str, waited: Waited, act: Act) -> String {
    let mut line = match waited {
        Waited::Link => format!("the link to the engine at {address} did not come up within 30 s"),
        Waited::Stream => {
            format!("the engine at {address} took the connection but its stream did not answer within 30 s")
        }
        Waited::Read => format!("the engine at {address} did not answer a read within 30 s"),
        Waited::Again(Again::Busy) => {
            format!("the engine at {address} last answered this act busy; no outcome within 30 s")
        }
        Waited::Again(Again::NotReady) => {
            format!("the engine at {address} last answered this act not ready; no outcome within 30 s")
        }
        Waited::Again(Again::Lost) => {
            format!("the last answer to this act from the engine at {address} was lost; no outcome within 30 s")
        }
        Waited::Outcome => format!("the engine at {address} gave no outcome for this act within 30 s"),
    };
    line.push_str(match act {
        Act::None => "",
        Act::NotSent => "; it was not sent",
        Act::Kept { named: true } => "; it is kept, and the same --key may be tried again",
        Act::Kept { named: false } => "; it is kept, and the next call that sends tries it again",
        Act::Once { named: true } => "; it is not kept, and the same --key and file may be tried again",
        Act::Once { named: false } => "; it is not kept, and calling again is a new request",
    });
    line
}
