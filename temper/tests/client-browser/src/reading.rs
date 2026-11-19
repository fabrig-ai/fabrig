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
    // The first request also appears in the system's task brief. Keep each chat's
    // first cue among the eight asked scripts, so later messages choose their own replies.
    let mut replies: Vec<_> = (1..=POSITION_MESSAGES)
        .map(|number| {
            let paragraphs = if number >= POSITION_MESSAGES - 1 { POSITION_PARAGRAPHS } else { MIDDLE_PARAGRAPHS };
            let reply = reading_reply(POSITION_CODE_WORDS, paragraphs);
            (
                String::from_utf8(position_words(number).into_vec()).expect("position words"),
                format!("{reply}\n\nEnd of reply {number}."),
            )
        })
        .collect();
    replies.push(("Latest paragraph".into(), "Keep these words available while reading back. ".repeat(4)));
    replies.push((SHORT_START.into(), SHORT_END.into()));
    replies.extend((1..=LONG_MESSAGES).map(|number| {
        let paragraphs = if number < LONG_MESSAGES { WINDOW_PARAGRAPHS } else { POSITION_PARAGRAPHS };
        let reply = reading_reply(LONG_CODE_WORDS, paragraphs);
        (
            String::from_utf8(long_words(number).into_vec()).expect("long words"),
            format!("{reply}\n\nEnd of reply {number}."),
        )
    }));
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
pub(crate) fn configure(replies: &[(String, String)]) -> serde_json::Value {
    let (asked, cued) = replies.split_at(replies.len().min(8));
    serde_json::json!({"asked": scripts(asked), "cued": scripts(cued), "otherwise": ["scripted model"]})
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
        serde_json::from_slice(&temper_config_shell::init_document(false)).expect("init start")
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
        let value = demo_deployment();
        let replies = replies();
        let scripts = configure(&replies);
        let document = crate::scripted::document("127.0.0.1:8370", std::path::Path::new("/test/state/model.key"));
        let mut joined = temper_fake::llm::ScriptJoin::empty();
        joined
            .push(
                &serde_json::to_vec(&document).expect("model document"),
                &serde_json::to_vec(&scripts).expect("reading scripts"),
            )
            .expect("the product's join admits the world's scripts");
        let llm: serde_json::Value = serde_json::from_slice(&joined.into_bytes()).expect("joined scripts");
        assert_eq!(llm["asked"].as_array().expect("asked").len(), 8);
        let cued = llm["cued"].as_array().expect("cued");
        assert_eq!(cued.len(), replies.len() - 8);
        for first in [SHORT_START.as_bytes(), &position_words(1), &long_words(1)] {
            let first = std::str::from_utf8(first).expect("first request");
            assert!(
                !cued.iter().any(|script| first.contains(script["cue"].as_str().expect("cue"))),
                "a first request in the system brief must not select a cued script on later turns"
            );
        }
        let complete: Vec<_> = llm["asked"]
            .as_array()
            .expect("offered scripts")
            .iter()
            .chain(llm["cued"].as_array().expect("unoffered scripts"))
            .cloned()
            .collect();
        assert_eq!(serde_json::Value::Array(complete), super::scripts(&replies), "every reading reply is kept whole");
        crate::configuration::shape(value);
        temper_fake::llm::Document::decode_with_scripts(
            &serde_json::to_vec(&document).expect("model document"),
            Some(&serde_json::to_vec(&llm).expect("joined reading scripts")),
        )
        .expect("the standalone fake admits every history's replies");
    }

    #[test]
    fn the_scripted_provider_fits_beside_the_stock_and_box_endpoints() {
        let stock = temper_agent_fixtures::providers::stock();
        let settings = crate::scripted::settings("127.0.0.1:8370");
        let items = settings["items"].as_array().expect("settings items");
        let provider = |number| {
            items
                .iter()
                .find(|item| item["setting"]["kind"] == "provider" && item["setting"]["number"] == number)
                .expect("provider setting")["set"]
                .clone()
        };
        let anthropic = provider(2);
        let browser = provider(4);
        let endpoints = stock["endpoints"].as_array().expect("stock endpoints");
        let retained_stock = endpoints.iter().filter(|endpoint| endpoint["provider"] != "anthropic").count();
        assert_eq!(
            retained_stock
                + anthropic["endpoints"].as_array().expect("Anthropic endpoints").len()
                + browser["endpoints"].as_array().expect("world endpoints").len()
                + 1,
            4,
            "the box's endpoint and the world's fit the standard agent profile"
        );
        let mut expected = temper_box_kit::provider_setup::items(&stock)
            .into_iter()
            .find(|item| item["setting"]["number"] == 2)
            .expect("stock Anthropic")["set"]
            .clone();
        expected.as_object_mut().expect("provider").remove("direct");
        expected["endpoints"].as_array_mut().expect("endpoints").retain(|endpoint| endpoint["number"] == 2);
        assert_eq!(anthropic, expected, "only Anthropic's unused direct route changes");
        let model = &items.iter().find(|item| item["setting"]["kind"] == "model").expect("model setting")["set"];
        assert_eq!(model["number"], 8);
        assert_eq!(model["model"]["provider"], 4);
        assert_eq!(model["model"]["endpoint"], browser["endpoints"][0]["number"]);
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
