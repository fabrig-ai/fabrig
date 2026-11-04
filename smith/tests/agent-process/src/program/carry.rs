//! Host-owned window transfers and activation replay on a retained fake filesystem.
//! Saved frames cross the channel; session markers stay on the fake machine.
//! No agent state is inspected (smith/docs/design/protocol/programs.md, 5.8 and 5.10).
use skein_lib::Reader;

/// Extract the transcript bodies the agent gave its host, without inspecting domains.
#[must_use]
pub fn from_frames(frames: &[skein_fake_channel::Observed]) -> Vec<Box<[u8]>> {
    frames
        .iter()
        .filter(|frame| frame.kind == 0x0109)
        .map(|frame| {
            let turn = smith_channel::Turn::decode(&smith_channel::CEILINGS, &mut Reader::new(&frame.body))
                .expect("turn envelope");
            Box::from(turn.body())
        })
        .collect()
}

/// Supply a saved window with the next run's charter and grant.
#[must_use]
pub fn start_with_window(
    charter: &[u8],
    value: &smith_channel::GrantValue,
    turns: &[Box<[u8]>],
) -> skein_channel::Frame {
    let message = super::waiting::initial(true);
    let frame = crate::start_workspace_message(charter, 8, false, message.as_ref(), value, None);
    crate::wire::edit_start(&frame, |start| {
        start.transcript = smith_protocol_translate::transcript_piece(turns).expect("saved window");
    })
}

/// Resume on the same fake filesystem, preserving Claude's session rejection.
#[must_use]
pub fn resume_children(
    old: &mut super::Run,
    story: &smith_fake_claude::Story,
    charter: &[u8],
    activation: u64,
) -> super::Run {
    let turns = from_frames(&old.frames);
    super::run_scenario(
        super::Case::Events,
        story,
        charter,
        &crate::program(b"source"),
        super::RunOptions {
            history: Some(&turns),
            files: Some(old.files.take().expect("settled fake filesystem")),
            activation,
            ..super::RunOptions::default()
        },
    )
}

/// Reopen the retained fixture and replace scenario inputs without removing sessions.
pub(super) fn machine(
    files: Option<super::Files>,
    story: &smith_fake_claude::Story,
    login: Option<&[u8]>,
    case: super::Case,
) -> (super::Files, super::Opened) {
    let Some(mut files) = files else { return super::fixture(story, login, case) };
    let base = files.reopen_root(0);
    // Only scenario inputs and observations reset; session files stay in place.
    for (path, bytes) in [
        (b"programs/source/fake-claude.story".as_slice(), story.encode()),
        (b"programs/source/fake-claude.observations".as_slice(), Vec::new()),
        (b"home/events.jsonl".as_slice(), Vec::new()),
    ] {
        let file = files.open(base, path, super::How::Write).expect("scenario file");
        files.truncate(file, 0).expect("reset observation or scenario");
        files.write(file, 0, &bytes).expect("scenario contents");
        files.close(file);
    }
    (files, base)
}

/// Restore history and supply a later activation's message and workspace authority.
pub(super) fn activation(
    parent: &mut super::Parent,
    case: super::Case,
    charter: &[u8],
    value: &smith_channel::GrantValue,
    options: &super::RunOptions<'_>,
) {
    if case != super::Case::Transcript && options.history.is_none() {
        return;
    }
    let frame = match options.history {
        Some(turns) => start_with_window(charter, value, turns),
        None => super::with_transcript(charter, value),
    };
    let frame = crate::wire::edit_start(&frame, |start| {
        if options.activation > 0 {
            start.activation = options.activation;
            start.messages = skein_lib::List::with_capacity(1);
            start
                .messages
                .push(
                    smith_channel::Message::new(
                        &smith_channel::CEILINGS,
                        smith_channel::MessageParts {
                            finish_requested: false,
                            name: options.activation,
                            label: b"host".as_slice().into(),
                            text: b"FOLLOWUP Have one sub-agent read README.md once more.".as_slice().into(),
                        },
                    )
                    .expect("follow-up message"),
                )
                .expect("one message");
            // Workspace authority is supplied again with each activation.
            let workspace = super::tools_start(charter, value);
            let start_workspace = smith_channel::Start::decode(
                &smith_channel::CEILINGS,
                &mut Reader::new(workspace.bytes().get(8..).expect("Start header")),
            )
            .expect("workspace start");
            start.workspace.clone_from(start_workspace.workspace());
        }
    });
    parent.peer = crate::host();
    parent.peer.play(frame);
}

/// Drive an inspect-only chat and deliver follow-ups while its program is alive.
#[must_use]
pub fn children_chat(story: &smith_fake_claude::Story, charter: &[u8], releases: &[super::HostMessage]) -> super::Run {
    super::run_scenario(
        super::Case::Events,
        story,
        charter,
        &crate::program(b"source"),
        super::RunOptions { login: Some(b"claude.ai"), releases: Some(releases), ..super::RunOptions::default() },
    )
}
