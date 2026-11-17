//! The stock's per-worker ceiling fits the u32 engine channel cap (protocol/limits.md).
#[cfg(test)]
mod tests {
    use jig_protocol_config::{Profile, stock};
    use jig_protocol_worker_wire::sizes;

    #[test]
    fn stock_engine_link_cap_fits() {
        let stock = stock();
        let engine = jig_test_service::limits::for_profile(Profile::Standard, &stock).workers;
        let cap = sizes::engine_cap(&engine.sizes).expect("stock engine link cap fits u32");
        assert_eq!(cap.bytes, 2_503_418_860);
        let mut candidate = engine.sizes;
        candidate.slots = 64; // Jig alone holds 64; the application's startup sizes bind first.
        assert_eq!(sizes::engine_cap(&candidate).unwrap().bytes, 4_005_464_116);
        candidate.slots = 70; // The next round figure above 64.
        assert_eq!(sizes::engine_cap(&candidate), None);
        let mut worker = engine.sizes;
        worker.slots = stock.hub.slots;
        assert!(sizes::worker_cap(&worker).is_some(), "stock hub's worker cap fits u32");
    }
}
