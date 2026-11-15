use super::fixtures;
use crate::{jobs, keys, records};
use jig_protocol_store::{JobPage, JobRoom, JobRow, JobWrite};
use temper_engine_domain_forge_vocabulary::{ChangeRow, Key, Stored};

fn row(task: u64) -> JobRow {
    let mut change = fixtures::change_row(temper_engine_domain_forge_change::State::Producing { requested: false });
    change.task = task;
    change.change.task = task;
    JobRow {
        key: keys::encode(&Key::Change(task), u32::MAX).unwrap(),
        value: records::encode(&Stored::Change(Box::new(change))),
    }
}
fn page(rows: Box<[JobRow]>, writes: u32, bytes: u32) -> JobPage {
    JobPage { end: rows.is_empty(), rows, layout: None, room: JobRoom { writes, bytes: u64::from(bytes) } }
}

#[test]
fn change_parts_job_moves_rows_and_consumes_only_whole_changes_within_the_edge() {
    let input = page(Box::new([row(12), row(13)]), 6, u32::MAX);
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let answer = jobs::page("change-parts", &input).unwrap();
    assert_eq!(answer.through, None);
    assert_eq!(answer.writes.len(), 6);
    let mut total = 0;
    for (writes, old) in answer.writes.as_chunks::<3>().0.iter().zip(&input.rows) {
        let [
            JobWrite::Put { key: cold_key, value: cold },
            JobWrite::Put { key: hot_key, value: hot },
            JobWrite::Erase { key },
        ] = writes
        else {
            panic!("two parts and erase")
        };
        let Stored::ChangeCold(cold) = records::row(cold_key, cold, &bounds).unwrap() else { panic!("cold") };
        let Stored::ChangeHot(hot) = records::row(hot_key, hot, &bounds).unwrap() else { panic!("hot") };
        let Stored::Change(whole) = records::row(&old.key, &old.value, &bounds).unwrap() else { panic!("whole") };
        assert_eq!(ChangeRow::join(*cold, *hot), *whole);
        assert_eq!(*key, old.key);
        for write in writes {
            total += match write {
                JobWrite::Put { key, value } => {
                    skein_kv::put_bytes(u32::try_from(key.len()).unwrap(), u32::try_from(value.len()).unwrap()).unwrap()
                }
                JobWrite::Erase { key } => skein_kv::erase_bytes(u32::try_from(key.len()).unwrap()).unwrap(),
            };
        }
    }
    for (writes, bytes) in [(3, u32::MAX), (5, u32::MAX), (6, total - 1)] {
        let cut = jobs::page("change-parts", &page(Box::new([row(12), row(13)]), writes, bytes)).unwrap();
        assert_eq!(cut.writes.len(), 3);
        assert_eq!(cut.through.as_deref(), Some(input.rows[0].key.as_ref()));
    }
    let exact = jobs::page("change-parts", &page(Box::new([row(12), row(13)]), 6, total)).unwrap();
    assert_eq!(exact.writes.len(), answer.writes.len());
    assert_eq!(exact.through, None);
    let empty = jobs::page("change-parts", &page(Box::new([]), 0, 0)).unwrap();
    assert!(empty.writes.is_empty());
    assert_eq!(empty.through, None);
}

#[test]
fn change_parts_job_refuses_the_unreadable_key_and_a_row_that_cannot_fit() {
    let mut bad = row(12);
    bad.value = Box::new([0]);
    for input in
        [page(Box::new([bad]), 3, u32::MAX), page(Box::new([row(12)]), 2, u32::MAX), page(Box::new([row(12)]), 3, 0)]
    {
        let why = jobs::page("change-parts", &input).unwrap_err();
        assert_eq!(why.key, Some(input.rows[0].key.clone()));
    }
    let mut wrong_key = row(12);
    wrong_key.key = keys::encode(&Key::Change(13), u32::MAX).unwrap();
    assert_eq!(
        jobs::page(
            "change-parts",
            &page(Box::new([JobRow { key: wrong_key.key.clone(), value: wrong_key.value.clone() }]), 3, u32::MAX)
        )
        .unwrap_err()
        .key,
        Some(wrong_key.key)
    );
}
