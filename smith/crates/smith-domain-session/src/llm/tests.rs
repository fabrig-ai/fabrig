use super::{Block, Decoded, NotRun, Returned, ToolCall};
use alloc::boxed::Box;

#[test]
fn block_accessors_select_only_their_variant() {
    let call = ToolCall {
        id: Box::from(&b"call"[..]),
        name: Box::from(&b"read"[..]),
        input: Box::from(&b"{}"[..]),
        call: Decoded::Historical,
        replay: None,
    };
    let cases = [
        (Block::Opaque { bytes: Box::from(&b"opaque"[..]) }, None, None),
        (Block::Refusal { text: Box::from(&b"refusal"[..]), replay: None }, None, None),
        (Block::Text { text: Box::from(&b"text"[..]), replay: None }, None, Some(&b"text"[..])),
        (Block::ToolCall(call.clone()), Some(&call), None),
        (Block::ToolResult { id: call.id.clone(), result: Returned::NotRun { why: NotRun::Stopped } }, None, None),
    ];
    for (block, expected_call, expected_text) in cases {
        assert_eq!(block.tool_call(), expected_call, "{block:?}");
        assert_eq!(block.text(), expected_text, "{block:?}");
    }
}
