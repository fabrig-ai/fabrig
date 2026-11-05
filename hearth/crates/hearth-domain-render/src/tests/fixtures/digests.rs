use hearth_lib::encode::Digest;

pub use hearth_test_fixtures::digits::hex;

pub fn digest(text: &str) -> Digest {
    let bytes = hex(text);
    Digest(bytes.as_ref().try_into().expect("a digest has thirty-two bytes"))
}
