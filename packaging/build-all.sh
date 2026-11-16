#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"

"$project_dir/packaging/bump-version.sh"
export LEAF_VERSION_BUMPED=1
"$project_dir/packaging/build-linux.sh"
"$project_dir/packaging/build-windows.sh"
"$project_dir/packaging/build-installer.sh"
"$project_dir/packaging/build-android.sh"
