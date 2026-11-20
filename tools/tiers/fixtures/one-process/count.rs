// Instruction inheritance fixture (tools/tiers/design.md, 11.10).
#[test]
fn spin() {
    std::thread::spawn(|| {
        for value in 0..10_000_000 {
            std::hint::black_box(value);
        }
    })
    .join()
    .unwrap();
}

#[test]
fn idle() {}
