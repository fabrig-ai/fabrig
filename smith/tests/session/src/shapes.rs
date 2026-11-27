//! The `smith/2` completion vocabulary and its per-case reach (smith/docs/design/shapes.md,
//! 2–5). A shape retains only allowlisted words, not provider or workspace data.

use std::collections::BTreeMap;
use std::io::Write as _;

use skein_world::sweep::Shapes;
use smith_domain_session::{llm, record};
use smith_domain_tools::{Call as WorkspaceCall, Exit, JobEnd, Mismatch, Outcome};

/// The version of the words and completion text the worlds write.
pub const VOCABULARY: &str = "smith/2";

macro_rules! words {
    ($(#[$meta:meta])* $name:ident, $all:ident; $($(#[$variant_meta:meta])* $variant:ident => $word:literal),+ $(,)?) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant),+
        }

        const $all: &[$name] = &[$($name::$variant),+];

        impl $name {
            /// This variant's word in the vocabulary.
            #[must_use]
            pub const fn word(self) -> &'static str {
                match self { $(Self::$variant => $word),+ }
            }
        }
    };
}

words! {
    /// Which tool the model named, without the call's input.
    Tool, TOOLS;
    /// Read a file.
    Read => "read",
    /// List paths.
    List => "list",
    /// Search files.
    Search => "search",
    /// Write a file.
    Write => "write",
    /// Edit a file.
    Edit => "edit",
    /// Remove a path.
    Remove => "remove",
    /// Run a command.
    Shell => "shell",
    /// Read or stop a held command.
    Job => "job",
    /// Read a task.
    ReadTask => "read_task",
    /// Send words, a question, or an answer.
    Message => "message",
    /// Subscribe to a task.
    Subscribe => "subscribe",
    /// Record words as read.
    MarkRead => "mark_read",
    /// Another tool the host serves.
    Delegated => "delegated",
    /// No known tool.
    Unknown => "unknown",
}

words! {
    /// The answer to a call, without its data.
    Answer, ANSWERS;
    /// A file tool or search succeeded.
    Ok => "ok",
    /// A search returned hits at its deadline.
    SearchTimedOut => "search_timed_out",
    /// A command is still running.
    Running => "running",
    /// The command exited with zero.
    Exited => "exited",
    /// The command exited with a nonzero status.
    ExitNonzero => "exit_nonzero",
    /// A signal ended the command.
    Signalled => "signalled",
    /// The command reached its deadline.
    CommandTimedOut => "command_timed_out",
    /// A stop ended the job.
    Stopped => "stopped",
    /// The owner cancelled the job or operation.
    Cancelled => "cancelled",
    /// A restart interrupted the command or wait.
    Interrupted => "interrupted",
    /// The kit did not have the tool grant.
    NotGranted => "not_granted",
    /// The path exceeded its length bound.
    TooLong => "too_long",
    /// The path escaped its workspace.
    Outside => "outside",
    /// The kit cannot write to the repository.
    ReadOnly => "read_only",
    /// The path is protected from changes.
    Protected => "protected",
    /// The path did not exist.
    NotFound => "not_found",
    /// The tool did not find a regular file where required.
    NotFile => "not_file",
    /// The tool did not find a directory where required.
    NotDirectory => "not_directory",
    /// The change would pass through a symbolic link.
    Linked => "linked",
    /// The directory to remove had entries.
    NotEmpty => "not_empty",
    /// An argument contained NUL.
    NulByte => "nul_byte",
    /// An input or file was too large for the tools.
    TooLarge => "too_large",
    /// The kit had not read the file's current version.
    NotRead => "not_read",
    /// The file's version changed before the operation completed.
    Stale => "stale",
    /// The edit's old text did not match.
    EditNoMatch => "edit_no_match",
    /// The edit's old text matched more than once.
    EditAmbiguous => "edit_ambiguous",
    /// The edit's old and new text were identical.
    EditUnchanged => "edit_unchanged",
    /// The operation's IO failed.
    Failed => "failed",
    /// The operation reached its deadline.
    TimedOut => "timed_out",
    /// The tool kit had no free operation slot.
    Busy => "busy",
    /// The call did not name an offered tool.
    UnknownTool => "unknown_tool",
    /// The call's input could not be decoded.
    Malformed => "malformed",
    /// The call itself exceeded a bound.
    CallTooLarge => "call_too_large",
    /// The model stopped before finishing the input.
    CutOff => "cut_off",
    /// The kit did not hold the named job.
    UnknownJob => "unknown_job",
    /// All the kit's job cells were held.
    JobsFull => "jobs_full",
    /// The chosen tools excluded the call.
    NotRunOutsideChoice => "not_run_outside_choice",
    /// A previous failure prevented this call from running.
    NotRunAfterFailure => "not_run_after_failure",
    /// The completion stopped without asking to run the call.
    NotRunStopped => "not_run_stopped",
    /// A withdrawal won the delegated answer's race.
    Withdrawn => "withdrawn",
    /// The call's answer came from the opener.
    Delegated => "delegated",
    /// The completion did not settle an answer to the call.
    Unanswered => "unanswered",
    /// A task was read.
    Read => "read",
    /// A message was sent.
    Sent => "sent",
    /// A subscription was added.
    Subscribed => "subscribed",
    /// Words were recorded as read.
    Marked => "marked",
    /// The caller could neither reference nor read the task as required by the tool.
    NoReference => "no_reference",
    /// The message recipient had ended.
    RecipientEnded => "recipient_ended",
    /// The question already had an answer.
    QuestionAnswered => "question_answered",
    /// No unread words could be marked.
    NotUnread => "not_unread",
    /// An application tool refused for another reason.
    Refused => "refused",
    /// No other word describes the answer.
    Unknown => "unknown",
}

