use super::{Bounds, Undecodable, answers, api, entries, names, w};
use alloc::boxed::Box;
use skein_lib::{List, Time};

fn encode_what(value: &skein_forge_client::What) -> w::LiveWhat {
    match value {
        skein_forge_client::What::Repository => w::LiveWhat::Repository,
        skein_forge_client::What::Branch(branch) => w::LiveWhat::Branch(w::LiveBranch { branch: branch.clone() }),
        skein_forge_client::What::Pull(number) => w::LiveWhat::Pull(w::Number { number: *number }),
        skein_forge_client::What::Issue(number) => w::LiveWhat::Issue(w::Number { number: *number }),
    }
}
fn decode_what(value: w::LiveWhat) -> skein_forge_client::What {
    match value {
        w::LiveWhat::Repository => skein_forge_client::What::Repository,
        w::LiveWhat::Branch(value) => skein_forge_client::What::Branch(value.branch),
        w::LiveWhat::Pull(value) => skein_forge_client::What::Pull(value.number),
        w::LiveWhat::Issue(value) => skein_forge_client::What::Issue(value.number),
    }
}
fn encode_watch(value: &skein_forge_client::Watch) -> w::Watch {
    let skein_forge_client::Watch { resource: skein_forge_client::Resource { repository, what }, participating } =
        value;
    let resource = w::LiveResource { repository: names::encode_repository(*repository), what: encode_what(what) };
    w::Watch { resource, participating: *participating }
}
fn decode_watch(value: w::Watch) -> skein_forge_client::Watch {
    let w::Watch { resource, participating } = value;
    let w::LiveResource { repository, what } = resource;
    skein_forge_client::Watch {
        resource: skein_forge_client::Resource {
            repository: names::decode_repository(repository),
            what: decode_what(what),
        },
        participating,
    }
}
fn encode_cached(value: &skein_forge_client::Cached) -> w::Cached {
    let skein_forge_client::Cached { item, pull, tip } = value;
    w::Cached {
        item: match item {
            Some(value) => Some(api::encode_summary(value)),
            None => None,
        },
        pull: match pull {
            Some(value) => Some(answers::encode_pull(value)),
            None => None,
        },
        tip: *tip,
    }
}
fn decode_cached(value: w::Cached, bounds: &Bounds) -> Result<skein_forge_client::Cached, Undecodable> {
    let w::Cached { item, pull, tip } = value;
    Ok(skein_forge_client::Cached {
        item: match item {
            Some(value) => Some(api::decode_summary(value, bounds)?),
            None => None,
        },
        pull: match pull {
            Some(value) => Some(answers::decode_pull(value)),
            None => None,
        },
        tip,
    })
}
fn encode_live(value: &skein_forge_client::LiveRecord) -> w::LiveRecord {
    let skein_forge_client::LiveRecord {
        watch,
        position,
        listed,
        comment_after,
        comment_page,
        comment_scanning,
        comment_repair,
        review_page,
        comments,
        reviews,
        cached,
        pushed,
        echoes,
    } = value;
    w::LiveRecord {
        watch: encode_watch(watch),
        position: entries::encode_position(*position),
        listed: listed.as_nanos(),
        comment_after: *comment_after,
        comment_page: *comment_page,
        comment_scanning: *comment_scanning,
        comment_repair: *comment_repair,
        review_page: *review_page,
        comments: encode_deliveries(comments),
        reviews: encode_deliveries(reviews),
        cached: encode_cached(cached),
        pushed: *pushed,
        echoes: encode_echoes(echoes),
    }
}
fn decode_live(value: w::LiveRecord, bounds: &Bounds) -> Result<skein_forge_client::LiveRecord, Undecodable> {
    let w::LiveRecord {
        watch,
        position,
        listed,
        comment_after,
        comment_page,
        comment_scanning,
        comment_repair,
        review_page,
        comments,
        reviews,
        cached,
        pushed,
        echoes,
    } = value;
    Ok(skein_forge_client::LiveRecord {
        watch: decode_watch(watch),
        position: entries::decode_position(position),
        listed: Time::from_nanos(listed),
        comment_after,
        comment_page,
        comment_scanning,
        comment_repair,
        review_page,
        comments: decode_deliveries(comments),
        reviews: decode_deliveries(reviews),
        cached: Box::new(decode_cached(cached, bounds)?),
        pushed,
        echoes: decode_echoes(echoes),
    })
}
fn encode_repository(value: skein_forge_client::RepositoryRecord) -> w::RepositoryRecord {
    let skein_forge_client::RepositoryRecord { repository, mark, clock } = value;
    w::RepositoryRecord {
        repository: names::encode_repository(repository),
        mark: match mark {
            Some(value) => Some(value.as_nanos()),
            None => None,
        },
        clock: clock.as_nanos(),
    }
}
fn decode_repository(value: w::RepositoryRecord) -> skein_forge_client::RepositoryRecord {
    let w::RepositoryRecord { repository, mark, clock } = value;
    skein_forge_client::RepositoryRecord {
        repository: names::decode_repository(repository),
        mark: match mark {
            Some(value) => Some(Time::from_nanos(value)),
            None => None,
        },
        clock: Time::from_nanos(clock),
    }
}
pub(super) fn encode(value: &skein_forge_client::Stored) -> w::ClientRow {
    let value = match value {
        skein_forge_client::Stored::Writer(_) => {
            unreachable!("the connector keeps the client's writer as Stored::Writer")
        }
        skein_forge_client::Stored::Live(value) => w::ClientValue::Live(Box::new(encode_live(value))),
        skein_forge_client::Stored::Repository(value) => w::ClientValue::Repository(encode_repository(*value)),
    };
    w::ClientRow { value }
}
pub(super) fn decode(value: w::ClientRow, bounds: &Bounds) -> Result<skein_forge_client::Stored, Undecodable> {
    Ok(match value.value {
        w::ClientValue::Live(value) => skein_forge_client::Stored::Live(decode_live(*value, bounds)?),
        w::ClientValue::Repository(value) => skein_forge_client::Stored::Repository(decode_repository(value)),
    })
}
fn encode_deliveries(values: &[skein_forge_client::Delivery]) -> List<w::Delivery> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let skein_forge_client::Delivery { id, revision } = *value;
        let item = w::Delivery { id, revision };
        result.push(item).expect("same count");
    }
    result
}
fn decode_deliveries(values: List<w::Delivery>) -> Box<[skein_forge_client::Delivery]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let w::Delivery { id, revision } = value;
        result.push(skein_forge_client::Delivery { id, revision }).expect("same count");
    }
    result.into_boxed()
}
fn encode_echoes(values: &[skein_forge_client::Echo]) -> List<w::Echo> {
    let mut result = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for value in values {
        let item = match value {
            skein_forge_client::Echo::Comment(number) => w::Echo::Comment(w::Number { number: *number }),
            skein_forge_client::Echo::Review(number) => w::Echo::Review(w::Number { number: *number }),
        };
        result.push(item).expect("same count");
    }
    result
}
fn decode_echoes(values: List<w::Echo>) -> Box<[skein_forge_client::Echo]> {
    let mut result = List::with_capacity(values.len());
    for value in values.into_boxed() {
        let item = match value {
            w::Echo::Comment(value) => skein_forge_client::Echo::Comment(value.number),
            w::Echo::Review(value) => skein_forge_client::Echo::Review(value.number),
        };
        result.push(item).expect("same count");
    }
    result.into_boxed()
}
