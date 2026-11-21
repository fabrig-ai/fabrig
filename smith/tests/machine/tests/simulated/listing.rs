use skein_io::kernel;
use smith_domain::tools;

#[test]
fn the_tools_fake_lists_like_the_machine() {
    let long = [vec![b'a'; 200], b"\0lib.rs\0main.rs\0".to_vec()].concat();
    for output in [
        b"".as_slice(),
        b"Cargo.toml\0big.txt\0long.txt\0src/lib.rs\0src/main.rs\0vendor/lib/lib.rs\0",
        b"lib.rs\0main.rs\0",
        b"lib.rs\0",
        b"guide.md\0",
        b"a\0b\0c\0d\0e\0f\0",
        &long,
        b"src/deep/only.rs\0src/empty.rs\0",
    ] {
        for depth in [tools::Depth::Whole, tools::Depth::Levels(1), tools::Depth::Levels(2)] {
            for max in [0, 1, 2, 16] {
                for bytes in [0, 32, 37, 38, 39, 66, 512, 4096] {
                    for code in [0, 1, 127] {
                        let mut listing = smith_protocol_machine::Listing::new(max, bytes, depth, 4096);
                        for byte in output {
                            listing.push(*byte);
                        }
                        let expected = listing.finish(Some(kernel::Exit::Code(code)), false);
                        assert_eq!(
                            smith_tools_world::translate::listed(
                                output,
                                skein_fake_checkout::Exit::Code(code),
                                depth,
                                max,
                                bytes
                            ),
                            expected
                        );
                    }
                }
            }
        }
    }
}
