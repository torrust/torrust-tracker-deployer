#!/bin/bash

set -euo pipefail

usage() {
    cat <<'EOF'
Usage: ./scripts/update-dependencies.sh --branch <branch-name> [options]

Automates the dependency update workflow:
- sync the base branch from the base remote
- create a fresh working branch
- run cargo update
- run pre-commit checks
- create a signed commit with the full cargo update output
- optionally push the branch and create a pull request

Options:
  --branch <name>                  Working branch to create (required)
  --base-branch <name>             Base branch to update from (default: main)
  --base-remote <name>             Remote that owns the base branch (default: torrust, then origin, then first remote)
  --push-remote <name>             Remote used to push the branch
  --repo <owner/repo>              Repository slug for PR creation
  --commit-title <title>           Commit title and default PR title
  --pr-title <title>               Pull request title override
  --delete-existing-branch         Delete an existing local/remote branch with the same name
  --skip-pre-commit                Skip ./scripts/pre-commit.sh
  --create-pr                      Create a PR after pushing the branch
  --no-sign-commit                 Do not use git commit -S
  --help                           Show this help message

Examples:
  ./scripts/update-dependencies.sh \
    --branch 445-update-dependencies \
    --push-remote josecelano \
    --create-pr

  ./scripts/update-dependencies.sh \
    --branch update-dependencies \
    --push-remote josecelano \
    --delete-existing-branch \
    --commit-title "chore: update dependencies"
EOF
}

log() {
    echo "[update-dependencies] $*"
}

