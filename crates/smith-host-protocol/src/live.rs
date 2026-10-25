//! Order checks for live content (protocol/channel.md, section 5).
use smith_channel::{Live, LiveKind};
use smith_host_domain::channel::LiveAsk;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Rule {
    NotAsked,
    Turn,
    Retry,
    Offset,
    Call,
}

impl Rule {
    pub(super) const fn reason(self) -> &'static [u8] {
        match self {
            Self::NotAsked => b"live kind not asked",
            Self::Turn => b"live turn",
            Self::Retry => b"live retry",
            Self::Offset => b"live offset",
            Self::Call => b"live call",
        }
    }
}

#[derive(Debug)]
pub(super) struct Streams {
    ask: Option<LiveAsk>,
    last_turn: u32,
    ends: [Option<(u32, u64)>; 3],
    input: Option<smith_channel::LiveCall>,
}

impl Streams {
    pub(super) const fn new(ask: Option<LiveAsk>, last_turn: u32) -> Self {
        Self { ask, last_turn, ends: [None; 3], input: None }
    }

    pub(super) fn turn(&mut self, turn: u32) {
        self.last_turn = turn;
        self.ends = [None; 3];
        self.input = None;
    }

    pub(super) fn check(&mut self, piece: &Live) -> Result<(), Rule> {
        let Some(ask) = self.ask else {
            return Err(Rule::NotAsked);
        };
        let (index, asked) = match piece.kind() {
            LiveKind::Text => (0, ask.text),
            LiveKind::Reasoning => (1, ask.reasoning),
            LiveKind::Input => (2, ask.input),
        };
        if !asked {
            return Err(Rule::NotAsked);
        }
        if self.last_turn.checked_add(1) != Some(piece.turn()) {
            return Err(Rule::Turn);
        }
        if (*piece.kind() == LiveKind::Input) != piece.call().is_some() {
            return Err(Rule::Call);
        }
        let held = self.ends.get_mut(index).expect("one slot per live kind");
        if let Some((retry, end)) = *held {
            if piece.retry() < retry {
                return Err(Rule::Retry);
            }
            let same_call = *piece.kind() != LiveKind::Input || self.input.as_ref() == piece.call().as_ref();
            if piece.retry() == retry && same_call && piece.offset() < end {
                return Err(Rule::Offset);
            }
        }
        let Some(end) = piece.offset().checked_add(u64::try_from(piece.bytes().len()).expect("piece length fits u64"))
        else {
            return Err(Rule::Offset);
        };
        *held = Some((piece.retry(), end));
        if *piece.kind() == LiveKind::Input {
            self.input.clone_from(piece.call());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Rule, Streams};
    use smith_channel::{CEILINGS, Live, LiveCall, LiveCallParts, LiveKind, LiveParts};
    use smith_host_domain::channel::LiveAsk;

    fn piece(turn: u32, retry: u32, kind: LiveKind, offset: u64, length: usize, call: bool) -> Live {
        let call = if call {
            Some(
                LiveCall::new(
                    &CEILINGS,
                    LiveCallParts { id: b"call".as_slice().into(), tool: b"tool".as_slice().into() },
                )
                .unwrap(),
            )
        } else {
            None
        };
        let bytes = [b'a'; 4096];
        Live::new(
            &CEILINGS,
            LiveParts { turn, retry, kind, offset, bytes: bytes.get(..length).expect("bounded fixture").into(), call },
        )
        .unwrap()
    }

    fn text(turn: u32, retry: u32, offset: u64, length: usize) -> Live {
        piece(turn, retry, LiveKind::Text, offset, length, false)
    }

    #[test]
    fn each_order_rule_refuses_its_breach() {
        let ask = LiveAsk { text: true, reasoning: false, input: true };
        let rows: [(&str, &[Live], Option<Rule>); 10] = [
            ("in order", &[text(1, 0, 0, 3), text(1, 0, 3, 2)], None),
            ("a gap", &[text(1, 0, 0, 3), text(1, 0, 9, 2)], None),
            ("not asked", &[piece(1, 0, LiveKind::Reasoning, 0, 3, false)], Some(Rule::NotAsked)),
            ("another turn", &[text(2, 0, 0, 3)], Some(Rule::Turn)),
            ("back in the stream", &[text(1, 0, 0, 3), text(1, 0, 2, 2)], Some(Rule::Offset)),
            ("an earlier retry", &[text(1, 1, 0, 3), text(1, 0, 0, 3)], Some(Rule::Retry)),
            ("input without call", &[piece(1, 0, LiveKind::Input, 0, 3, false)], Some(Rule::Call)),
            ("call on text", &[piece(1, 0, LiveKind::Text, 0, 3, true)], Some(Rule::Call)),
            ("retry starts past zero", &[text(1, 0, 0, 3), text(1, 1, 9, 2)], None),
            ("input with call", &[piece(1, 0, LiveKind::Input, 0, 3, true)], None),
        ];
        for (case, pieces, refused) in rows {
            let mut streams = Streams::new(Some(ask), 0);
            let mut last = None;
            for piece in pieces {
                if let Err(rule) = streams.check(piece) {
                    last = Some(rule);
                    break;
                }
            }
            assert_eq!(last, refused, "{case}");
        }
    }
    #[test]
    fn each_inputs_call_starts_a_new_cursor_and_backwards_offsets_are_refused() {
        let mut streams = Streams::new(Some(LiveAsk { text: true, reasoning: false, input: true }), 0);
        let first = piece(1, 0, LiveKind::Input, 0, 3, true);
        assert_eq!(streams.check(&first), Ok(()));
        assert_eq!(streams.check(&piece(1, 0, LiveKind::Input, 2, 1, true)), Err(Rule::Offset));
        let mut second = first.into_parts();
        second.call = Some(
            LiveCall::new(
                &CEILINGS,
                LiveCallParts { id: b"second".as_slice().into(), tool: b"tool".as_slice().into() },
            )
            .unwrap(),
        );
        assert_eq!(streams.check(&Live::new(&CEILINGS, second).unwrap()), Ok(()));
        assert_eq!(streams.check(&text(1, 0, 0, 3)), Ok(()));
        assert_eq!(streams.check(&text(1, 0, 2, 1)), Err(Rule::Offset));
    }
}
