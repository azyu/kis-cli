# AGENTS.md

## Agent Workflow

**작업 시작 전 반드시 다음 세 파일을 읽는다:**

1. `.context/TASKS.md` - 현재 진행 중인 작업 목록과 상태
2. `.context/STEERING.md` - 작업 방향, 우선순위, 에이전트 간 조율 지침
3. `docs/SPEC.md` - 현재 Rust 2-crate 아키텍처와 모듈 경계 기술 명세

### TASKS.md 규칙

- 각 태스크는 `[ ]` (미완료) / `[x]` (완료) / `[~]` (진행중) 체크박스로 상태를 표시한다
- 작업을 시작하면 `[~]`로 변경하고, 완료 시 `[x]`로 변경한다
- 새 태스크가 필요하면 추가하되, 기존 태스크를 임의로 삭제하지 않는다

### STEERING.md 규칙

- 현재 프로젝트 방향, 우선순위, 주의사항을 기록한다
- 에이전트 간 충돌 방지를 위한 작업 영역 분담 정보를 포함한다
- 작업 중 발견한 중요한 결정사항이나 블로커를 기록한다

### SPEC.md 규칙

- Rust 기술 구조의 single source of truth로 취급한다
- crate 경계, 모듈 배치, 의존 방향을 바꾸면 함께 갱신한다
- 구현 세부보다 현재 유지해야 하는 구조 제약을 우선 기록한다

### 커밋 규칙

- 작업이 정상적으로 완료되면 반드시 커밋한다
- 커밋 메시지는 Conventional Commits 형식을 따른다 (`feat:`, `fix:`, `docs:`, `refactor:`, `test:` 등)

### Definition of Done (DoD)

- 사용자 요청 범위를 충족하는 최소 변경만 반영한다
- 이번 변경으로 영향받는 문서/명세/협업 문서(`AGENTS.md`, `.context/TASKS.md`, `.context/STEERING.md`, `docs/SPEC.md`)를 필요한 범위에서 함께 갱신한다
- 적용한 검증을 실행해 결과를 확인한다. 검증을 실행하지 못했다면 이유와 미실행 범위를 작업 보고에 명시한다
- 남은 제약, 후속 작업, 운영 결정이 생기면 `.context/TASKS.md` 또는 `.context/STEERING.md`에 기록한다
- 결과, 검증, 남은 리스크를 사용자에게 보고할 수 있는 상태가 되면 완료로 간주한다

### Git/PR 절차

- 모든 작업은 `main`에서 분기한 작업 브랜치에서 시작한다
- 커밋은 논리적 단계별로 나누고 Conventional Commits 형식을 유지한다
- DoD를 충족하면 현재 작업 브랜치를 원격에 push한다
- push 후 변경 요약, 검증 결과, 남은 리스크를 포함한 PR을 작성한다

---

## Agent Roles

에이전트별 담당 영역. 현재 저장소는 Rust 구현을 기준으로 작업한다.

### Agent 1: Core (설정 + 인증 + HTTP 클라이언트)

- **담당:** `rust/kis-core/src/auth.rs`, `rust/kis-core/src/client.rs`, `rust/kis-core/src/config.rs`, `rust/kis-core/src/error.rs`, `rust/kis-core/src/ws.rs`, `rust/Cargo.toml`
- **역할:** YAML 설정 로딩, 환경 매핑, OAuth 토큰 발급/캐싱, hashkey, 공통 HTTP 클라이언트, Rate Limiting
- **Skills:** `api-design`, `api-security-hardening`

### Agent 2: Domain (주식 도메인 로직)

- **담당:** `rust/kis-core/src/api_client.rs`, `rust/kis-core/src/domestic/`, `rust/kis-core/src/overseas/`
- **역할:** 국내/해외 시세 조회, 주문, 잔고 등 API 구현, TR ID 매핑, 거래소 코드 변환
- **Skills:** `api-design`

### Agent 3: CLI (명령어 인터페이스)

- **담당:** `rust/kis-cli/`
- **역할:** `clap` 기반 서브커맨드 구조, stdout 출력 포맷팅, `kis` 바이너리 엔트리포인트
- **Skills:** `domain-cli`, `conventional-commit`

### Agent 4: Quality (테스트 + 리뷰)

- **담당:** `rust/**/tests`, Rust 테스트 모듈, CI 설정
- **역할:** 단위/통합 테스트 작성, 회귀 검증, 모의투자 환경 smoke test 준비
- **Skills:** `rust-testing`, `conventional-commit`

---

## Installed Skills

