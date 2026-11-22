//! Deployment file preparation (tools/tiers/layers.md, 6).
mod drive {
    mod deployment {
        pub(super) use temper_client_journeys_tool::{drive::deployment::secret_pair, testing::configuration};
    }
    mod tests {
        mod deployment;
    }
}
