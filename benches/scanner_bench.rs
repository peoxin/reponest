use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use reponest::config::AppConfig;
use std::fs;
use std::hint::black_box;
use std::path::Path;
use tempfile::TempDir;

// Helper to create a test directory structure with repositories
fn create_test_structure(base: &Path, num_repos: usize, depth: usize) {
    for i in 0..num_repos {
        let repo_path = if depth > 0 {
            // Create nested structure
            let mut path = base.to_path_buf();
            for d in 0..depth {
                path.push(format!("level{}", d));
            }
            path.push(format!("repo{}", i));
            path
        } else {
            base.join(format!("repo{}", i))
        };

        fs::create_dir_all(&repo_path).unwrap();
        fs::create_dir_all(repo_path.join(".git")).unwrap();

        // Add some dummy files
        fs::write(repo_path.join("README.md"), "# Test repo").unwrap();
        fs::write(repo_path.join(".git/config"), "[core]").unwrap();
    }
}

// Helper to create noise directories (non-git directories)
fn create_noise_directories(base: &Path, num_dirs: usize) {
    for i in 0..num_dirs {
        let dir_path = base.join(format!("noise{}", i));
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(dir_path.join("file.txt"), "noise").unwrap();
    }
}

// Helper to create nested noise directories (simulating node_modules, build outputs, etc.)
fn create_nested_noise(base: &Path, num_trees: usize, depth: usize, width: usize) {
    for i in 0..num_trees {
        let tree_root = base.join(format!("noise_tree{}", i));
        create_nested_noise_recursive(&tree_root, depth, width, 0);
    }
}

fn create_nested_noise_recursive(
    path: &Path,
    max_depth: usize,
    width: usize,
    current_depth: usize,
) {
    if current_depth >= max_depth {
        return;
    }

    fs::create_dir_all(path).unwrap();
    fs::write(path.join("file.txt"), "noise").unwrap();

    for i in 0..width {
        let child = path.join(format!("subdir{}", i));
        create_nested_noise_recursive(&child, max_depth, width, current_depth + 1);
    }
}

// Helper to create realistic project structure with repos and noise
fn create_realistic_structure(base: &Path, num_projects: usize) {
    for i in 0..num_projects {
        let project_dir = base.join(format!("project{}", i));
        fs::create_dir_all(&project_dir).unwrap();

        // Create a git repo in the project
        let git_dir = project_dir.join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        fs::write(git_dir.join("config"), "[core]").unwrap();
        fs::write(project_dir.join("README.md"), "# Project").unwrap();

        // Create node_modules with nested structure
        let node_modules = project_dir.join("node_modules");
        for j in 0..3 {
            let pkg = node_modules.join(format!("package{}", j));
            fs::create_dir_all(&pkg).unwrap();
            fs::write(pkg.join("index.js"), "module.exports = {}").unwrap();

            // Nested dependencies
            let nested_nm = pkg.join("node_modules");
            for k in 0..2 {
                let nested_pkg = nested_nm.join(format!("dep{}", k));
                fs::create_dir_all(&nested_pkg).unwrap();
                fs::write(nested_pkg.join("index.js"), "module.exports = {}").unwrap();
            }
        }

        // Create build output directories
        let build_dirs = ["target", "dist", "build", ".next"];
        for build_dir in &build_dirs {
            let dir = project_dir.join(build_dir);
            fs::create_dir_all(&dir).unwrap();
            for j in 0..5 {
                let subdir = dir.join(format!("subdir{}", j));
                fs::create_dir_all(&subdir).unwrap();
                fs::write(subdir.join("output.bin"), &[0u8; 100]).unwrap();
            }
        }

        // Create source directories with some structure
        let src = project_dir.join("src");
        fs::create_dir_all(&src).unwrap();
        for j in 0..3 {
            let module = src.join(format!("module{}", j));
            fs::create_dir_all(&module).unwrap();
            fs::write(module.join("code.rs"), "fn main() {}").unwrap();
        }
    }
}

// Benchmark scanning with different numbers of repositories
fn bench_scan_small(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scan_small_5_repos", |b| {
        let temp_dir = TempDir::new().unwrap();
        create_test_structure(temp_dir.path(), 5, 0);
        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });
}

fn bench_scan_medium(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scan_medium_20_repos", |b| {
        let temp_dir = TempDir::new().unwrap();
        create_test_structure(temp_dir.path(), 20, 0);
        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });
}

fn bench_scan_large(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scan_large_50_repos", |b| {
        let temp_dir = TempDir::new().unwrap();
        create_test_structure(temp_dir.path(), 50, 0);
        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });
}

// Benchmark scanning with different depths
fn bench_scan_depths(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("scan_by_depth");

    for depth in [1, 2, 3, 4].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &depth| {
            let temp_dir = TempDir::new().unwrap();
            create_test_structure(temp_dir.path(), 10, depth);
            let mut config = AppConfig::default();
            config.main.max_depth = depth + 2;
            let path = temp_dir.path().to_str().unwrap().to_string();

            b.to_async(&rt).iter(|| async {
                reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                    .await
                    .unwrap()
            });
        });
    }
    group.finish();
}

