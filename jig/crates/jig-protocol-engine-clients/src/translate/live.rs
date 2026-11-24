//! Whole live document facts (protocol/client/wire.md, 1, and protocol/client/README.md, 5).
use super::{chips, pieces, task, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_views as views;
use skein_lib::{List, Writer, bytes};
#[must_use]
pub fn current_call(value: &views::Call) -> w::CurrentCall {
    mapped::current_call_to_record(value)
}
pub fn writing(value: &views::Writing) -> Result<w::RunText, w::Problem> {
    Ok(w::RunText {
        writing: match &value.writing {
            Some(call) => writing_call(call)?,
            None => None,
        },
        attempt: value.turn.attempt,
        began: None,
        turn: mapped::provisional_turn_name_to_record(&value.turn),
        from: value.from,
        through: value.to,
        text: bytes::copy_of(&value.text),
        cut: value.cut,
        call: match &value.call {
            Some(call) => Some(w::LiveCall::Raw(w::LiveCallRaw { value: current_call(call) })),
            None => None,
        },
    })
}
#[expect(
    clippy::disallowed_methods,
    reason = "protocol/client/README.md 5: this protocol owner validates retained input UTF-8"
)]
pub fn writing_call(value: &views::WritingCall) -> Result<Option<w::WritingCall>, w::Problem> {
    let mut skip: usize = 0;
    if value.from != 0 {
        for byte in &value.input {
            if byte & 0xc0 != 0x80 {
                break;
            }
            skip = skip.checked_add(1).expect("input boundary");
        }
    }
    let input = value.input.get(skip..).expect("input suffix");
    let length = match core::str::from_utf8(input) {
        Ok(_) => input.len(),
        Err(error) => match error.error_len() {
            Some(_) => return Ok(None),
            None => error.valid_up_to(),
        },
    };
    let Some(from) = value.from.checked_add(u32::try_from(skip).expect("bounded input")) else {
        return Ok(None);
    };
    let Some(through) = from.checked_add(u32::try_from(length).expect("bounded input")) else {
        return Ok(None);
    };
    Ok(Some(w::WritingCall {
        id: bytes::copy_of(&value.id),
        tool: bytes::copy_of(&value.tool),
        from,
        through,
        input: bytes::copy_of(input.get(..length).expect("complete character prefix")),
        cut: value.cut,
    }))
}

pub fn run(value: &d::Run) -> Result<w::RunState, w::Problem> {
    mapped::run_state_to_record(value)
}
#[must_use]
pub fn count(value: d::Count) -> w::Count {
    let (number, limited) = match value {
        d::Count::Known(n) => (n, false),
        d::Count::AtLeast(n) => (n, true),
    };
    w::Count { value: number, limited }
}
#[must_use]
pub fn phases(value: d::PhaseCounts) -> w::PhaseCounts {
    mapped::phase_counts_to_record(&value)
}

pub fn tree_row(value: &d::TreeRow, limits: &w::Limits) -> Result<w::TreeRow, w::Problem> {
    let mut proposals = List::with_capacity(value.proposals.len());
    for row in &value.proposals {
        proposals.push(task::proposal(row)).expect("supplied proposals fit");
    }
    let mut dependencies = List::with_capacity(value.dependencies.len());
    for row in &value.dependencies {
        dependencies.push(task::link(row, limits)?).expect("supplied dependencies fit");
    }
    let dependencies =
        task::links_pieces(&dependencies, limits.tree_row_dependencies, w::paths::TREE_ROW_DEPENDENCIES, limits)?;
    Ok(w::TreeRow {
        chip: chips::chip(&value.chip, limits)?,
        parent: value.parent,
        depth: value.depth,
        proposals,
        spent: value.spent,
        dependencies,
        cut: value.cut,
    })
}
pub fn tree(value: &d::Tree, next: Option<w::Cursor>, limits: &w::Limits) -> Result<w::Tree, w::Problem> {
    let mut rows = List::with_capacity(value.rows.len());
    for row in &value.rows {
        rows.push(tree_row(row, limits)?).expect("supplied rows fit");
    }
    let rows = tree_pieces(&rows, limits.tree_rows, w::paths::TREE_ROWS, limits)?;
    let mut folded = List::with_capacity(value.folded.len());
    for row in &value.folded {
        folded.push(mapped::tree_fold_to_record(row)).expect("supplied folds fit");
    }
    Ok(w::Tree { root: chips::chip(&value.root, limits)?, rows, folded, held_below: value.held_below, next })
}

fn tree_pieces(
    rows: &List<w::TreeRow>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    pieces(&encoded, bound, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn call(from: u32, input: &[u8]) -> views::WritingCall {
        views::WritingCall {
            id: Box::from(b"call".as_slice()),
            tool: Box::from(b"tool".as_slice()),
            from,
            through: from.checked_add(u32::try_from(input.len()).expect("fixture bytes")).expect("fixture offset"),
            input: Box::from(input),
            cut: from != 0,
        }
    }
    #[test]
    fn an_incomplete_character_waits_until_the_next_piece_completes_it() {
        let first = writing_call(&call(0, b"\xc3")).expect("wire").expect("identity");
        assert_eq!((first.from, first.through, first.input.as_ref()), (0, 0, b"".as_slice()));
        let second = writing_call(&call(0, b"\xc3\xa9")).expect("wire").expect("whole character");
        assert_eq!((second.from, second.through, second.input.as_ref()), (0, 2, "é".as_bytes()));
    }
    #[test]
    fn a_trim_skips_continuation_bytes_and_invalid_utf8_omits_writing() {
        let value = writing_call(&call(1, b"\xa9x")).expect("wire").expect("suffix");
        assert_eq!((value.from, value.through, value.input.as_ref(), value.cut), (2, 3, b"x".as_slice(), true));
        assert!(writing_call(&call(0, b"\xff")).expect("wire").is_none());
    }
}
