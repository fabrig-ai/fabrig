//! The locked-screen words of a need's push (attention.md, 7.2).
//! The root composes them because it holds the core's project name.
use alloc::boxed::Box;
use skein_json::writer::{Encoder, Limits};
use skein_lib::{Decimal, Writer};

fn limits() -> Limits {
    Limits { depth: 1, length: u32::try_from(skein_push::PAYLOAD_BYTES).expect("push payload fits u32") }
}

pub(crate) fn words(project: u32, needs: u32, name: &str) -> (Box<[u8]>, Box<[u8]>) {
    let number = Decimal::of(u64::from(project));
    let mut topic = Writer::new(number.as_bytes().len().saturating_add(1));
    topic.put(b"p").expect("topic measured");
    topic.put(number.as_bytes()).expect("topic measured");
    let limits = limits();
    let mut measure = Encoder::measure(&limits);
    encode(&mut measure, project, needs, name, b"");
    let (name, suffix, length) = match measure.measured() {
        Ok(length) => (name, b"".as_slice(), length),
        Err(skein_json::writer::Refusal::TooLong) => {
            let name = shortened(project, needs, name);
            let mut measure = Encoder::measure(&limits);
            encode(&mut measure, project, needs, name, "…".as_bytes());
            (name, "…".as_bytes(), measure.measured().expect("shortened push fits"))
        }
        Err(
            skein_json::writer::Refusal::TooDeep
            | skein_json::writer::Refusal::Text
            | skein_json::writer::Refusal::Number,
        ) => {
            unreachable!("one object of UTF-8 words and unsigned numbers")
        }
    };
    let mut write = Encoder::write(length, &limits);
    encode(&mut write, project, needs, name, suffix);
    (topic.finish(), write.finish())
}

// Search whole UTF-8 prefixes, measuring their JSON spelling, including escapes.
fn shortened(project: u32, needs: u32, name: &str) -> &str {
    let mut low = 0_usize;
    let mut high = name.len().min(skein_push::PAYLOAD_BYTES);
    for _ in 0..3_u32 {
        if name.is_char_boundary(high) {
            break;
        }
        high = high.saturating_sub(1);
    }
    for _ in 0..u32::BITS {
        if low >= high {
            break;
        }
        let mut middle = low.saturating_add(high.saturating_sub(low).div_ceil(2));
        for _ in 0..3_u32 {
            if name.is_char_boundary(middle) {
                break;
            }
            middle = middle.saturating_add(1);
        }
        let prefix = name.get(..middle).expect("whole UTF-8 prefix");
        let mut measure = Encoder::measure(&limits());
        encode(&mut measure, project, needs, prefix, "…".as_bytes());
        if measure.measured().is_ok() {
            low = middle;
        } else {
            let last = prefix.chars().next_back().expect("nonempty prefix");
            high = middle.saturating_sub(last.len_utf8());
        }
    }
    name.get(..low).expect("whole UTF-8 prefix")
}

fn encode(json: &mut Encoder, project: u32, needs: u32, name: &str, suffix: &[u8]) {
    let count = Decimal::of(u64::from(needs));
    let noun = if needs == 1 { b" thing needs you in ".as_slice() } else { b" things need you in ".as_slice() };
    json.object_start();
    json.key(b"title");
    json.string(b"temper");
    json.key(b"body");
    json.string_parts(&[count.as_bytes(), noun, name.as_bytes(), suffix]);
    json.key(b"project");
    json.unsigned(u64::from(project));
    json.object_end();
}

#[cfg(test)]
mod tests {
    #![expect(clippy::disallowed_methods, reason = "assert the push payload's UTF-8 JSON text")]
    use super::*;

    #[test]
    fn words_count_needs_and_escape_only_the_project_name() {
        let (topic, payload) = words(7, 1, "NAME");
        assert_eq!(topic.as_ref(), b"p7");
        assert_eq!(payload.as_ref(), br#"{"title":"temper","body":"1 thing needs you in NAME","project":7}"#);
        let (topic, payload) = words(u32::MAX, 23, "é\"\\\n");
        assert_eq!(topic.as_ref(), b"p4294967295");
        assert_eq!(
            core::str::from_utf8(&payload).unwrap(),
            "{\"title\":\"temper\",\"body\":\"23 things need you in é\\\"\\\\\\n\",\"project\":4294967295}"
        );
    }

    #[test]
    fn a_long_name_is_cut_at_a_whole_character_with_room_for_its_json_and_ellipsis() {
        for character in ["é", "界", "🦀", "\"", "\\", "\u{1}"] {
            let name = character.repeat(skein_push::PAYLOAD_BYTES);
            let (_, payload) = words(u32::MAX, u32::MAX, &name);
            assert!(payload.len() <= skein_push::PAYLOAD_BYTES);
            let text = core::str::from_utf8(&payload).unwrap();
            assert!(text.ends_with("…\",\"project\":4294967295}"));
            assert!(payload.len() >= skein_push::PAYLOAD_BYTES - 5, "maximal prefix, even when escaped");
        }
        let overhead = words(1, 1, "").1.len();
        let fits = "x".repeat(skein_push::PAYLOAD_BYTES - overhead);
        let (_, payload) = words(1, 1, &fits);
        assert_eq!(payload.len(), skein_push::PAYLOAD_BYTES);
        assert!(!core::str::from_utf8(&payload).unwrap().contains('…'));
        let (_, payload) = words(1, 1, &(fits + "é"));
        assert_eq!(payload.len(), skein_push::PAYLOAD_BYTES);
        assert!(core::str::from_utf8(&payload).unwrap().ends_with("…\",\"project\":1}"));
    }
}