words! {
    /// The kind of reference a call carries.
    Reference, REFERENCES;
    /// A workspace path.
    Path => "path",
    /// A read offset.
    Offset => "offset",
    /// A yielded command's number.
    Job => "job",
    /// A task number named by a read, message, or subscription.
    Task => "task",
    /// The question a message answers.
    Question => "question",
    /// The message whose words a call marks read.
    Message => "message",
}

words! {
    /// What the world knows of a reference when the model names it.
    Class, CLASSES;
    /// A live job, a present path, or an extant line.
    Live => "live",
    /// A path that the workspace once held.
    Stale => "stale",
    /// A job of this kit ended, and its cell remains held.
    Held => "held",
    /// A job's end has been read and its cell is free.
    Consumed => "consumed",
    /// A restored job number is from before this opening.
    BeforeOpening => "before_opening",
    /// The path is outside the workspace, or another kit issued the job.
    Foreign => "foreign",
    /// No session issued the job, or the path or line never existed.
    Unissued => "unissued",
    /// A named task had already ended.
    Ended => "ended",
    /// The call did not name a reference the world could classify.
    Unknown => "unknown",
}

/// The answer's word, or `unanswered` if the completion left the call without one.
#[must_use]
pub fn answer(returned: Option<&llm::Returned>) -> Answer {
    use llm::{NotRun, Problem, Returned};
    match returned {
        None => Answer::Unanswered,
        Some(Returned::Owned { outcome }) => owned(outcome),
        Some(Returned::Invalid { problem }) => match problem {
            Problem::UnknownTool => Answer::UnknownTool,
            Problem::Nesting
            | Problem::NotAnObject
            | Problem::Missing { .. }
            | Problem::WrongType { .. }
            | Problem::BadValue { .. } => Answer::Malformed,
            Problem::TooLarge | Problem::Oversize { .. } => Answer::CallTooLarge,
            Problem::CutOff { .. } => Answer::CutOff,
            Problem::UnknownJob { .. } => Answer::UnknownJob,
            Problem::JobsFull { .. } => Answer::JobsFull,
        },
        Some(Returned::NotRun { why }) => match why {
            NotRun::OutsideChoice { .. } => Answer::NotRunOutsideChoice,
            NotRun::AfterFailedWrite { .. } => Answer::NotRunAfterFailure,
            NotRun::Stopped => Answer::NotRunStopped,
        },
        Some(Returned::Text { .. }) => Answer::Delegated,
        Some(Returned::Withdrawn) => Answer::Withdrawn,
    }
}