| Skill | 용도 |
|-------|------|
| `api-design` | REST API 클라이언트 설계 패턴 |
| `api-security-hardening` | API 키/토큰 관리, 보안 하드닝 |
| `conventional-commit` | Conventional Commits 규칙 적용 |
| `domain-cli` | `clap` 기반 Rust CLI 설계, 서브커맨드/출력/환경변수 우선순위 |
| `rust-testing` | Rust 단위/통합/CLI 테스트 패턴 |
| `ratatui-tui` | Rust TUI 설계 참고용 (`kis tui` 검토 시 사용) |

---

## Project Overview

**kis-cli** - 한국투자증권(KIS) Open API를 활용한 CLI 도구.
동일 저장소 내 `rust/` workspace에서 Rust CLI `kis`를 구현하고 유지한다.

## Tech Stack

- **Language:** Rust
- **Target API:** KIS Open Trading API (REST + WebSocket)
- **Config format:** YAML (`kis_devlp.yaml` 형식 기반)

## Project Structure

```
kis-cli/
├── AGENTS.md              # 이 파일
├── .context/
│   ├── TASKS.md           # 작업 목록과 상태
│   └── STEERING.md        # 방향/우선순위/결정 로그
├── docs/
│   ├── SPEC.md            # Rust 2-crate 기술 명세
│   └── reference.md       # KIS API 레퍼런스 (엔드포인트, TR ID, 거래소 코드 등)
└── rust/                  # Rust workspace
```

## Local Test Binary

- Final local test install path: `~/.local/bin/kis`
- Release build artifact source: `rust/target/release/kis`

## Key References

- `docs/SPEC.md` - 현재 Rust 2-crate 아키텍처 명세.
  - `kis-core` / `kis-cli` 책임 분리
  - 공개 모듈 경로와 의존 방향
  - 새 기능 추가 시 배치 기준
- `docs/reference.md` - API 레퍼런스 문서. 작업 전 반드시 참조할 것.
  - 섹션 1-3: 도메인, 인증 흐름, 공통 헤더/응답 구조
  - 섹션 4: 카테고리별 엔드포인트 (국내주식 156, 해외주식 50, 채권 18, 선물옵션 78, ETF/ETN 6, ELW 24)
  - 섹션 5-6: 거래소 코드, TR ID 매핑 테이블
  - 섹션 7: 설정 파일 구조
- 원본 소스: https://github.com/koreainvestment/open-trading-api
- API 포털: https://apiportal.koreainvestment.com

## Architecture Decisions

### API 환경

두 가지 환경이 존재하며, 설정으로 전환한다:
- **실전투자**: `https://openapi.koreainvestment.com:9443`
- **모의투자**: `https://openapivts.koreainvestment.com:29443`

TR ID 접두어로 환경을 구분한다: 실전 `T`/`F`/`H`, 모의 `V`.

### 인증

- OAuth2 토큰 기반 (`/oauth2/tokenP`)
- 토큰 유효기간 24시간, 로컬 캐싱 필요 (`~/.KIS/config/`)
- 모든 REST 요청에 `authorization`, `appkey`, `appsecret`, `tr_id` 헤더 필수

### Rate Limiting

- 실전: 0.05초 간격
- 모의: 0.5초 간격
- WebSocket: 최대 40개 동시 구독

## Coding Conventions

- API 응답의 모든 필드명은 대문자 스네이크 케이스 (예: `CANO`, `ACNT_PRDT_CD`, `ORD_DVSN`)
- 에러 처리: 응답의 `rt_cd` 필드가 `"0"`이면 성공, 그 외 실패
- 시크릿 정보 (앱키, 시크릿키, 토큰)는 절대 로그에 출력하지 않는다

## Important Patterns

### 해외주식 주문 TR ID 분기

해외주식 주문은 거래소별로 TR ID가 다르다. `docs/reference.md` 섹션 6의 매핑 테이블을 반드시 참조할 것.
예: 미국 매수 `TTTT1002U`, 홍콩 매수 `TTTS1002U`, 일본 매수 `TTTS0308U`

### 거래소 코드 이중 체계

주문용 코드(`NASD`, `NYSE` 등)와 시세용 코드(`NAS`, `NYS` 등)가 다르다. 혼용 주의.

### 연속조회 (Pagination)

- 요청 헤더 `tr_cont`: `""` (최초), `"N"` (다음)
- 응답 헤더 `tr_cont`로 다음 페이지 존재 여부 확인
- 최대 재귀 깊이 10회 권장

## Security

- `.env`, `kis_devlp.yaml` 등 인증 정보 파일은 `.gitignore`에 반드시 포함
- 앱키/시크릿키를 코드에 하드코딩하지 않는다
- 토큰은 파일 시스템에 캐싱하되, 권한을 `0600`으로 설정한다
