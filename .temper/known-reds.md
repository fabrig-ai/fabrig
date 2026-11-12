# Known reds

Failing on the goal branch, inherited from main, owned by no live change.

The test below fails under load (it asserts settling within 180 ms of wall
clock) and passes alone. The person listed it on 2026-11-12. A separate
health goal makes it hold under load and removes this entry when it lands.

- test smith-bench::suite runner::an_overflowing_stderr_retains_its_prefix_and_settles_the_tree
