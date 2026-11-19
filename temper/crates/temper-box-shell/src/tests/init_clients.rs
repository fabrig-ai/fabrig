//! Init's document text fits its clients (temper/docs/design/protocol/setup/init.md, 1, and temper/docs/design/protocol/setup/README.md, 9).

#[test]
fn every_client_receives_the_text_inits_documents_allow() {
    for (document, bytes) in
        [("box", temper_config_shell::init_document(true)), ("bare", temper_config_shell::init_document(false))]
    {
        let value: serde_json::Value = serde_json::from_slice(&bytes).expect("init JSON");
        assert!(value["engine"].get("limits").is_none());
        assert!(value["engine"].get("memory_bytes").is_none());
        assert!(value["application"].get("limits").is_none());
        let parsed = jig_protocol_config::document(
            &bytes,
            &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
        )
        .expect("init document");
        let engine = temper_service::startup::engine_with(
            &parsed,
            parsed.member(0, b"engine").expect("engine"),
            &jig_protocol_config::Directories::working(),
            &temper_service::stock::Given::Stock,
        )
        .expect("init engine");
        let text = u64::from(engine.limits.core.documents.text_bytes);
        for (name, client) in [
            ("web", temper_client_shell_web::limits::client().client),
            ("terminal", temper_client_shell_terminal::limits::native().client.client),
            ("tools", temper_client_tools::limits::client().client),
        ] {
            let link = jig_client_protocol_link::Limits::new(client.domain, client.exchanges).expect("client limits");
            let receiving = link.receiving().document_text_bytes;
            assert!(text <= u64::from(receiving), "{document}: {text} text bytes exceed {name}'s {receiving}");
        }
        jig_protocol_config::check_effective_limits(&engine.limits.core).expect("startup receiving limits fit codecs");
        let web = temper_client_shell_web::limits::client().client.domain;
        let clients = engine.limits.clients;
        assert!(clients.watches_per_stream >= web.watches);
        let queue = jig_protocol_engine_clients::limits::stream_queue_bytes(web.watches, &clients.wire)
            .expect("queue fits u32");
        assert_eq!(clients.queued_bytes, queue);
        let memory = temper_service::limits::stock().root.transcript_memory;
        assert!(u64::from(queue) <= memory, "one full first-delivery batch fits the domain byte budget");
        assert!(
            engine.limits.clients_http.event <= temper_client_shell_web::limits::BODY_BYTES,
            "the browser's SSE decoder carries the largest HTTP event"
        );
    }
}
