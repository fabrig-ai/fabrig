//! The names that cross layer boundaries (programming-model.md, 4.2 and 4.4).

/// An opaque name for an entity of another layer, or of this layer as another
/// layer knows it.
///
/// A layer makes tokens from its own handles (`Id::token`) and turns back into
/// handles only the tokens it issued, in the record variant it issued them for.
/// The layer that holds a token stores it without interpreting it, and echoes
/// it on every record about that entity.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Token(u64);

impl Token {
    #[must_use]
    pub const fn new(raw: u64) -> Token {
        Token(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Tags a namespace in the top four bits (skein/docs/design/lib.md, 4).
    /// Refuses tags above 15 and values that need more than 60 bits.
    #[must_use]
    pub fn tagged(tag: u8, value: u64) -> Option<Token> {
        if tag > 15 || value >> 60_u32 != 0 {
            return None;
        }
        Some(Token((u64::from(tag) << 60_u32) | value))
    }

    /// The namespace in the top four bits.
    #[must_use]
    pub fn tag(&self) -> u8 {
        u8::try_from(self.0 >> 60_u32).expect("four bits fit in a u8")
    }

    /// The value below the namespace's top four bits.
    #[must_use]
    pub const fn untagged(&self) -> u64 {
        self.0 & 0x0fff_ffff_ffff_ffff
    }
}

/// The right to answer one call.
///
/// A call that comes up to a layer carries a `ReplyTo`, and the layer answers
/// with exactly one reply, which consumes it, now or later. It is neither
/// `Copy` nor `Clone`, so a second reply does not compile; the simulator
/// catches a missing one.
#[derive(PartialEq, Eq, Hash, Debug)]
#[must_use = "every call is answered exactly once"]
pub struct ReplyTo(Token);

impl ReplyTo {
    /// Issued by the layer the call came from, naming the call in its own terms.
    pub const fn new(token: Token) -> ReplyTo {
        ReplyTo(token)
    }

    /// The token of the call, for the layer that issued it, once the reply
    /// has reached it.
    #[must_use]
    pub const fn into_token(self) -> Token {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Token;

    #[test]
    fn namespaces_preserve_all_sixty_value_bits_and_refuse_overflow() {
        let max = 0x0fff_ffff_ffff_ffff;
        for tag in 0..=15 {
            for value in [0, 1, max] {
                let token = Token::tagged(tag, value).expect("tag and value fit");
                assert_eq!(token.tag(), tag);
                assert_eq!(token.untagged(), value);
                assert_eq!(Token::new(token.raw()), token);
            }
        }
        assert_eq!(Token::tagged(15, max), Some(Token::new(u64::MAX)));
        assert_eq!(Token::new(u64::MAX).tag(), 15);
        assert_eq!(Token::new(u64::MAX).untagged(), max);
        assert_eq!(Token::tagged(0, 1 << 60_u32), None);
        assert_eq!(Token::tagged(15, u64::MAX), None);
        assert_eq!(Token::tagged(16, 0), None);
        assert_eq!(Token::tagged(u8::MAX, 0), None);
    }
}
