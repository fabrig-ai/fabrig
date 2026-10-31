//! Tool JSON admission and borrowed navigation.
use super::fixtures::LIMITS;
use crate::{Problem, Why, json};

#[test]
fn unused_nested_objects_refuse_decoded_duplicate_keys() {
    let input = br#"{"unused":[{"key":1,"k\u0065y":2}]}"#;
    assert_eq!(json::parse(input, &LIMITS), Err(Problem { field: b"", why: Why::Malformed }));
}

#[test]
fn every_container_keeps_its_direct_value_limit() {
    for count in [128_u32, 129] {
        for object in [false, true] {
            let mut input = skein_lib::List::with_capacity(LIMITS.input_bytes);
            append(&mut input, br#"{"unused":"#);
            append(&mut input, if object { b"{" } else { b"[" });
            for index in 0..count {
                if index != 0 {
                    append(&mut input, b",");
                }
                if object {
                    append(&mut input, b"\"");
                    append(&mut input, skein_lib::Decimal::of(u64::from(index)).as_bytes());
                    append(&mut input, b"\":");
                }
                append(&mut input, b"null");
            }
            append(&mut input, if object { b"}}" } else { b"]}" });
            let result = json::parse(input.as_slice(), &LIMITS);
            if count == 128 {
                assert!(result.is_ok());
            } else {
                assert_eq!(result, Err(Problem { field: b"", why: Why::TooLarge }));
            }
        }
    }
}

fn append(out: &mut skein_lib::List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("bounded JSON fixture");
    }
}

#[test]
fn array_walk_skips_nested_values_and_preserves_error_fields() {
    let doc = json::parse(br#"{"references":[{},[1],false,18446744073709551616,"ok"]}"#, &LIMITS).expect("JSON");
    let mut items = json::Node::root(&doc, &LIMITS).required(b"references").expect("field").array().expect("array");
    assert_eq!(items.len(), 5);
    assert!(items.next().expect("object").object());
    let mut nested = items.next().expect("nested").array().expect("array");
    assert_eq!(nested.next().expect("number").text(), Err(Problem { field: b"", why: Why::Type }));
    assert_eq!(items.next().expect("boolean").boolean(), Ok(false));
    assert_eq!(items.next().expect("number").number(), Err(Problem { field: b"references", why: Why::Range }));
    assert_eq!(items.next().expect("string").text().expect("text").as_ref(), b"ok");
    assert!(items.next().is_none());
}

#[test]
fn settings_bytes_have_their_own_type_and_size_rules() {
    let limits = crate::Limits { words: 1, ..LIMITS };
    let crate::Decoded::Settings { set, .. } =
        crate::decode(b"propose_settings", br#"{"set":"000102","against":1,"reason":"r"}"#, &limits)
    else {
        unreachable!("settings bytes")
    };
    assert_eq!(set.as_ref(), [0, 1, 2]);
    let crate::Decoded::Problem(problem) = crate::decode(b"propose_settings", br#"{"set":false}"#, &limits) else {
        unreachable!("settings type refusal")
    };
    assert_eq!(problem, Problem { field: b"set", why: Why::Type });
}

#[test]
fn root_and_number_refusals_keep_their_kind() {
    for input in [b"[]".as_slice(), b"null", b"1"] {
        assert_eq!(json::parse(input, &LIMITS), Err(Problem { field: b"", why: Why::Type }));
    }
    for input in [b"".as_slice(), b"{}{}", b"{", b"{} x"] {
        assert_eq!(json::parse(input, &LIMITS), Err(Problem { field: b"", why: Why::Malformed }));
    }
    for input in [
        br#"{"target":-1}"#.as_slice(),
        br#"{"target":1.0}"#,
        br#"{"target":1e2}"#,
        br#"{"target":18446744073709551616}"#,
    ] {
        let doc = json::parse(input, &LIMITS).expect("JSON");
        assert_eq!(
            json::Node::root(&doc, &LIMITS).required(b"target").expect("field").number(),
            Err(Problem { field: b"target", why: Why::Range })
        );
    }
}

#[test]
fn retained_tokens_are_admitted_at_the_same_bound() {
    for last in [121_u32, 122] {
        let mut input = skein_lib::List::with_capacity(LIMITS.input_bytes);
        append(&mut input, br#"{"unused":["#);
        for array in 0_u32..32 {
            if array != 0 {
                append(&mut input, b",");
            }
            append(&mut input, b"[");
            let count = if array == 31 { last } else { 126 };
            for item in 0..count {
                if item != 0 {
                    append(&mut input, b",");
                }
                append(&mut input, b"0");
            }
            append(&mut input, b"]");
        }
        append(&mut input, b"]}");
        let result = json::parse(input.as_slice(), &LIMITS);
        if last == 121 {
            assert_eq!(result.expect("4096 tokens").len(), 4096);
        } else {
            assert_eq!(result, Err(Problem { field: b"", why: Why::TooLarge }));
        }
    }
}
