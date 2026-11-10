//! Capped body text (skein/docs/design/forge/README.md, 4.4).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Body {
    Kept(Box<[u8]>),
    Long(u64),
}
impl Body {
    /// Reads the tokenizer's capped body string, retaining a long body's length alone.
    pub fn read(event: &skein_json::tokenizer::Event, field: &'static [u8]) -> Result<Self, crate::decode::Malformed> {
        match event {
            skein_json::tokenizer::Event::Token(skein_json::Token::String(bytes)) => Ok(Self::Kept(bytes.clone())),
            skein_json::tokenizer::Event::Long(length) => Ok(Self::Long(*length)),
            skein_json::tokenizer::Event::Failed(error) => Err(crate::decode::Malformed::Json(*error)),
            skein_json::tokenizer::Event::Token(
                skein_json::Token::ObjectStart
                | skein_json::Token::ObjectEnd
                | skein_json::Token::ArrayStart
                | skein_json::Token::ArrayEnd
                | skein_json::Token::Key(_)
                | skein_json::Token::Number(_)
                | skein_json::Token::True
                | skein_json::Token::False
                | skein_json::Token::Null,
            )
            | skein_json::tokenizer::Event::Skipped(_)
            | skein_json::tokenizer::Event::Done
            | skein_json::tokenizer::Event::Closed => Err(crate::decode::Malformed::Shape { field }),
        }
    }
}
