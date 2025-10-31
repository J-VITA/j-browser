# SyncFlo Browser 실행 가이드

## 필수 설치 사항

### 1. Rust 및 Cargo
이미 설치되어 있습니다. 확인하려면:
```bash
rustc --version
cargo --version
```

환경 변수가 로드되지 않았다면:
```bash
source "$HOME/.cargo/env"
```

### 2. macOS 시스템 요구사항
macOS에서는 WebKit이 기본적으로 설치되어 있으므로 **추가 설치가 필요하지 않습니다**.

## 실행 방법

### 방법 1: 개발 모드로 실행
```bash
source "$HOME/.cargo/env"
cd /Users/jaykim/TEST/syncflo-browser
cargo run
```

### 방법 2: 릴리즈 빌드 후 실행
```bash
source "$HOME/.cargo/env"
cd /Users/jaykim/TEST/syncflo-browser

# 릴리즈 빌드
cargo build --release

# 실행
./target/release/syncflo-browser
```

### 방법 3: 디버그 로그와 함께 실행
```bash
source "$HOME/.cargo/env"
cd /Users/jaykim/TEST/syncflo-browser
RUST_LOG=debug cargo run
```

## 문제 해결

### 문제: "command not found: cargo"
**해결책:**
```bash
source "$HOME/.cargo/env"
```

또는 `~/.zshrc` 또는 `~/.bash_profile`에 다음을 추가:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### 문제: WebView 관련 에러
macOS에서는 일반적으로 문제가 없지만, 만약 에러가 발생한다면:
1. 시스템 업데이트 확인
2. Xcode Command Line Tools 설치:
```bash
xcode-select --install
```

### 문제: 빌드 실패
의존성을 업데이트:
```bash
cargo update
```

## 실행 확인

프로그램이 성공적으로 실행되면:
- "SyncFlo Browser" 타이틀의 창이 열립니다
- Google 홈페이지가 표시됩니다
- 개발자 도구가 활성화되어 있습니다 (Cmd+Option+I)

## 빠른 시작 스크립트

편의를 위해 실행 스크립트를 만들 수 있습니다:

```bash
cat > ~/run-browser.sh << 'EOF'
#!/bin/bash
source "$HOME/.cargo/env"
cd /Users/jaykim/TEST/syncflo-browser
cargo run
EOF

chmod +x ~/run-browser.sh
```

실행:
```bash
~/run-browser.sh
```

