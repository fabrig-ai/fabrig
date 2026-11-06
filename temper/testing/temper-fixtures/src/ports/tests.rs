use super::*;
use std::collections::BTreeSet;

const DRAWS: usize = 3 * BLOCK as usize;

#[test]
fn the_range_is_read_from_the_kernels_file() {
    assert_eq!(first_of("32768\t60999\n"), Some(32768));
    assert_eq!(first_of("20000\t60999\n"), Some(20000));
    assert_eq!(first_of("garbage"), None);
}

#[test]
fn a_chosen_port_is_below_the_ephemeral_range() {
    let listeners = reserve::<DRAWS>(&[]);
    let first = ephemeral_first();
    eprintln!("{RANGE_FILE}: first {first}");
    for listener in listeners {
        assert!((LOWEST..first).contains(&listener.local_addr().expect("address").port()));
    }
}

#[test]
fn two_processes_choose_no_port_in_common() {
    let mut one = Blocks::new(ephemeral_first(), 0);
    let mut two = Blocks::new(ephemeral_first(), 0);
    let first: BTreeSet<_> = (0..DRAWS).map(|_| one.listener(&[]).local_addr().expect("address")).collect();
    let second: BTreeSet<_> = (0..DRAWS).map(|_| two.listener(&[]).local_addr().expect("address")).collect();
    assert_eq!(first.len(), DRAWS);
    assert_eq!(second.len(), DRAWS);
    assert!(first.is_disjoint(&second));
}

#[test]
fn a_released_port_is_no_other_processs_and_listens_again() {
    let mut one = Blocks::new(ephemeral_first(), 0);
    let listener = one.listener(&[]);
    let address = listener.local_addr().expect("address");
    drop(listener);
    let mut two = Blocks::new(ephemeral_first(), 0);
    for _ in 0..DRAWS {
        assert_ne!(two.listener(&[]).local_addr().expect("other address"), address);
    }
    let _again = TcpListener::bind(address).expect("released port listens again");
}

#[test]
fn a_block_is_free_once_its_process_ends() {
    let mut one = Blocks::new(ephemeral_first(), 0);
    let address = one.listener(&[]).local_addr().expect("first address");
    drop(one);
    let mut two = Blocks::new(ephemeral_first(), 0);
    assert_eq!(two.listener(&[]).local_addr().expect("second address"), address);
}

#[test]
fn a_port_someone_holds_is_passed_over() {
    let mut blocks = Blocks::new(ephemeral_first(), 0);
    let _first = blocks.listener(&[]);
    let occupied = TcpListener::bind((Ipv4Addr::LOCALHOST, blocks.next.start)).expect("hold next port");
    assert_ne!(
        blocks.listener(&[]).local_addr().expect("chosen address"),
        occupied.local_addr().expect("held address")
    );
}

#[test]
#[should_panic(expected = "ip_local_port_range")]
fn a_range_too_narrow_says_so() {
    let _blocks = Blocks::new(LOWEST, 0);
}
