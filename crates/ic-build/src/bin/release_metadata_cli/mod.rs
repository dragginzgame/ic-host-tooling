use ic_build::cargo::{
    lock_projection::WorkspaceVersionProjection,
    release_metadata::{
        DependencyVersionProjection, render_workspace_lock, render_workspace_manifest,
    },
};
use serde_json::Value;
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    io::Write,
    path::{Component, Path, PathBuf},
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() < 6 || arguments[0] != "--max-input-bytes" {
        return Err("usage: ic-build-release-metadata --max-input-bytes N manifest SOURCE PREVIOUS NEXT [WORKSPACE_INPUT_JSON] | lock SOURCE PREVIOUS NEXT PACKAGE...".into());
    }
    let limit: usize = text(&arguments[1])?.parse()?;
    if limit == 0 {
        return Err("input byte allowance must be positive".into());
    }
    let source = Path::new(&arguments[3]);
    check_path(source)?;
    let input = read(source, limit)?;
    let previous = text(&arguments[4])?;
    let next = text(&arguments[5])?;
    let output = match text(&arguments[2])? {
        "manifest" if matches!(arguments.len(), 6 | 7) => {
            let dependencies = if arguments.len() == 7 {
                let context: Value = serde_json::from_str(&read(Path::new(&arguments[6]), limit)?)?;
                dependency_versions(source, &input, next, &context)?
            } else {
                Vec::new()
            };
            let projections: Vec<_> = dependencies
                .iter()
                .map(|(alias, previous, next)| DependencyVersionProjection {
                    alias,
                    previous,
                    next,
                })
                .collect();
            render_workspace_manifest(&input, previous, next, &projections)?
        }
        "lock" if arguments.len() > 6 => {
            let projections = arguments[6..]
                .iter()
                .map(|name| {
                    Ok(WorkspaceVersionProjection {
                        package: text(name)?,
                        previous,
                        next,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            render_workspace_lock(&input, &projections)?
        }
        _ => return Err("invalid rendering operation or arguments".into()),
    };
    std::io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}
fn text(value: &OsString) -> Result<&str> {
    value
        .to_str()
        .ok_or_else(|| "expected UTF-8 argument".into())
}
fn check_path(path: &Path) -> Result<()> {
    if path
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .any(|byte| matches!(byte, 0 | b'\r' | b'\n'))
    {
        return Err("policy path contains NUL/CR/LF".into());
    }
    Ok(())
}
fn read(path: &Path, limit: usize) -> Result<String> {
    check_path(path)?;
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    check_path(&absolute)?;
    check_path(
        &absolute
            .parent()
            .ok_or("input has no parent")?
            .canonicalize()?,
    )?;
    // Parent namespace custody remains caller-owned. Host admits the final opened
    // descriptor, refuses final symlinks/special files and enforces the byte limit.
    Ok(String::from_utf8(ic_host_fs::read::read_file_no_follow(
        path, limit,
    )?)?)
}
fn identity(path: &Path) -> Result<PathBuf> {
    check_path(path)?;
    if !path.is_absolute() {
        return Err("saved identity must be absolute".into());
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Normal(_) => result.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return Err("identity escapes filesystem root".into());
                }
            }
            Component::Prefix(_) => return Err("unsupported path prefix".into()),
        }
    }
    Ok(result)
}
fn required<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string {key}").into())
}
fn requirement(previous: &str, next: &str) -> Option<String> {
    let offset = usize::from(previous.starts_with(['~', '^', '=']));
    let version = previous[offset..].trim_start_matches([' ', '\t']);
    let prefix = &previous[..previous.len() - version.len()];
    let parts: Vec<_> = version.split('.').collect();
    if !(1..=3).contains(&parts.len())
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    Some(format!(
        "{prefix}{}",
        next.split('.')
            .take(parts.len())
            .collect::<Vec<_>>()
            .join(".")
    ))
}
fn dependency_versions(
    source: &Path,
    input: &str,
    next: &str,
    context: &Value,
) -> Result<Vec<(String, String, String)>> {
    let saved = context.get("source_root").is_some();
    let root = if saved {
        identity(Path::new(required(context, "source_root")?))?
    } else {
        source
            .canonicalize()?
            .parent()
            .ok_or("manifest has no parent")?
            .to_owned()
    };
    let mut members = HashMap::new();
    for member in context
        .get("members")
        .and_then(Value::as_array)
        .ok_or("missing members array")?
    {
        let name = required(member, "name")?;
        let path = identity(Path::new(required(member, "manifest_path")?))?;
        if members.insert(name, path).is_some() {
            return Err("duplicate member identity".into());
        }
    }
    let manifest: toml::Value = toml::from_str(input)?;
    let actual = manifest
        .get("workspace")
        .and_then(|value| value.get("dependencies"));
    let expected = context
        .get("manifest")
        .and_then(|value| value.get("workspace"))
        .and_then(|value| value.get("dependencies"));
    if serde_json::to_value(actual)? != expected.cloned().unwrap_or(Value::Null) {
        return Err("saved dependency declarations differ from source".into());
    }
    let mut selected = Vec::new();
    if let Some(dependencies) = expected.and_then(Value::as_object) {
        for (alias, dependency) in dependencies {
            let (Some(path), Some(previous)) = (
                dependency.get("path").and_then(Value::as_str),
                dependency.get("version").and_then(Value::as_str),
            ) else {
                continue;
            };
            let package = dependency
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or(alias);
            let Some(member) = members.get(package) else {
                continue;
            };
            let selected_path = root.join(path).join("Cargo.toml");
            let owned = if saved {
                if identity(&selected_path)? != *member {
                    return Err("cannot establish saved dependency ownership".into());
                }
                true
            } else {
                selected_path.canonicalize()? == *member
            };
            if owned && let Some(candidate) = requirement(previous, next) {
                selected.push((alias.clone(), previous.to_owned(), candidate));
            }
        }
    }
    Ok(selected)
}
