//! Text tokens for the fixture's arguments and one-answer-per-line report.
use super::{Answer, Ids, Question, Report, Rule};
use core::fmt::Write;
use core::str::SplitWhitespace;
use skein_io::kernel::{Error, Rights, Unenforced};

fn path_token(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::from("-");
    }
    let mut text = String::new();
    for byte in bytes {
        text.push(char::from(skein_lib::bytes::hex_digit(*byte >> 4_u32)));
        text.push(char::from(skein_lib::bytes::hex_digit(*byte)));
    }
    text
}

fn unhex(text: &str) -> Result<Box<[u8]>, Error> {
    if text == "-" {
        return Ok(Box::new([]));
    }
    if !text.len().is_multiple_of(2) {
        return Err(Error::InvalidArgument);
    }
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let [upper, lower] = *pair;
            let upper = skein_lib::bytes::hex_value(upper).ok_or(Error::InvalidArgument)?;
            let lower = skein_lib::bytes::hex_value(lower).ok_or(Error::InvalidArgument)?;
            Ok((upper << 4_u32) | lower)
        })
        .collect()
}

fn token<'a>(tokens: &mut SplitWhitespace<'a>) -> Result<&'a str, Error> {
    tokens.next().ok_or(Error::InvalidArgument)
}
fn number(tokens: &mut SplitWhitespace<'_>) -> Result<u32, Error> {
    token(tokens)?.parse().map_err(|_| Error::InvalidArgument)
}
fn signed(tokens: &mut SplitWhitespace<'_>) -> Result<i32, Error> {
    token(tokens)?.parse().map_err(|_| Error::InvalidArgument)
}

pub(super) fn question(question: &Question) -> String {
    match question {
        Question::Read(path) => format!("read {}", path_token(path)),
        Question::Write(path) => format!("write {}", path_token(path)),
        Question::List(path) => format!("list {}", path_token(path)),
        Question::Execute(path) => format!("execute {}", path_token(path)),
        Question::SetId(path) => format!("set-id {}", path_token(path)),
        Question::NoNewPrivs => String::from("no-new-privs"),
        Question::SignalParent => String::from("signal-parent"),
        Question::Descriptors => String::from("descriptors"),
        Question::Nest { rules, questions } => {
            let mut text = format!("nest {}", rules.len());
            for rule in rules {
                write!(text, " {} {}", path_token(&rule.path), rule.rights.0).expect("writing a string");
            }
            write!(text, " {}", questions.len()).expect("writing a string");
            for next in questions {
                write!(text, " {}", self::question(next)).expect("writing a string");
            }
            text
        }
    }
}

pub(super) fn read_question(bytes: &[u8]) -> Result<Question, Error> {
    let text = core::str::from_utf8(bytes).map_err(|_| Error::InvalidArgument)?;
    let mut tokens = text.split_whitespace();
    let question = parse_question(&mut tokens)?;
    if tokens.next().is_some() {
        return Err(Error::InvalidArgument);
    }
    Ok(question)
}

fn parse_question(tokens: &mut SplitWhitespace<'_>) -> Result<Question, Error> {
    let tag = token(tokens)?;
    if tag == "read" {
        return Ok(Question::Read(unhex(token(tokens)?)?));
    }
    if tag == "write" {
        return Ok(Question::Write(unhex(token(tokens)?)?));
    }
    if tag == "list" {
        return Ok(Question::List(unhex(token(tokens)?)?));
    }
    if tag == "execute" {
        return Ok(Question::Execute(unhex(token(tokens)?)?));
    }
    if tag == "set-id" {
        return Ok(Question::SetId(unhex(token(tokens)?)?));
    }
    if tag == "no-new-privs" {
        return Ok(Question::NoNewPrivs);
    }
    if tag == "signal-parent" {
        return Ok(Question::SignalParent);
    }
    if tag == "descriptors" {
        return Ok(Question::Descriptors);
    }
    if tag != "nest" {
        return Err(Error::InvalidArgument);
    }
    let mut rules = Vec::new();
    for _ in 0..number(tokens)? {
        let path = unhex(token(tokens)?)?;
        let rights = Rights(u16::try_from(number(tokens)?).map_err(|_| Error::InvalidArgument)?);
        rules.push(Rule { path, rights });
    }
    let mut questions = Vec::new();
    for _ in 0..number(tokens)? {
        questions.push(parse_question(tokens)?);
    }
    Ok(Question::Nest { rules: rules.into_boxed_slice(), questions: questions.into_boxed_slice() })
}

