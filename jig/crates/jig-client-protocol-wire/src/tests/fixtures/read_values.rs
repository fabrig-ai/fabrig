//! Every contract read variant with its existing document's smallest value.
use crate::*;
use skein_lib::{List, Reader};

macro_rules! value {
    ($rows:expr, $limits:expr, $variant:ident, $record:ident, $length:expr, $wrap:path) => {{
        let bytes = super::bytes($length);
        let record = $record::decode($limits, &mut Reader::new(&bytes)).expect("smallest existing document");
        $rows.push((stringify!($variant), ReadValue::$variant($wrap(record)))).expect("26 read variants");
    }};
    ($rows:expr, $limits:expr, $variant:ident, $record:ident, $length:expr) => {
        value!($rows, $limits, $variant, $record, $length, core::convert::identity);
    };
}

pub fn read_values(limits: &Limits) -> List<(&'static str, ReadValue)> {
    let mut values = List::with_capacity(26);
    value!(values, limits, Project, Project, 116);
    value!(values, limits, Ended, Ended, 299, Box::new);
    value!(values, limits, Me, Me, 83);
    value!(values, limits, Turn, ArchivedTurn, 42);
    value!(values, limits, Task, Task, 272);
    value!(values, limits, Proposal, Proposal, 61);
    value!(values, limits, Escalation, Escalation, 276);
    value!(values, limits, Question, Question, 31);
    value!(values, limits, PersonTask, PersonTask, 71);
    value!(values, limits, Result, ResultDocument, 15);
    value!(values, limits, EndedPage, EndedPage, 7);
    value!(values, limits, History, HistoryPage, 7);
    value!(values, limits, Activity, ActivityPage, 7);
    value!(values, limits, Inbox, InboxPage, 8);
    value!(values, limits, Spend, SpendPage, 7);
    value!(values, limits, Notes, NotePage, 7);
    value!(values, limits, Deployment, Deployment, 148);
    values
}
