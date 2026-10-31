//! The client's blocks rendered as bounded agent-readable lines.
use alloc::boxed::Box;
use jig_client_domain::contract::{Block, Speaker, Turn};
use skein_lib::{Decimal, Writer};

pub(super) fn render(turn: &Turn, bound: u32) -> Box<[u8]> {
    let bound = skein_lib::length::usize_of(bound);
    let mut measured = Output { writer: None, left: bound };
    blocks(turn, &mut measured);
    let length = bound.checked_sub(measured.left).expect("rendered length within bound");
    let mut output = Output { writer: Some(Writer::new(length)), left: bound };
    blocks(turn, &mut output);
    output.writer.expect("render pass has its measured writer").finish()
}

fn blocks(turn: &Turn, output: &mut Output) {
    for message in &turn.messages {
        let speaker: &[u8] = match message.speaker {
            Speaker::Person => b"person: ",
            Speaker::Assistant => b"assistant: ",
            Speaker::Tool => b"tool: ",
            Speaker::System => b"system: ",
        };
        for block in &message.blocks {
            match block {
                Block::Text(text) => output.line(&[speaker, text.bytes()]),
                Block::Call(call) => output.line(&[b"call ", call.tool.bytes(), b": ", call.input.bytes()]),
                Block::Result { summary, failed, .. } => {
                    let prefix: &[u8] = if *failed { b"failed: " } else { b"result: " };
                    output.line(&[prefix, summary.bytes()]);
                }
                Block::Opaque { dialect, size } => {
                    output.line(&[
                        b"[opaque ",
                        dialect.bytes(),
                        b" block, ",
                        Decimal::of(*size).as_bytes(),
                        b" bytes]",
                    ]);
                }
            }
        }
    }
}

#[derive(Debug)]
struct Output {
    writer: Option<Writer>,
    left: usize,
}
impl Output {
    fn line(&mut self, parts: &[&[u8]]) {
        if self.left == 0 {
            return;
        }
        self.left = self.left.checked_sub(1).expect("room for newline");
        for part in parts {
            let prefix = skein_lib::bytes::utf8_prefix(part, self.left);
            let keep = prefix.len();
            if let Some(writer) = &mut self.writer {
                writer.put(prefix).expect("measured output");
            }
            self.left = self.left.checked_sub(keep).expect("remaining output budget");
            if keep < part.len() {
                break;
            }
        }
        if let Some(writer) = &mut self.writer {
            writer.put(b"\n").expect("measured newline");
        }
    }
}
