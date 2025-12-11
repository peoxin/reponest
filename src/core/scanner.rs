//! This module provides asynchronous directory traversal to discover Git repositories.

use anyhow::Result;
use std::path::PathBuf;

use crate::config::AppConfig;

/// Scan a single directory for Git repositories
pub async fn scan_directory(base_path: &str, cfg: &AppConfig) -> Result<Vec<PathBuf>> {
    let base = PathBuf::from(base_path);
    let mut paths = Vec::new();
    scan_recursive(base, cfg, 0, &mut paths).await?;
    Ok(paths)
}

/// Scan multiple directories for Git repositories
pub async fn scan_directories(base_paths: &[String], cfg: &AppConfig) -> Result<Vec<PathBuf>> {
    let mut all_paths = Vec::new();
    for base in base_paths {
        if let Ok(mut paths) = scan_directory(base, cfg).await {
            all_paths.append(&mut paths);
        }
    }
    Ok(all_paths)
}

/// Recursively traverse directory tree to find Git repositories
fn scan_recursive<'a>(
    path: PathBuf,
    cfg: &'a AppConfig,
    depth: usize,
    paths: &'a mut Vec<PathBuf>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        if cfg.main.max_depth > 0 && depth >= cfg.main.max_depth {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }

            let file_name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // If we find a .git directory, record the parent as a Git repository.
            // After that, we will continue scanning other directories, thus finding nested repos.
            if file_name == ".git" {
                if let Some(repo_path) = entry_path.parent() {
                    paths.push(repo_path.to_path_buf());
                }
                continue;
            }

            if is_excluded(file_name, &cfg.internal.exclude_dirs) {
                continue;
            }
            let _ = scan_recursive(entry_path, cfg, depth + 1, paths).await;
        }

        Ok(())
    })
}

/// Check if a directory should be excluded from scanning
#[inline]
fn is_excluded(dir_name: &str, exclude_patterns: &[String]) -> bool {
    // Skip all hidden directories
    if dir_name.starts_with('.') {
        return true;
    }

    exclude_patterns
        .iter()
        .any(|pattern| matches_wildcard(dir_name, pattern))
}

