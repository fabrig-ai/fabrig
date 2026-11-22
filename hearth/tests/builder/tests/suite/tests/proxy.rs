use hearth_test_mirror::{Proxy, testing as fixtures, testing::URL};
use skein_scratch::Scratch;

const FOUND: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nsigned\n";
#[test]
fn the_proxy_serves_a_laid_mirror_over_loopback() {
    let scratch = Scratch::new("test-mirror-proxy");
    let proxy = Proxy::serve(&fixtures::laid(scratch.path())).expect("a loopback port");
    assert_eq!(fixtures::get(&proxy, URL), FOUND);
}
