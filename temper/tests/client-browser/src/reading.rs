//! Reading's histories and anchoring steps (temper/docs/design/client/browser.md, 4.3 and 4.6, item 14).
use skein_fake_person_browser::Step;

pub const LONG_MESSAGES: u32 = 8;
pub const POSITION_MESSAGES: u32 = 3;
pub const LATEST_MESSAGES: u32 = 6;
pub const SHORT_START: &str = "A short conversation";
pub const SHORT_END: &str = "The whole answer.";

// The tier's bundle holds an eighth of a person's window, with the same mechanism.
// A person's bundle keeps the whole bound; a long chat at that size is read by hand
// (temper/docs/design/client/browser.md, 4.3).
#[cfg(test)]
const READING_WINDOW_BYTES: usize = 131_072;
const LONG_CODE_WORDS: usize = 9_600;
const POSITION_CODE_WORDS: usize = 7_000;

/// "Reading paragraph 01."
#[must_use]
pub fn long_words(number: u32) -> Box<[u8]> {
    format!("Reading paragraph {number:02}.").into_bytes().into_boxed_slice()
}

/// "Position paragraph 01."
#[must_use]
pub fn position_words(number: u32) -> Box<[u8]> {
    format!("Position paragraph {number:02}.").into_bytes().into_boxed_slice()
}

/// Latest's numbered paragraph, with four repetitions of its reading words.
#[must_use]
pub fn latest_words(number: u32) -> Box<[u8]> {
    format!("Latest paragraph {number}. {}", "Keep these words available while reading back. ".repeat(4))
        .into_bytes()
        .into_boxed_slice()
}

/// Each history's cues and replies, in `Tier::reading`'s `asked` scripts.
#[must_use]
pub fn replies() -> Vec<(String, String)> {
    // The last reply makes opening read the preceding one. An evicting window needs room at both read margins.
    const POSITION_PARAGRAPHS: u32 = 30;
    const MIDDLE_PARAGRAPHS: u32 = 10;
    const WINDOW_PARAGRAPHS: u32 = 40;
    let mut replies: Vec<_> = (1..=LONG_MESSAGES)
        .map(|number| {
            let paragraphs = if number < LONG_MESSAGES { WINDOW_PARAGRAPHS } else { POSITION_PARAGRAPHS };
            let reply = reading_reply(LONG_CODE_WORDS, paragraphs);
            (
                String::from_utf8(long_words(number).into_vec()).expect("long words"),
                format!("{reply}\n\nEnd of reply {number}."),
            )
        })
        .collect();
    replies.extend((1..=POSITION_MESSAGES).map(|number| {
        let paragraphs = if number >= POSITION_MESSAGES - 1 { POSITION_PARAGRAPHS } else { MIDDLE_PARAGRAPHS };
        let reply = reading_reply(POSITION_CODE_WORDS, paragraphs);
        (
            String::from_utf8(position_words(number).into_vec()).expect("position words"),
            format!("{reply}\n\nEnd of reply {number}."),
        )
    }));
    replies.push(("Latest paragraph".into(), "Keep these words available while reading back. ".repeat(4)));
    replies.push((SHORT_START.into(), SHORT_END.into()));
    replies
}

fn reading_reply(code_words: usize, paragraphs: u32) -> String {
    use std::fmt::Write as _;
    // A horizontally scrollable code line carries the window's bytes in one node.
    let mut reply = format!("```text\n{}\n```\n\nEnd of the large reply.", "data ".repeat(code_words));
    for line in 1..=paragraphs {
        write!(reply, "\n\nNumbered line {line:02}. Keep this paragraph available while reading.").expect("reply text");
    }
    reply
}

/// The page at the height the private stylesheet turns anchoring off at.
#[must_use]
pub fn anchoring_off() -> Step {
    Step::Viewport { width: 1280, height: 801 }
}

/// The page back at the kit's height, with the browser's anchoring.
#[must_use]
pub fn anchoring_on() -> Step {
    Step::Viewport { width: 1280, height: 800 }
}

pub(crate) fn scripts(replies: &[(String, String)]) -> serde_json::Value {
    serde_json::json!(
        replies
            .iter()
            .map(|(cue, text)| {
                serde_json::json!({"cue":cue,"turns":[{"lines":[{"text":text}],"finish":"stop","tokens":1}]})
            })
            .collect::<Vec<_>>()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_long_history_spans_three_windows_and_the_position_history_stays_inside_one() {
        let replies = replies();
        let long_bytes: usize =
            replies.iter().filter(|(cue, _)| cue.starts_with("Reading paragraph")).map(|(_, reply)| reply.len()).sum();
        let position_bytes: usize =
            replies.iter().filter(|(cue, _)| cue.starts_with("Position paragraph")).map(|(_, reply)| reply.len()).sum();
        assert!(long_bytes >= 3 * READING_WINDOW_BYTES, "the long history spans at least three windows: {long_bytes}");
        assert!(
            position_bytes < READING_WINDOW_BYTES,
            "the position history stays inside its window: {position_bytes}"
        );
    }

    /// A startup refusal would otherwise show only in the browser tier.
    #[test]
    fn the_fake_admits_every_historys_replies() {
        let document = serde_json::to_vec(&serde_json::json!({
            "listen":{"socket":"127.0.0.1:0"},"key":"/tmp/key","tls":null,
            "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],
            "asked":scripts(&replies()),"otherwise":["fallback"]
        }))
        .expect("fake document JSON");
        temper_fake::llm::Document::decode(&document).expect("the fake admits every history's replies");
    }

    /// The fake chooses by the cue that comes first in a request's words.
    #[test]
    fn each_request_cues_only_its_own_reply() {
        let replies = replies();
        let mut requests: Vec<_> =
            (1..=LONG_MESSAGES).map(|number| (long_words(number), format!("Reading paragraph {number:02}."))).collect();
        requests.extend(
            (1..=POSITION_MESSAGES).map(|number| (position_words(number), format!("Position paragraph {number:02}."))),
        );
        requests.extend((1..=LATEST_MESSAGES).map(|number| (latest_words(number), "Latest paragraph".into())));
        requests.push((SHORT_START.as_bytes().into(), SHORT_START.into()));
        for (words, expected) in requests {
            let words = std::str::from_utf8(&words).expect("request words");
            let cues: Vec<_> =
                replies.iter().filter(|(cue, _)| words.contains(cue)).map(|(cue, _)| cue.as_str()).collect();
            assert_eq!(cues, [expected.as_str()], "{words}: exactly its own cue selects the reply");
        }
    }
}
