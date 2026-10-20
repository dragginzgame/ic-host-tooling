//! Explicit Cargo execution, bounded observations and feature-safe package grouping.
//!
//! Responsibility: execute selected recipes, observe complete Cargo evidence and preserve isolated dependency trees.
//! Does not own: recipe selection, source custody, role contracts or runtime lifecycle.
//! Boundary: shared dependency feature unification must not change a role's isolated build.

#[cfg(test)]
mod tests;

pub mod diagnostics;
pub mod discovery;
pub mod environment;
pub mod execution;
pub mod flags;
pub mod lock_projection;
pub(crate) mod lockfile;
pub(crate) mod messages;
#[cfg(unix)]
pub mod native;
pub mod observation;
pub mod packages;
pub mod release_metadata;
pub mod selection;

use ic_host_artifacts::batch::validate_labels;
use std::{collections::BTreeMap, path::Path};

/// Partition packages without changing their resolved normal/build dependency trees.
///
/// The caller supplies the recipe's Cargo selection, target and tree observer.
/// The observer executes each specification with the selected command, complete
/// environment, source custody and output/deadline policy. It must admit success
/// and completeness before returning UTF-8 text. Every isolated
/// and combined observation uses that selection's features, lock and network
/// choices. Profiles do not affect Cargo tree and are not rendered here.
/// This does not publish artifacts or
/// establish a consumer's release validity.
/// Package names must be nonempty and unique; invalid inputs are rejected before
/// invoking the observer, with zero-based indices in the error. An empty
/// package list returns an empty plan without invoking the observer.
///
/// # Errors
/// Returns invalid package input, the observer's original boxed error without
/// conversion, or invalid dependency-tree evidence. Capture error identities
/// belong to the observer's runner, independently of Build's Host generation.
pub fn compatible_package_groups(
    workspace: &Path,
    packages: &[String],
    target: &str,
    selection: &selection::CargoSelection,
    mut observe_tree: impl FnMut(
        &observation::CargoTreeSpec<'_>,
    ) -> Result<String, Box<dyn std::error::Error>>,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    validate_labels(packages.iter().map(String::as_str))?;
    let mut isolated = BTreeMap::new();
    let mut groups: Vec<Vec<String>> = Vec::new();
    for package in packages {
        let mut tree = resolved_trees(
            workspace,
            &[package.as_str()],
            target,
            selection,
            &mut observe_tree,
        )?;
        if tree.len() != 1 {
            return Err("Cargo returned additional isolated package roots".into());
        }
        let selected = tree
            .remove(package)
            .ok_or("Cargo omitted the selected package tree")?;
        isolated.insert(package.clone(), selected);
        let mut selected_group = None;
        for (index, group) in groups.iter().enumerate() {
            let candidate = group
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(package.as_str()))
                .collect::<Vec<_>>();
            let combined =
                resolved_trees(workspace, &candidate, target, selection, &mut observe_tree)?;
            if preserves_individual_trees(&isolated, &combined, &candidate) {
                selected_group = Some(index);
                break;
            }
        }
        if let Some(index) = selected_group {
            groups[index].push(package.clone());
        } else {
            groups.push(vec![package.clone()]);
        }
    }
    Ok(groups)
}

fn resolved_trees(
    workspace: &Path,
    packages: &[&str],
    target: &str,
    selection: &selection::CargoSelection,
    observe_tree: &mut impl FnMut(
        &observation::CargoTreeSpec<'_>,
    ) -> Result<String, Box<dyn std::error::Error>>,
) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let manifest = workspace.join("Cargo.toml");
    let tree = observe_tree(&observation::CargoTreeSpec {
        manifest_path: &manifest,
        packages,
        target,
        edges: "normal,build",
        format: "{p}\t{f}",
        locked: selection.locked,
        offline: selection.offline,
        features: &selection.features,
    })?;
    parse_trees(&tree)
}

fn parse_trees(text: &str) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let mut trees = BTreeMap::new();
    let mut root = None;
    for line in text.lines().filter(|line| !line.is_empty()) {
        if let Some(package) = line.strip_prefix('0') {
            let name = package
                .split_whitespace()
                .next()
                .ok_or("empty Cargo tree root")?;
            if trees.insert(name.to_string(), String::new()).is_some() {
                return Err("duplicate Cargo tree root".into());
            }
            root = Some(name);
        }
        let tree = root
            .and_then(|name| trees.get_mut(name))
            .ok_or("Cargo dependency has no selected root")?;
        tree.push_str(line);
        tree.push('\n');
    }
    if trees.is_empty() {
        return Err("empty Cargo dependency forest".into());
    }
    Ok(trees)
}

fn preserves_individual_trees(
    isolated: &BTreeMap<String, String>,
    combined: &BTreeMap<String, String>,
    packages: &[impl AsRef<str>],
) -> bool {
    combined.len() == packages.len()
        && packages.iter().all(|name| {
            isolated
                .get(name.as_ref())
                .is_some_and(|expected| combined.get(name.as_ref()) == Some(expected))
        })
}
