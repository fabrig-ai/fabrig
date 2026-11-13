//! Preparation rows for standard descriptor file actions (runtime/io/processes.md, 1).

use skein_io::kernel::{Pipe, Spawn, Way};

#[test]
fn inherited_standard_entries_prepare_no_pipe_sources_or_default_nulls() {
    let root = crate::open_root(std::path::Path::new(".")).expect("spawn root");
    for descriptor in 0..3 {
        let command = Spawn {
            program: Box::from(&b"/bin/true"[..]),
            args: Box::new([]),
            env: Box::new([]),
            root,
            dir: Box::new([]),
            pipes: Box::new([Pipe { child: descriptor, way: Way::Inherit, parent: None }]),
        };
        let (_dir, parents, children, nulls) = super::prepare(&command).expect("prepare inherited descriptor");
        assert!(parents.iter().all(Option::is_none), "inherit has no parent end");
        assert!(children.iter().all(Option::is_none), "inherit makes no pipe source");
        for (standard, source) in (0..3).zip(nulls) {
            assert_eq!(source.is_none(), standard == descriptor, "only unrequested standards default to null");
        }
    }
    crate::close_startup_fd(root);
}
