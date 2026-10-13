//! Native process-leader RSS observations, independent of workload policy.
//! Linux retains page-based /proc measurements; Darwin reports native ps RSS.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(target_os = "macos")]
use std::process::{Command, Stdio};
#[cfg(target_os = "linux")]
use std::{
    fs::{self, File},
    io::Read as _,
    path::Path,
};

use std::{fmt, io, path::PathBuf, process::ExitStatus};

/// Failure to observe a native process table or sum its resident bytes.
#[derive(Debug)]
pub enum SamplingError {
    /// Native filesystem or process invocation failed.
    Io(io::Error),
    /// A native record was malformed or its byte arithmetic overflowed.
    InvalidData(&'static str),
    /// The native process-table command failed.
    CommandFailed {
        /// Executed native program.
        program: PathBuf,
        /// Original exit status.
        status: ExitStatus,
    },
    /// This host has no supported process-table observation backend.
    UnsupportedHost,
}

impl fmt::Display for SamplingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(formatter),
            Self::InvalidData(message) => formatter.write_str(message),
            Self::CommandFailed { program, status } => {
                write!(formatter, "{} exited with {status}", program.display())
            }
            Self::UnsupportedHost => formatter.write_str("native sampling requires Linux or macOS"),
        }
    }
}

impl std::error::Error for SamplingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for SamplingError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// One process leader observed in a native table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Process {
    /// Observed parent process identifier.
    pub parent: u32,
    /// Native resident memory in bytes; shared pages may occur in multiple processes.
    pub resident_bytes: u64,
}

/// Native observations indexed by process-leader identifier.
pub type Snapshot = BTreeMap<u32, Process>;

/// Measurement-source identity; consumers retain it with their comparisons.
#[must_use]
pub const fn source() -> &'static str {
    if cfg!(target_os = "linux") {
        "linux-proc-stat"
    } else if cfg!(target_os = "macos") {
        "macos-ps-rss"
    } else {
        "unsupported"
    }
}

#[cfg(any(target_os = "linux", test))]
fn parse_stat(stat: &str, page_bytes: u64) -> Result<Process, SamplingError> {
    // comm can contain spaces and closing parentheses. Remaining fields start
    // at state (3), then PPID (4), and RSS in pages (24).
    let (_, fields) = stat.rsplit_once(')').ok_or(SamplingError::InvalidData(
        "missing process command terminator",
    ))?;
    let mut fields = fields.split_whitespace();
    let parent = fields
        .nth(1)
        .and_then(|field| field.parse().ok())
        .ok_or(SamplingError::InvalidData("invalid process parent"))?;
    let pages: u64 = fields
        .nth(19)
        .and_then(|field| field.parse().ok())
        .ok_or(SamplingError::InvalidData("invalid process RSS pages"))?;
    let resident_bytes = pages
        .checked_mul(page_bytes)
        .ok_or(SamplingError::InvalidData("RSS byte count overflow"))?;
    Ok(Process {
        parent,
        resident_bytes,
    })
}

#[cfg(target_os = "linux")]
fn process_snapshot(
    proc: &Path,
    page_bytes: u64,
    record_bytes: u64,
) -> Result<Snapshot, SamplingError> {
    let mut processes = Snapshot::new();
    for entry in fs::read_dir(proc)? {
        let entry = entry?;
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<u32>().ok())
        else {
            continue;
        };
        let stat = (|| -> io::Result<String> {
            let mut stat = String::new();
            File::open(entry.path().join("stat"))?
                .take(record_bytes)
                .read_to_string(&mut stat)?;
            Ok(stat)
        })();
        match stat {
            Ok(stat) => {
                processes.insert(pid, parse_stat(&stat, page_bytes)?);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
                ) => {}
            Err(error) if error.raw_os_error() == Some(rustix::io::Errno::SRCH.raw_os_error()) => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(processes)
}

#[cfg(any(target_os = "macos", test))]
fn parse_ps(output: &str) -> Result<Snapshot, SamplingError> {
    let mut processes = Snapshot::new();
    for line in output.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split_whitespace();
        let pid = fields
            .next()
            .and_then(|field| field.parse::<u32>().ok())
            .ok_or(SamplingError::InvalidData("invalid ps PID"))?;
        let parent = fields
            .next()
            .and_then(|field| field.parse().ok())
            .ok_or(SamplingError::InvalidData("invalid ps parent"))?;
        let rss: u64 = fields
            .next()
            .and_then(|field| field.parse().ok())
            .ok_or(SamplingError::InvalidData("invalid ps RSS"))?;
        if fields.next().is_some() {
            return Err(SamplingError::InvalidData("unexpected ps fields"));
        }
        // Darwin ps documents RSS in 1024-byte units, not host memory pages.
        let resident_bytes = rss
            .checked_mul(1024)
            .ok_or(SamplingError::InvalidData("ps RSS byte count overflow"))?;
        if processes
            .insert(
                pid,
                Process {
                    parent,
                    resident_bytes,
                },
            )
            .is_some()
        {
            return Err(SamplingError::InvalidData(
                "duplicate process leader in ps output",
            ));
        }
    }
    Ok(processes)
}