/// Match a name against a pattern with wildcard support
#[inline]
fn matches_wildcard(name: &str, pattern: &str) -> bool {
    if !pattern.contains('*') {
        return name == pattern;
    }

    let parts: Vec<&str> = pattern.split('*').collect();

    match parts.len() {
        1 => true, // pattern is just "*"
        2 => {
            let (prefix, suffix) = (parts[0], parts[1]);
            match (prefix.is_empty(), suffix.is_empty()) {
                (true, false) => name.ends_with(suffix),   // "*suffix"
                (false, true) => name.starts_with(prefix), // "prefix*"
                (false, false) => {
                    // "prefix*suffix"
                    name.starts_with(prefix)
                        && name.ends_with(suffix)
                        && name.len() >= prefix.len() + suffix.len()
                }
                (true, true) => true, // "*"
            }
        }
        _ => name == pattern, // complex patterns fallback to exact match
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create a test git repository
    fn create_git_repo(path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
        fs::create_dir_all(path.join(".git")).unwrap();
        fs::write(path.join(".git/config"), "[core]").unwrap();
    }

    /// Create a regular directory (not a git repo)
    fn create_dir(path: &std::path::Path) {
        fs::create_dir_all(path).unwrap();
    }

    #[tokio::test]
    async fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = AppConfig::default();

        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 0);
    }

    #[tokio::test]
    async fn test_scan_single_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("repo1");
        create_git_repo(&repo_path);

        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], repo_path);
    }

    #[tokio::test]
    async fn test_scan_multiple_repos() {
        let temp_dir = TempDir::new().unwrap();

        // Create 3 repos
        for i in 1..=3 {
            let repo_path = temp_dir.path().join(format!("repo{}", i));
            create_git_repo(&repo_path);
        }

        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_scan_nested_repos() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure: parent/child1 and parent/child2
        let parent = temp_dir.path().join("parent");
        create_git_repo(&parent);

        let child1 = parent.join("child1");
        create_git_repo(&child1);

        let child2 = parent.join("child2");
        create_git_repo(&child2);

        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        // Should find all 3 repos
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_scan_with_max_depth() {
        let temp_dir = TempDir::new().unwrap();

        // Create structure with depth: temp_dir/level1/level2/level3
        let level1 = temp_dir.path().join("level1");
        create_git_repo(&level1);

        let level2 = level1.join("level2");
        create_git_repo(&level2);

        let level3 = level2.join("level3");
        create_git_repo(&level3);

        // Test with max_depth = 2 (should find only level1, depth 1 from base)
        let mut config = AppConfig::default();
        config.main.max_depth = 2;

        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("level1"));

        // Test with max_depth = 3 (should find level1 and level2)
        config.main.max_depth = 3;
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);

        // Test with max_depth = 0 (unlimited)
        config.main.max_depth = 0;
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_scan_excludes_hidden_dirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create regular repo
        let normal_repo = temp_dir.path().join("normal");
        create_git_repo(&normal_repo);

        // Create repo in hidden directory (should be excluded)
        let hidden_dir = temp_dir.path().join(".hidden");
        create_git_repo(&hidden_dir);

        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        // Should only find the normal repo, not the hidden one
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], normal_repo);
    }

    #[tokio::test]
    async fn test_scan_with_exclude_patterns() {
        let temp_dir = TempDir::new().unwrap();

        // Create various repos
        create_git_repo(&temp_dir.path().join("repo1"));
        create_git_repo(&temp_dir.path().join("node_modules"));
        create_git_repo(&temp_dir.path().join("target"));
        create_git_repo(&temp_dir.path().join("build"));

        // Configure exclusions
        let mut config = AppConfig::default();
        config.internal.exclude_dirs = vec![
            "node_modules".to_string(),
            "target".to_string(),
            "build".to_string(),
        ];

        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        // Should only find repo1
        assert_eq!(result.len(), 1);
        assert!(result[0].ends_with("repo1"));
    }

    #[tokio::test]
    async fn test_scan_ignores_non_git_dirs() {
        let temp_dir = TempDir::new().unwrap();

        // Create git repo
        let repo_path = temp_dir.path().join("repo");
        create_git_repo(&repo_path);

        // Create non-git directories
        create_dir(&temp_dir.path().join("not_a_repo"));
        create_dir(&temp_dir.path().join("another_dir"));

        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        // Should only find the actual git repo
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], repo_path);
    }

    #[tokio::test]
    async fn test_scan_multiple_directories() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Create repos in both directories
        create_git_repo(&temp_dir1.path().join("repo1"));
        create_git_repo(&temp_dir1.path().join("repo2"));
        create_git_repo(&temp_dir2.path().join("repo3"));

        let paths = vec![
            temp_dir1.path().to_str().unwrap().to_string(),
            temp_dir2.path().to_str().unwrap().to_string(),
        ];

        let config = AppConfig::default();
        let result = scan_directories(&paths, &config).await.unwrap();

        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_matches_wildcard_exact() {
        assert!(matches_wildcard("node_modules", "node_modules"));
        assert!(!matches_wildcard("node_modules", "target"));
    }

    #[test]
    fn test_matches_wildcard_prefix() {
        assert!(matches_wildcard("test_file", "test*"));
        assert!(matches_wildcard("test", "test*"));
        assert!(!matches_wildcard("other", "test*"));
    }

    #[test]
    fn test_matches_wildcard_suffix() {
        assert!(matches_wildcard("file.txt", "*.txt"));
        assert!(matches_wildcard(".txt", "*.txt"));
        assert!(!matches_wildcard("file.rs", "*.txt"));
    }

    #[test]
    fn test_matches_wildcard_prefix_suffix() {
        assert!(matches_wildcard("test_file.txt", "test*.txt"));
        assert!(matches_wildcard("test.txt", "test*.txt"));
        assert!(!matches_wildcard("other_file.txt", "test*.txt"));
        assert!(!matches_wildcard("test", "test*.txt"));
    }

    #[test]
    fn test_matches_wildcard_star_only() {
        assert!(matches_wildcard("anything", "*"));
        assert!(matches_wildcard("", "*"));
    }

    #[test]
    fn test_is_excluded_hidden_dirs() {
        let patterns = vec![];
        assert!(is_excluded(".hidden", &patterns));
        assert!(is_excluded(".git", &patterns));
        assert!(!is_excluded("normal", &patterns));
    }

    #[test]
    fn test_is_excluded_with_patterns() {
        let patterns = vec![
            "node_modules".to_string(),
            "target".to_string(),
            "*.tmp".to_string(),
        ];

        assert!(is_excluded("node_modules", &patterns));
        assert!(is_excluded("target", &patterns));
        assert!(is_excluded("file.tmp", &patterns));
        assert!(!is_excluded("src", &patterns));
    }

    #[tokio::test]
    async fn test_scan_complex_structure() {
        let temp_dir = TempDir::new().unwrap();

        // Create a complex structure mimicking real projects
        // project1/
        //   .git/
        //   src/
        //   subproject/
        //     .git/
        // project2/
        //   .git/

        let project1 = temp_dir.path().join("project1");
        create_git_repo(&project1);
        create_dir(&project1.join("src"));

        let subproject = project1.join("subproject");
        create_git_repo(&subproject);

        let project2 = temp_dir.path().join("project2");
        create_git_repo(&project2);
        create_dir(&project2.join("build"));

        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        // Should find project1, subproject, and project2
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_scan_deep_nesting() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure at moderate depth
        let level1 = temp_dir.path().join("a");
        let level2 = level1.join("b");
        let level3 = level2.join("c");

        fs::create_dir_all(&level3).unwrap();
        let repo_path = level3.join("deep_repo");
        create_git_repo(&repo_path);

        // Test with no max_depth (should find it)
        let config = AppConfig::default();
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], repo_path);

        // Test with max_depth = 2 (should not find it at depth 4)
        let mut config = AppConfig::default();
        config.main.max_depth = 2;
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 0);

        // Test with max_depth = 5 (should find it)
        let mut config = AppConfig::default();
        config.main.max_depth = 5;
        let result = scan_directory(temp_dir.path().to_str().unwrap(), &config)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
    }
}
