use skein_forge_connection::settings::Kind;
use skein_forge_setup::item::Bounds;
use skein_lib::{Rng, Secret};
use temper_protocol_records::{self as records, Item};
use temper_protocol_records_forge as forge_records;
use temper_protocol_records_forge::{NotConnected, Origin, Rows};
fn bounds() -> Bounds {
    Bounds { token_bytes: 64, name_bytes: 64, key_bytes: 1196, installations: 8 }
}
fn expected(kind: Kind) -> Origin {
    Origin { kind, api: Box::from("127.0.0.1:3000") }
}
fn put(rows: &mut Rows, item: Item, bytes: &[u8]) {
    rows.push(
        records::number(1, item),
        Secret::new(records::encode(item, bytes).unwrap_or_else(|_| vec![0]).into_boxed_slice()),
        4096,
    );
}
fn complete(kind: Kind) -> Rows {
    let mut rows = Rows::new();
    put(&mut rows, Item::Origin, &expected(kind).encode());
    put(&mut rows, Item::User, &9_u64.to_le_bytes());
    match kind {
        Kind::Forgejo => {
            let byte = u8::try_from(Rng::new(83).between(33, 126)).expect("ASCII");
            put(&mut rows, Item::ApiToken, &[byte; 32]);
        }
        Kind::GitHub => {
            put(&mut rows, Item::AppId, &7_u64.to_le_bytes());
            put(&mut rows, Item::AppSlug, b"test-app");
            put(&mut rows, Item::AppKey, skein_fake_forge_github::keys::KEYS[0]);
        }
    }
    rows
}
#[test]
fn record_numbers_and_origin_addresses_keep_their_meaning() {
    assert_eq!(records::number(0x1234, Item::Origin), 0x0001_0000_1234_000a);
    assert_eq!(Item::AppMade.code(), 12);
    assert_eq!(records::parse(0x0001_0000_1234_000c), Some((1, 0x1234, Item::AppMade)));
    for (i, address) in ["127.0.0.1:3000", "[::1]:3000", "forge.test:443", "api.github.com:443"].iter().enumerate() {
        let kind = if i == 3 { Kind::GitHub } else { Kind::Forgejo };
        let configured = if let Ok(socket) = address.parse() {
            jig_protocol_config::Address::Addr(skein_io_address(socket))
        } else {
            let (name, port) = address.rsplit_once(':').expect("host port");
            jig_protocol_config::Address::Host { name: name.as_bytes().into(), port: port.parse().expect("port") }
        };
        let origin = forge_records::origin(kind, &configured);
        assert_eq!(Origin::decode(&origin.encode()), Ok(origin));
    }
    for item in [
        Item::AdminToken,
        Item::ApiToken,
        Item::GitToken,
        Item::User,
        Item::WebhookSecret,
        Item::AppId,
        Item::AppSlug,
        Item::AppKey,
        Item::Accepted,
        Item::Origin,
        Item::AppMade,
    ] {
        assert_eq!(records::parse(records::number(65535, item)), Some((1, 65535, item)));
    }
    for number in [0, 1 << 48, (1 << 48) | 0x000b, (1 << 48) | 0xffff, records::CARRIED, u64::MAX] {
        assert_eq!(records::parse(number), None);
    }
    for bytes in [&b""[..], &[2, 1], &[0]] {
        assert_eq!(records::decode(Item::ApiToken, bytes), Err(records::Unreadable));
    }
}
fn skein_io_address(socket: core::net::SocketAddr) -> skein_io::Addr {
    skein_io::Addr::Ip(socket)
}
#[test]
fn access_checks_each_record_and_preserves_the_access_and_accepted_ids() {
    for kind in [Kind::Forgejo, Kind::GitHub] {
        let mut rows = complete(kind);
        put(&mut rows, Item::Accepted, &1_u64.to_le_bytes());
        let byte = u8::try_from(Rng::new(84).between(33, 126)).expect("ASCII");
        put(&mut rows, Item::WebhookSecret, &[byte; 32]);
        for (hooks, made) in [(false, false), (true, true)] {
            if made && kind == Kind::GitHub {
                put(&mut rows, Item::AppMade, &7_u64.to_le_bytes());
            }
            let start = forge_records::access(kind, &expected(kind), rows.forge(1).expect("rows"), &bounds(), hooks)
                .expect("complete");
            assert_eq!(start.access.user, 9);
            assert_eq!(start.access.webhook_secret.is_some(), hooks && kind == Kind::Forgejo);
            match start.access.credential {
                skein_forge::credential::Credential::Token(token) => assert_eq!(token.expose().len(), 32),
                skein_forge::credential::Credential::App { id, key } => {
                    assert_eq!(id, 7);
                    assert!(key.expose() == skein_fake_forge_github::keys::KEYS[0]);
                    assert_eq!(start.accepted.as_ref(), &[1]);
                    assert_eq!(start.slug.as_deref(), Some(b"test-app".as_slice()));
                }
            }
        }
    }
    assert!(Rows::new().forge(1).is_none());
    let mut incomplete = Rows::new();
    put(&mut incomplete, Item::Origin, &expected(Kind::GitHub).encode());
    assert_eq!(
        forge_records::access(
            Kind::GitHub,
            &expected(Kind::GitHub),
            incomplete.forge(1).expect("rows"),
            &bounds(),
            false
        )
        .err(),
        Some(NotConnected::Incomplete)
    );
    for (item, bytes) in [
        (Item::AppId, vec![0; 8]),
        (Item::AppMade, vec![0; 8]),
        (Item::AppSlug, b"Bad".to_vec()),
        (Item::AppKey, vec![0; 1197]),
        (Item::Accepted, vec![1]),
        (Item::User, vec![0; 8]),
    ] {
        let mut rows = complete(Kind::GitHub);
        put(&mut rows, item, &bytes);
        assert_eq!(
            forge_records::access(
                Kind::GitHub,
                &expected(Kind::GitHub),
                rows.forge(1).expect("rows"),
                &bounds(),
                false
            )
            .err(),
            Some(NotConnected::Unreadable)
        );
    }
    for code in [4, 11] {
        let mut rows = complete(Kind::GitHub);
        rows.push((1 << 48) | (1 << 16) | code, Secret::new(Box::from([2, 1])), 4096);
        assert_eq!(
            forge_records::access(
                Kind::GitHub,
                &expected(Kind::GitHub),
                rows.forge(1).expect("rows"),
                &bounds(),
                false
            )
            .err(),
            Some(NotConnected::Unreadable)
        );
    }
    for kind in [Kind::Forgejo, Kind::GitHub] {
        let mut rows = complete(kind);
        let other = if kind == Kind::Forgejo { Kind::GitHub } else { Kind::Forgejo };
        put(&mut rows, Item::Origin, &expected(other).encode());
        assert_eq!(
            forge_records::access(kind, &expected(kind), rows.forge(1).expect("rows"), &bounds(), false).err(),
            Some(NotConnected::OtherForge { kind: other, api: expected(other).api })
        );
    }
}