/// Observe process leaders using Linux /proc or the macOS native ps table.
///
/// `record_bytes` bounds each Linux stat record. Consumers own sampling cadence,
/// workload selection and total snapshot memory; no elapsed-time deadline is set.
/// Processes that disappear or deny stat access during Linux enumeration are skipped.
/// macOS ps RSS uses 1024-byte units, independently of native memory page size.
///
/// # Errors
/// Returns native I/O, command, malformed-record or checked-arithmetic failures.
pub fn snapshot(record_bytes: u64) -> Result<Snapshot, SamplingError> {
    #[cfg(not(target_os = "linux"))]
    let _ = record_bytes;
    #[cfg(target_os = "linux")]
    {
        let page_bytes = rustix::param::page_size();
        let page_bytes = u64::try_from(page_bytes)
            .ok()
            .filter(|value| *value > 0)
            .ok_or(SamplingError::InvalidData("native page size unavailable"))?;
        process_snapshot(Path::new("/proc"), page_bytes, record_bytes)
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/bin/ps")
            .args(["-A", "-o", "pid=,ppid=,rss="])
            .env("LC_ALL", "C")
            .stderr(Stdio::inherit())
            .output()?;
        if !output.status.success() {
            return Err(SamplingError::CommandFailed {
                program: "/bin/ps".into(),
                status: output.status,
            });
        }
        let output = std::str::from_utf8(&output.stdout)
            .map_err(|_| SamplingError::InvalidData("ps output is not UTF-8"))?;
        parse_ps(output)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    Err(SamplingError::UnsupportedHost)
}

/// Sum observed resident bytes for a root and its descendants, counting each leader once.
///
/// Missing roots yield zero; cycles cannot repeat an observation. This is a sampled
/// sum of native RSS, not unique physical memory or proof that a process is still live.
///
/// # Errors
/// Returns an arithmetic overflow rather than wrapping the resident byte total.
pub fn tree_rss(root: u32, processes: &Snapshot) -> Result<(u64, usize), SamplingError> {
    if !processes.contains_key(&root) {
        return Ok((0, 0));
    }
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (pid, process) in processes {
        children.entry(process.parent).or_default().push(*pid);
    }
    let mut pending = vec![root];
    let mut seen = BTreeSet::new();
    let mut resident = 0_u64;
    while let Some(pid) = pending.pop() {
        if !seen.insert(pid) {
            continue;
        }
        resident = resident
            .checked_add(processes[&pid].resident_bytes)
            .ok_or(SamplingError::InvalidData("process-tree RSS overflow"))?;
        if let Some(children) = children.get(&pid) {
            pending.extend(children);
        }
    }
    Ok((resident, seen.len()))
}

#[cfg(test)]
mod tests {
    use super::SamplingError;
    use super::{Process, Snapshot, parse_ps, parse_stat, snapshot, tree_rss};

    #[test]
    fn tree_counts_runner_and_descendants_but_not_parent_or_sibling() {
        let processes: Snapshot = [
            (1, 0, 10000),
            (10, 1, 100),
            (11, 1, 20000),
            (20, 10, 200),
            (21, 20, 300),
            (22, 10, 400),
        ]
        .into_iter()
        .map(|(pid, parent, resident_bytes)| {
            (
                pid,
                Process {
                    parent,
                    resident_bytes,
                },
            )
        })
        .collect();
        assert_eq!(tree_rss(10, &processes).unwrap(), (1000, 4));
        assert_eq!(tree_rss(999, &processes).unwrap(), (0, 0));
        let cycles = [
            (
                1,
                Process {
                    parent: 2,
                    resident_bytes: 1,
                },
            ),
            (
                2,
                Process {
                    parent: 1,
                    resident_bytes: 2,
                },
            ),
        ]
        .into();
        assert_eq!(tree_rss(1, &cycles).unwrap(), (3, 2));
    }

    #[test]
    fn kernel_stat_with_parentheses_in_comm_preserves_parent_and_rss() {
        let fields = ["S", "42"]
            .into_iter()
            .chain(["0"; 19])
            .chain(["7"])
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            parse_stat(&format!("123 (name with ) parentheses) {fields}"), 4096).unwrap(),
            Process {
                parent: 42,
                resident_bytes: 7 * 4096
            }
        );
        assert!(matches!(
            parse_stat("123 (name) S bad", 4096),
            Err(SamplingError::InvalidData(_))
        ));
    }

    #[test]
    fn darwin_ps_preserves_parents_and_kib_units_and_rejects_bad_rows() {
        let processes = parse_ps(" 10 1 4\n 20 10 8\n 21 20 0\n").unwrap();
        assert_eq!(tree_rss(10, &processes).unwrap(), (12 * 1024, 3));
        for malformed in [
            "10 1",
            "10 1 -1",
            "10 1 18446744073709551615",
            "10 1 1 extra",
            "10 1 1\n10 1 2",
        ] {
            assert!(matches!(
                parse_ps(malformed),
                Err(SamplingError::InvalidData(_))
            ));
        }
    }

    #[test]
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn native_snapshot_includes_this_process_once() {
        let processes = snapshot(64 * 1024).unwrap();
        let own = processes
            .get(&std::process::id())
            .expect("native sampler observes test process");
        assert!(own.parent > 0);
        assert!(own.resident_bytes > 0);
        assert!(tree_rss(std::process::id(), &processes).unwrap().0 >= own.resident_bytes);
    }
}
