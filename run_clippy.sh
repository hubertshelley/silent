#!/bin/bash

# 定义项目的根目录
PROJECT_ROOT="examples"

# 遍历项目根目录下的所有项目
for project in "$PROJECT_ROOT"/*; do
    echo "Checking $project"
    if [ -d "$project" ]; then
        # 进入项目目录
        cd "$project"
        # 运行 cargo clippy
        cargo clippy --all-targets --all-features --tests --benches -- -D warnings
        # 返回项目组根目录
        cd -
    fi
done
