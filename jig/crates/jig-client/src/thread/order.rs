//! Whole activity lines ordered newest first (jig/docs/design/client/domain/client/conversations.md).
use crate::contract::DecodedActivityLine;
use skein_lib::{List, Map};

pub(super) fn order(rows: &mut List<DecodedActivityLine>) {
    let mut ordered = Map::with_capacity(rows.len());
    for row in &*rows {
        if ordered.insert(row.source.position, None).expect("one entry per row fits").is_some() {
            order_repeated(rows);
            return;
        }
    }
    let incoming = core::mem::replace(rows, List::with_capacity(rows.capacity()));
    for row in incoming.into_boxed() {
        let position = row.source.position;
        *ordered.get_mut(&position).expect("every position was indexed") = Some(row);
    }
    for _ in 0..ordered.len() {
        let position = *ordered.last().expect("one indexed row remains").0;
        let row = ordered.remove(&position).expect("indexed position exists").expect("indexed row was filled");
        rows.push(row).expect("ordering keeps the row count and capacity");
    }
}

// Repeated positions keep the pairwise ordering of their supplied rows.
fn order_repeated(rows: &mut List<DecodedActivityLine>) {
    for first in 0..rows.len() {
        for second in first.saturating_add(1)..rows.len() {
            let a = rows.get(first).expect("bounded row exists");
            let b = rows.get(second).expect("bounded row exists");
            if b.source.position > a.source.position {
                let b = b.clone();
                let a = core::mem::replace(rows.get_mut(first).expect("bounded row exists"), b);
                *rows.get_mut(second).expect("bounded row exists") = a;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::order;
    use crate::contract::{Activity, ActivityLine, DecodedActivityLine, RunWhy};
    use skein_lib::{List, Wall};

    #[test]
    fn ordering_keeps_whole_lines_and_their_room() {
        for positions in [&[][..], &[1], &[3, 2, 1], &[1, 2, 3], &[2, 1, 3]] {
            let mut rows = List::with_capacity(8);
            let mut sources = List::with_capacity(rows.capacity());
            for position in positions {
                let source = Box::new(ActivityLine {
                    task: 7,
                    position: *position,
                    at: Wall::EPOCH,
                    line: Activity::RunBegan { attempt: 1, why: RunWhy::Made, resumed: false },
                });
                sources.push((core::cmp::Reverse(*position), core::ptr::from_ref(source.as_ref()))).unwrap();
                rows.push(DecodedActivityLine { source, decoded: None }).unwrap();
            }
            order(&mut rows);
            let mut sources = sources.into_boxed();
            sources.sort_unstable();
            assert_eq!(rows.capacity(), 8, "ordering retains admission room");
            assert_eq!(rows.len(), u32::try_from(positions.len()).unwrap());
            for (row, (core::cmp::Reverse(position), source)) in rows.iter().zip(sources) {
                assert_eq!(row.source.position, position, "newest first for {positions:?}");
                assert_eq!(core::ptr::from_ref(row.source.as_ref()), source, "whole line moved for {positions:?}");
            }
        }
    }

    #[test]
    fn repeated_positions_keep_every_line_in_pairwise_order() {
        let mut rows = List::with_capacity(8);
        for (task, position) in [1, 1, 2].into_iter().enumerate() {
            rows.push(DecodedActivityLine {
                source: Box::new(ActivityLine {
                    task: u64::try_from(task).unwrap(),
                    position,
                    at: Wall::EPOCH,
                    line: Activity::RunBegan { attempt: 1, why: RunWhy::Made, resumed: false },
                }),
                decoded: None,
            })
            .unwrap();
        }
        order(&mut rows);
        assert_eq!(rows.capacity(), 8);
        let mut tasks = List::with_capacity(rows.len());
        for row in &rows {
            tasks.push(row.source.task).unwrap();
        }
        assert_eq!(tasks.as_slice(), &[2, 1, 0], "repeated positions retain the pairwise swaps' order");
    }
}
