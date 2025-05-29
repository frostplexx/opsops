use git2::Repository;
use std::path::PathBuf;

use super::print_status::print_warning;

pub fn find_project_root() -> Option<PathBuf> {
    // Root indicators to fall back on
    let root_indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];

    // Try to find Git repository root
    Repository::discover(".")
        .ok()
        .and_then(|repo| repo.workdir().map(|p| p.to_path_buf()))
        .or_else(|| find_root_by_indicators(&root_indicators))
}

/// Fallback method to find root by walking up directories looking for indicators.
fn find_root_by_indicators(indicators: &[&str]) -> Option<PathBuf> {
    find_root_by_indicators_from_dir(indicators, &std::env::current_dir().ok()?)
}

/// Internal function that takes a starting directory - useful for testing
fn find_root_by_indicators_from_dir(
    indicators: &[&str],
    start_dir: &std::path::Path,
) -> Option<PathBuf> {
    let mut current_dir = start_dir.to_path_buf();

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
    Some(start_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_find_root_by_indicators_with_git() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];
        let result = find_root_by_indicators_from_dir(&indicators, temp_dir.path());

        assert!(result.is_some());
        let expected = temp_dir.path().canonicalize().unwrap();
        let actual = result.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_root_by_indicators_with_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_file = temp_dir.path().join("Cargo.toml");
        File::create(&cargo_file).unwrap();

        let indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];
        let result = find_root_by_indicators_from_dir(&indicators, temp_dir.path());

        assert!(result.is_some());
        let expected = temp_dir.path().canonicalize().unwrap();
        let actual = result.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_root_by_indicators_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        // Create indicator in root
        let cargo_file = temp_dir.path().join("Cargo.toml");
        File::create(&cargo_file).unwrap();

        let indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];
        let result = find_root_by_indicators_from_dir(&indicators, &nested_dir);

        assert!(result.is_some());
        let expected = temp_dir.path().canonicalize().unwrap();
        let actual = result.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_root_by_indicators_no_indicators() {
        let temp_dir = TempDir::new().unwrap();

        let indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];
        let result = find_root_by_indicators_from_dir(&indicators, temp_dir.path());

        // The function falls back to the starting directory when no indicators are found
        assert!(result.is_some());
        let expected = temp_dir.path().canonicalize().unwrap();
        let actual = result.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_find_root_by_indicators_no_indicators_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested_dir).unwrap();

        let indicators = vec![".git", "src", "flake.nix", "package.json", "Cargo.toml"];
        let result = find_root_by_indicators_from_dir(&indicators, &nested_dir);

        // Should fall back to the nested directory (where we started from)
        assert!(result.is_some());
        let expected = nested_dir.canonicalize().unwrap();
        let actual = result.unwrap().canonicalize().unwrap();
        assert_eq!(actual, expected);
    }
}