#[test]
fn foreign_credentials_and_values_past_the_bound_are_refused() {
    let token = [u8::try_from(Rng::new(85).between(33, 126)).expect("ASCII"); 32];
    for (kind, item, bytes, other) in [
        (Kind::Forgejo, Item::AppKey, skein_fake_forge_github::keys::KEYS[0], Kind::GitHub),
        (Kind::GitHub, Item::ApiToken, token.as_slice(), Kind::Forgejo),
    ] {
        let mut rows = complete(kind);
        put(&mut rows, item, bytes);
        assert_eq!(
            forge_records::access(kind, &expected(kind), rows.forge(1).expect("rows"), &bounds(), false).err(),
            Some(NotConnected::OtherForge { kind: other, api: expected(kind).api })
        );
    }
    let mut rows = complete(Kind::GitHub);
    let start =
        forge_records::access(Kind::GitHub, &expected(Kind::GitHub), rows.forge(1).expect("rows"), &bounds(), false)
            .expect("complete without Accepted");
    assert!(start.accepted.is_empty());
    rows.push(
        records::number(1, Item::AppKey),
        Secret::new(records::encode(Item::AppKey, &[0x30, 0]).expect("DER envelope fits schema").into_boxed_slice()),
        1,
    );
    assert_eq!(
        forge_records::access(Kind::GitHub, &expected(Kind::GitHub), rows.forge(1).expect("rows"), &bounds(), false)
            .err(),
        Some(NotConnected::Unreadable)
    );
}

// Literal sizes are the schema ceilings: worker-wire's token, name and
// installation limits, setup's 0x82 DER envelope, and jig's DNS host plus port.
const TOKEN_BYTES: usize = 4096;
const SLUG_BYTES: usize = 1024;
const KEY_BYTES: usize = 65539;
const INSTALLATIONS: usize = 256;
const API_BYTES: usize = 259;

