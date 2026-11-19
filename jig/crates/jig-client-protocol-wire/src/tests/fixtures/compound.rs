use super::*;
pub const COMPOUND_PROJECT: u32 = 3;
const COMPOUND_TASK: u64 = 8;
pub fn compound_operations() -> [Operation; 2] {
    [
        Operation::Message(
            Message { project: COMPOUND_PROJECT, task: COMPOUND_TASK, words: b"words".as_slice().into() },
        ),
        Operation::Release(
            Release {
                    project: COMPOUND_PROJECT,
                    task: COMPOUND_TASK,
                    revision: Revision { value: 4 },
                    again: None,
                },
        ),
    ]
}
