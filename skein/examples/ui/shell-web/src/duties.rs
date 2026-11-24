//! The example's view duties as transport requests (skein/docs/design/ui/web/README.md, 4.3).
use skein_lib::Queue;
use skein_ui_example_view::{Duties, StreamDuty};
use skein_ui_web::{fetch, store, transport};

pub(crate) fn empty() -> Duties {
    Duties {
        writes: Queue::with_capacity(8),
        removes: Queue::with_capacity(8),
        exchanges: Queue::with_capacity(8),
        show: None,
        stream: None,
    }
}

pub(crate) fn requests(mut duties: Duties) -> transport::Requests {
    let mut requests = transport::Requests::default();
    while let Some(record) = duties.writes.pop() {
        requests.writes.push(store::Record { key: record.key, value: record.value });
    }
    while let Some(key) = duties.removes.pop() {
        requests.removes.push(key);
    }
    while let Some(exchange) = duties.exchanges.pop() {
        requests.exchanges.push(fetch::Exchange {
            id: exchange.id,
            path: exchange.path,
            method: if exchange.post { "POST" } else { "GET" }.into(),
            headers: vec![("Content-Type".into(), "application/octet-stream".into())],
            body: exchange.body,
            bound: exchange.bound,
            pieces: None,
            cache: fetch::Cache::Browser,
            keep: &[],
        });
    }
    requests.stream = duties.stream.map(|duty| match duty {
        StreamDuty::Open { path, headers, bound } => {
            transport::StreamDuty::Open { path, headers: headers.into_vec(), bound }
        }
        StreamDuty::Close => transport::StreamDuty::Close,
    });
    requests
}
