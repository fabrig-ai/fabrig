use skein_world::testing::open_cgroup_member;

#[test]
fn a_cgroup_candidate_must_prove_its_opened_membership() {
    let pid = std::process::id();
    let foreign = std::path::Path::new("/sys/fs/cgroup/skein-foreign-candidate");
    assert_eq!(open_cgroup_member(pid, Some(foreign)), Ok(None), "a foreign cgroup grants no ownership");
    assert_eq!(open_cgroup_member(pid, None), Ok(None), "a number alone grants no ownership");
    let membership = std::fs::read_to_string("/proc/self/cgroup").expect("own cgroup");
    let relative = membership.lines().find_map(|line| line.strip_prefix("0::")).expect("cgroup v2");
    let own = std::path::Path::new("/sys/fs/cgroup").join(relative.trim_start_matches('/'));
    let descriptor = open_cgroup_member(pid, Some(&own)).expect("validate membership").expect("own member");
    skein_shell::keeper::close_keeper_fd(descriptor);
}
