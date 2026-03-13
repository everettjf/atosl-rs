#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

usage() {
  cat <<'EOF'
Usage:
  ./deploy.sh
  ./deploy.sh patch
  ./deploy.sh minor
  ./deploy.sh major
  ./deploy.sh 0.1.16

Behavior:
  - Verifies the git working tree is clean
  - Bumps the version in Cargo.toml
  - Regenerates Cargo.lock
  - Runs validation
  - Commits and tags the release
  - Runs cargo publish --dry-run
  - Runs cargo publish
  - Pushes the current branch and the release tag

Notes:
  - Default bump is patch
  - Requires cargo publish access and a configured git remote
EOF
}

require_clean_tree() {
  if [[ -n "$(git status --short)" ]]; then
    echo "working tree is not clean; commit or stash changes first" >&2
    exit 1
  fi
}

current_branch() {
  git branch --show-current
}

current_version() {
  awk -F'"' '
    $1 ~ /^version = / {
      print $2
      exit
    }
  ' Cargo.toml
}

is_semver() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]
}

bump_version() {
  local current="$1"
  local mode="$2"
  local major minor patch
  IFS='.' read -r major minor patch <<<"$current"

  case "$mode" in
    major)
      echo "$((major + 1)).0.0"
      ;;
    minor)
      echo "${major}.$((minor + 1)).0"
      ;;
    patch)
      echo "${major}.${minor}.$((patch + 1))"
      ;;
    *)
      echo "unsupported bump mode: $mode" >&2
      exit 1
      ;;
  esac
}

replace_version() {
  local from="$1"
  local to="$2"
  perl -0pi -e "s/version = \"\Q$from\E\"/version = \"$to\"/" Cargo.toml
}

validate_release() {
  cargo fmt
  cargo clippy --all-targets -- -D warnings
  cargo test --all-targets
  cargo bench --bench batch_symbolize --no-run
}

main() {
  if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
  fi

  require_clean_tree

  local branch version_arg old_version new_version tag
  branch="$(current_branch)"
  old_version="$(current_version)"
  version_arg="${1:-patch}"

  if is_semver "$version_arg"; then
    new_version="$version_arg"
  else
    new_version="$(bump_version "$old_version" "$version_arg")"
  fi

  if [[ "$new_version" == "$old_version" ]]; then
    echo "new version matches current version: $old_version" >&2
    exit 1
  fi

  tag="v${new_version}"

  if git rev-parse "$tag" >/dev/null 2>&1; then
    echo "git tag already exists: $tag" >&2
    exit 1
  fi

  echo "releasing $old_version -> $new_version on branch $branch"

  replace_version "$old_version" "$new_version"

  cargo check >/dev/null
  validate_release

  git add Cargo.toml Cargo.lock
  git commit -m "Release v${new_version}"
  git tag "$tag"

  cargo publish --dry-run
  cargo publish

  git push origin "$branch"
  git push origin "$tag"

  echo "released ${new_version}"
}

main "$@"
