//! Line-oriented presentation of the bounded account projection. Reading the
//! same projection preserves identities, omissions and digest facts in both formats.
use super::Text;
use skein_json::{Token, tokenizer};
use skein_lib::{
    Env, Intake, Queue, Time, Wall,
    stream::{Down, Up},
};

const DEPTH: usize = 32;

pub(super) fn text(bytes: &[u8], out: &mut Text) {
    let length = u32::try_from(bytes.len()).expect("bounded account projection");
    let limits = tokenizer::Limits {
        depth: u32::try_from(DEPTH).expect("projection depth fits"),
        string: length,
        number: 20,
        chunk: length.max(1),
        length,
    };
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut machine = tokenizer::Tokenizer::new(&limits);
    let mut intake = Intake::with_capacity(length.max(tokenizer::largest_demand(&limits)));
    intake.append(bytes).expect("projection fits intake");
    let mut above = Queue::with_capacity(1);
    let mut below = Queue::with_capacity(1);
    let mut demanded = None;
    let mut lines = Lines { arrays: [false; DEPTH], depth: 0, keyed: false };
    let steps = bytes.len().checked_mul(4).expect("projection steps fit").checked_add(8).expect("projection steps fit");
    for _ in 0..steps {
        match demanded.take() {
            Some(read) => match intake.meet(read) {
                Some(bytes) => tokenizer::up(&mut machine, &env, Up::Bytes(bytes), &mut above, &mut below),
                None => tokenizer::up(&mut machine, &env, Up::End, &mut above, &mut below),
            },
            None => tokenizer::down(&mut machine, &env, tokenizer::Request::Next, &mut above, &mut below),
        }
        match below.pop() {
            Some(Down::Demand { read, .. }) => demanded = Some(read),
            Some(Down::Send(_) | Down::Finish) => unreachable!("projection tokenizer only demands"),
            None => {}
        }
        match above.pop() {
            Some(tokenizer::Event::Token(token)) => lines.token(&token, out),
            Some(tokenizer::Event::Done) => return,
            Some(
                tokenizer::Event::Failed(_)
                | tokenizer::Event::Closed
                | tokenizer::Event::Long(_)
                | tokenizer::Event::Skipped(_),
            ) => unreachable!("valid bounded projection"),
            None => {}
        }
    }
    unreachable!("projection tokenizer finishes within its byte-derived steps")
}
struct Lines {
    arrays: [bool; DEPTH],
    depth: usize,
    keyed: bool,
}
impl Lines {
    fn indent(&self, out: &mut Text) {
        for _ in 1..self.depth {
            out.put(b"  ");
        }
    }
    fn prefix(&mut self, out: &mut Text) {
        if self.keyed {
            self.keyed = false;
        } else if self.depth > 0 && *self.arrays.get(self.depth.checked_sub(1).expect("parent")).expect("depth") {
            self.indent(out);
            out.put(b"- ");
        }
    }
    fn token(&mut self, token: &Token, out: &mut Text) {
        match token {
            Token::Key(key) => {
                self.indent(out);
                out.put(key);
                out.put(b": ");
                self.keyed = true;
            }
            Token::ArrayStart | Token::ObjectStart => {
                self.prefix(out);
                if self.depth > 0 {
                    out.put(b"\n");
                }
                *self.arrays.get_mut(self.depth).expect("bounded depth") = *token == Token::ArrayStart;
                self.depth = self.depth.checked_add(1).expect("bounded depth");
            }
            Token::ArrayEnd | Token::ObjectEnd => self.depth = self.depth.checked_sub(1).expect("open container"),
            Token::String(value) | Token::Number(value) => {
                self.prefix(out);
                for byte in value {
                    match *byte {
                        b'\n' => out.put(b"\\n"),
                        b'\r' => out.put(b"\\r"),
                        b'\t' => out.put(b"\\t"),
                        b'\\' => out.put(b"\\\\"),
                        byte if byte < 32 || byte == 127 => {
                            out.put(b"\\x");
                            for digit in [byte >> 4u32, byte & 15] {
                                out.put(&[if digit < 10 {
                                    b'0'.checked_add(digit).expect("digit")
                                } else {
                                    b'a'.checked_add(digit.checked_sub(10).expect("hex digit")).expect("hex")
                                }]);
                            }
                        }
                        byte => out.put(&[byte]),
                    }
                }
                out.put(b"\n");
            }
            Token::True | Token::False | Token::Null => {
                self.prefix(out);
                out.put(if *token == Token::True {
                    b"true"
                } else if *token == Token::False {
                    b"false"
                } else {
                    b"null"
                });
                out.put(b"\n");
            }
        }
    }
}
