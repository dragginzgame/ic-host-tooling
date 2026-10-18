//! Strict file-dependency reads; callers select records and output policy.

use crate::artifacts::cache_fs::canonicalize_allow_missing;
use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
};

/// Parse Cargo/rustc Make depfiles without interpreting unsupported Make syntax.
///
/// Supports escaped space/tab/backslash/hash/colon, `$$`, line continuations,
/// multiple rules and empty phony rules. Full-line comments are ignored; their
/// environment facts must be represented in caller identity. Relative paths
/// remain relative until a record read binds the caller's explicit working directory.
///
/// # Errors
/// Returns `InvalidData` for malformed or unsupported syntax, or an empty input set.
pub fn parse_cargo_depfile(text: &str) -> io::Result<Vec<PathBuf>> {
    let mut paths = BTreeSet::new();
    let mut has_target = false;
    let mut token = String::new();
    let mut separated = false;
    let mut chars = text.chars();
    while let Some(character) = chars.next() {
        match character {
            '\\' => match chars.next() {
                Some('\n') => flush_token(&mut token, separated, &mut has_target, &mut paths),
                Some('\r') if chars.next() == Some('\n') => {
                    flush_token(&mut token, separated, &mut has_target, &mut paths);
                }
                Some(escaped @ (' ' | '\t' | '\\' | '#' | ':')) => token.push(escaped),
                _ => return Err(invalid("unsupported depfile escape")),
            },
            '$' => {
                if chars.next() != Some('$') {
                    return Err(invalid("unsupported depfile expansion"));
                }
                token.push('$');
            }
            ':' => {
                if separated {
                    return Err(invalid("unescaped colon in depfile dependency"));
                }
                flush_token(&mut token, false, &mut has_target, &mut paths);
                if !has_target {
                    return Err(invalid("depfile target is empty"));
                }
                separated = true;
            }
            '#' if !separated && !has_target && token.is_empty() => {
                for comment in chars.by_ref() {
                    if comment == '\n' {
                        break;
                    }
                }
            }
            '\n' => {
                finish_rule(&mut token, &mut has_target, &mut separated, &mut paths)?;
            }
            ' ' | '\t' | '\r' => {
                flush_token(&mut token, separated, &mut has_target, &mut paths);
            }
            '#' | ';' | '|' | '*' | '?' | '[' | ']' | '=' | '%' => {
                return Err(invalid("unsupported Make depfile syntax"));
            }
            control if control.is_control() => return Err(invalid("control byte in depfile")),
            ordinary => token.push(ordinary),
        }
    }
    finish_rule(&mut token, &mut has_target, &mut separated, &mut paths)?;
    if paths.is_empty() {
        return Err(invalid("depfile has no source dependencies"));
    }
    Ok(paths.into_iter().collect())
}

/// Read source inputs from one explicitly selected Cargo/rustc depfile.
///
/// The caller owns target selection, so unrelated records in a shared output
/// directory are not inspected. Uses bounded, regular-file/no-follow
/// reads, strict parsing, alias resolution and generated-root exclusions.
/// Missing records return `NotFound`; the caller decides
/// whether absence is an admissible cold-build inventory.
///
/// # Errors
/// Returns non-absolute selections, record overflow, unsupported syntax or I/O.
pub fn read_cargo_depfile_inputs(
    current_dir: &Path,
    record: &Path,
    excluded_roots: &[PathBuf],
    maximum_record_bytes: usize,
) -> io::Result<Vec<PathBuf>> {
    if !current_dir.is_absolute()
        || !record.is_absolute()
        || excluded_roots.iter().any(|path| !path.is_absolute())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "depfile selections must be absolute",
        ));
    }
    let excluded = excluded_roots
        .iter()
        .map(|path| canonicalize_allow_missing(path))
        .collect::<io::Result<Vec<_>>>()?;
    let mut inputs = BTreeSet::new();
    read_record_inputs(
        current_dir,
        record,
        &excluded,
        maximum_record_bytes,
        &mut inputs,
    )?;
    Ok(inputs.into_iter().collect())
}

fn read_record_inputs(
    current_dir: &Path,
    record: &Path,
    excluded: &[PathBuf],
    maximum_record_bytes: usize,
    inputs: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    let bytes = ic_host_fs::read::read_file_no_follow(record, maximum_record_bytes)
        .map_err(io::Error::from)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    for path in parse_cargo_depfile(text)? {
        let path = if path.is_absolute() {
            path
        } else {
            current_dir.join(path)
        };
        let path = match path.canonicalize() {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => path,
            Err(error) => return Err(error),
        };
        if !path
            .components()
            .any(|component| component == std::path::Component::ParentDir)
            && excluded.iter().any(|root| path.starts_with(root))
        {
            continue;
        }
        inputs.insert(path);
    }
    Ok(())
}

fn flush_token(
    token: &mut String,
    separated: bool,
    has_target: &mut bool,
    paths: &mut BTreeSet<PathBuf>,
) {
    if !token.is_empty() {
        if separated {
            paths.insert(PathBuf::from(std::mem::take(token)));
        } else {
            *has_target = true;
            token.clear();
        }
    }
}

fn finish_rule(
    token: &mut String,
    has_target: &mut bool,
    separated: &mut bool,
    paths: &mut BTreeSet<PathBuf>,
) -> io::Result<()> {
    flush_token(token, *separated, has_target, paths);
    if *has_target && !*separated {
        return Err(invalid("depfile rule has no target separator"));
    }
    *has_target = false;
    *separated = false;
    Ok(())
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
