//! Reading's histories and anchoring steps (temper/docs/design/client/web/testing.md, 3 and 6, item 14).
use skein_fake_person_browser::Step;

/// Eight replies span three windows; the long story reads them by their place.
pub const LONG_MESSAGES: u32 = 8;
/// The second-last reply anchors the position story's read above it to the first.
pub const POSITION_MESSAGES: u32 = 3;
/// Two `PageUp` keys leave the log's foot and show Latest, while the chat opens whole.
pub const LATEST_MESSAGES: u32 = 6;
pub const SHORT_START: &str = "A short conversation";
pub const SHORT_END: &str = "The whole answer.";

// The page's small-window setting holds its long_text: a thirty-second of a person's
// document_bytes, using the same window mechanism (temper/docs/design/client/web/testing.md, 3).
#[cfg(test)]
const READING_WINDOW_BYTES: usize = 32_768;
// One code line carries each reply's bytes: eight span just past three windows,
// with about two and a half replies held at once, apart from their paragraph height.
const LONG_CODE_WORDS: usize = 2_020;
// Three position replies fit one window, leaving room for its lines and their parts.
const POSITION_CODE_WORDS: usize = 1_520;

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

/// Each history's disjoint cues and replies for `Tier::reading`.
#[must_use]
pub fn replies() -> Vec<(String, String)> {
    // The last long reply and the last two position replies have the height the
    // story's PageUps cross; the shorter last long reply makes opening read its predecessor.
    const POSITION_PARAGRAPHS: u32 = 30;
    // The first position reply is short enough for the read above the anchor to reach it.
    const MIDDLE_PARAGRAPHS: u32 = 10;
    // Full long replies give the PageUp and PageDown counts height to cross at both read margins.
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

/// Offer at most eight reading replies; the rest stay cued without a suggestion.
pub(crate) fn configure(value: &mut serde_json::Value, replies: &[(String, String)]) {
    let (asked, cued) = replies.split_at(replies.len().min(8));
    let llm = &mut value["application"]["demo"]["llm"];
    llm["asked"] = scripts(asked);
    llm["cued"] = scripts(cued);
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

    fn demo_deployment() -> serde_json::Value {
        use std::num::NonZeroU16;
        let facts = temper_init::Facts {
            binary: b"/test/temper",
            git: Some(b"/usr/bin/git"),
            claude: None,
            login: None,
            sign_in: false,
            ports: temper_init::Ports {
                clients: NonZeroU16::new(8367).expect("clients port"),
                owned: NonZeroU16::new(8368).expect("owned port"),
                forge: NonZeroU16::new(8369).expect("forge port"),
                llm: NonZeroU16::new(8370).expect("LLM port"),
                webhooks: NonZeroU16::new(8371).expect("webhooks port"),
            },
        };
        let directories = temper_init::Directories { config: b"/test/config", data: b"/test/state" };
        let answer = temper_init::files(&facts, &directories, &[0; 32], &[1; 32]).expect("demo init");
        let file = answer.files.iter().find(|file| file.name == b"temper.json").expect("deployment file");
        let temper_init::Content::Bytes { bytes, .. } = &file.content else { panic!("deployment bytes") };
        serde_json::from_slice(bytes).expect("demo deployment JSON")
    }

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
        let mut value = demo_deployment();
        configure(&mut value, &replies());
        let llm = &value["application"]["demo"]["llm"];
        let complete: Vec<_> = llm["asked"]
            .as_array()
            .expect("offered scripts")
            .iter()
            .chain(llm["cued"].as_array().expect("unoffered scripts"))
            .cloned()
            .collect();
        assert_eq!(serde_json::Value::Array(complete), scripts(&replies()), "every reading reply is kept whole");
        let bytes = serde_json::to_vec(&value).expect("reading deployment JSON");
        temper_service::startup::shape(&bytes, &jig_protocol_config::Directories::working()).unwrap_or_else(
            |problems| panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems))),
        );
        let engine = crate::configuration::stock_engine(&value);
        let document = jig_protocol_config::document(
            &bytes,
            &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
        )
        .expect("reading deployment document");
        let mut root = jig_protocol_config::read::Object::new(&document, 0).expect("deployment object");
        let application = root.member(&document, b"application").expect("application");
        let mut application =
            jig_protocol_config::read::Object::new(&document, application).expect("application object");
        let demo = application.member(&document, b"demo").expect("demo");
        let directories = jig_protocol_config::Directories::given(b"/test/config", b"/test/state", b"/test/cache")
            .expect("absolute demo directories");
        temper_fake::demo::read(
            &document,
            demo,
            &temper_fake::demo::Facts { llm: &engine.llm, listeners: &[], directories: &directories },
        )
        .expect("the complete derived fake admits every history's replies")
        .expect("reading demo");
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
