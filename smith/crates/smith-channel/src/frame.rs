//! Encode and frame typed channel bodies (smith/docs/design/protocol/channel.md, sections 3–4).
use crate::{Record, kind, v13};
use skein_channel::{Frame, FrameError, frame_writer};
use skein_lib::Writer;

/// One owned channel body whose kind follows its codec.
#[derive(Debug)]
pub enum Body {
    /// The `Start` wire body.
    Start(v13::Start),
    /// The `Message` wire body.
    Message(v13::Message),
    /// The `HostAnswer` wire body.
    HostAnswer(v13::HostAnswer),
    /// The `Acknowledge` wire body.
    Acknowledge(v13::Acknowledge),
    /// The `GrantRefresh` wire body.
    GrantRefresh(v13::GrantRefresh),
    /// The `Cancel` wire body.
    Cancel(v13::Cancel),
    /// The `Drain` wire body.
    Drain(v13::Drain),
    /// The `Renewals` wire body.
    Renewals(v13::Renewals),
    /// The `Admitted` wire body.
    Admitted(v13::Admitted),
    /// The `Call` wire body.
    Call(v13::Call),
    /// The `Withdraw` wire body.
    Withdraw(v13::Withdraw),
    /// The `Turn` wire body.
    Turn(v13::Turn),
    /// The `Waiting` wire body.
    Waiting(v13::Waiting),
    /// The `Long` wire body.
    Long(v13::Long),
    /// The `LongDone` wire body.
    LongDone(v13::LongDone),
    /// The `Unauthorized` wire body.
    Unauthorized(v13::Unauthorized),
    /// The `Limited` wire body.
    Limited(v13::Limited),
    /// One account allowance window.
    Usage(v13::Usage),
    /// The `Fact` wire body.
    Fact(v13::Fact),
    /// The `Answer` wire body.
    Answer(v13::Answer),
    /// The `MessageRefused` wire body.
    MessageRefused(v13::MessageRefused),
    /// The `Live` wire body.
    Live(v13::Live),
    /// The `AccountRefused` wire body.
    AccountRefused(v13::AccountRefused),
    /// The `Signalled` wire body.
    Signalled(v13::Signalled),
}

/// Encode a body with its matching kind and exactly measured frame length.
pub fn frame(body: Body) -> Result<Frame, FrameError> {
    let (kind, length) = match &body {
        Body::Start(record) => (kind::START, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Message(record) => (kind::MESSAGE, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::HostAnswer(record) => (kind::HOST_ANSWER, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Acknowledge(record) => (kind::ACKNOWLEDGE, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::GrantRefresh(record) => {
            (kind::GRANT_REFRESH, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
        Body::Cancel(record) => (kind::CANCEL, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Drain(record) => (kind::DRAIN, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Renewals(record) => (kind::RENEWALS, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Admitted(record) => (kind::ADMITTED, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Call(record) => (kind::CALL, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Withdraw(record) => (kind::WITHDRAW, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Turn(record) => (kind::TURN, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Waiting(record) => (kind::WAITING, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Long(record) => (kind::LONG, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::LongDone(record) => (kind::LONG_DONE, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Unauthorized(record) => {
            (kind::UNAUTHORIZED, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
        Body::Limited(record) => (kind::LIMITED, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Usage(record) => (kind::USAGE, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Fact(record) => (kind::FACT, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Answer(record) => (kind::ANSWER, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::MessageRefused(record) => {
            (kind::MESSAGE_REFUSED, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
        Body::Signalled(record) => (kind::SIGNALLED, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Live(record) => (kind::LIVE, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::AccountRefused(record) => {
            (kind::ACCOUNT_REFUSED, record.measure(&v13::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
    };
    let mut writer = Writer::new(usize::try_from(length).or(Err(FrameError::TooLarge))?);
    let encoded = match body {
        Body::Start(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Message(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::HostAnswer(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Acknowledge(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::GrantRefresh(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Cancel(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Drain(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Renewals(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Admitted(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Call(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Withdraw(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Turn(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Waiting(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Long(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::LongDone(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Unauthorized(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Limited(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Usage(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Fact(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Answer(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::MessageRefused(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Signalled(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::Live(record) => record.write(&v13::CEILINGS, &mut writer),
        Body::AccountRefused(record) => record.write(&v13::CEILINGS, &mut writer),
    };
    encoded.or(Err(FrameError::Incomplete))?;
    let mut frame = frame_writer(kind, length)?;
    frame.put(&writer.finish())?;
    frame.finish()
}
