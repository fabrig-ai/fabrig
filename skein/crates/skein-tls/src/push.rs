//! Web Push's signature and seal (net/tls.md, 4; RFC 8291, 3).
use alloc::boxed::Box;
use ring::{
    aead, agreement, hkdf,
    rand::{SecureRandom, SystemRandom},
    signature::{self, KeyPair},
};
use skein_lib::Writer;

pub const ES256_KEY_BYTES: usize = 256;

fn key(bytes: &[u8]) -> Option<signature::EcdsaKeyPair> {
    if cfg!(all(target_arch = "wasm32", target_os = "unknown")) || bytes.len() > ES256_KEY_BYTES {
        return None;
    }
    signature::EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_FIXED_SIGNING, bytes, &SystemRandom::new()).ok()
}

/// Signs PKCS#8 P-256 with its public half; none on the portable target.
#[must_use]
pub fn sign_es256(bytes: &[u8], message: &[u8]) -> Option<[u8; 64]> {
    let signature = key(bytes)?.sign(&SystemRandom::new(), message).ok()?;
    signature.as_ref().try_into().ok()
}

#[must_use]
pub fn es256_public(bytes: &[u8]) -> Option<[u8; 65]> {
    key(bytes)?.public_key().as_ref().try_into().ok()
}

#[must_use]
pub fn verify_es256(public: &[u8; 65], message: &[u8], signature: &[u8; 64]) -> bool {
    if cfg!(all(target_arch = "wasm32", target_os = "unknown")) {
        return false;
    }
    signature::UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, public).verify(message, signature).is_ok()
}

#[must_use]
pub fn seal_aes128gcm(p256dh: &[u8; 65], auth: &[u8; 16], payload: &[u8]) -> Option<Box<[u8]>> {
    if cfg!(all(target_arch = "wasm32", target_os = "unknown")) || !(1..=3993).contains(&payload.len()) {
        return None;
    }
    let rng = SystemRandom::new();
    let private = agreement::EphemeralPrivateKey::generate(&agreement::ECDH_P256, &rng).ok()?;
    let sender = private.compute_public_key().ok()?;
    let sender = sender.as_ref().try_into().ok()?;
    let mut salt = [0; 16];
    rng.fill(&mut salt).ok()?;
    let public = agreement::UnparsedPublicKey::new(&agreement::ECDH_P256, p256dh);
    // ring's agreement consumes the private key, and lends the secret only here.
    agreement::agree_ephemeral(private, &public, |secret| {
        seal_aes128gcm_from(secret.try_into().ok()?, &sender, &salt, p256dh, auth, payload)
    })
    .ok()?
}

/// RFC 8291's deterministic part: one record, no padding beyond its delimiter.
#[must_use]
pub fn seal_aes128gcm_from(
    secret: &[u8; 32],
    sender: &[u8; 65],
    salt: &[u8; 16],
    p256dh: &[u8; 65],
    auth: &[u8; 16],
    payload: &[u8],
) -> Option<Box<[u8]>> {
    if cfg!(all(target_arch = "wasm32", target_os = "unknown")) || !(1..=3993).contains(&payload.len()) {
        return None;
    }
    let ikm = expand(auth, secret, &[b"WebPush: info\0", p256dh, sender])?;
    let cek = expand(salt, &ikm, &[b"Content-Encoding: aes128gcm\0"])?;
    let nonce = expand(salt, &ikm, &[b"Content-Encoding: nonce\0"])?;
    let key = aead::UnboundKey::new(&aead::AES_128_GCM, cek.get(..16)?).ok()?;
    let nonce = aead::Nonce::assume_unique_for_key(nonce.get(..12)?.try_into().ok()?);
    let mut record = Writer::new(payload.len().checked_add(1)?);
    record.put(payload).ok()?;
    record.put(&[2]).ok()?;
    let mut record = record.finish();
    let tag = aead::LessSafeKey::new(key).seal_in_place_separate_tag(nonce, aead::Aad::empty(), &mut record).ok()?;
    let mut body = Writer::new(payload.len().checked_add(103)?);
    body.put(salt).ok()?;
    body.put(&4096_u32.to_be_bytes()).ok()?;
    body.put(&[65]).ok()?;
    body.put(sender).ok()?;
    body.put(&record).ok()?;
    body.put(tag.as_ref()).ok()?;
    Some(body.finish())
}

// HKDF's first 32 output bytes; shorter CEK and nonce take their prefixes.
fn expand(salt: &[u8], secret: &[u8], info: &[&[u8]]) -> Option<[u8; 32]> {
    let prk = hkdf::Salt::new(hkdf::HKDF_SHA256, salt).extract(secret);
    let okm = prk.expand(info, hkdf::HKDF_SHA256).ok()?;
    let mut bytes = [0; 32];
    okm.fill(&mut bytes).ok()?;
    Some(bytes)
}
