//! Checkpoint requests draw at the service route; terminals retain their token and figures.
use super::fixtures;
use crate::store::route::{self, Up};
use jig_protocol_store as store;
use skein_lib::{CryptoRng, Queue, Secret, Token};
use temper_engine_domain as root;

#[test]
fn checkpoint_requests_draw_sixteen_fresh_bytes_at_the_store_route() {
    let mut routes = fixtures::routes();
    let mut random = CryptoRng::new(&Secret::new(Box::new([7; 32])), 1).unwrap();
    for token in [Token::new(42), Token::new(43)] {
        let mut expected = [0; 16];
        random.fill(&mut expected);
        let Some(store::Request::Checkpoint { owner, draw }) =
            route::down(&mut routes, root::StoreRequest::Checkpoint { token })
        else {
            panic!("checkpoint request")
        };
        assert_eq!((owner, draw), (token, expected));
    }
}
#[test]
fn checkpoint_store_terminals_reach_the_root_with_their_token_and_figures() {
    let mut routes = fixtures::routes();
    let token = Token::new(42);
    let mut out = Queue::uncapped();
    route::up(
        &mut routes,
        store::Event::Checkpointed {
            owner: token,
            checkpoint: store::Checkpointed {
                name: store::CheckpointName { store: 42, suffix: 0xab },
                secrets: 7,
                linked: 5,
                copied: 2,
                bytes: 91,
                paused: 1_234_567,
                took: 9_876_543,
                held: 123,
                removed: 1,
                skipped: 3,
            },
        },
        &mut out,
    );
    let Some(Up::Root(event)) = out.pop() else { panic!("root terminal") };
    let root::Event::Store(root::StoreInput::Checkpointed { token: found, checkpoint }) = *event else {
        panic!("checkpoint terminal")
    };
    assert_eq!(found, token);
    let expected = root::Checkpointed {
        name: b"00000000000000000042-00000000000000ab".as_slice().into(),
        store: 42,
        secrets: 7,
        linked: 5,
        copied: 2,
        bytes: 91,
        paused: 1_234_567,
        took: 9_876_543,
        held: 123,
        removed: 1,
        skipped: 3,
    };
    assert_eq!(checkpoint, expected);
    for (why, expected) in [
        (store::CheckpointRefusal::Unavailable, root::CheckpointWhy::Unavailable),
        (
            store::CheckpointRefusal::Failed { path: "store/log-1".into(), error: skein_io::kernel::Error::Permission },
            root::CheckpointWhy::Failed { path: "store/log-1".into(), error: "Permission".into() },
        ),
    ] {
        route::up(&mut routes, store::Event::CheckpointRefused { owner: token, why }, &mut out);
        let Some(Up::Root(event)) = out.pop() else { panic!("root refusal") };
        let root::Event::Store(root::StoreInput::CheckpointRefused { token: found, why }) = *event else {
            panic!("checkpoint refusal")
        };
        assert_eq!((found, why), (token, expected));
    }
    assert!(out.is_empty());
}