pub(super) fn answer(answer: &Answer) -> String {
    match answer {
        Answer::Granted => String::from("granted"),
        Answer::Refused(error) => format!("refused {}", error_text(*error)),
        Answer::NoNewPrivs(value) => format!("no-new-privs {}", u8::from(*value)),
        Answer::Ids(Ids::Kept) => String::from("ids kept"),
        Answer::Ids(Ids::Gained) => String::from("ids gained"),
        Answer::Descriptors(descriptors) => {
            let mut text = format!("descriptors {}", descriptors.len());
            for fd in descriptors {
                write!(text, " {fd}").expect("writing a string");
            }
            text
        }
        Answer::Nested(report) => {
            let mut text = format!("nested {}", report.answers.len());
            for next in &report.answers {
                write!(text, " {}", self::answer(next)).expect("writing a string");
            }
            text
        }
    }
}

pub(super) fn read_answer(bytes: &[u8]) -> Result<Answer, Error> {
    let text = core::str::from_utf8(bytes).map_err(|_| Error::InvalidArgument)?;
    let mut tokens = text.split_whitespace();
    let answer = parse_answer(&mut tokens)?;
    if tokens.next().is_some() {
        return Err(Error::InvalidArgument);
    }
    Ok(answer)
}

fn parse_answer(tokens: &mut SplitWhitespace<'_>) -> Result<Answer, Error> {
    let tag = token(tokens)?;
    if tag == "granted" {
        return Ok(Answer::Granted);
    }
    if tag == "refused" {
        return Ok(Answer::Refused(parse_error(tokens)?));
    }
    if tag == "no-new-privs" {
        let value = number(tokens)?;
        if value > 1 {
            return Err(Error::InvalidArgument);
        }
        return Ok(Answer::NoNewPrivs(value == 1));
    }
    if tag == "ids" {
        let ids = token(tokens)?;
        if ids == "kept" {
            return Ok(Answer::Ids(Ids::Kept));
        }
        if ids == "gained" {
            return Ok(Answer::Ids(Ids::Gained));
        }
        return Err(Error::InvalidArgument);
    }
    if tag == "descriptors" {
        let mut descriptors = Vec::new();
        for _ in 0..number(tokens)? {
            descriptors.push(number(tokens)?);
        }
        return Ok(Answer::Descriptors(descriptors.into_boxed_slice()));
    }
    if tag == "nested" {
        let mut answers = Vec::new();
        for _ in 0..number(tokens)? {
            answers.push(parse_answer(tokens)?);
        }
        return Ok(Answer::Nested(Report { answers: answers.into_boxed_slice() }));
    }
    Err(Error::InvalidArgument)
}

