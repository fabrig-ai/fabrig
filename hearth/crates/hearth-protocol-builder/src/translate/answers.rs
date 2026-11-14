//! Render answers become the client's typed records (hearth/docs/design/domain/render/README.md, 5.1).
use crate::{records::outcome as target, render::vocabulary::Outcome};

#[must_use]
pub fn outcome(outcome: Outcome) -> target::Outcome {
    match outcome {
        Outcome::Usage => target::Outcome::Usage,
        Outcome::Failed => target::Outcome::Failed,
        Outcome::TryAgain => target::Outcome::TryAgain,
        Outcome::Negative => target::Outcome::Negative,
        Outcome::Done => target::Outcome::Done,
    }
}

/// A build's layer, by kind, key and state (hearth/docs/design/client/README.md, 3.1).
#[must_use]
pub fn layer(
    layer: crate::builder::build::Layer,
    key: crate::render::vocabulary::Digest,
    state: crate::builder::vocabulary::State,
) -> crate::records::build::Layer {
    use crate::{builder::vocabulary::State, records::build};
    build::Layer {
        kind: kind(layer),
        key: key.0,
        state: match state {
            State::Found => build::State::Found,
            State::ToBuild => build::State::ToBuild,
        },
    }
}
fn kind(layer: crate::builder::build::Layer) -> crate::records::build::Kind {
    use crate::{builder::build::Layer, records::build::Kind};
    match layer {
        Layer::Packages => Kind::Packages,
        Layer::Software => Kind::Software,
        Layer::Config => Kind::Config,
    }
}
/// The step a build ended on, named with its layer (hearth/docs/design/domain/build/README.md, 3).
#[must_use]
pub fn failed(
    step: crate::builder::build::Step,
    why: crate::builder::build::vocabulary::Why,
) -> crate::records::build::Failed {
    use crate::{
        builder::build::{Layer, Step, vocabulary::Why},
        records::build,
    };
    let (layer, step) = match step {
        Step::Fetch(layer) => (layer, build::Step::Fetch),
        Step::Bootstrap => (Layer::Packages, build::Step::Mmdebstrap),
        Step::Verify => (Layer::Packages, build::Step::Verify),
        Step::Place => (Layer::Software, build::Step::Place),
        Step::Software => (Layer::Software, build::Step::Software),
        Step::Users => (Layer::Config, build::Step::Users),
        Step::Files => (Layer::Config, build::Step::Tree),
        Step::Tmpfiles => (Layer::Config, build::Step::Tmpfiles),
        Step::Presets => (Layer::Config, build::Step::Presets),
        Step::Customize => (Layer::Config, build::Step::Customize),
    };
    build::Failed {
        layer: kind(layer),
        step,
        why: match why {
            // The records name one reason until the client grows more (hearth/docs/design/client/README.md, 3.3).
            Why::Unavailable | Why::Failed | Why::Stage | Why::Seal => build::Why::Unavailable,
        },
    }
}
/// The store full at a limit, as an operational error of kind store (hearth/docs/design/domain/store/README.md, 6.1).
#[must_use]
pub fn full(limit: crate::builder::store::vocabulary::Limit, most: u32, store: &[u8]) -> crate::records::errors::Error {
    use crate::{
        builder::store::vocabulary::Limit,
        records::errors::{Class, Error, Kind, StoreError, StoreFault, StoreLimit},
    };
    let limit = match limit {
        Limit::Resolutions => StoreLimit::Resolutions,
        Limit::Layers => StoreLimit::Layers,
        Limit::CacheFiles => StoreLimit::CacheFiles,
    };
    Error {
        side: None,
        kind: Kind::Store(StoreError {
            store: skein_lib::bytes::copy_of(store),
            fault: StoreFault::Full { limit, most },
        }),
        class: Class::Operational,
        at: None,
        also: None,
        target: None,
        task: None,
    }
}
