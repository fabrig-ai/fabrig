#[test]
fn delegate_resumes_its_withheld_wait_on_nothing_to_await() {
    temper_one_process_world::parks::withheld_wait();
}
