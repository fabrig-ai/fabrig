//! Unread words survive task documents at both ends of their bounds.
use super::*;

#[test]
fn task_documents_round_trip_empty_and_full_unread_words() {
    let limits = CEILINGS;
    for count in [0, limits.task_unread] {
        let mut task = fixtures::task(&limits);
        let mut words = skein_lib::List::with_capacity(count);
        for message in 0..count {
            words
                .push(UnreadWords {
                    message: u64::from(message),
                    by: Requester::Person(RequesterPerson { value: 42 }),
                    at: u64::MAX,
                    seen: message % 2 == 0,
                    head: Box::new([b'x'; 256]),
                    bytes: 300,
                })
                .unwrap();
        }
        task.unread = words;
        let bytes = fixtures::encoded(&task);
        let decoded = Task::decode(&limits, &mut Reader::new(&bytes)).unwrap();
        assert_eq!(decoded.unread.len(), count);
        assert_eq!(decoded, task);
    }
}