fn owned(outcome: &Outcome) -> Answer {
    match outcome {
        Outcome::Read { .. }
        | Outcome::Listed { .. }
        | Outcome::Found { timed_out: false, .. }
        | Outcome::Written { .. }
        | Outcome::Edited { .. }
        | Outcome::Removed => Answer::Ok,
        Outcome::Found { timed_out: true, .. } => Answer::SearchTimedOut,
        Outcome::Running { .. } | Outcome::Job { end: JobEnd::Running, .. } => Answer::Running,
        Outcome::Exited { exit, .. } | Outcome::Job { end: JobEnd::Ended(exit), .. } => match exit {
            Exit::Code { code: 0 } => Answer::Exited,
            Exit::Code { .. } => Answer::ExitNonzero,
            Exit::Signal { .. } => Answer::Signalled,
            Exit::TimedOut => Answer::CommandTimedOut,
        },
        Outcome::Job { end: JobEnd::Stopped, .. } => Answer::Stopped,
        Outcome::Cancelled | Outcome::Job { end: JobEnd::Cancelled, .. } => Answer::Cancelled,
        Outcome::Interrupted { .. } => Answer::Interrupted,
        Outcome::NotGranted => Answer::NotGranted,
        Outcome::TooLong => Answer::TooLong,
        Outcome::Outside { .. } => Answer::Outside,
        Outcome::ReadOnly { .. } => Answer::ReadOnly,
        Outcome::Protected => Answer::Protected,
        Outcome::NotFound { .. } => Answer::NotFound,
        Outcome::NotFile => Answer::NotFile,
        Outcome::NotDirectory { .. } => Answer::NotDirectory,
        Outcome::Linked => Answer::Linked,
        Outcome::NotEmpty => Answer::NotEmpty,
        Outcome::NulByte => Answer::NulByte,
        Outcome::TooLarge { .. } => Answer::TooLarge,
        Outcome::NotRead => Answer::NotRead,
        Outcome::Stale => Answer::Stale,
        Outcome::EditFailed { why, .. } => match why {
            Mismatch::NoMatch => Answer::EditNoMatch,
            Mismatch::Ambiguous { .. } => Answer::EditAmbiguous,
            Mismatch::Unchanged => Answer::EditUnchanged,
        },
        Outcome::Failed { .. } => Answer::Failed,
        Outcome::TimedOut => Answer::TimedOut,
        Outcome::Busy => Answer::Busy,
    }
}

/// One call in a completion's shape.
pub struct Call {
    pub tool: Tool,
    pub answer: Answer,
    /// In path, offset, job, task, question, message order, omitting absent references.
    pub references: Vec<(Reference, Class)>,
}

/// What one case reached, with each completion's shape in visitation order.
#[derive(Debug, PartialEq, Eq)]
pub struct Reached {
    shapes: Vec<String>,
    answers: [bool; ANSWERS.len()],
    classes: [[bool; CLASSES.len()]; REFERENCES.len()],
}

impl Reached {
    /// An empty case.
    #[must_use]
    pub fn new() -> Reached {
        Reached { shapes: Vec::new(), answers: [false; ANSWERS.len()], classes: Default::default() }
    }

    /// Adds one completion, with its calls in order, or none for text alone.
    pub fn completion(&mut self, calls: &[Call]) {
        self.completion_refs(calls.iter().map(|call| (call.tool, call.answer, call.references.iter().copied())));
    }

    fn completion_refs(
        &mut self,
        calls: impl IntoIterator<Item = (Tool, Answer, impl IntoIterator<Item = (Reference, Class)>)>,
    ) {
        let mut shape = String::with_capacity(64);
        for (tool, answer, references) in calls {
            if !shape.is_empty() {
                shape.push(',');
            }
            shape.push_str(tool.word());
            shape.push(':');
            shape.push_str(answer.word());
            self.answers[answer as usize] = true;
            for (reference, class) in references {
                shape.push(':');
                shape.push_str(reference.word());
                shape.push('=');
                shape.push_str(class.word());
                self.classes[reference as usize][class as usize] = true;
            }
        }
        if shape.is_empty() {
            shape.push_str("text");
        }
        shape.push_str(" end=unknown begin=unknown wake=unknown");
        self.shapes.push(shape);
    }

