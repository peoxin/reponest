#!/usr/bin/env bash
# Setup test environment for reponest TUI testing
# Creates multiple Git repositories with various states for realistic testing

set -e

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default test directory (in project root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DIR="${PROJECT_ROOT}/test-repos"

# Parse arguments
REPO_COUNT=10
INCLUDE_NOISE=true

print_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Setup test environment for reponest TUI testing.

OPTIONS:
    -d, --dir PATH          Test directory path (default: ./test-repos)
    -n, --count NUMBER      Number of repositories to create (default: 10)
    --no-noise              Don't create noise directories
    -h, --help              Show this help message

EXAMPLES:
    $0                      # Create default test environment
    $0 -n 20                # Create 20 repositories
    $0 -d /tmp/test         # Create in custom directory
    $0 -n 50 --no-noise     # Create 50 repos without noise directories
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--dir)
            TEST_DIR="$2"
            shift 2
            ;;
        -n|--count)
            REPO_COUNT="$2"
            shift 2
            ;;
        --no-noise)
            INCLUDE_NOISE=false
            shift
            ;;
        -h|--help)
            print_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            print_help
            exit 1
            ;;
    esac
done

echo -e "${BLUE}=== Reponest Test Environment Setup ===${NC}\n"
echo -e "Test directory: ${GREEN}${TEST_DIR}${NC}"
echo -e "Repository count: ${GREEN}${REPO_COUNT}${NC}"
echo -e "Include noise: ${GREEN}${INCLUDE_NOISE}${NC}\n"

# Check if directory exists
if [ -d "$TEST_DIR" ]; then
    echo -e "${YELLOW}Warning: Directory $TEST_DIR already exists${NC}"
    read -p "Do you want to remove it and recreate? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Removing existing directory...${NC}"
        rm -rf "$TEST_DIR"
    else
        echo -e "${RED}Aborted${NC}"
        exit 1
    fi
fi

# Create test directory
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo -e "${GREEN}✓${NC} Created test directory: $TEST_DIR\n"

# Function to create a simple repository
create_simple_repo() {
    local name=$1
    mkdir -p "$name"
    cd "$name"
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    echo "# $name" > README.md
    git add README.md
    git commit -q -m "Initial commit"
    cd ..
}

# Function to create a repository with changes
create_repo_with_changes() {
    local name=$1
    mkdir -p "$name"
    cd "$name"
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"

    # Initial commit
    echo "# $name" > README.md
    git add README.md
    git commit -q -m "Initial commit"

    # Add some files and changes
    echo "console.log('Hello');" > index.js
    echo "def main(): pass" > main.py
    git add index.js main.py
    git commit -q -m "Add source files"

    # Create uncommitted changes
    echo "// Modified" >> index.js
    echo "new-file.txt" > new-file.txt

    cd ..
}

# Function to create a repository with multiple branches
create_repo_with_branches() {
    local name=$1
    mkdir -p "$name"
    cd "$name"
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"

    # Main branch
    echo "# $name" > README.md
    git add README.md
    git commit -q -m "Initial commit"

    # Feature branch
    git checkout -q -b feature/new-feature
    echo "feat: new feature" > feature.txt
    git add feature.txt
    git commit -q -m "Add new feature"

    # Dev branch
    git checkout -q -b dev
    echo "dev work" > dev.txt
    git add dev.txt
    git commit -q -m "Dev work"

    git checkout -q main
    cd ..
}

# Function to create a repository with stashes
create_repo_with_stash() {
    local name=$1
    mkdir -p "$name"
    cd "$name"
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"

    echo "# $name" > README.md
    git add README.md
    git commit -q -m "Initial commit"

    # Create and stash changes
    echo "work in progress" > wip.txt
    git add wip.txt
    git stash -q

    cd ..
}

