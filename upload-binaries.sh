#!/bin/sh

# create an array that will contain the targets
target_containers=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")

# create an array that will contain architectures
target_archs=("x64" "arm64")

# build and package the binaries
if [ $# -gt 0 ]; then
    for i in "${!target_containers[@]}"; do
        echo "Building for ${target_containers[$i]}"
        TARGET=${target_containers[$i]} npm run build-cross:release && npm run package --target_arch=${target_archs[$i]}
        echo "Uploading release binaries"
        NODE_PRE_GYP_GITHUB_TOKEN=$1 npx node-pre-gyp-github publish --release
        echo "Removing output files to continue"
        rm -r ./build/stage
    done
fi
