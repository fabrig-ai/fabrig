//! Keyed views of the independently observed durable receipts.
//! Contract: jig/docs/design/protocol/channel/testing.md, 1.
use super::{Store, core, root};

impl Store {
    pub(crate) fn core_rows(
        &self,
        first: core::CoreKey,
        last: core::CoreKey,
    ) -> impl Iterator<Item = &core::CoreRecord> {
        self.rows.range(root::Key::Core(core::Key::Core(first))..=root::Key::Core(core::Key::Core(last))).filter_map(
            |(_, row)| match row {
                root::Record::Core(core::Record::Core(row)) => Some(row),
                root::Record::Core(_) | root::Record::Connector { .. } => None,
            },
        )
    }

    pub(crate) fn turns(&self) -> impl Iterator<Item = &core::TurnRecord> {
        self.core_rows(
            core::CoreKey::Turn { task: 0, attempt: 0, turn: 0 },
            core::CoreKey::Turn { task: u64::MAX, attempt: u64::MAX, turn: u32::MAX },
        )
        .filter_map(|row| match row {
            core::CoreRecord::Turn(turn) => Some(turn),
            _ => None,
        })
    }

    pub(crate) fn terminals(&self) -> impl Iterator<Item = &core::TerminalRecord> {
        self.core_rows(
            core::CoreKey::Terminal { task: 0, attempt: 0 },
            core::CoreKey::Terminal { task: u64::MAX, attempt: u64::MAX },
        )
        .filter_map(|row| match row {
            core::CoreRecord::Terminal(terminal) => Some(terminal),
            _ => None,
        })
    }

    pub(crate) fn parts(&self) -> impl Iterator<Item = &core::PartRecord> {
        self.core_rows(
            core::CoreKey::Part(core::PartKey { task: 0, attempt: 0, number: 0 }),
            core::CoreKey::Part(core::PartKey { task: u64::MAX, attempt: u64::MAX, number: u32::MAX }),
        )
        .filter_map(|row| match row {
            core::CoreRecord::Part(part) => Some(part),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::Wall;

    fn turn(task: u64, attempt: u64, turn: u32) -> core::CoreRecord {
        core::CoreRecord::Turn(core::TurnRecord {
            task,
            attempt,
            turn,
            opens_window: false,
            spent: 0,
            read: None,
            at: Wall::EPOCH,
            transcript: Box::default(),
        })
    }

    fn terminal(task: u64, attempt: u64) -> core::CoreRecord {
        core::CoreRecord::Terminal(core::TerminalRecord {
            task,
            attempt,
            cumulative: 0,
            end: jig_domain_tasks::End::Parked,
        })
    }

    fn part(task: u64, attempt: u64, number: u32) -> core::CoreRecord {
        core::CoreRecord::Part(core::PartRecord {
            key: core::PartKey { task, attempt, number },
            turn: 1,
            thread: 0,
            call: 0,
            kind: jig_domain_link::PartKind::Answer,
            body: Box::default(),
        })
    }

    #[test]
    fn receipt_views_keep_key_order_and_live_changes_without_snapshots() {
        let mut store = Store::new();
        let middle = (7, 2, 3);
        let last = (u64::MAX, u64::MAX, u32::MAX);
        for (task, attempt, number) in [last, middle, (0, 0, 0)] {
            for row in [turn(task, attempt, number), terminal(task, attempt), part(task, attempt, number)] {
                let row = root::Record::Core(core::Record::Core(row));
                store.rows.insert(row.key(), row);
            }
        }
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let turns = store.turns().map(|turn| (turn.task, turn.attempt, turn.turn));
        assert!(turns.eq([(0, 0, 0), middle, last]));
        let terminals = store.terminals().map(|end| (end.task, end.attempt));
        assert!(terminals.eq([(0, 0), (middle.0, middle.1), (last.0, last.1)]));
        let parts = store.parts().map(|part| (part.key.task, part.key.attempt, part.key.number));
        assert!(parts.eq([(0, 0, 0), middle, last]));
        let measured = meter.end();
        assert_eq!(measured.peak(), 0, "receipt queries borrow rows, without collecting snapshots");

        store.rows.remove(&root::Key::Core(core::Key::Core(core::CoreKey::Turn {
            task: middle.0,
            attempt: middle.1,
            turn: middle.2,
        })));
        let row = root::Record::Core(core::Record::Core(turn(7, 2, 4)));
        store.rows.insert(row.key(), row);
        assert!(store.turns().map(|turn| turn.turn).eq([0, 4, u32::MAX]));
        store.recovering();
        assert!(store.turns().next().is_none());
        assert!(store.terminals().next().is_none());
        assert!(store.parts().next().is_none());
    }
}
