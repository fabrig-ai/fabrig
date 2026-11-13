//! A run of calls and its rows (temper/docs/design/client/pages/README.md).
use crate::{
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Wall, Writer};
use skein_ui::{Builder, Key, Role, Run as TextRun, phrases::Hole};
use temper_client_domain::pages::calls::{self, Touched};

#[derive(Clone, Debug)]
struct Call<'a> {
    source: &'a c::Call,
    touched: Touched,
    summary: Option<&'a c::Text>,
}
/// A run of calls (temper/docs/design/client/pages/README.md, its calls).
#[derive(Debug)]
pub struct Run<'a> {
    calls: List<Call<'a>>,
    first: Option<Wall>,
    key: Option<Key>,
    last: Option<Wall>,
}
impl<'a> Run<'a> {
    /// Starts an empty run within the conversation window (temper/docs/design/client/pages/README.md, its calls).
    #[must_use]
    pub fn with_capacity(calls: u32) -> Self {
        Self { calls: List::with_capacity(calls), first: None, key: None, last: None }
    }
    pub(crate) fn key(&mut self, key: Key) {
        self.key.get_or_insert(key);
    }
    /// Adds an ended call at its activity time (temper/docs/design/client/pages/README.md, its calls).
    pub fn call(&mut self, call: &'a c::Call, at: Wall, since: Option<Wall>) {
        self.first.get_or_insert(since.unwrap_or(at));
        self.last = Some(at);
        self.calls
            .push(Call { source: call, summary: None, touched: calls::touched(call.tool.bytes(), call.input.bytes()) })
            .expect("window bounds run calls");
    }
    /// Whether this run holds the numbered call (temper/docs/design/client/pages/README.md, its calls).
    #[must_use]
    pub fn has_call(&self, number: u64) -> bool {
        for call in &self.calls {
            if call.source.number == number {
                return true;
            }
        }
        false
    }
    /// Adds the summary of a call whose result followed it (temper/docs/design/client/pages/README.md, its calls).
    pub fn result(&mut self, call: u64, summary: &'a c::Text) {
        for row in self.calls.iter_mut().rev() {
            if row.source.number == call {
                row.summary = Some(summary);
                return;
            }
        }
    }
    /// Whether no ended call is held (temper/docs/design/client/pages/README.md, its calls).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
    /// The run's words, emptied after reading (temper/docs/design/client/pages/README.md, its calls).
    pub fn line(
        &mut self,
        current: Option<&c::CurrentCall>,
        running: bool,
        words: &Words,
        terminal: bool,
    ) -> Option<crate::shared::RunLine> {
        if self.is_empty() && current.is_none() {
            return None;
        }
        let name = if let Some(call) = self.calls.as_slice().first() {
            if self.calls.len() == 1 { alone(call.source, words) } else { phrases(self.calls.as_slice(), words) }
        } else {
            let current = current.expect("nonempty run or current call");
            under_way(current, words, true)
        };
        let mut failed = 0_u64;
        let mut failure = None;
        for call in &self.calls {
            if call.source.failed {
                failed = failed.checked_add(1).expect("window call count");
                if failure.is_none() {
                    failure = Some(call);
                }
            }
        }
        let duration = match (self.first, self.last) {
            (Some(first), Some(last)) => time(first, last, words),
            (Some(_) | None, None) | (None, Some(_)) => None,
        };
        let description = if terminal && let Some(call) = failure {
            let error = match call.source.outcome.as_ref().or(call.summary) {
                Some(text) => text.bytes(),
                None => b"",
            };
            let end = skein_lib::bytes::find(error, b"\n").unwrap_or(error.len());
            let description = joined(&[
                &super::terminal::words::say(words, super::terminal::words::TerminalWord::Failed),
                error.get(..end).expect("failure first line").trim_ascii_end(),
            ]);
            Some(match duration {
                Some(duration) => joined(&[&description, " · ".as_bytes(), &duration]),
                None => description,
            })
        } else if running {
            match current {
                Some(call) => {
                    let description = under_way(call, words, false);
                    Some(if words.shell == Shell::Terminal {
                        joined(&[&description, "…".as_bytes()])
                    } else {
                        description
                    })
                }
                None => None,
            }
        } else if failed == 0 {
            duration
        } else {
            let description = words.say(Word::WorkFailedCount, &[Hole::Number(failed)]);
            Some(match duration {
                Some(duration) => joined(&[&description, " · ".as_bytes(), &duration]),
                None => description,
            })
        };
        self.calls.clear();
        self.first = None;
        self.key = None;
        self.last = None;
        Some(crate::shared::RunLine {
            name: cut(&name, 256),
            description: match description {
                Some(description) => Some(cut(&description, 256)),
                None => None,
            },
            failed,
            running,
        })
    }
    /// Draws its turn and empties; nothing when empty and with nothing under way.
    pub(crate) fn flush(
        &mut self,
        b: &mut Builder,
        current: Option<&c::CurrentCall>,
        running: bool,
        words: &Words,
        terminal: bool,
    ) {
        let key = self.key;
        let calls = self.calls.clone();
        let Some(line) = self.line(current, running, words, terminal) else { return };
        b.open(Role::Slot);
        b.key(key.unwrap_or(Key { owner: look::owner(), kind: 52, a: 0, b: 0 }));
        b.style(look::TURN);
        b.open(Role::Disclosure);
        b.style(look::WORK);
        b.style(look::TURN_TOOL);
        b.style(look::WORK_RUN);
        if line.running {
            b.style(look::WORK_RUNNING);
        }
        if line.failed != 0 {
            b.style(look::WORK_FAILED);
        }
        b.name(&line.name);
        if let Some(description) = line.description {
            b.description(&description);
        }
        b.open(Role::Slot);
        b.style(look::WORK);
        b.style(look::WORK_CALLS);
        for call in &calls {
            let outcome = match call.source.outcome.as_ref().or(call.summary) {
                Some(text) => Some(text.bytes()),
                None => None,
            };
            row(b, call.source.tool.bytes(), &call.touched, outcome, call.source.failed, words);
        }
        if let Some(call) = current {
            row(b, call.tool.bytes(), &calls::touched(call.tool.bytes(), call.input.bytes()), None, false, words);
        }
        b.close();
        b.close();
        b.close();
    }
}
fn phrases(calls: &[Call<'_>], words: &Words) -> Box<[u8]> {
    let mut parts = List::with_capacity(u32::try_from(calls.len()).expect("bounded calls"));
    let mut length = 0_usize;
    for (at, call) in calls.iter().enumerate() {
        let tool = call.source.tool.bytes();
        let mut earlier = false;
        for previous in calls.get(..at).expect("earlier calls") {
            if same_group(previous, call) {
                earlier = true;
                break;
            }
        }
        if earlier || see_more_beside_another(call, calls) {
            continue;
        }
        let see = see_words(tool, &call.touched);
        let table = completed_words(call.source, &call.touched);
        let mut count = 0_u64;
        for (index, other) in calls.iter().enumerate() {
            if !same_group(other, call) {
                continue;
            }
            let mut repeated = false;
            if let Some(table) = &table {
                for previous in calls.get(..index).expect("earlier calls") {
                    if same_group(previous, call) && same_argument(table.count, &previous.touched, &other.touched) {
                        repeated = true;
                        break;
                    }
                }
            }
            if let Some(see) = &see {
                for previous in calls.get(..index).expect("earlier calls") {
                    if same_group(previous, call) && see_same_argument(see, &previous.touched, &other.touched) {
                        repeated = true;
                        break;
                    }
                }
            }
            if !repeated {
                count = count.checked_add(1).expect("window calls");
            }
        }
        let part = match &see {
            Some(see) => see_counted(see, count),
            None => counted(tool, table.as_ref(), count, words),
        };
        let part = capitalized(part, parts.is_empty() && (table.is_some() || see.is_some()));
        let extra = if parts.is_empty() { 0 } else { 2 };
        let next =
            length.checked_add(extra).expect("bounded separators").checked_add(part.len()).expect("bounded phrase");
        if next > 253 {
            parts.push(skein_lib::bytes::copy_of("…".as_bytes())).expect("omission replaces next phrase");
            length = length.checked_add(3).expect("bounded ellipsis");
            break;
        }
        let part = if parts.is_empty() { part } else { joined(&[b", ", &part]) };
        length = next;
        parts.push(part).expect("one phrase per tool");
    }
    let mut out = Writer::new(length);
    for part in &parts {
        out.put(part).expect("measured phrase");
    }
    out.finish()
}
fn same_group(first: &Call<'_>, other: &Call<'_>) -> bool {
    if first.source.tool.bytes() == b"see" && other.source.tool.bytes() == b"see" {
        return match (see_words(b"see", &first.touched), see_words(b"see", &other.touched)) {
            (Some(first), Some(other)) => first.group == other.group,
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        };
    }
    first.source.tool.bytes() == other.source.tool.bytes()
        && (first.source.tool.bytes() != b"propose_settings" || first.source.failed == other.source.failed)
}
#[derive(Clone, Copy)]
enum Count {
    Calls,
    Arguments,
    Docs,
}
struct ToolWords {
    count: Count,
    one: Word,
    numbered: bool,
    many: Word,
    alone: Word,
    present: Word,
}
fn tool_words(tool: &[u8], touched: &Touched) -> Option<ToolWords> {
    use Word::{
        WorkChecked, WorkCheckedSettings, WorkCheckedSettingsCount, WorkChecking, WorkEditedArgument, WorkEditedFile,
        WorkEditedFiles, WorkEditing, WorkListedArgument, WorkListedFolder, WorkListedFolders, WorkListing,
        WorkRanArgument, WorkRanCommand, WorkRanCommands, WorkReadArgument, WorkReadDoc, WorkReadDocs, WorkReadFile,
        WorkReadFiles, WorkReadPage, WorkReading, WorkRemovedArgument, WorkRemovedFile, WorkRemovedFiles, WorkRemoving,
        WorkRunning, WorkSearched, WorkSearchedArgument, WorkSearchedFiles, WorkSearchedFilesCount, WorkSearching,
        WorkSearchingFiles, WorkWriting, WorkWroteArgument, WorkWroteFile, WorkWroteFiles,
    };
    let (count, one, many, alone, present) = match tool {
        b"read" | b"Read" => (Count::Arguments, WorkReadFile, WorkReadFiles, WorkReadArgument, WorkReading),
        b"list" | b"Glob" => (Count::Arguments, WorkListedFolder, WorkListedFolders, WorkListedArgument, WorkListing),
        b"write" | b"Write" => (Count::Arguments, WorkWroteFile, WorkWroteFiles, WorkWroteArgument, WorkWriting),
        b"edit" | b"Edit" => (Count::Arguments, WorkEditedFile, WorkEditedFiles, WorkEditedArgument, WorkEditing),
        b"remove" => (Count::Arguments, WorkRemovedFile, WorkRemovedFiles, WorkRemovedArgument, WorkRemoving),
        b"search" | b"Grep" => {
            (Count::Calls, WorkSearchedFiles, WorkSearchedFilesCount, WorkSearchedArgument, WorkSearchingFiles)
        }
        b"shell" | b"Bash" => (Count::Calls, WorkRanCommand, WorkRanCommands, WorkRanArgument, WorkRunning),
        b"read_docs" => match touched {
            Touched::Search { .. } => (Count::Docs, WorkReadDoc, WorkReadDocs, WorkSearched, WorkSearching),
            Touched::Doc { .. }
            | Touched::Inventory
            | Touched::Setting { .. }
            | Touched::SettingHistory
            | Touched::Other { .. } => (Count::Docs, WorkReadDoc, WorkReadDocs, WorkReadPage, WorkReading),
        },
        b"propose_settings" => (
            Count::Calls,
            Word::WorkProposedChange,
            Word::WorkProposedChangeCount,
            Word::WorkProposedChange,
            Word::WorkProposingChange,
        ),
        b"settings" => (Count::Calls, WorkCheckedSettings, WorkCheckedSettingsCount, WorkChecked, WorkChecking),
        _ => return None,
    };
    Some(ToolWords {
        count,
        one,
        many,
        alone,
        present,
        numbered: one != WorkCheckedSettings && one != WorkSearchedFiles && one != Word::WorkProposedChange,
    })
}
fn same_argument(count: Count, previous: &Touched, other: &Touched) -> bool {
    match (distinct_argument(count, previous), distinct_argument(count, other)) {
        (Some(previous), Some(other)) => previous == other,
        (Some(_) | None, None) | (None, Some(_)) => false,
    }
}
fn distinct_argument(count: Count, touched: &Touched) -> Option<&[u8]> {
    match count {
        Count::Calls => None,
        Count::Arguments => match touched {
            Touched::Other { argument } => argument.as_deref(),
            Touched::Doc { .. }
            | Touched::Search { .. }
            | Touched::Inventory
            | Touched::Setting { .. }
            | Touched::SettingHistory => None,
        },
        Count::Docs => match touched {
            Touched::Doc { topic, .. } => Some(topic),
            Touched::Search { .. }
            | Touched::Inventory
            | Touched::Setting { .. }
            | Touched::SettingHistory
            | Touched::Other { .. } => None,
        },
    }
}
fn counted(tool: &[u8], table: Option<&ToolWords>, count: u64, words: &Words) -> Box<[u8]> {
    match table {
        Some(table) => {
            let word = if count == 1 { table.one } else { table.many };
            if count == 1 && !table.numbered { words.say(word, &[]) } else { words.say(word, &[Hole::Number(count)]) }
        }
        None if count == 1 => skein_lib::bytes::copy_of(tool),
        None => words.say(Word::WorkRepeated, &[crate::cards::name(&cut(tool, 256)), Hole::Number(count)]),
    }
}
fn completed_words(call: &c::Call, touched: &Touched) -> Option<ToolWords> {
    let mut table = tool_words(call.tool.bytes(), touched)?;
    if call.tool.bytes() == b"propose_settings" && call.failed {
        table.one = Word::WorkProposalFailed;
        table.many = Word::WorkProposalFailedCount;
        table.alone = Word::WorkProposalFailed;
    }
    Some(table)
}
fn alone(call: &c::Call, words: &Words) -> Box<[u8]> {
    let touched = calls::touched(call.tool.bytes(), call.input.bytes());
    if let Some(see) = see_words(call.tool.bytes(), &touched) {
        return capitalized(see_alone(&see, false), true);
    }
    let Some(table) = completed_words(call, &touched) else {
        return skein_lib::bytes::copy_of(call.tool.bytes());
    };
    if call.tool.bytes() == b"propose_settings" {
        return capitalized(counted(call.tool.bytes(), Some(&table), 1, words), true);
    }
    let argument = match &touched {
        Touched::Doc { topic, .. } => topic.clone(),
        Touched::Search { .. }
        | Touched::Inventory
        | Touched::Setting { .. }
        | Touched::SettingHistory
        | Touched::Other { .. } => argument(&touched, words),
    };
    let phrase = if argument.is_empty() {
        counted(call.tool.bytes(), Some(&table), 1, words)
    } else {
        words.say(table.alone, &[Hole::Name(argument)])
    };
    capitalized(phrase, true)
}
fn under_way(call: &c::CurrentCall, words: &Words, capitalize: bool) -> Box<[u8]> {
    if call.tool.bytes() == b"propose_settings" {
        return capitalized(words.say(Word::WorkProposingChange, &[]), capitalize);
    }
    let touched = calls::touched(call.tool.bytes(), call.input.bytes());
    if let Some(see) = see_words(call.tool.bytes(), &touched) {
        return capitalized(see_alone(&see, true), capitalize);
    }
    let Some(table) = tool_words(call.tool.bytes(), &touched) else {
        return skein_lib::bytes::copy_of(call.tool.bytes());
    };
    let argument = argument(&touched, words);
    if argument.is_empty() {
        return skein_lib::bytes::copy_of(call.tool.bytes());
    }
    capitalized(words.say(table.present, &[Hole::Name(argument)]), capitalize)
}
fn time(first: Wall, last: Wall, words: &Words) -> Option<Box<[u8]>> {
    let seconds = last.as_nanos().checked_sub(first.as_nanos())?.checked_div(1_000_000_000)?;
    if seconds == 0 {
        return None;
    }
    Some(words.say(
        if seconds < 60 { Word::WorkSeconds } else { Word::WorkMinutes },
        &[Hole::Number(if seconds < 60 { seconds } else { seconds.checked_div(60)? })],
    ))
}
fn argument(touched: &Touched, words: &Words) -> Box<[u8]> {
    match touched {
        Touched::Doc { topic, from } => match from {
            Some(from) => words.say(Word::WorkFromLine, &[crate::cards::name(topic), Hole::Number(*from)]),
            None => topic.clone(),
        },
        Touched::Search { words } => words.clone(),
        Touched::Inventory => words.say(Word::WorkInventory, &[]),
        Touched::SettingHistory => words.say(Word::WorkHistory, &[]),
        Touched::Setting { kind, number } => match number {
            Some(number) => words.say(Word::WorkNumber, &[crate::cards::name(kind), Hole::Number(*number)]),
            None => kind.clone(),
        },
        Touched::Other { argument } => argument.clone().unwrap_or_default(),
    }
}
fn row(b: &mut Builder, tool: &[u8], touched: &Touched, outcome: Option<&[u8]>, failed: bool, words: &Words) {
    let tool = cut(tool, 160);
    let argument = argument(touched, words);
    let argument = cut(&argument, 160_usize.saturating_sub(tool.len()).saturating_sub(1));
    let used = tool
        .len()
        .checked_add(argument.len())
        .expect("bounded row")
        .checked_add(usize::from(!argument.is_empty()))
        .expect("row separator");
    let outcome = if failed { Some(outcome.unwrap_or(b"")) } else { outcome };
    let outcome = match outcome {
        Some(outcome) if used < 158 => {
            let line = calls::first_line(outcome);
            let line = if failed {
                words.say(Word::WorkFailedOut, &[crate::cards::name(&cut(line, 160))])
            } else {
                skein_lib::bytes::copy_of(line)
            };
            Some(cut(&line, 160_usize.saturating_sub(used).saturating_sub(2)))
        }
        Some(_) | None => None,
    };
    let name = if argument.is_empty() { tool.clone() } else { joined(&[&tool, b" ", &argument]) };
    let name = match &outcome {
        Some(outcome) => joined(&[&name, b": ", outcome]),
        None => name,
    };
    b.open(Role::Paragraph);
    b.name(&name);
    if failed {
        b.style(look::WORK_FAILED);
    }
    b.text(TextRun::Code, &tool);
    if !argument.is_empty() {
        let argument = if words.shell == Shell::Terminal { joined(&[b" ", &argument]) } else { argument };
        b.text(TextRun::Plain, &argument);
    }
    if let Some(outcome) = outcome {
        let outcome = if words.shell == Shell::Terminal { joined(&[b": ", &outcome]) } else { outcome };
        b.open(Role::Text);
        b.style(look::WORK_OUT);
        b.run(TextRun::Plain, &outcome);
        b.close();
    }
    b.close();
}
fn capitalized(mut bytes: Box<[u8]>, capitalize: bool) -> Box<[u8]> {
    if capitalize && let Some(first) = bytes.first_mut() {
        first.make_ascii_uppercase();
    }
    bytes
}
fn cut(bytes: &[u8], most: usize) -> Box<[u8]> {
    if bytes.len() <= most {
        return skein_lib::bytes::copy_of(bytes);
    }
    if most < 3 {
        return Box::default();
    }
    let mut end = most.saturating_sub(3);
    for _continuation in 0_u8..3 {
        match bytes.get(end) {
            Some(byte) if byte & 0xc0 == 0x80 => end = end.saturating_sub(1),
            Some(_) | None => break,
        }
    }
    joined(&[bytes.get(..end).expect("UTF-8 prefix"), "…".as_bytes()])
}
fn joined(parts: &[&[u8]]) -> Box<[u8]> {
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded text join");
    }
    let mut out = Writer::new(length);
    for part in parts {
        out.put(part).expect("measured text join");
    }
    out.finish()
}