fail() {
    echo "Error: $*" >&2
    exit 1
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

detect_default_remote() {
    if git remote | grep -qx "torrust"; then
        echo "torrust"
        return
    fi

    if git remote | grep -qx "origin"; then
        echo "origin"
        return
    fi

    git remote | head -n 1
}

parse_github_slug_from_remote() {
    local remote=$1
    local remote_url
    local slug

    remote_url=$(git remote get-url "$remote")
    slug=$(printf '%s' "$remote_url" | sed -E 's#^(git@github.com:|https://github.com/|ssh://git@github.com/)##; s#\.git$##')

    [[ "$slug" == */* ]] || fail "Could not parse GitHub repository slug from remote '$remote'"

    echo "$slug"
}

parse_github_owner_from_remote() {
    local remote=$1
    local slug

    slug=$(parse_github_slug_from_remote "$remote")
    echo "${slug%%/*}"
}

branch_exists_local() {
    local branch=$1
    git show-ref --verify --quiet "refs/heads/$branch"
}

branch_exists_remote() {
    local remote=$1
    local branch=$2
    git ls-remote --exit-code --heads "$remote" "$branch" >/dev/null 2>&1
}

ensure_clean_worktree() {
    git diff --quiet || fail "Working tree has unstaged changes"
    git diff --cached --quiet || fail "Working tree has staged changes"
}

cleanup_files() {
    rm -f "$CARGO_UPDATE_OUTPUT_FILE" "$COMMIT_MESSAGE_FILE" "$PR_BODY_FILE"
}

BRANCH_NAME=""
BASE_BRANCH="main"
BASE_REMOTE=""
PUSH_REMOTE=""
REPOSITORY_SLUG=""
COMMIT_TITLE="chore: update dependencies"
PR_TITLE=""
DELETE_EXISTING_BRANCH=false
RUN_PRE_COMMIT=true
CREATE_PR=false
SIGN_COMMIT=true

while [[ $# -gt 0 ]]; do
    case "$1" in
        --branch)
            BRANCH_NAME=${2:-}
            shift 2
            ;;
        --base-branch)
            BASE_BRANCH=${2:-}
            shift 2
            ;;
        --base-remote)
            BASE_REMOTE=${2:-}
            shift 2
            ;;
        --push-remote)
            PUSH_REMOTE=${2:-}
            shift 2
            ;;
        --repo)
            REPOSITORY_SLUG=${2:-}
            shift 2
            ;;
        --commit-title)
            COMMIT_TITLE=${2:-}
            shift 2
            ;;
        --pr-title)
            PR_TITLE=${2:-}
            shift 2
            ;;
        --delete-existing-branch)
            DELETE_EXISTING_BRANCH=true
            shift
            ;;
        --skip-pre-commit)
            RUN_PRE_COMMIT=false
            shift
            ;;
        --create-pr)
            CREATE_PR=true
            shift
            ;;
        --no-sign-commit)
            SIGN_COMMIT=false
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            fail "Unknown option: $1"
            ;;
    esac
done

[[ -n "$BRANCH_NAME" ]] || fail "--branch is required"

command_exists git || fail "git is required"
command_exists cargo || fail "cargo is required"

BASE_REMOTE=${BASE_REMOTE:-$(detect_default_remote)}
[[ -n "$BASE_REMOTE" ]] || fail "Could not determine a base remote"

if [[ -z "$REPOSITORY_SLUG" ]]; then
    REPOSITORY_SLUG=$(parse_github_slug_from_remote "$BASE_REMOTE")
fi

if [[ "$CREATE_PR" == true ]]; then
    [[ -n "$PUSH_REMOTE" ]] || fail "--push-remote is required when --create-pr is used"
    command_exists gh || fail "gh is required when --create-pr is used"
fi

if [[ -n "$PUSH_REMOTE" ]]; then
    git remote get-url "$PUSH_REMOTE" >/dev/null 2>&1 || fail "Remote '$PUSH_REMOTE' does not exist"
fi

CARGO_UPDATE_OUTPUT_FILE=$(mktemp)
COMMIT_MESSAGE_FILE=$(mktemp)
PR_BODY_FILE=$(mktemp)
trap cleanup_files EXIT

ensure_clean_worktree

if branch_exists_local "$BRANCH_NAME"; then
    if [[ "$DELETE_EXISTING_BRANCH" == true ]]; then
        log "Deleting local branch '$BRANCH_NAME'"
        current_branch=$(git branch --show-current)
        if [[ "$current_branch" == "$BRANCH_NAME" ]]; then
            git checkout "$BASE_BRANCH"
        fi
        git branch -D "$BRANCH_NAME"
    else
        fail "Local branch '$BRANCH_NAME' already exists. Use --delete-existing-branch to replace it."
    fi
fi

if [[ -n "$PUSH_REMOTE" ]] && branch_exists_remote "$PUSH_REMOTE" "$BRANCH_NAME"; then
    if [[ "$DELETE_EXISTING_BRANCH" == true ]]; then
        log "Deleting remote branch '$BRANCH_NAME' from '$PUSH_REMOTE'"
        git push "$PUSH_REMOTE" --delete "$BRANCH_NAME"
    else
        fail "Remote branch '$BRANCH_NAME' already exists on '$PUSH_REMOTE'. Use --delete-existing-branch to replace it."
    fi
fi

log "Fetching '$BASE_REMOTE/$BASE_BRANCH'"
git fetch "$BASE_REMOTE" "$BASE_BRANCH"

log "Checking out '$BASE_BRANCH'"
git checkout "$BASE_BRANCH"

log "Fast-forwarding '$BASE_BRANCH' from '$BASE_REMOTE/$BASE_BRANCH'"
git merge --ff-only "$BASE_REMOTE/$BASE_BRANCH"

log "Creating branch '$BRANCH_NAME'"
git checkout -b "$BRANCH_NAME"

log "Running cargo update"
cargo update 2>&1 | tee "$CARGO_UPDATE_OUTPUT_FILE"

if git diff --quiet; then
    log "No dependency changes were produced by cargo update"
    git checkout "$BASE_BRANCH"
    git branch -D "$BRANCH_NAME"
    exit 0
fi

if [[ "$RUN_PRE_COMMIT" == true ]]; then
    log "Running pre-commit checks"
    ./scripts/pre-commit.sh
    PRE_COMMIT_SUMMARY="- run \`./scripts/pre-commit.sh\` successfully"
else
    PRE_COMMIT_SUMMARY="- skip \`./scripts/pre-commit.sh\` by request"
fi

{
    printf '%s\n\n' "$COMMIT_TITLE"
    printf '%s\n' 'cargo update output:'
    printf '%s\n' '```'
    cat "$CARGO_UPDATE_OUTPUT_FILE"
    printf '%s\n' '```'
} > "$COMMIT_MESSAGE_FILE"

log "Creating commit"
git add -u
if [[ "$SIGN_COMMIT" == true ]]; then
    git commit -S -F "$COMMIT_MESSAGE_FILE"
else
    git commit -F "$COMMIT_MESSAGE_FILE"
fi

if [[ -n "$PUSH_REMOTE" ]]; then
    log "Pushing branch to '$PUSH_REMOTE'"
    git push -u "$PUSH_REMOTE" "$BRANCH_NAME"
fi

if [[ "$CREATE_PR" == true ]]; then
    HEAD_OWNER=$(parse_github_owner_from_remote "$PUSH_REMOTE")
    PR_TITLE=${PR_TITLE:-$COMMIT_TITLE}

    {
        printf '%s\n' '## Summary'
        printf '%s\n' "- run \`cargo update\`"
        printf '%s\n' "- commit the resulting \`Cargo.lock\` changes"
        printf '%s\n\n' "$PRE_COMMIT_SUMMARY"
        printf '%s\n' '## cargo update output'
        printf '%s\n' '```'
        cat "$CARGO_UPDATE_OUTPUT_FILE"
        printf '%s\n' '```'
    } > "$PR_BODY_FILE"

    log "Creating pull request in '$REPOSITORY_SLUG'"
    gh pr create \
        --repo "$REPOSITORY_SLUG" \
        --base "$BASE_BRANCH" \
        --head "$HEAD_OWNER:$BRANCH_NAME" \
        --title "$PR_TITLE" \
        --body-file "$PR_BODY_FILE"
fi

log "Dependency update workflow completed"
