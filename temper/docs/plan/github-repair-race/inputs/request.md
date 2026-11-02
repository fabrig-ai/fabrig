# Request: the github_repair replacement race, fixed on main

From dogfood/plan via agents/loop, 2026-11-02 ~20:25Z. The test
`temper-real-loop-world::suite github_repair::a_replacement_github_repair_after_release_receives_the_job_log_again`
(from main, finding-71, 302072db31) has a race. Its last `change show --until held` can match
before the replacement repair runs, so under load it fails with "story completion obligations:
undeclared engine task 1: Running". It has failed 5 times across plans, and at 20:20Z it rejected
reference-m4's trunk in the queue. Every landing to main can meet it.

`reference-m7a/repair-1` fixed it on M7a's trunk in commit 063edf0d7e ("Wait for the replacement
repair verdict before ending its story"; `temper/tests/real-loop/tests/suite/github_repair.rs`,
`replacement_acts`, +8 -2). The patch is beside this request as `fix.patch`.

One task: apply the same fix to main, cherry-picking 063edf0d7e's change to `github_repair.rs`
and nothing else. Then run the test alone, and in its suite, several times under heavy. No other
change. When M7a's trunk later lands, the same change on both sides merges cleanly.
