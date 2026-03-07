# Rust 2-Crate Architecture Spec

## Summary

현재 공식 Rust workspace는 `kis-core`와 `kis-cli` 두 크레이트만 유지한다.

- `kis-core`: 설정, 인증, HTTP/WebSocket 클라이언트, 국내/해외 도메인 API
- `kis-cli`: `clap` 파서, runtime dispatch, text/JSON 출력, `kis` 바이너리 엔트리포인트

## Workspace Layout

```text
rust/
├── kis-core/
│   └── src/
│       ├── api_client.rs
│       ├── auth.rs
│       ├── client.rs
│       ├── config.rs
│       ├── error.rs
│       ├── ws.rs
│       ├── domestic/
│       └── overseas/
└── kis-cli/
    └── src/
        ├── cli.rs
        ├── lib.rs
        ├── main.rs
        ├── render.rs
        └── runtime.rs
```

## Dependency Rules

- `kis-cli`는 `kis-core`만 의존한다.
- `kis-core`는 `kis-cli`를 참조하지 않는다.
- 별도 `kis-api` crate는 두지 않는다.
- 새 국내/해외 API는 `kis-core/src/domestic/` 또는 `kis-core/src/overseas/`에 추가한다.
- 도메인 공통 API helper trait/envelope 파싱은 `kis-core/src/api_client.rs`에 둔다.
- 공통 HTTP, auth, config, error, websocket은 각각 `client.rs`, `auth.rs`, `config.rs`, `error.rs`, `ws.rs`에 둔다.

## Public Module Contract

`kis-core`는 다음 모듈 경로를 공개 표면으로 유지한다.

- `kis_core::api_client`
- `kis_core::auth`
- `kis_core::client`
- `kis_core::config`
- `kis_core::domestic`
- `kis_core::error`
- `kis_core::overseas`
- `kis_core::ws`

`kis-cli`는 도메인 접근 시 `kis_core::domestic::*`, `kis_core::overseas::*`를 사용한다.

## Ownership Guidance

- Core 변경: 설정, 인증, HTTP/WebSocket, 공통 에러, workspace manifest
- Domain 변경: 국내/해외 REST 도메인, TR ID 매핑, 거래소 코드 변환, pagination
- CLI 변경: 명령어 표면, 실행 라우팅, stdout/stderr/JSON 계약
- 구조를 바꾸면 `AGENTS.md`, `.claude/STEERING.md`, `README.md`와 함께 이 문서를 갱신한다