fn error_text(error: Error) -> String {
    match error {
        Error::Reach(reason) => format!("reach {}", reach_text(reason)),
        Error::Other(errno) => format!("other {errno}"),
        Error::Busy => String::from("busy"),
        Error::Refused => String::from("refused"),
        Error::Reset => String::from("reset"),
        Error::BrokenPipe => String::from("broken-pipe"),
        Error::NotConnected => String::from("not-connected"),
        Error::AddressInUse => String::from("address-in-use"),
        Error::AddressNotAvailable => String::from("address-not-available"),
        Error::Unreachable => String::from("unreachable"),
        Error::TimedOut => String::from("timed-out"),
        Error::TooManyOpenFiles => String::from("too-many-open-files"),
        Error::NoBufferSpace => String::from("no-buffer-space"),
        Error::Cancelled => String::from("cancelled"),
        Error::TooLate => String::from("too-late"),
        Error::NotFound => String::from("not-found"),
        Error::Exists => String::from("exists"),
        Error::NotADirectory => String::from("not-a-directory"),
        Error::IsADirectory => String::from("is-a-directory"),
        Error::NotEmpty => String::from("not-empty"),
        Error::Permission => String::from("permission"),
        Error::NoSpace => String::from("no-space"),
        Error::ReadOnly => String::from("read-only"),
        Error::TooManyLinks => String::from("too-many-links"),
        Error::NameTooLong => String::from("name-too-long"),
        Error::Escape => String::from("escape"),
        Error::NotAFile => String::from("not-a-file"),
        Error::InvalidArgument => String::from("invalid-argument"),
        Error::NotALink => unreachable!("the reach probe never reads link targets"),
    }
}

fn reach_text(reason: Unenforced) -> String {
    match reason {
        Unenforced::Bounds => String::from("bounds"),
        Unenforced::Unsupported => String::from("unsupported"),
        Unenforced::Absent => String::from("absent"),
        Unenforced::Disabled => String::from("disabled"),
        Unenforced::Abi { found, needs } => format!("abi {found} {needs}"),
        Unenforced::Rule { index, errno } => format!("rule {index} {errno}"),
        Unenforced::Restrict { errno } => format!("restrict {errno}"),
        Unenforced::Helper => String::from("helper"),
    }
}

fn parse_error(tokens: &mut SplitWhitespace<'_>) -> Result<Error, Error> {
    let tag = token(tokens)?;
    if tag == "other" {
        return Ok(Error::Other(signed(tokens)?));
    }
    if tag == "reach" {
        return Ok(Error::Reach(parse_reach(tokens)?));
    }
    // The named errors are finite; this pairs their spelling with the same
    // vocabulary used by the encoder without interpreting a Debug string.
    for error in [
        Error::Busy,
        Error::Refused,
        Error::Reset,
        Error::BrokenPipe,
        Error::NotConnected,
        Error::AddressInUse,
        Error::AddressNotAvailable,
        Error::Unreachable,
        Error::TimedOut,
        Error::TooManyOpenFiles,
        Error::NoBufferSpace,
        Error::Cancelled,
        Error::TooLate,
        Error::NotFound,
        Error::Exists,
        Error::NotADirectory,
        Error::IsADirectory,
        Error::NotEmpty,
        Error::Permission,
        Error::NoSpace,
        Error::ReadOnly,
        Error::TooManyLinks,
        Error::NameTooLong,
        Error::Escape,
        Error::NotAFile,
        Error::InvalidArgument,
    ] {
        if error_text(error) == tag {
            return Ok(error);
        }
    }
    Err(Error::InvalidArgument)
}

fn parse_reach(tokens: &mut SplitWhitespace<'_>) -> Result<Unenforced, Error> {
    let tag = token(tokens)?;
    if tag == "bounds" {
        return Ok(Unenforced::Bounds);
    }
    if tag == "unsupported" {
        return Ok(Unenforced::Unsupported);
    }
    if tag == "absent" {
        return Ok(Unenforced::Absent);
    }
    if tag == "disabled" {
        return Ok(Unenforced::Disabled);
    }
    if tag == "helper" {
        return Ok(Unenforced::Helper);
    }
    if tag == "abi" {
        return Ok(Unenforced::Abi { found: number(tokens)?, needs: number(tokens)? });
    }
    if tag == "rule" {
        return Ok(Unenforced::Rule { index: number(tokens)?, errno: signed(tokens)? });
    }
    if tag == "restrict" {
        return Ok(Unenforced::Restrict { errno: signed(tokens)? });
    }
    Err(Error::InvalidArgument)
}
