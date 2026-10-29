//! The shell's bounded trust bytes composed with TLS's own parser (tls.md, 3.4).

use std::fs;

use skein_scratch::Scratch;
use skein_shell::{RootBounds, Trust, read_trust_roots};
use skein_tls::Config;
use skein_tls_world::pki;

fn certificate(bytes: &[u8]) -> Vec<u8> {
    let alphabet = skein_lib::base64::Alphabet::Standard;
    let len =
        skein_lib::base64::encoded_len(alphabet, u32::try_from(bytes.len()).expect("fixture fits")).expect("fits");
    let mut writer = skein_lib::Writer::new(usize::try_from(len).expect("fits"));
    skein_lib::base64::encode(alphabet, bytes, &mut writer).expect("measured room");
    let mut text = Vec::from(b"-----BEGIN CERTIFICATE-----\n".as_slice());
    text.extend_from_slice(&writer.finish());
    text.extend_from_slice(b"\n-----END CERTIFICATE-----\n");
    text
}

#[test]
fn startup_pem_and_der_files_supply_tls_configuration() {
    let scratch = Scratch::new("tls-startup");
    let path = scratch.path().join("roots.pem");
    let text = certificate(pki::ROOT);
    fs::write(&path, &text).expect("plant fixture bundle");
    let bounds = RootBounds {
        bundle_bytes: u32::try_from(text.len()).expect("bundle fits"),
        certificates: 1,
        certificate_bytes: u32::try_from(pki::ROOT.len()).expect("root fits"),
    };
    let roots = read_trust_roots(&Trust::Machine { ssl_cert_file: Some(path) }, &bounds).expect("bounded PEM roots");
    // PEM skips are owned by skein-shell's startup::lower_tests::
    // pem_preserves_certificate_order_and_skips_other_blocks_without_decoding and
    // pem_recovers_malformed_blocks_without_spending_the_certificate_count.
    let (_, parsed) = Config::from_der(&roots.certificates, &[b"h2"]).expect("fixture root remains");
    assert_eq!(parsed.taken, 1);
    // DER skips and empty roots are owned by skein-tls's tests::config::
    // der_roots_count_supported_certificates_and_skip_the_rest.
    let roots = read_trust_roots(&Trust::Der("fixtures/root.der".into()), &bounds).expect("DER file");
    let (_, parsed) = Config::from_der(&roots.certificates, &[]).expect("DER root remains");
    assert_eq!(parsed.taken, 1);
}