// The phrase book's declarations live in words.rs. These literal phrases keep its
// fixed capacities while sharing the run's words (client/pages/README.md, its calls).
#[derive(Clone, Copy)]
enum SeeCount {
    Calls,
    Distinct,
    Docs,
}
struct SeeWords<'a> {
    group: &'a [u8],
    count: SeeCount,
    noun: Option<&'static [u8]>,
    done: &'static [u8],
    present: &'static [u8],
    argument: &'a [u8],
}
fn address(touched: &Touched) -> Option<&[u8]> {
    match touched {
        Touched::Other { argument } => argument.as_deref(),
        Touched::Doc { .. }
        | Touched::Search { .. }
        | Touched::Inventory
        | Touched::Setting { .. }
        | Touched::SettingHistory => None,
    }
}
fn first_word(words: &[u8]) -> (&[u8], &[u8]) {
    let words = words.trim_ascii();
    for (at, byte) in words.iter().enumerate() {
        if byte.is_ascii_whitespace() {
            return (words.get(..at).expect("word prefix"), words.get(at..).expect("word suffix").trim_ascii_start());
        }
    }
    (words, b"")
}
fn see_words<'a>(tool: &[u8], touched: &'a Touched) -> Option<SeeWords<'a>> {
    if tool != b"see" {
        return None;
    }
    let (group, rest) = first_word(address(touched)?);
    let (done, present): (&[u8], &[u8]) = match group {
        b"changes" => (b"listed changes", b"listing changes"),
        b"tasks" => (b"listed tasks", b"listing tasks"),
        b"goals" => (b"listed goals", b"listing goals"),
        b"inbox" => (b"checked the inbox", b"checking the inbox"),
        b"me" => (b"checked your role", b"checking your role"),
        b"project" => (b"read the project", b"reading the project"),
        b"settings" => (b"checked settings", b"checking settings"),
        b"ways" => (b"read the ways", b"reading the ways"),
        b"revisions" | b"revision" => (b"read the history", b"reading the history"),
        b"more" => (b"read on", b"reading on"),
        b"change" | b"task" | b"proposal" | b"help" | b"file" => (b"read ", b"reading "),
        _ => return None,
    };
    let mut see = SeeWords { group, count: SeeCount::Calls, noun: None, done, present, argument: b"" };
    match group {
        b"change" | b"task" => {
            let (number, _) = first_word(rest);
            if number.is_empty() {
                return None;
            }
            for byte in number {
                if !byte.is_ascii_digit() {
                    return None;
                }
            }
            // Numbers compare without their leading zeros.
            let mut start = 0;
            for (at, byte) in number.iter().enumerate() {
                start = at;
                if *byte != b'0' {
                    break;
                }
            }
            see.argument = number.get(start..).expect("number suffix");
            see.count = SeeCount::Distinct;
            see.noun = Some(if group == b"change" { b"change" } else { b"task" });
        }
        b"proposal" => {
            see.argument = first_word(rest).0;
            if see.argument.is_empty() {
                return None;
            }
            see.count = SeeCount::Distinct;
            see.noun = Some(b"proposal");
        }
        b"file" => {
            let (_, path) = first_word(rest);
            see.argument = first_word(path).0;
            if see.argument.is_empty() {
                return None;
            }
            see.count = SeeCount::Distinct;
            see.noun = Some(b"file");
        }
        b"help" => {
            see.count = SeeCount::Docs;
            see.noun = Some(b"doc");
            let (page, rest) = first_word(rest);
            if page == b"search" && !rest.is_empty() {
                see.done = b"searched the docs for ";
                see.present = b"searching the docs for ";
                see.argument = rest;
            } else if page.is_empty() {
                see.done = b"read the docs";
                see.present = b"reading the docs";
            } else {
                see.done = b"read the page ";
                see.argument = page;
            }
        }
        b"changes" | b"tasks" | b"goals" | b"inbox" | b"me" | b"project" | b"settings" | b"ways" | b"revisions"
        | b"revision" | b"more" => {}
        _ => return None,
    }
    Some(see)
}
fn see_same_argument(see: &SeeWords<'_>, previous: &Touched, other: &Touched) -> bool {
    match see.count {
        SeeCount::Calls => return false,
        SeeCount::Distinct | SeeCount::Docs => {}
    }
    match (see_words(b"see", previous), see_words(b"see", other)) {
        (Some(previous), Some(other)) => {
            let distinct = match see.count {
                SeeCount::Calls => false,
                SeeCount::Distinct => true,
                SeeCount::Docs => other.done == b"read the page ",
            };
            distinct && previous.done == other.done && previous.argument == other.argument
        }
        (Some(_) | None, None) | (None, Some(_)) => false,
    }
}
fn see_counted(see: &SeeWords<'_>, count: u64) -> Box<[u8]> {
    let number = skein_lib::Decimal::of(count);
    match see.noun {
        Some(noun) => joined(&[b"read ", number.as_bytes(), b" ", noun, if count == 1 { b"" } else { b"s" }]),
        None if count == 1 => see.done.into(),
        None => joined(&[see.done, b" ", number.as_bytes(), "×".as_bytes()]),
    }
}
fn see_alone(see: &SeeWords<'_>, present: bool) -> Box<[u8]> {
    let prefix = if present { see.present } else { see.done };
    if see.group == b"change" || see.group == b"task" || see.group == b"proposal" {
        joined(&[prefix, see.group, b" ", see.argument])
    } else {
        joined(&[prefix, see.argument])
    }
}
fn see_more_beside_another(call: &Call<'_>, calls: &[Call<'_>]) -> bool {
    let Some(see) = see_words(call.source.tool.bytes(), &call.touched) else { return false };
    if see.group != b"more" {
        return false;
    }
    for other in calls {
        if other.source.tool.bytes() == b"see" {
            match see_words(b"see", &other.touched) {
                Some(see) if see.group == b"more" => {}
                Some(_) | None => return true,
            }
        }
    }
    false
}
