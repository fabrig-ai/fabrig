//! The stock's per-worker ceiling fits the u32 engine channel cap (protocol/limits.md).
#[cfg(test)]
mod tests {
    use jig_protocol_config::stock;
    use jig_protocol_worker_wire::sizes;

    #[test]
    fn stock_engine_link_cap_fits() {
        let stock = stock();
        let engine = jig_test_service::limits::stock(&stock).workers;
        let cap = sizes::engine_cap(&engine.sizes, engine.room).expect("stock engine link cap fits u32");
        assert!(cap.bytes >= engine.room, "room for a fitted assignment and its notices");
        let mut worker = engine.sizes;
        worker.slots = stock.hub.slots;
        assert!(sizes::worker_cap(&worker).is_some(), "stock hub's worker cap fits u32");
    }
}