    /// Adds each assistant message's shape from one conversation's recorded turns.
    /// Results can follow in a later turn. References are classed `Unknown`.
    pub fn turns(&mut self, turns: &[record::Turn]) {
        let first = self.shapes.len();
        let mut results = BTreeMap::<&[u8], &llm::Returned>::new();
        // Walk back from results to calls, consuming each result at its call
        // so a reused provider id doesn't answer an earlier completion too.
        for message in turns.iter().rev().flat_map(|turn| turn.messages.iter().rev()) {
            match message.role {
                llm::Role::User => {
                    for block in message.content.iter().rev() {
                        if let llm::Block::ToolResult { id, result } = block {
                            results.insert(id, result);
                        }
                    }
                }
                llm::Role::Assistant => {
                    self.completion_refs(completion_tools(&message.content).map(|(position, tool)| {
                        let call = message.content[position].tool_call().expect("a tool block");
                        let returned = results.remove(call.id.as_ref());
                        let answer = answer(returned);
                        let tool = historical_tool(call, tool, answer);
                        (tool, answer, unknown_references(call, tool))
                    }));
                }
            }
        }
        self.shapes[first..].reverse();
    }

    /// The case reached this answer in at least one completion.
    #[must_use]
    pub fn answered(&self, answer: Answer) -> bool {
        self.answers[answer as usize]
    }

    /// The case reached this reference and class in at least one completion.
    #[must_use]
    pub fn classed(&self, reference: Reference, class: Class) -> bool {
        self.classes[reference as usize][class as usize]
    }

    /// The case reached this completion shape.
    #[must_use]
    pub fn contains(&self, shape: &str) -> bool {
        self.shapes.iter().any(|reached| reached == shape)
    }

    pub(crate) fn recorded(
        &mut self,
        turn: &record::Turn,
        tools: impl IntoIterator<Item = (usize, Tool)>,
        mut references: impl FnMut(usize) -> [Option<Class>; 3],
    ) {
        let assistant = turn.messages.iter().position(|message| message.role == llm::Role::Assistant);
        let calls = tools.into_iter().map(|(position, tool)| {
            // A compaction records only the new window, not the model's calls or results.
            let returned = assistant.and_then(|index| {
                let call = turn.messages[index].content[position].tool_call().expect("the recorded call's position");
                recorded_result(&turn.messages[index + 1..], &call.id)
            });
            let references = REFERENCES
                .iter()
                .copied()
                .zip(references(position))
                .filter_map(|(reference, class)| Some((reference, class?)));
            (tool, answer(returned), references)
        });
        self.completion_refs(calls);
    }
}

impl Default for Reached {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn completion_tools(blocks: &[llm::Block]) -> impl Iterator<Item = (usize, Tool)> + '_ {
    blocks.iter().enumerate().filter_map(|(position, block)| {
        let call = block.tool_call()?;
        let tool = match &call.call {
            llm::Decoded::Delegated { .. } | llm::Decoded::Invalid { descriptor: Some(_), .. } => Tool::Delegated,
            llm::Decoded::Invalid { problem: llm::Problem::UnknownTool, .. } => Tool::Unknown,
            llm::Decoded::Owned { .. }
            | llm::Decoded::Invalid { .. }
            | llm::Decoded::Builtin { .. }
            | llm::Decoded::Historical => TOOLS
                .iter()
                .copied()
                .find(|tool| {
                    !matches!(tool, Tool::Delegated | Tool::Unknown) && tool.word().as_bytes() == call.name.as_ref()
                })
                .unwrap_or(Tool::Unknown),
        };
        Some((position, tool))
    })
}

// Concrete transcript turns retain names and results but erase the live decoder's tickets.
fn historical_tool(call: &llm::ToolCall, tool: Tool, answer: Answer) -> Tool {
    if call.call != llm::Decoded::Historical {
        return tool;
    }
    match answer {
        Answer::Delegated | Answer::Withdrawn => Tool::Delegated,
        Answer::UnknownTool => Tool::Unknown,
        _ if matches!(&*call.name, b"wait" | b"save" | b"sub_agent" | b"deliver" | b"finish") => Tool::Delegated,
        _ => tool,
    }
}

