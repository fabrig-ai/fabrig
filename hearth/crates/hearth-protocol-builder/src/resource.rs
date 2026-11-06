//! Resources held through close acknowledgement (skein/docs/foundation/programming-model.md, 5.2).
use skein_lib::Token;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Resource {
    Awaited,
    Open(Token),
    Closing(Token),
    Closed,
}

impl Resource {
    pub fn token(self) -> Option<Token> {
        match self {
            Self::Open(token) | Self::Closing(token) => Some(token),
            Self::Awaited | Self::Closed => None,
        }
    }
    pub fn is_open(self) -> bool {
        match self {
            Self::Open(_) => true,
            Self::Awaited | Self::Closing(_) | Self::Closed => false,
        }
    }
    pub fn request_close(&mut self) -> Option<Token> {
        match *self {
            Self::Open(token) => {
                *self = Self::Closing(token);
                Some(token)
            }
            Self::Awaited | Self::Closing(_) | Self::Closed => None,
        }
    }
}
