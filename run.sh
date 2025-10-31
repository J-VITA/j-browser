#!/bin/bash

# SyncFlo Browser 실행 스크립트

# Rust 환경 변수 로드
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
else
    echo "경고: $HOME/.cargo/env를 찾을 수 없습니다."
    echo "Rust가 설치되어 있는지 확인하세요."
fi

# 프로젝트 디렉토리로 이동
cd "$(dirname "$0")"

# 빌드 및 실행
echo "SyncFlo Browser를 시작합니다..."
cargo run

