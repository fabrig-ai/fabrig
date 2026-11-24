use skein_lib::Duration;

/// The core's figures for waits with no clock of their own (jig/docs/design/domain/engine/waits.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Figures {
    pub question: Duration,
    pub party: Duration,
    pub news: Duration,
    pub host: Duration,
    pub account: Duration,
    pub step: Duration,
    pub room: Duration,
    pub sign_off: Duration,
}
