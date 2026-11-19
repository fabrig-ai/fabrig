use super::names::{decode_name, decode_repository, encode_name, encode_repository};
use super::{Bounds, Undecodable, w};
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::List;
use temper_engine_domain_forge_vocabulary as d;

fn encode_topic(value: &d::Topic) -> w::Topic {
    match value {
        d::Topic::Findings { project } => w::Topic::Findings(w::FindingsTopic { project: *project }),
        d::Topic::Landings { repository, branch } => {
            w::Topic::Landings(w::LandingsTopic { repository: encode_repository(*repository), branch: branch.clone() })
        }
        d::Topic::Ci { repository, head } => {
            w::Topic::Ci(w::CiTopic { repository: encode_repository(*repository), head: *head })
        }
        d::Topic::Pull { repository, number } => {
            w::Topic::Pull(w::NumberTopic { repository: encode_repository(*repository), number: *number })
        }
        d::Topic::Participation { repository, number } => {
            w::Topic::Participation(w::NumberTopic { repository: encode_repository(*repository), number: *number })
        }
    }
}
fn decode_topic(value: w::Topic) -> d::Topic {
    match value {
        w::Topic::Findings(value) => d::Topic::Findings { project: value.project },
        w::Topic::Landings(value) => {
            let w::LandingsTopic { repository, branch } = value;
            d::Topic::Landings { repository: decode_repository(repository), branch }
        }
        w::Topic::Ci(value) => {
            let w::CiTopic { repository, head } = value;
            d::Topic::Ci { repository: decode_repository(repository), head }
        }
        w::Topic::Pull(value) => {
            let w::NumberTopic { repository, number } = value;
            d::Topic::Pull { repository: decode_repository(repository), number }
        }
        w::Topic::Participation(value) => {
            let w::NumberTopic { repository, number } = value;
            d::Topic::Participation { repository: decode_repository(repository), number }
        }
    }
}
pub(super) fn encode_names(task: u64, resources: &[d::Name]) -> w::Names {
    let mut names = List::with_capacity(u32::try_from(resources.len()).expect("admitted count"));
    for name in resources {
        names.push(encode_name(name)).expect("same count");
    }
    w::Names { task, resources: names }
}
pub(super) fn decode_names(resources: List<w::Name>, bounds: &Bounds) -> Result<Box<[d::Name]>, Undecodable> {
    let mut names = List::with_capacity(resources.len());
    for name in resources.into_boxed() {
        names.push(decode_name(name, bounds)?).expect("same count");
    }
    Ok(names.into_boxed())
}

pub(super) fn encode_ci(value: api::Ci) -> w::Ci {
    match value {
        api::Ci::None => w::Ci::None,
        api::Ci::Pending => w::Ci::Pending,
        api::Ci::Passed => w::Ci::Passed,
        api::Ci::Failed => w::Ci::Failed,
    }
}
pub(super) fn decode_ci(value: w::Ci) -> api::Ci {
    match value {
        w::Ci::None => api::Ci::None,
        w::Ci::Pending => api::Ci::Pending,
        w::Ci::Passed => api::Ci::Passed,
        w::Ci::Failed => api::Ci::Failed,
    }
}

pub(super) fn encode_state(value: api::State) -> w::ItemState {
    match value {
        api::State::Open => w::ItemState::Open,
        api::State::Closed => w::ItemState::Closed,
    }
}
pub(super) fn decode_state(value: w::ItemState) -> api::State {
    match value {
        w::ItemState::Open => api::State::Open,
        w::ItemState::Closed => api::State::Closed,
    }
}

fn encode_ending(value: d::ReleaseEnding) -> w::ReleaseEnding {
    match value {
        d::ReleaseEnding::Done => w::ReleaseEnding::Done,
        d::ReleaseEnding::Failed => w::ReleaseEnding::Failed,
        d::ReleaseEnding::Cancelled => w::ReleaseEnding::Cancelled,
    }
}
fn decode_ending(value: w::ReleaseEnding) -> d::ReleaseEnding {
    match value {
        w::ReleaseEnding::Done => d::ReleaseEnding::Done,
        w::ReleaseEnding::Failed => d::ReleaseEnding::Failed,
        w::ReleaseEnding::Cancelled => d::ReleaseEnding::Cancelled,
    }
}

