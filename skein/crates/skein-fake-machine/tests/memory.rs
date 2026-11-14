//! A synced file retains its contents once (testing/simulator.md, 3.3).
#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

#[cfg(test)]
mod tests {
    use skein_fake_machine::{How, Item, Machine};
    use skein_heap::Span;

    #[test]
    fn a_synced_file_retains_one_copy_without_growth_capacity() {
        const FILE_BYTES: i64 = 1_048_576;
        let span = Span::start();
        let mut machine = Machine::new();
        let root = machine.lay(&[Item::file(b"file", b"")]);
        let file = machine.open(root, b"file", How::Write).unwrap();
        // Separate writes leave both pending copies and spare growth capacity
        // before sync, as an appended store segment does.
        for _ in 0..1_025_u32 {
            machine.append(file, &[7; 1024]).unwrap();
        }
        machine.truncate(file, u64::try_from(FILE_BYTES).unwrap()).unwrap();
        machine.sync(file);
        machine.sync(file);
        let held = span.end().net;
        assert!(held >= FILE_BYTES, "the file's bytes are counted: {held}");
        assert!(held <= FILE_BYTES + FILE_BYTES / 4, "synced file retained {held} bytes");
        assert_eq!(machine.read(file, 0, 16).unwrap(), [7; 16]);
        machine.close(file);
        machine.close(root);
    }
}
