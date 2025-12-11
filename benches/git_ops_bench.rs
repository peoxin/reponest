use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use git2::{Repository, Signature};
use reponest::core::git_ops::{RepoInfoWorker, get_repos_info_parallel};
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

/// Create a test repository with initial commit
fn create_test_repo(path: &Path) -> Repository {
    fs::create_dir_all(path).unwrap();
    let repo = Repository::init(path).unwrap();

    // Configure test user
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();

    // Create initial commit
    let sig = Signature::now("Test User", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        index.write_tree().unwrap()
    };
    {
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }

    repo
}

/// Create a repository with some changes
fn create_repo_with_changes(path: &Path, num_files: usize) {
    let repo = create_test_repo(path);

    // Add some untracked files
    for i in 0..num_files {
        fs::write(
            path.join(format!("file{}.txt", i)),
            format!("content {}", i),
        )
        .unwrap();
    }

    // Stage some files
    let mut index = repo.index().unwrap();
    for i in 0..num_files / 2 {
        index
            .add_path(Path::new(&format!("file{}.txt", i)))
            .unwrap();
    }
    index.write().unwrap();
}

/// Create a repository with commits
fn create_repo_with_commits(path: &Path, num_commits: usize) {
    let repo = create_test_repo(path);
    let sig = Signature::now("Test User", "test@example.com").unwrap();

    for i in 1..num_commits {
        // Create a file
        fs::write(
            path.join(format!("commit{}.txt", i)),
            format!("commit {}", i),
        )
        .unwrap();

        // Stage and commit
        let mut index = repo.index().unwrap();
        index
            .add_path(Path::new(&format!("commit{}.txt", i)))
            .unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let parent = repo.head().unwrap().peel_to_commit().unwrap();

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Commit {}", i),
            &tree,
            &[&parent],
        )
        .unwrap();
    }
}

/// Create a repository with stashes
fn create_repo_with_stashes(path: &Path, num_stashes: usize) {
    let mut repo = create_test_repo(path);
    let sig = Signature::now("Test User", "test@example.com").unwrap();

    for i in 0..num_stashes {
        // Create changes
        fs::write(path.join(format!("stash{}.txt", i)), format!("stash {}", i)).unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_path(Path::new(&format!("stash{}.txt", i)))
            .unwrap();
        index.write().unwrap();

        // Create stash
        repo.stash_save(&sig, &format!("Stash {}", i), None)
            .unwrap();
    }
}

/// Create multiple test repositories
fn create_test_repos(base: &Path, count: usize, repo_type: &str) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(count);

    for i in 0..count {
        let repo_path = base.join(format!("repo{}", i));

        match repo_type {
            "simple" => {
                create_test_repo(&repo_path);
            }
            "with_changes" => {
                create_repo_with_changes(&repo_path, 10);
            }
            "with_commits" => {
                create_repo_with_commits(&repo_path, 20);
            }
            "with_stashes" => {
                create_repo_with_stashes(&repo_path, 5);
            }
            "complex" => {
                // Create a complex repo with everything
                create_repo_with_commits(&repo_path, 50);
                create_repo_with_changes(&repo_path, 20);

                // Add remote
                let repo = Repository::open(&repo_path).unwrap();
                repo.remote("origin", "https://github.com/test/repo.git")
                    .unwrap();

                // Create remote tracking branch
                let head = repo.head().unwrap();
                let commit = head.peel_to_commit().unwrap();
                repo.reference(
                    "refs/remotes/origin/main",
                    commit.id(),
                    false,
                    "create remote tracking branch",
                )
                .unwrap();
            }
            _ => {
                create_test_repo(&repo_path);
            }
        }

        paths.push(repo_path);
    }

    paths
}

// Benchmark parallel processing with rayon
fn bench_parallel_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_simple");

    for count in [5, 10, 20, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let temp_dir = TempDir::new().unwrap();
            let paths = create_test_repos(temp_dir.path(), count, "simple");

            b.iter(|| {
                let results = get_repos_info_parallel(black_box(&paths));
                assert_eq!(results.len(), count);
            });
        });
    }

    group.finish();
}

fn bench_parallel_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_complex");
    group.sample_size(10);

    for count in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let temp_dir = TempDir::new().unwrap();
            let paths = create_test_repos(temp_dir.path(), count, "complex");

            b.iter(|| {
                let results = get_repos_info_parallel(black_box(&paths));
                assert_eq!(results.len(), count);
            });
        });
    }

    group.finish();
}