// Benchmark scanning with noise (non-git directories)
fn bench_scan_with_noise(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("scan_with_noise");

    for noise_ratio in [0, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}%_noise", noise_ratio)),
            noise_ratio,
            |b, &noise_ratio| {
                let temp_dir = TempDir::new().unwrap();
                let num_repos = 10;
                let num_noise = (num_repos * noise_ratio) / 100;

                create_test_structure(temp_dir.path(), num_repos, 0);
                create_noise_directories(temp_dir.path(), num_noise);

                let config = AppConfig::default();
                let path = temp_dir.path().to_str().unwrap().to_string();

                b.to_async(&rt).iter(|| async {
                    reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                        .await
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

// Benchmark max_depth limits
fn bench_max_depth_limits(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("max_depth_limits");

    for max_depth in [0, 1, 2, 3, 5].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{}", max_depth)),
            max_depth,
            |b, &max_depth| {
                let temp_dir = TempDir::new().unwrap();
                create_test_structure(temp_dir.path(), 10, 4);

                let mut config = AppConfig::default();
                config.main.max_depth = max_depth;
                let path = temp_dir.path().to_str().unwrap().to_string();

                b.to_async(&rt).iter(|| async {
                    reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                        .await
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

// Benchmark multiple directories scanning
fn bench_scan_multiple_dirs(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("scan_multiple_3_dirs_10_repos_each", |b| {
        let temp_dirs: Vec<TempDir> = (0..3).map(|_| TempDir::new().unwrap()).collect();

        for dir in &temp_dirs {
            create_test_structure(dir.path(), 10, 0);
        }

        let paths: Vec<String> = temp_dirs
            .iter()
            .map(|d| d.path().to_str().unwrap().to_string())
            .collect();

        let config = AppConfig::default();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directories(black_box(&paths), black_box(&config))
                .await
                .unwrap()
        });
    });
}

// Benchmark exclude patterns
fn bench_exclude_patterns(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("exclude_patterns");

    for num_patterns in [0, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_patterns", num_patterns)),
            num_patterns,
            |b, &num_patterns| {
                let temp_dir = TempDir::new().unwrap();
                create_test_structure(temp_dir.path(), 10, 1);

                let mut config = AppConfig::default();
                config.internal.exclude_dirs = (0..num_patterns)
                    .map(|i| format!("excluded{}", i))
                    .collect();

                let path = temp_dir.path().to_str().unwrap().to_string();

                b.to_async(&rt).iter(|| async {
                    reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                        .await
                        .unwrap()
                });
            },
        );
    }
    group.finish();
}

// Benchmark realistic large workspace scenario
fn bench_realistic_large_workspace(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("realistic_large_workspace");

    // Configure for longer running tests
    group.sample_size(10);

    group.bench_function("workspace_10_projects", |b| {
        let temp_dir = TempDir::new().unwrap();
        create_realistic_structure(temp_dir.path(), 10);

        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });

    group.bench_function("workspace_25_projects", |b| {
        let temp_dir = TempDir::new().unwrap();
        create_realistic_structure(temp_dir.path(), 25);

        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });

    group.finish();
}

// Benchmark with deeply nested noise directories
fn bench_nested_noise(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("nested_noise");

    for depth in [2, 3, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("depth_{}_width_3", depth)),
            depth,
            |b, &depth| {
                let temp_dir = TempDir::new().unwrap();

                // Create 10 git repos
                create_test_structure(temp_dir.path(), 10, 0);

                // Create 5 deeply nested noise trees
                create_nested_noise(temp_dir.path(), 5, depth, 3);

                let config = AppConfig::default();
                let path = temp_dir.path().to_str().unwrap().to_string();

                b.to_async(&rt).iter(|| async {
                    reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                        .await
                        .unwrap()
                });
            },
        );
    }

    group.finish();
}

// Benchmark extreme case: very large directory with mixed structure
fn bench_extreme_case(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("extreme_case");

    // Configure for very long running test
    group.sample_size(10);

    group.bench_function("500_repos_2000_noise_dirs", |b| {
        let temp_dir = TempDir::new().unwrap();

        // Create 500 repos at various depths (simulating large monorepo or workspace)
        for i in 0..500 {
            let depth = i % 5; // Vary depth from 0 to 4
            create_test_structure(temp_dir.path(), 1, depth);
        }

        // Create 2000 flat noise directories (simulating logs, temp files, etc.)
        create_noise_directories(temp_dir.path(), 2000);

        // Create 50 deep noise trees (simulating node_modules, target, etc.)
        create_nested_noise(temp_dir.path(), 50, 5, 4);

        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });

    group.bench_function("1000_repos_deep_nesting", |b| {
        let temp_dir = TempDir::new().unwrap();

        // Create 1000 repos with extreme depth variation
        for i in 0..1000 {
            let depth = (i % 7) + 1; // Depths from 1 to 7
            create_test_structure(temp_dir.path(), 1, depth);
        }

        // Create massive nested noise
        create_nested_noise(temp_dir.path(), 100, 6, 5);

        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });

    group.bench_function("mega_workspace_5000_total_dirs", |b| {
        let temp_dir = TempDir::new().unwrap();

        // Create 200 actual git repos
        for i in 0..200 {
            let depth = i % 6;
            create_test_structure(temp_dir.path(), 1, depth);
        }

        // Create 3000 flat noise directories
        create_noise_directories(temp_dir.path(), 3000);

        // Create 150 nested noise trees with extreme depth
        create_nested_noise(temp_dir.path(), 150, 6, 6);

        // Add some realistic projects mixed in
        create_realistic_structure(temp_dir.path(), 50);

        let config = AppConfig::default();
        let path = temp_dir.path().to_str().unwrap().to_string();

        b.to_async(&rt).iter(|| async {
            reponest::core::scanner::scan_directory(black_box(&path), black_box(&config))
                .await
                .unwrap()
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scan_small,
    bench_scan_medium,
    bench_scan_large,
    bench_scan_depths,
    bench_scan_with_noise,
    bench_max_depth_limits,
    bench_scan_multiple_dirs,
    bench_exclude_patterns,
    bench_realistic_large_workspace,
    bench_nested_noise,
    bench_extreme_case
);

criterion_main!(benches);