fn scalar_rows() -> [(Item, Vec<u8>); 11] {
    [
        (Item::AdminToken, b"admin".to_vec()),
        (Item::ApiToken, b"api".to_vec()),
        (Item::GitToken, b"git".to_vec()),
        (Item::User, 9_u64.to_le_bytes().to_vec()),
        (Item::WebhookSecret, b"hook".to_vec()),
        (Item::AppId, 7_u64.to_le_bytes().to_vec()),
        (Item::AppMade, 7_u64.to_le_bytes().to_vec()),
        (Item::AppSlug, b"test-app".to_vec()),
        (Item::AppKey, vec![0x30, 0]),
        (Item::Accepted, 1_u64.to_le_bytes().to_vec()),
        (Item::Origin, expected(Kind::GitHub).encode().into_vec()),
    ]
}
#[test]
fn every_row_round_trips_through_its_generated_record() {
    for (item, bytes) in scalar_rows() {
        let encoded = records::encode(item, &bytes).expect("row fits schema");
        assert_eq!(records::decode(item, &encoded).expect("generated row").expose(), bytes);
    }
}
#[test]
fn old_rows_and_rows_under_another_item_are_unreadable() {
    let other = records::encode(Item::GitToken, b"token").expect("token row");
    for bytes in [&[1, b't', b'o', b'k', b'e', b'n'][..], other.as_slice()] {
        assert_eq!(records::decode(Item::ApiToken, bytes), Err(records::Unreadable));
    }
}
#[test]
fn replacing_an_unreadable_row_restores_the_item() {
    let mut rows = Rows::new();
    rows.push(records::number(1, Item::ApiToken), Secret::new(Box::from([1, b'x'])), 4096);
    put(&mut rows, Item::ApiToken, b"new-token");
    assert_eq!(rows.forge(1).expect("configured forge").get(Item::ApiToken), Ok(b"new-token".as_slice()));
}
fn origin_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0]; // Forgejo, followed by a schema text's u32 length.
    bytes.extend_from_slice(&u32::try_from(length).expect("bounded field length").to_be_bytes());
    bytes.extend_from_slice(&vec![b'a'; length]);
    bytes
}
fn bounded_rows(extra: usize) -> [(Item, Vec<u8>); 8] {
    [
        (Item::AdminToken, vec![b'a'; TOKEN_BYTES + extra]),
        (Item::ApiToken, vec![b'a'; TOKEN_BYTES + extra]),
        (Item::GitToken, vec![b'a'; TOKEN_BYTES + extra]),
        (Item::WebhookSecret, vec![b'a'; TOKEN_BYTES + extra]),
        (Item::AppSlug, vec![b'a'; SLUG_BYTES + extra]),
        (Item::AppKey, vec![0; KEY_BYTES + extra]),
        (Item::Accepted, vec![0; (INSTALLATIONS + extra) * 8]),
        (Item::Origin, origin_bytes(API_BYTES + extra)),
    ]
}
#[test]
fn each_row_bound_accepts_its_limit() {
    for (item, bytes) in bounded_rows(0) {
        let encoded = records::encode(item, &bytes).expect("at ceiling");
        assert_eq!(records::decode(item, &encoded).expect("at ceiling").expose(), bytes);
    }
}
#[test]
fn each_row_bound_refuses_one_past_its_limit() {
    for (item, bytes) in bounded_rows(1) {
        assert_eq!(records::encode(item, &bytes), Err(records::Unreadable), "{item:?}");
    }
}
#[test]
fn numeric_rows_are_big_endian_beyond_the_boundary() {
    assert_eq!(
        records::encode(Item::User, &9_u64.to_le_bytes()).expect("user row"),
        &[0, 1, 3, 0, 0, 0, 0, 0, 0, 0, 9]
    );
    assert_eq!(
        records::encode(Item::Accepted, &9_u64.to_le_bytes()).expect("accepted row"),
        &[0, 1, 8, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 9]
    );
}
#[test]
fn pasted_payload_fits_its_unchanged_aggregate_budget() {
    let bytes = vec![b'p'; records::PASTED_PAYLOAD];
    for code in [1, 11] {
        let encoded = records::encode_pasted(&[(code, &bytes)]).expect("pasted value at budget");
        assert_eq!(encoded.len(), 4096);
        let decoded = records::decode_pasted(encoded.expose()).expect("producer record");
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].0, code);
        assert!(decoded[0].1.expose() == bytes);
    }
}
#[test]
fn pasted_payload_one_past_the_aggregate_budget_is_refused() {
    for code in [1, 11] {
        let bytes = vec![b'p'; records::PASTED_PAYLOAD + 1];
        let items = [(code, bytes.as_slice())];
        assert_eq!(records::encode_pasted(&items), Err(records::Unreadable));
        assert_eq!(records::decode_pasted(&untrusted_pasted(&items)), Err(records::PastedError::Unreadable));
    }
}

