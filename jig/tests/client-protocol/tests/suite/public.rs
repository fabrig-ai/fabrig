//! Public HTTP integration (jig/docs/design/protocol/client/public-reads.md).
use jig_client_protocol_wire::public_v1;
use jig_client_protocol_world::public;
use skein_lib::Reader;

#[test]
fn a_public_board_is_read_over_http_bytes_through_the_service() {
    for seed in [3101, 3102] {
        eprintln!("public HTTP seed {seed}");
        let (project, replies) = public::run(seed);
        let [shown, hidden] = replies.as_slice() else { panic!("two HTTP reads seed {seed}") };
        assert_eq!(shown.status, 200, "seed {seed}");
        let mut bytes = Reader::new(&shown.body);
        let board = public_v1::PublicBoard::decode(&public_v1::CEILINGS, &mut bytes).expect("public board body");
        assert_eq!(bytes.remaining(), 0);
        assert_eq!(board.project(), project);
        assert_eq!(board.live().len(), 1);
        assert_eq!(board.live().get(0).expect("goal").line().title(), public::TITLE);
        let cache = shown.headers.iter().find(|h| h.is(b"cache-control")).expect("cache-control");
        let cache = std::str::from_utf8(&cache.value).expect("ASCII header");
        assert!(cache.contains("must-revalidate"));
        let max_age = cache
            .split(',')
            .find_map(|v| v.trim().strip_prefix("max-age="))
            .expect("max-age")
            .parse::<u32>()
            .expect("seconds");
        assert!(max_age <= jig_client_protocol_world::fixtures::public_service_limits().clients.public.max_age);
        assert_eq!(hidden.status, 404, "private seed {seed}");
        assert_eq!(hidden.body.len(), 0);
    }
}
