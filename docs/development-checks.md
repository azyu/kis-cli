# Development Checks

저장소 표준 개발 체크는 루트 `Makefile`을 진입점으로 사용한다. 로컬 훅과 GitHub Actions도 같은 명령을 재사용한다.

## Prerequisites

- 로컬 개발 워크플로우는 POSIX shell + `make` 기준이다.
- macOS/Linux는 그대로 사용할 수 있다.
- Windows에서는 Git Bash 또는 WSL 사용을 전제로 한다.

## Standard Commands

```bash
make fmt
make fmt-check
make lint
make test
make hooks-install
```

- `make fmt`: Rust workspace 포맷 적용
- `make fmt-check`: 포맷 검증
- `make lint`: workspace 전체 `clippy` (`--all-targets`, `-D warnings`)
- `make test`: workspace 전체 테스트
- `make hooks-install`: repo-local Git hooks 경로를 `.githooks`로 고정

## Hook Policy

- `.githooks/pre-commit`: `make fmt-check`
- `.githooks/pre-push`: `make lint`, `make test`

새로 clone한 뒤 한 번만 아래 명령을 실행하면 된다.

```bash
make hooks-install
```

## CI Policy

- PR/`main` CI는 `make fmt-check`, `make lint`, `make test`를 다시 실행한다.
- 릴리스 바이너리를 배포하는 저장소이므로 PR CI에서는 release `kis` 빌드 검증을 별도 step으로 유지한다.

## Copy Checklist

1. 루트 `rust-toolchain.toml`을 추가하고 Rust 채널과 `clippy`/`rustfmt` 컴포넌트를 고정한다.
2. 루트 `Makefile`에 `fmt`, `fmt-check`, `lint`, `test`, `hooks-install` 표준 진입점을 정의한다.
3. `.githooks/pre-commit`, `.githooks/pre-push`를 같이 복사하고, clone 후 `make hooks-install`을 실행한다.
4. CI에서 `make fmt-check`, `make lint`, `make test`를 재사용하고, 필요하면 release build 검증은 별도 step으로 유지한다.
5. README 같은 온보딩 문서도 raw 명령 대신 `make` 진입점을 가리키도록 같이 갱신한다.