fn untrusted_row(item: Item, bytes: &[u8]) -> Vec<u8> {
    let mut encoded = vec![0, 1, u8::try_from(item.code() - 1).expect("ten item variants")];
    if item != Item::Origin {
        let count = if item == Item::Accepted { bytes.len() / 8 } else { bytes.len() };
        encoded.extend_from_slice(&u32::try_from(count).expect("bounded item count").to_be_bytes());
    }
    encoded.extend_from_slice(bytes);
    encoded
}
#[test]
fn each_row_decoder_refuses_one_past_its_limit() {
    for (item, bytes) in bounded_rows(1) {
        assert_eq!(records::decode(item, &untrusted_row(item, &bytes)), Err(records::Unreadable), "{item:?}");
    }
}

#[test]
fn pasted_items_round_trip_in_their_order() {
    let mut rng = Rng::new(86);
    let bytes = temper_codecs_world::drawn(&mut rng, 32);
    let items = [(2, bytes.as_slice()), (3, bytes.as_slice()), (5, bytes.as_slice())];
    let encoded = records::encode_pasted(&items).expect("three items");
    let decoded = records::decode_pasted(encoded.expose()).expect("three items");
    assert_eq!(decoded.len(), items.len());
    for ((code, value), (expected, bytes)) in decoded.iter().zip(items) {
        assert_eq!(*code, expected);
        assert!(value.expose() == bytes);
    }
}

// Use generated item encoders with an unchecked count to reach lists the schema refuses.
fn untrusted_pasted(items: &[(u16, &[u8])]) -> Box<[u8]> {
    use records::v1;
    let mut size = size_of::<u16>() + size_of::<u32>();
    for &(_, bytes) in items {
        size += size_of::<u8>() + size_of::<u32>() + bytes.len();
    }
    let mut out = skein_lib::Writer::new(size);
    out.put(&records::VERSION.to_be_bytes()).expect("version");
    out.put(&u32::try_from(items.len()).expect("test count").to_be_bytes()).expect("count");
    for &(code, bytes) in items {
        let item = match code {
            1 => v1::PastedItem::AdminToken(
                v1::AdminToken::new(&v1::CEILINGS, v1::AdminTokenParts { value: Secret::new(bytes.into()) })
                    .expect("token"),
            ),
            2 => v1::PastedItem::ApiToken(
                v1::ApiToken::new(&v1::CEILINGS, v1::ApiTokenParts { value: Secret::new(bytes.into()) })
                    .expect("token"),
            ),
            3 => v1::PastedItem::GitToken(
                v1::GitToken::new(&v1::CEILINGS, v1::GitTokenParts { value: Secret::new(bytes.into()) })
                    .expect("token"),
            ),
            5 => v1::PastedItem::WebhookSecret(
                v1::WebhookSecret::new(&v1::CEILINGS, v1::WebhookSecretParts { value: Secret::new(bytes.into()) })
                    .expect("secret"),
            ),
            11 => v1::PastedItem::Pem(
                v1::Pem::new(&v1::CEILINGS, v1::PemParts { value: Secret::new(bytes.into()) }).expect("PEM"),
            ),
            _ => panic!("test item"),
        };
        item.encode(&mut out).expect("test bundle fits");
    }
    out.finish()
}

#[test]
fn invalid_pasted_lists_are_unreadable() {
    let mut rng = Rng::new(87);
    let bytes = temper_codecs_world::drawn(&mut rng, 32);
    let value = bytes.as_slice();
    for items in [
        &[][..],
        &[(2, value), (2, value)],
        &[(2, &[][..])],
        &[(11, &[][..])],
        &[(1, value), (2, value), (3, value), (5, value)],
    ] {
        assert_eq!(records::encode_pasted(items), Err(records::Unreadable));
        assert_eq!(records::decode_pasted(&untrusted_pasted(items)), Err(records::PastedError::Unreadable));
    }
    for code in [0, 12, u16::MAX] {
        assert_eq!(records::encode_pasted(&[(code, value)]), Err(records::Unreadable));
    }
    let mut unknown = untrusted_pasted(&[(1, value)]);
    // The tag follows the generated record's version and list count.
    unknown[size_of::<u16>() + size_of::<u32>()] = 11;
    assert_eq!(records::decode_pasted(&unknown), Err(records::PastedError::Unreadable));
}

#[test]
fn pasted_bundles_of_another_version_are_refused() {
    let mut bytes = untrusted_pasted(&[]);
    bytes[..size_of::<u16>()].copy_from_slice(&(records::VERSION + 1).to_be_bytes());
    assert_eq!(records::decode_pasted(&bytes), Err(records::PastedError::Version));
}
