//! Bounded object decoding (skein/docs/design/data/json.md, 3.3 and 5.3).

use crate::Problem;
use crate::json::{Value, parse};

// The parser admits 16 open containers and 65,536 input bytes.
const DEPTH: u32 = 16;
const INPUT_BYTES: usize = 65_536;
const TOO_MANY_BYTES: usize = 65_537;

fn nested(depth: u32) -> (Box<[u8]>, Value) {
    let length = usize::try_from(depth).unwrap().checked_mul(6).unwrap().checked_sub(4).unwrap();
    let mut input = skein_lib::Writer::new(length);
    let mut value = Value::Object(Box::new([]));
    for _ in 1..depth {
        input.put(br#"{"a":"#).unwrap();
        value = Value::Object(Box::new([(b"a".as_slice().into(), value)]));
    }
    input.put(b"{}").unwrap();
    for _ in 1..depth {
        input.put(b"}").unwrap();
    }
    (input.finish(), value)
}

#[test]
fn bounded_json_objects_keep_values_and_refuse_bad_inputs() {
    let (deepest, deep_value) = nested(DEPTH);
    let (too_deep, _) = nested(DEPTH.checked_add(1).unwrap());
    let mut longest = skein_lib::bytes::zeroed(INPUT_BYTES);
    longest.fill(b' ');
    longest[0] = b'{';
    longest[1] = b'}';
    let oversized = skein_lib::bytes::zeroed(TOO_MANY_BYTES);
    for (input, expected) in [
        (b"{}".as_slice(), Ok(Value::Object(Box::new([])))),
        (br#"{"a":1}"#, Ok(Value::Object(Box::new([(b"a".as_slice().into(), Value::Number(b"1".as_slice().into()))])))),
        (
            br#"{"a":[{},[true,false,null,"\u0041"]]}"#,
            Ok(Value::Object(Box::new([(
                b"a".as_slice().into(),
                Value::Array(Box::new([
                    Value::Object(Box::new([])),
                    Value::Array(Box::new([
                        Value::Boolean(true),
                        Value::Boolean(false),
                        Value::Null,
                        Value::String(b"A".as_slice().into()),
                    ])),
                ])),
            )]))),
        ),
        (
            br#"{"n":-123.4500e+06}"#,
            Ok(Value::Object(Box::new([(b"n".as_slice().into(), Value::Number(b"-123.4500e+06".as_slice().into()))]))),
        ),
        (
            br#"{"n":123456789012345678901234}"#,
            Ok(Value::Object(Box::new([(
                b"n".as_slice().into(),
                Value::Number(b"123456789012345678901234".as_slice().into()),
            )]))),
        ),
        (&deepest, Ok(deep_value)),
        (&longest, Ok(Value::Object(Box::new([])))),
        (b"", Err(Problem::Malformed)),
        (b"{", Err(Problem::Malformed)),
        (br#"{"a":1}{}"#, Err(Problem::Malformed)),
        (br#"{"a":1,"a":2}"#, Err(Problem::Malformed)),
        (br#"{"a":1,"\u0061":2}"#, Err(Problem::Malformed)),
        (br#"{"a":{"b":1,"b":2}}"#, Err(Problem::Malformed)),
        (br#"{"n":1234567890123456789012345}"#, Err(Problem::Malformed)),
        (&too_deep, Err(Problem::Malformed)),
        (b"[]", Err(Problem::Type(b"input"))),
        (b"1", Err(Problem::Type(b"input"))),
        (br#""a""#, Err(Problem::Type(b"input"))),
        (&oversized, Err(Problem::TooLarge)),
    ] {
        assert_eq!(parse(input), expected, "input: {input:?}");
    }
}
