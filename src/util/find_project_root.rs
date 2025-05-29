use git2::Repository;
use std::path::PathBuf;

use super::print_status::print_warning;

pub fn find_project_root() -> Option<PathBuf> {
    // Root indicators to fall back on
    let root_indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];

    // Try to find Git repository root
    let project_root = Repository::discover(".")
        .ok()
        .and_then(|repo| repo.workdir().map(|p| p.to_path_buf()))
        .or_else(|| find_root_by_indicators(&root_indicators));

    return project_root;
}

/// Fallback method to find root by walking up directories looking for indicators.
fn find_root_by_indicators(indicators: &[&str]) -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        for indicator in indicators {
            if current_dir.join(indicator).exists() {
                return Some(current_dir.clone());
            }
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            break;
        }
    }

    print_warning(
        "Couldn't find project root, falling back to current directory. Please use the --sops-file flag",
    );
    Some(std::env::current_dir().ok()?)
}