// Benchmark worker-based async processing
fn bench_worker_simple(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("worker_simple");

    for count in [5, 10, 20, 50].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let temp_dir = TempDir::new().unwrap();
            let paths = create_test_repos(temp_dir.path(), count, "simple");

            b.to_async(&rt).iter(|| async {
                let worker = Arc::new(RepoInfoWorker::for_repo_info());
                worker.submit_repos(black_box(&paths));

                let mut results = Vec::new();
                while !worker.is_complete() {
                    let batch = worker.poll_results();
                    results.extend(batch);
                    if !worker.is_complete() {
                        tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
                    }
                }

                // Collect remaining results
                results.extend(worker.poll_results());

                assert_eq!(results.len(), count);
            });
        });
    }

    group.finish();
}

fn bench_worker_complex(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("worker_complex");
    group.sample_size(10);

    for count in [5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let temp_dir = TempDir::new().unwrap();
            let paths = create_test_repos(temp_dir.path(), count, "complex");

            b.to_async(&rt).iter(|| async {
                let worker = Arc::new(RepoInfoWorker::for_repo_info());
                worker.submit_repos(black_box(&paths));

                let mut results = Vec::new();
                while !worker.is_complete() {
                    let batch = worker.poll_results();
                    results.extend(batch);
                    if !worker.is_complete() {
                        tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
                    }
                }

                results.extend(worker.poll_results());

                assert_eq!(results.len(), count);
            });
        });
    }

    group.finish();
}

// Benchmark different repository types
fn bench_repo_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("repo_types");

    for repo_type in ["simple", "with_changes", "with_commits", "with_stashes"].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(repo_type),
            repo_type,
            |b, &repo_type| {
                let temp_dir = TempDir::new().unwrap();
                let paths = create_test_repos(temp_dir.path(), 10, repo_type);

                b.iter(|| {
                    let results = get_repos_info_parallel(black_box(&paths));
                    assert_eq!(results.len(), 10);
                });
            },
        );
    }

    group.finish();
}

// Benchmark worker vs parallel comparison
fn bench_worker_vs_parallel(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("worker_vs_parallel");
    group.sample_size(20);

    let temp_dir = TempDir::new().unwrap();
    let paths = create_test_repos(temp_dir.path(), 30, "complex");

    group.bench_function("parallel_30_repos", |b| {
        b.iter(|| {
            let results = get_repos_info_parallel(black_box(&paths));
            assert_eq!(results.len(), 30);
        });
    });

    group.bench_function("worker_30_repos", |b| {
        b.to_async(&rt).iter(|| async {
            let worker = Arc::new(RepoInfoWorker::for_repo_info());
            worker.submit_repos(black_box(&paths));

            let mut results = Vec::new();
            while !worker.is_complete() {
                let batch = worker.poll_results();
                results.extend(batch);
                if !worker.is_complete() {
                    tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
                }
            }

            results.extend(worker.poll_results());

            assert_eq!(results.len(), 30);
        });
    });

    group.finish();
}

// Benchmark extreme case with many repos
fn bench_extreme_many_repos(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("extreme_many_repos");
    group.sample_size(10);

    group.bench_function("parallel_100_simple_repos", |b| {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_repos(temp_dir.path(), 100, "simple");

        b.iter(|| {
            let results = get_repos_info_parallel(black_box(&paths));
            assert_eq!(results.len(), 100);
        });
    });

    group.bench_function("worker_100_simple_repos", |b| {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_repos(temp_dir.path(), 100, "simple");

        b.to_async(&rt).iter(|| async {
            let worker = Arc::new(RepoInfoWorker::for_repo_info());
            worker.submit_repos(black_box(&paths));

            let mut results = Vec::new();
            while !worker.is_complete() {
                let batch = worker.poll_results();
                results.extend(batch);
                if !worker.is_complete() {
                    tokio::time::sleep(tokio::time::Duration::from_micros(50)).await;
                }
            }

            results.extend(worker.poll_results());

            assert_eq!(results.len(), 100);
        });
    });

    group.bench_function("parallel_50_complex_repos", |b| {
        let temp_dir = TempDir::new().unwrap();
        let paths = create_test_repos(temp_dir.path(), 50, "complex");

        b.iter(|| {
            let results = get_repos_info_parallel(black_box(&paths));
            assert_eq!(results.len(), 50);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parallel_simple,
    bench_parallel_complex,
    bench_worker_simple,
    bench_worker_complex,
    bench_repo_types,
    bench_worker_vs_parallel,
    bench_extreme_many_repos
);

criterion_main!(benches);
