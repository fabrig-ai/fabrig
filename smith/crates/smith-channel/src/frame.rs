//! Encode and frame typed channel bodies (smith/docs/design/protocol/channel.md, sections 3–4).
use crate::{Record, kind, v11};
use skein_channel::{Frame, FrameError, frame_writer};
use skein_lib::Writer;

/// One owned channel body whose kind follows its codec.
#[derive(Debug)]
pub enum Body {
    /// The `Start` wire body.
    Start(v11::Start),
    /// The `Message` wire body.
    Message(v11::Message),
    /// The `HostAnswer` wire body.
    HostAnswer(v11::HostAnswer),
    /// The `Acknowledge` wire body.
    Acknowledge(v11::Acknowledge),
    /// The `GrantRefresh` wire body.
    GrantRefresh(v11::GrantRefresh),
    /// The `Cancel` wire body.
    Cancel(v11::Cancel),
    /// The `Drain` wire body.
    Drain(v11::Drain),
    /// The `Renewals` wire body.
    Renewals(v11::Renewals),
    /// The `Admitted` wire body.
    Admitted(v11::Admitted),
    /// The `Call` wire body.
    Call(v11::Call),
    /// The `Withdraw` wire body.
    Withdraw(v11::Withdraw),
    /// The `Turn` wire body.
    Turn(v11::Turn),
    /// The `Waiting` wire body.
    Waiting(v11::Waiting),
    /// The `Long` wire body.
    Long(v11::Long),
    /// The `LongDone` wire body.
    LongDone(v11::LongDone),
    /// The `Unauthorized` wire body.
    Unauthorized(v11::Unauthorized),
    /// The `Limited` wire body.
    Limited(v11::Limited),
    /// The `Fact` wire body.
    Fact(v11::Fact),
    /// The `Answer` wire body.
    Answer(v11::Answer),
    /// The `MessageRefused` wire body.
    MessageRefused(v11::MessageRefused),
    /// The `Live` wire body.
    Live(v11::Live),
    /// The `AccountRefused` wire body.
    AccountRefused(v11::AccountRefused),
}

/// Encode a body with its matching kind and exactly measured frame length.
pub fn frame(body: Body) -> Result<Frame, FrameError> {
    let (kind, length) = match &body {
        Body::Start(record) => (kind::START, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Message(record) => (kind::MESSAGE, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::HostAnswer(record) => (kind::HOST_ANSWER, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Acknowledge(record) => (kind::ACKNOWLEDGE, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::GrantRefresh(record) => {
            (kind::GRANT_REFRESH, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
        Body::Cancel(record) => (kind::CANCEL, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Drain(record) => (kind::DRAIN, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Renewals(record) => (kind::RENEWALS, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Admitted(record) => (kind::ADMITTED, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Call(record) => (kind::CALL, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Withdraw(record) => (kind::WITHDRAW, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Turn(record) => (kind::TURN, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Waiting(record) => (kind::WAITING, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Long(record) => (kind::LONG, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::LongDone(record) => (kind::LONG_DONE, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Unauthorized(record) => {
            (kind::UNAUTHORIZED, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
        Body::Limited(record) => (kind::LIMITED, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Fact(record) => (kind::FACT, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::Answer(record) => (kind::ANSWER, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::MessageRefused(record) => {
            (kind::MESSAGE_REFUSED, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
        Body::Live(record) => (kind::LIVE, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?),
        Body::AccountRefused(record) => {
            (kind::ACCOUNT_REFUSED, record.measure(&v11::CEILINGS).or(Err(FrameError::TooLarge))?)
        }
    };
    let mut writer = Writer::new(usize::try_from(length).or(Err(FrameError::TooLarge))?);
    let encoded = match body {
        Body::Start(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Message(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::HostAnswer(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Acknowledge(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::GrantRefresh(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Cancel(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Drain(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Renewals(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Admitted(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Call(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Withdraw(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Turn(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Waiting(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Long(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::LongDone(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Unauthorized(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Limited(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Fact(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Answer(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::MessageRefused(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::Live(record) => record.write(&v11::CEILINGS, &mut writer),
        Body::AccountRefused(record) => record.write(&v11::CEILINGS, &mut writer),
    };
    encoded.or(Err(FrameError::Incomplete))?;
    let mut frame = frame_writer(kind, length)?;
    frame.put(&writer.finish())?;
    frame.finish()
}