# Function to create nested repositories
create_nested_repos() {
    local base=$1
    mkdir -p "$base"
    cd "$base"

    # Parent repo
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    echo "# Parent Repo" > README.md
    git add README.md
    git commit -q -m "Initial commit"

    # Nested repo 1
    mkdir -p packages/pkg1
    cd packages/pkg1
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    echo "# Package 1" > README.md
    git add README.md
    git commit -q -m "Initial commit"
    cd ../..

    # Nested repo 2
    mkdir -p packages/pkg2
    cd packages/pkg2
    git init -q
    git config user.name "Test User"
    git config user.email "test@example.com"
    echo "# Package 2" > README.md
    git add README.md
    git commit -q -m "Initial commit"
    cd ../..

    cd ..
}

# Function to create noise directories (non-git)
create_noise_dirs() {
    local count=$1
    for i in $(seq 1 $count); do
        local dir="noise-dir-$i"
        mkdir -p "$dir"
        echo "This is not a git repository" > "$dir/README.txt"

        # Add some subdirectories
        mkdir -p "$dir/subdir1" "$dir/subdir2"
        echo "data" > "$dir/subdir1/data.txt"
        echo "more data" > "$dir/subdir2/data.txt"
    done
}

echo -e "${BLUE}Creating repositories...${NC}\n"

# Create various types of repositories
created=0

# Simple repos (40%)
simple_count=$((REPO_COUNT * 40 / 100))
for i in $(seq 1 $simple_count); do
    create_simple_repo "simple-repo-$i"
    created=$((created + 1))
    echo -e "${GREEN}✓${NC} Created simple repository: simple-repo-$i"
done

# Repos with changes (30%)
changes_count=$((REPO_COUNT * 30 / 100))
for i in $(seq 1 $changes_count); do
    create_repo_with_changes "project-$i"
    created=$((created + 1))
    echo -e "${GREEN}✓${NC} Created repository with changes: project-$i"
done

# Repos with branches (20%)
branches_count=$((REPO_COUNT * 20 / 100))
for i in $(seq 1 $branches_count); do
    create_repo_with_branches "multi-branch-$i"
    created=$((created + 1))
    echo -e "${GREEN}✓${NC} Created repository with branches: multi-branch-$i"
done

# Repos with stash (5%)
stash_count=$((REPO_COUNT * 5 / 100))
if [ $stash_count -lt 1 ]; then
    stash_count=1
fi
for i in $(seq 1 $stash_count); do
    create_repo_with_stash "stash-repo-$i"
    created=$((created + 1))
    echo -e "${GREEN}✓${NC} Created repository with stash: stash-repo-$i"
done

# Nested repos (remaining)
remaining=$((REPO_COUNT - created))
if [ $remaining -gt 0 ]; then
    create_nested_repos "monorepo"
    created=$((created + 3)) # parent + 2 nested
    echo -e "${GREEN}✓${NC} Created nested repositories: monorepo (with 2 packages)"
fi

# Create noise directories if enabled
if [ "$INCLUDE_NOISE" = true ]; then
    echo -e "\n${BLUE}Creating noise directories...${NC}\n"
    noise_count=$((REPO_COUNT * 2))
    create_noise_dirs $noise_count
    echo -e "${GREEN}✓${NC} Created $noise_count noise directories"
fi

# Summary
echo -e "\n${GREEN}=== Setup Complete ===${NC}\n"
echo -e "Test environment created at: ${BLUE}${TEST_DIR}${NC}"
echo -e "Total repositories created: ${GREEN}${created}${NC}"
if [ "$INCLUDE_NOISE" = true ]; then
    echo -e "Noise directories: ${YELLOW}${noise_count}${NC}"
fi
echo -e "\n${BLUE}Test reponest with:${NC}"
echo -e "  cd ${TEST_DIR}"
echo -e "  reponest"
echo -e "\n${YELLOW}Clean up with:${NC}"
echo -e "  ./scripts/cleanup-test-repos.sh -d ${TEST_DIR}"
echo
