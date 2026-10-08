use rustix::process::Pid;

// Darwin can reject killpg with EPERM when only the zombie leader remains.
// Do not dismiss permission failures for groups containing any other member.
#[allow(
    unsafe_code,
    reason = "reviewed fixed-buffer libproc query unavailable through rustix"
)]
pub(super) fn sole_group_member(group: Pid) -> bool {
    let raw = group.as_raw_nonzero().get();
    let mut members: [libc::pid_t; 2] = [0; 2];
    let size = libc::c_int::try_from(std::mem::size_of_val(&members))
        .expect("two process IDs fit the native buffer size");
    // SAFETY: the buffer is writable for exactly size bytes. The caller retains
    // the exited, unreaped group leader, reserving its PID throughout this query.
    // Two slots distinguish the sole leader from a larger or truncated group.
    let count = unsafe { libc::proc_listpgrppids(raw, members.as_mut_ptr().cast(), size) };
    count == 1 && members[0] == raw
}