fn unknown_references(call: &llm::ToolCall, tool: Tool) -> impl Iterator<Item = (Reference, Class)> {
    let known = match tool {
        Tool::Delegated | Tool::Unknown | Tool::Shell => [false; 3],
        _ => {
            let decoded;
            let owned = if let llm::Decoded::Owned { call } = &call.call {
                Some(call)
            } else {
                decoded = crate::translate::decode(&call.name, &call.input);
                if let llm::Decoded::Owned { call } = &decoded { Some(call) } else { None }
            };
            match owned {
                Some(WorkspaceCall::Read { offset, .. }) => [true, *offset != 1 || path_fields(&call.input).1, false],
                Some(WorkspaceCall::Search { .. }) => [path_fields(&call.input).0, false, false],
                Some(
                    WorkspaceCall::List { .. }
                    | WorkspaceCall::Write { .. }
                    | WorkspaceCall::Edit { .. }
                    | WorkspaceCall::Remove { .. },
                ) => [true, false, false],
                Some(WorkspaceCall::Job { .. }) => [false, false, true],
                Some(WorkspaceCall::Shell { .. }) | None => [false; 3],
            }
        }
    };
    REFERENCES
        .iter()
        .copied()
        .zip(known)
        .filter_map(|(reference, present)| present.then_some((reference, Class::Unknown)))
}

// Optional typed fields have lost their presence bit. Read it from the original object.
pub(crate) fn path_fields(input: &[u8]) -> (bool, bool) {
    let client = skein_llm_kit::limits();
    let bytes = u32::try_from(input.len()).unwrap_or(u32::MAX).min(client.input);
    let bounds = skein_llm::DocumentLimits { bytes, strings: bytes, tokens: bytes, depth: client.depth };
    skein_llm::Json::from_bytes(input, &bounds).map_or((false, false), |json| {
        let doc = json.document();
        (doc.member(0, b"path").is_some(), doc.member(0, b"offset").is_some())
    })
}

// The caller scopes the search to one turn segment, which may reuse another's provider id.
pub(crate) fn recorded_result<'a>(messages: &'a [llm::Message], id: &[u8]) -> Option<&'a llm::Returned> {
    messages.iter().flat_map(|message| &message.content).find_map(|block| match block {
        llm::Block::ToolResult { id: result_id, result } if result_id.as_ref() == id => Some(result),
        _ => None,
    })
}

/// Counts the shapes, answers and reference classes a sweep's cases reached.
pub struct Sample {
    sweep: &'static str,
    shapes: Shapes,
    answers: [u64; ANSWERS.len()],
    classes: [[u64; CLASSES.len()]; REFERENCES.len()],
}

impl Sample {
    /// An empty sweep sample in `smith/2`.
    #[must_use]
    pub fn new(sweep: &'static str) -> Sample {
        Sample {
            sweep,
            shapes: Shapes::new(sweep, VOCABULARY),
            answers: [0; ANSWERS.len()],
            classes: Default::default(),
        }
    }

    /// Folds a case, counting each shape, answer and reference class in it once.
    ///
    /// # Errors
    /// Refuses a case if any shape is not in skein's permitted form.
    pub fn ran(&mut self, case: &Reached) -> Result<(), String> {
        let shapes: Vec<_> = case.shapes.iter().map(String::as_str).collect();
        self.shapes.ran(&shapes)?;
        for &answer in ANSWERS {
            self.answers[answer as usize] += u64::from(case.answered(answer));
        }
        for &reference in REFERENCES {
            for &class in CLASSES {
                self.classes[reference as usize][class as usize] += u64::from(case.classed(reference, class));
            }
        }
        Ok(())
    }

    /// The cases folded, including those with no completion.
    #[must_use]
    pub fn cases(&self) -> u64 {
        self.shapes.cases()
    }

    /// The cases reaching this answer.
    #[must_use]
    pub fn answered(&self, answer: Answer) -> u64 {
        self.answers[answer as usize]
    }

    /// The cases reaching this reference and class.
    #[must_use]
    pub fn classed(&self, reference: Reference, class: Class) -> u64 {
        self.classes[reference as usize][class as usize]
    }

    /// Writes skein's lines and then the answer and class counts, including zeroes.
    pub fn write(&self) {
        self.shapes.write();
        let sweep = self.sweep;
        let mut stderr = std::io::stderr().lock();
        for &answer in ANSWERS {
            let _written = writeln!(stderr, "shapes answer {sweep} {} {}", answer.word(), self.answered(answer));
        }
        for &reference in REFERENCES {
            for &class in CLASSES {
                let _written = writeln!(
                    stderr,
                    "shapes class {sweep} {} {} {}",
                    reference.word(),
                    class.word(),
                    self.classed(reference, class)
                );
            }
        }
    }
}
