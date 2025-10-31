# SyncFlo Browser

AI-powered browsing assistant browser built with Rust.

## Features

- Modern web browsing experience
- AI-powered assistance for web navigation
- Built with Rust for performance and safety
- Cross-platform support (Windows, macOS, Linux)

## Prerequisites

### 이미 설치됨
- ✅ Rust (1.91.0)
- ✅ Cargo (1.91.0)

### macOS 시스템 요구사항
- ✅ WebKit (기본 설치됨)
- ✅ 추가 설치 불필요

### Linux 시스템의 경우
```bash
# Debian/Ubuntu
sudo apt-get install libwebkit2gtk-4.0-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel
```

## 빠른 시작

### 1단계: 환경 변수 로드
```bash
source "$HOME/.cargo/env"
```

### 2단계: 프로젝트 디렉토리로 이동
```bash
cd /Users/jaykim/TEST/syncflo-browser
```

### 3단계: 실행
```bash
cargo run
```

또는 릴리즈 빌드:
```bash
cargo build --release
./target/release/syncflo-browser
```

## 상세한 실행 가이드

더 자세한 내용은 [RUN.md](RUN.md)를 참조하세요.

## Development

```bash
# With logging
RUST_LOG=debug cargo run
```

## Project Structure

```
syncflo-browser/
├── src/
│   ├── main.rs          # Entry point
│   ├── browser/         # Browser core
│   ├── ai/             # AI integration
│   └── ui/             # User interface
├── Cargo.toml
└── README.md
```

## License

MIT OR Apache-2.0
