#!/usr/bin/env bash

set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
cd -- "$script_dir"

target_containers=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
target_archs=("x64" "arm64")
version=$(node -p "require('./package.json').version")

if [[ -n $(git status --porcelain) ]]; then
    echo "The working tree must be clean before uploading release binaries." >&2
    exit 1
fi

if ! tag_commit=$(git rev-parse --verify "refs/tags/$version^{commit}" 2>/dev/null); then
    echo "Local tag $version is required before uploading release binaries." >&2
    exit 1
fi

head_commit=$(git rev-parse HEAD)
if [[ "$head_commit" != "$tag_commit" ]]; then
    echo "HEAD must match tag $version before uploading release binaries." >&2
    exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
    echo "GitHub CLI (gh) is required to upload release binaries." >&2
    exit 1
fi

gh auth status >/dev/null
rm -rf ./build/stage

for i in "${!target_containers[@]}"; do
    echo "Building for ${target_containers[$i]}"
    TARGET=${target_containers[$i]} npm run build-cross:release
    npm run package -- --target_arch="${target_archs[$i]}"
done

if ! gh release view "$version" >/dev/null 2>&1; then
    gh release create "$version" --verify-tag --title "v$version" --generate-notes
fi

echo "Uploading release binaries"
gh release upload "$version" build/stage/"$version"/*.tar.gz

echo "Removing output files"
rm -rf ./build/stage
