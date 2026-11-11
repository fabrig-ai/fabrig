//! Public vectors from RFC 8291, Appendix A (and its Section 5 body).
use crate::{ES256_KEY_BYTES, es256_public, seal_aes128gcm, seal_aes128gcm_from, sign_es256, verify_es256};
use alloc::boxed::Box;
use alloc::vec::Vec;
use skein_lib::base64::{Alphabet, decode};

const SECRET: &[u8] = b"kyrL1jIIOHEzg3sM2ZWRHDRB62YACZhhSlknJ672kSs";
const SENDER: &[u8] = b"BP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A8";
const PRIVATE: &[u8] = b"yfWPiYE-n46HLnH0KqZOF1fJJU3MYrct3AELtAQ-oRw";
const UA: &[u8] = b"BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4";
const SALT: &[u8] = b"DGv6ra1nlYgDCS1FRnbzlw";
const AUTH: &[u8] = b"BTBZMqHH6r4Tts7J_aSIgg";
const TEXT: &[u8] = b"When I grow up, I want to be a watermelon";
const BODY: &[u8] = b"DGv6ra1nlYgDCS1FRnbzlwAAEABBBP4z9KsN6nGRTbVYI_c7VJSPQTBtkgcy27mlmlMoZIIgDll6e3vCYLocInmYWAmS6TlzAC8wEqKK6PBru3jl7A_yl95bQpu6cVPTpK4Mqgkf1CXztLVBSt2Ks3oZwbuwXPXLWyouBWLVWGNWQexSgSxsj_Qulcy4a-fN";
fn raw(text: &[u8]) -> Box<[u8]> {
    decode(Alphabet::Url, text, 512).expect("RFC base64url")
}
fn key() -> Vec<u8> {
    // OpenSSL's PKCS#8 v1 PrivateKeyInfo / ECPrivateKey, with the RFC's public test key.
    let mut key = Vec::from([
        0x30, 0x81, 0x87, 2, 1, 0, 0x30, 0x13, 6, 7, 0x2a, 0x86, 0x48, 0xce, 0x3d, 2, 1, 6, 8, 0x2a, 0x86, 0x48, 0xce,
        0x3d, 3, 1, 7, 4, 0x6d, 0x30, 0x6b, 2, 1, 1, 4, 0x20,
    ]);
    key.extend_from_slice(&raw(PRIVATE));
    key.extend_from_slice(&[0xa1, 0x44, 3, 0x42, 0]);
    key.extend_from_slice(&raw(SENDER));
    key
}
#[test]
fn es256_signs_and_refuses_changed_message_or_signature() {
    let key = key();
    let public = es256_public(&key).expect("OpenSSL PKCS#8 v1 form accepted");
    assert_eq!(public.as_slice(), raw(SENDER).as_ref());
    let signature = sign_es256(&key, b"message").expect("P-256 signs");
    assert!(verify_es256(&public, b"message", &signature));
    assert!(!verify_es256(&public, b"Message", &signature));
    let mut changed = signature;
    changed[0] ^= 1;
    assert!(!verify_es256(&public, b"message", &changed));
}
#[test]
fn es256_refuses_other_keys_and_past_bound() {
    let p384 = ring::signature::EcdsaKeyPair::generate_pkcs8(
        &ring::signature::ECDSA_P384_SHA384_FIXED_SIGNING,
        &ring::rand::SystemRandom::new(),
    )
    .expect("P-384 test key");
    let mut oversized = key();
    oversized.resize(ES256_KEY_BYTES.checked_add(1).expect("past bound"), 0);
    let rsa = super::fixture("rsa-1024.der");
    let mut pkcs8 = Vec::from([0x30, 0x82]);
    pkcs8.extend_from_slice(
        &u16::try_from(rsa.len().checked_add(22).expect("PKCS#8 wrapper")).expect("length").to_be_bytes(),
    );
    pkcs8.extend_from_slice(&[2, 1, 0, 0x30, 0x0d, 6, 9, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 1, 1, 1, 5, 0, 4, 0x82]);
    pkcs8.extend_from_slice(&u16::try_from(rsa.len()).expect("RSA length").to_be_bytes());
    pkcs8.extend_from_slice(&rsa);
    for key in [&[][..], pkcs8.as_slice(), p384.as_ref(), &oversized] {
        assert!(sign_es256(key, TEXT).is_none());
        assert!(es256_public(key).is_none());
    }
}
#[test]
fn seal_reproduces_rfc8291_appendix_a() {
    let body = seal_aes128gcm_from(
        &raw(SECRET).as_ref().try_into().expect("secret"),
        &raw(SENDER).as_ref().try_into().expect("sender"),
        &raw(SALT).as_ref().try_into().expect("salt"),
        &raw(UA).as_ref().try_into().expect("ua"),
        &raw(AUTH).as_ref().try_into().expect("auth"),
        TEXT,
    )
    .expect("seal");
    assert_eq!(body, raw(BODY));
}
#[test]
fn drawn_seal_has_header_and_bounds() {
    let ua = raw(UA).as_ref().try_into().expect("ua");
    let auth = raw(AUTH).as_ref().try_into().expect("auth");
    let first = seal_aes128gcm(&ua, &auth, TEXT).expect("drawn seal");
    let second = seal_aes128gcm(&ua, &auth, TEXT).expect("another seal");
    assert_eq!(first.len(), TEXT.len().checked_add(103).expect("body length"));
    assert_ne!(&first[..16], &second[..16], "fresh salt");
    assert_eq!(&first[16..22], &[0, 0, 16, 0, 65, 4]);
    assert_ne!(&first[21..86], &second[21..86], "fresh key");
    assert!(seal_aes128gcm(&ua, &auth, &[0; 3993]).is_some());
    assert!(seal_aes128gcm(&ua, &auth, b"").is_none());
    assert!(seal_aes128gcm(&ua, &auth, &[0; 3994]).is_none());
    assert!(seal_aes128gcm(&[0; 65], &auth, TEXT).is_none());
}
