use skein_json::{tokenizer, writer};

/// Native codec bounds for one endpoint; each dialect receives its own value.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Tool calls the caller accepts in one completion.
    pub calls_per_response: u32,
    pub request: u32,
    pub retained: u32,
    pub strings: u32,
    pub depth: u32,
    pub tokens: u32,
    pub output_items: u32,
    pub tools: u32,
    pub history_items: u32,
    pub metadata: u32,
    pub receiving: u32,
    pub skip: u32,
    pub input: u32,
    pub reasoning: u32,
    pub answer: u32,
    pub detail_bytes: u32,
}
impl Limits {
    /// Neutral admission for a request or standalone native fixture.
    #[must_use]
    pub const fn document(&self) -> crate::DocumentLimits {
        crate::DocumentLimits { bytes: self.skip, strings: self.strings, depth: self.depth, tokens: self.tokens }
    }
    /// Caller input is bounded by the whole request, without a sent-string cap.
    #[must_use]
    pub const fn request_document(&self) -> crate::DocumentLimits {
        crate::DocumentLimits { bytes: self.request, strings: self.request, depth: self.depth, tokens: self.request }
    }
    #[must_use]
    pub const fn metadata_document(&self) -> crate::DocumentLimits {
        crate::DocumentLimits { bytes: self.metadata, strings: self.strings, depth: self.depth, tokens: self.tokens }
    }
    #[must_use]
    pub const fn reasoning_document(&self) -> crate::DocumentLimits {
        crate::DocumentLimits { bytes: self.reasoning, strings: self.strings, depth: self.depth, tokens: self.tokens }
    }
    #[must_use]
    pub const fn writer_limits(&self) -> writer::Limits {
        writer::Limits { depth: self.depth, length: self.skip }
    }
    #[must_use]
    pub const fn tokenizer_limits(&self) -> tokenizer::Limits {
        tokenizer::Limits { depth: self.depth, string: self.strings, number: 32, chunk: 256, length: self.skip }
    }
}
/// Bounds request encoding workspace.
/// Fixed scratch for writing borrowed requests (llm/README.md, section 4.2).
/// The body belongs to its reservation; nesting and grammar are the only heap.
#[must_use]
pub fn encoder_bound(limits: &Limits) -> Option<u64> {
    let writer = writer::encoder_bound(&writer::Limits { depth: limits.depth, length: 0 })?;
    let grammar = tokenizer::validation_bound(&tokenizer::Limits {
        depth: limits.depth,
        string: 0,
        number: 32,
        chunk: 256,
        length: 0,
    })?;
    writer
        .checked_mul(2)?
        .checked_add(grammar)?
        .checked_add(u64::try_from(size_of::<writer::Encoder>()).ok()?.checked_mul(2)?)?
        .checked_add(u64::try_from(size_of::<tokenizer::Tokenizer>()).ok()?)
}
