#[test]
fn a_held_post_shell_request_retries_with_its_result_and_finishes() {
    temper_one_process_world::hung_model::run(true);
}

#[test]
fn exhausted_post_shell_requests_charge_one_run_try_without_stopping_the_agent() {
    temper_one_process_world::hung_model::run(false);
}