pub(super) fn encode_branch_head(value: &d::BranchHead) -> w::BranchHead {
    let d::BranchHead { name, commit, owned } = value;
    w::BranchHead { name: encode_name(name), commit: *commit, owned: *owned }
}
pub(super) fn decode_branch_head(value: w::BranchHead, bounds: &Bounds) -> Result<d::BranchHead, Undecodable> {
    let w::BranchHead { name, commit, owned } = value;
    Ok(d::BranchHead { name: decode_name(name, bounds)?, commit, owned })
}

pub(super) fn encode_pull_state(value: &d::PullState) -> w::PullState {
    let d::PullState { name, head, ci, state } = value;
    w::PullState { name: encode_name(name), head: *head, ci: encode_ci(*ci), state: encode_state(*state) }
}
pub(super) fn decode_pull_state(value: w::PullState, bounds: &Bounds) -> Result<d::PullState, Undecodable> {
    let w::PullState { name, head, ci, state } = value;
    Ok(d::PullState { name: decode_name(name, bounds)?, head, ci: decode_ci(ci), state: decode_state(state) })
}

pub(super) fn encode_ci_state(value: &d::CiState) -> w::CiState {
    let d::CiState { repository, head, status } = value;
    w::CiState { repository: encode_repository(*repository), head: *head, status: encode_ci(*status) }
}
pub(super) fn decode_ci_state(value: w::CiState) -> d::CiState {
    let w::CiState { repository, head, status } = value;
    d::CiState { repository: decode_repository(repository), head, status: decode_ci(status) }
}

pub(super) fn encode_subscriber(value: &d::Subscriber) -> w::Subscriber {
    let d::Subscriber { task, number, topic, own_change, goal_tasks, paths } = value;
    w::Subscriber {
        task: *task,
        number: *number,
        topic: encode_topic(topic),
        own_change: *own_change,
        goal_tasks: match goal_tasks {
            Some(v) => Some(encode_numbers(v)),
            None => None,
        },
        paths: encode_paths(paths),
    }
}
pub(super) fn decode_subscriber(value: w::Subscriber) -> d::Subscriber {
    let w::Subscriber { task, number, topic, own_change, goal_tasks, paths } = value;
    d::Subscriber {
        task,
        number,
        topic: decode_topic(topic),
        own_change,
        goal_tasks: match goal_tasks {
            Some(v) => Some(v.into_boxed()),
            None => None,
        },
        paths: paths.into_boxed(),
    }
}

pub(super) fn encode_landed(commit: api::Commit, task: u64) -> w::Landed {
    w::Landed { commit, task }
}
pub(super) fn encode_release(value: &d::ReleaseRow) -> w::ReleaseRow {
    let d::ReleaseRow { task, root, ending, close_pull, comment, pending, failed, effects_settled, releasing, room } =
        value;
    let pending = match pending {
        Some((entry, name)) => {
            let name = match name {
                Some(name) => Some(encode_name(name)),
                None => None,
            };
            Some(w::PendingRelease { entry: *entry, name })
        }
        None => None,
    };
    w::ReleaseRow {
        task: *task,
        root: *root,
        ending: encode_ending(*ending),
        close_pull: *close_pull,
        comment: comment.clone(),
        pending_release: pending,
        failed: *failed,
        effects_settled: *effects_settled,
        releasing: *releasing,
        room: *room,
    }
}
pub(super) fn decode_release(value: w::ReleaseRow, bounds: &Bounds) -> Result<d::ReleaseRow, Undecodable> {
    let w::ReleaseRow {
        task,
        root,
        ending,
        close_pull,
        comment,
        pending_release: pending,
        failed,
        effects_settled,
        releasing,
        room,
    } = value;
    let pending = match pending {
        Some(value) => {
            let w::PendingRelease { entry, name } = value;
            let name = match name {
                Some(name) => Some(decode_name(name, bounds)?),
                None => None,
            };
            Some((entry, name))
        }
        None => None,
    };
    Ok(d::ReleaseRow {
        room,
        task,
        root,
        ending: decode_ending(ending),
        close_pull,
        comment,
        pending,
        failed,
        effects_settled,
        releasing,
    })
}
fn encode_numbers(values: &[u64]) -> List<u64> {
    let mut numbers = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for number in values {
        numbers.push(*number).expect("same count");
    }
    numbers
}
fn encode_paths(values: &[Box<[u8]>]) -> List<Box<[u8]>> {
    let mut paths = List::with_capacity(u32::try_from(values.len()).expect("admitted count"));
    for path in values {
        paths.push(path.clone()).expect("same count");
    }
    paths
}
