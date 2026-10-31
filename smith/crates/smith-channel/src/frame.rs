//! Encode and frame typed channel bodies (smith/docs/design/protocol/channel.md, sections 3–4).
use crate::{kind, v5};
use skein_channel::{Frame, FrameError, frame_writer};
use skein_lib::Writer;

/// One owned channel body whose kind follows its codec.
#[derive(Debug)]
pub enum Body {
    /// The `Start` wire body.
    Start(v5::Start),
    /// The `Message` wire body.
    Message(v5::Message),
    /// The `HostAnswer` wire body.
    HostAnswer(v5::HostAnswer),
    /// The `Acknowledge` wire body.
    Acknowledge(v5::Acknowledge),
    /// The `GrantRefresh` wire body.
    GrantRefresh(v5::GrantRefresh),
    /// The `Cancel` wire body.
    Cancel(v5::Cancel),
    /// The `Admitted` wire body.
    Admitted(v5::Admitted),
    /// The `Call` wire body.
    Call(v5::Call),
    /// The `Withdraw` wire body.
    Withdraw(v5::Withdraw),
    /// The `Turn` wire body.
    Turn(v5::Turn),
    /// The `Waiting` wire body.
    Waiting(v5::Waiting),
    /// The `Long` wire body.
    Long(v5::Long),
    /// The `LongDone` wire body.
    LongDone(v5::LongDone),
    /// The `Unauthorized` wire body.
    Unauthorized(v5::Unauthorized),
    /// The `Limited` wire body.
    Limited(v5::Limited),
    /// The `Fact` wire body.
    Fact(v5::Fact),
    /// The `Answer` wire body.
    Answer(v5::Answer),
    /// The `MessageRefused` wire body.
    MessageRefused(v5::MessageRefused),
    /// The `Live` wire body.
    Live(v5::Live),
    /// The `AccountRefused` wire body.
    AccountRefused(v5::AccountRefused),
}

/// Encode a body with its matching kind and exactly measured frame length.
pub fn frame(body: Body) -> Result<Frame, FrameError> {
    let (kind, length) = match &body {
        Body::Start(record) => (kind::START, record.measure()),
        Body::Message(record) => (kind::MESSAGE, record.measure()),
        Body::HostAnswer(record) => (kind::HOST_ANSWER, record.measure()),
        Body::Acknowledge(record) => (kind::ACKNOWLEDGE, record.measure()),
        Body::GrantRefresh(record) => (kind::GRANT_REFRESH, record.measure()),
        Body::Cancel(record) => (kind::CANCEL, record.measure()),
        Body::Admitted(record) => (kind::ADMITTED, record.measure()),
        Body::Call(record) => (kind::CALL, record.measure()),
        Body::Withdraw(record) => (kind::WITHDRAW, record.measure()),
        Body::Turn(record) => (kind::TURN, record.measure()),
        Body::Waiting(record) => (kind::WAITING, record.measure()),
        Body::Long(record) => (kind::LONG, record.measure()),
        Body::LongDone(record) => (kind::LONG_DONE, record.measure()),
        Body::Unauthorized(record) => (kind::UNAUTHORIZED, record.measure()),
        Body::Limited(record) => (kind::LIMITED, record.measure()),
        Body::Fact(record) => (kind::FACT, record.measure()),
        Body::Answer(record) => (kind::ANSWER, record.measure()),
        Body::MessageRefused(record) => (kind::MESSAGE_REFUSED, record.measure()),
        Body::Live(record) => (kind::LIVE, record.measure()),
        Body::AccountRefused(record) => (kind::ACCOUNT_REFUSED, record.measure()),
    };
    let mut writer = Writer::new(usize::try_from(length).or(Err(FrameError::TooLarge))?);
    let encoded = match body {
        Body::Start(record) => record.encode(&mut writer),
        Body::Message(record) => record.encode(&mut writer),
        Body::HostAnswer(record) => record.encode(&mut writer),
        Body::Acknowledge(record) => record.encode(&mut writer),
        Body::GrantRefresh(record) => record.encode(&mut writer),
        Body::Cancel(record) => record.encode(&mut writer),
        Body::Admitted(record) => record.encode(&mut writer),
        Body::Call(record) => record.encode(&mut writer),
        Body::Withdraw(record) => record.encode(&mut writer),
        Body::Turn(record) => record.encode(&mut writer),
        Body::Waiting(record) => record.encode(&mut writer),
        Body::Long(record) => record.encode(&mut writer),
        Body::LongDone(record) => record.encode(&mut writer),
        Body::Unauthorized(record) => record.encode(&mut writer),
        Body::Limited(record) => record.encode(&mut writer),
        Body::Fact(record) => record.encode(&mut writer),
        Body::Answer(record) => record.encode(&mut writer),
        Body::MessageRefused(record) => record.encode(&mut writer),
        Body::Live(record) => record.encode(&mut writer),
        Body::AccountRefused(record) => record.encode(&mut writer),
    };
    encoded.or(Err(FrameError::Incomplete))?;
    let mut frame = frame_writer(kind, length)?;
    frame.put(&writer.finish())?;
    frame.finish()
}
