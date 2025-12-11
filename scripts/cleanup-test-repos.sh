#!/usr/bin/env bash
# Cleanup test environment for reponest TUI testing

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
FORCE=false

print_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Cleanup test environment for reponest TUI testing.

OPTIONS:
    -d, --dir PATH      Test directory path (default: ./test-repos)
    -f, --force         Force removal without confirmation
    -h, --help          Show this help message

EXAMPLES:
    $0                  # Remove default test environment (with confirmation)
    $0 -d /tmp/test     # Remove custom test directory
    $0 -f               # Force remove without asking
EOF
}

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--dir)
            TEST_DIR="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
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

echo -e "${BLUE}=== Reponest Test Environment Cleanup ===${NC}\n"
echo -e "Target directory: ${YELLOW}${TEST_DIR}${NC}\n"

# Check if directory exists
if [ ! -d "$TEST_DIR" ]; then
    echo -e "${RED}Error: Directory $TEST_DIR does not exist${NC}"
    exit 1
fi

# Count items
total_items=$(ls -1 "$TEST_DIR" | wc -l)
git_repos=$(find "$TEST_DIR" -maxdepth 2 -name ".git" -type d | wc -l)

echo -e "Directory contents:"
echo -e "  Total items: ${BLUE}${total_items}${NC}"
echo -e "  Git repositories: ${GREEN}${git_repos}${NC}\n"

# Confirmation
if [ "$FORCE" = false ]; then
    echo -e "${YELLOW}Warning: This will permanently delete the entire directory and all its contents!${NC}"
    read -p "Are you sure you want to continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Aborted${NC}"
        exit 0
    fi
fi

# Remove directory
echo -e "${YELLOW}Removing directory...${NC}"
rm -rf "$TEST_DIR"

if [ ! -d "$TEST_DIR" ]; then
    echo -e "${GREEN}✓ Successfully removed: ${TEST_DIR}${NC}"
else
    echo -e "${RED}✗ Failed to remove directory${NC}"
    exit 1
fi

echo -e "\n${GREEN}=== Cleanup Complete ===${NC}\n"
