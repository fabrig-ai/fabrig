//! Init's document text fits its clients (temper/docs/design/protocol/setup/init.md, 1, and temper/docs/design/protocol/setup/README.md, 9).
use core::num::NonZeroU16;

#[test]
fn every_client_receives_the_text_inits_documents_allow() {
    let port = NonZeroU16::new(8367).expect("nonzero fixture port");
    let facts = temper_init::Facts {
        sign_in: false,
        binary: b"/opt/temper/bin/temper",
        git: Some(b"/usr/bin/git"),
        claude: None,
        login: Some(b"person"),
        ports: temper_init::Ports { clients: port, webhooks: port },
    };
    let directories = temper_init::Directories { config: b"/etc/temper", data: b"/var/lib/temper" };
    let boxed = temper_init::files(&facts, &directories, &[0; 32]).expect("box init");
    let bare = temper_init::bare(
        &temper_init::Bare {
            binary: facts.binary,
            git: facts.git,
            claude: facts.claude,
            clients: facts.ports.clients,
            webhooks: facts.ports.webhooks,
        },
        &directories,
        &[0; 32],
    )
    .expect("bare init");
    for (document, files) in [("box", boxed.files.as_ref()), ("bare", bare.as_ref())] {
        let file = files.iter().find(|file| file.name == b"temper.json").expect("init writes temper.json");
        let temper_init::Content::Bytes { bytes, .. } = &file.content else { panic!("document bytes") };
        let value: serde_json::Value = serde_json::from_slice(bytes).expect("init JSON");
        assert!(value["engine"].get("limits").is_none());
        assert!(value["engine"].get("memory_bytes").is_none());
        assert!(value["application"].get("limits").is_none());
        let parsed = jig_protocol_config::document(
            bytes,
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
