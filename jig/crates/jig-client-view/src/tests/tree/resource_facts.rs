#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect whole public resource slots and literal trees"
)]

use jig_client::contract::*;
use skein_lib::List;

fn list<T>(values: impl IntoIterator<Item = T>) -> List<T> {
    let values: Vec<_> = values.into_iter().collect();
    let mut out = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        out.push(value).ok().unwrap();
    }
    out
}
