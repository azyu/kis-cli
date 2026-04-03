# Steering

## Current Direction

저장소는 `kis-core` + `kis-cli` 2-crate Rust workspace를 공식 구현으로 유지한다. 현재 공식 CLI 진입점은 `kis`이며 `kis-core`가 config/auth/client/ws와 국내/해외 도메인 API를 함께 소유한다. 지원 표면은 국내 price/order/balance/read APIs, 국내 시간외 REST, 해외 현재가/기간시세/호가/체결/주문/잔고/예약주문/예약취소/기간손익/기간거래/매수가능금액, WebSocket approval/정규장·시간외 실시간 시세, config 출력을 포함한다. 검증은 `cargo test --manifest-path rust/Cargo.toml` 기준으로 유지한다.

Go reference 구현과 관련 운영 문서를 제거해 저장소 기준을 Rust-only로 정리했다. 설정 파일 기본 위치는 `~/.config/kis/config.yaml`로 유지하고, 기존 `~/.kis/config.yaml` fallback은 제공하지 않는다. 토큰 캐시 경로는 별도 마일스톤 전까지 유지한다. 현재 crate/module 경계의 기술 기준 문서는 `docs/SPEC.md`다.

## Priorities

1. E2E 통합 테스트 (모의투자) 및 live smoke 기준 정리
2. `ratatui` 필요성 재평가 (`kis tui`는 후속)
3. WebSocket 표면 확대 여부 재평가 (정규장 시세/체결, 다중 구독 UX)
4. agent-friendly CLI 계약 후속 정리 (`--output`, JSON envelope, `--quiet`, 주문 `--dry-run` 이후 문서 동기화)
5. 태그 push 기반 GitHub Release/Homebrew tap 자동화 정착

## Agent Assignment

| Agent | 담당 영역 | 현재 상태 |
|-------|----------|----------|
| Core | `rust/kis-core/src/{auth,client,config,error,ws}.rs`, `rust/Cargo.toml` | Rust-only 기준 유지 |
| Domain | `rust/kis-core/src/api_client.rs`, `rust/kis-core/src/{domestic,overseas}/` | 다음 API 확장 대기 |
| CLI | `rust/kis-cli/` | `kis` 표면 유지 및 문서 정리 |
| Quality | `rust/*/tests`, Rust 테스트 모듈, CI | Go 제거 후 회귀 검증 |

## Coordination Rules

- 한 에이전트가 여러 역할을 겸할 수 있으나, 작업 시작 시 TASKS.md에 진행 상태를 기록한다
- 동일 파일을 두 에이전트가 동시에 수정하지 않는다
- 의존성이 있는 작업은 순서를 지킨다: Core -> Domain -> CLI -> Quality
- 작업 완료 기준과 후속 Git/PR 절차는 `AGENTS.md`의 DoD 규칙을 따른다
- Rust CLI는 `kis`를 공식 바이너리 이름으로 사용한다
- Rust-only 저장소 기준을 유지하고, 삭제된 Go 경로를 다시 참조하지 않는다
- 주문용 거래소 코드(`NASD`, `NYSE` 등)와 시세용 거래소 코드(`NAS`, `NYS` 등)를 동일 타입으로 섞지 않는다
- pagination이 필요한 해외 잔고/체결 API는 `tr_cont`/`CTX_AREA_*` 처리를 공통화한 뒤 붙인다
- WebSocket은 REST 마일스톤과 분리하고, `/oauth2/Approval` 발급을 Core 선행 작업으로 둔다
- 리뷰에서 확정된 결함은 새 기능보다 우선해서 재현 테스트를 추가한 뒤 수정한다
- 2026-03-07 현재 배치는 `rust/kis-core/src/{domestic,overseas}/*` 도메인 API, `rust/kis-core/src/{auth,client,config,error,ws}.rs` 공통 인프라, `rust/kis-cli/src/*` CLI/runtime/test 확장 순서로 진행한다
- 개발 체크 명령은 가능하면 `Makefile` 표준 진입점(`fmt`, `fmt-check`, `lint`, `test`, `hooks-install`)에 수렴시키고, README/CI/훅은 동일 명령을 재사용한다
- PR CI는 포맷/린트/테스트와 별개로 release `kis` 빌드 검증을 유지한다
- 2026-03-08 이후 남은 기능 구현은 오케스트레이터가 범위를 자르고 별도 리더 에이전트에 단계별로 위임한다. 각 단계 완료 후 다음 단계에 필요한 내용만 compact summary로 넘긴다

## Blockers

- 모의투자 `inquire-psbl-sell` 호출은 2026-03-06 실측 기준 `OPSQ0002` (`없는 서비스 코드 입니다`)를 반환한다. 구현은 KIS 오류를 그대로 surface 하며, 모의투자 지원 여부는 후속 확인이 필요하다.
- 모의투자 도메인에서 일부 읽기 API는 실측 기준 지원되지 않는다. 국내 `quote ask`는 `404 Not Found`, `market holiday`/`info search`는 `EGW2004` (`모의투자 TR 이 아닙니다.`), `info news`/`info opinion`은 `OPSQ0002` (`없는 서비스 코드 입니다`)를 반환했다.
- 모의투자 해외 시세/정보도 일부 읽기 API가 막힌다. `info detail --exchange NAS`는 실측 기준 `/uapi/overseas-price/v1/quotations/search-info` 호출 실패를 반환했다. live evidence 없이 endpoint/TR ID를 바꾸지 않고 blocker로 유지한다.
- 해외 주문/잔고 계열은 Rust 구현 기준으로도 payload shape와 모의투자 지원 여부를 단계별로 검증해야 한다.
- 미국 주간주문/정정취소는 공식 예제 기준 실전 TR ID만 확인된다. 모의투자 TR ID는 추정하지 말고 명시적으로 거절한다.
- `overtime_ccnl_krx`는 open-trading-api 예제 기준 WebSocket 실시간 체결(TR `H0STOUP0`)만 확인된다. REST 마일스톤에 섞지 않는다.
- `overtime_asking_price_krx`도 open-trading-api 예제 기준 WebSocket 실시간 호가(TR `H0STOAA0`)만 확인된다. REST 2차 범위에 다시 넣지 않는다.

## Decisions Log

- 2026-03-08: E2E 통합 테스트(모의투자) 1차는 기본 `make test`에 넣지 않고 ignored CLI smoke harness로만 운영한다. 실행 진입점은 `make test-e2e-virtual`이며 `KIS_E2E_VIRTUAL=1`, `KIS_E2E_VIRTUAL_CONFIG`, 선택적 `KIS_E2E_VIRTUAL_STOCK`를 요구한다. 현재 smoke 대상은 `config`, `price`, `price --daily`만 포함하고 known virtual blocker인 `quote ask`, `market holiday`, `info search`, `psbl-sell`은 제외한다.
- 2026-03-08: 해외 시세/시장정보 2차 잔여분의 `ranking` 1차로 `trade-vol`과 `market-cap`만 구현했다. CLI 표면은 `kis market volume --exchange <quote-exchange>`와 `kis market cap --exchange <quote-exchange>`로 제한하고, 국내 `market volume`/`market holiday` 동작은 유지한다.
- 2026-03-08: 해외 시세/시장정보 2차 잔여분의 `ranking` 2차로 `price_fluct`, `new_highlow`, `volume_surge`를 구현했다. CLI 표면은 `kis market price-fluct|new-highlow|volume-surge --exchange <quote-exchange>`로 제한하고, 세 endpoint의 추가 필터는 기본값(`MIXN=0`, `VOL_RANG=0`, `GUBN/GUBN2` 기본 조합)으로 고정해 계약을 최소화한다.
- 2026-03-08: `ranking` 후속 후보(`price_fluct`, `new_highlow`, `volume_surge` 등)는 이번 단계에서 제외한다. 현재 우선순위는 계약이 분리된 `inquire_search`보다 ranking 나머지 endpoints를 먼저 검토하는 것이다.
- 2026-03-08: 해외 시세/시장정보 2차 잔여분의 `search/info` 1차로 `search_info`만 구현했다. CLI 표면은 국내 `info search` 의미를 유지하기 위해 별도 `kis info detail <symbol> --exchange <quote-exchange>`로 추가했다.
- 2026-03-08: 해외 `inquire_search`는 조건검색 API라서 국내 `info search`의 키워드 검색 의미와 다르다. 다수 필터 플래그를 요구해 계약이 커지므로 이번 단계에서는 제외하고 후속 슬라이스로 남긴다.
- 2026-03-08: 해외 `inquire_search`를 별도 `kis info screener --exchange <quote-exchange> ...` 표면으로 구현했다. `info search`는 국내 키워드 검색 의미를 유지하고, screener는 API가 제공하는 8개 범위 필터만 `--*-start/--*-end` 쌍으로 노출한다.
- 2026-03-08: 해외 시세/시장정보 2차 잔여분(`chart`, `search_info`, `inquire_search`, `ranking`)은 이번 단계로 모두 완료됐다. 다음 우선순위는 해외 market-info가 아니라 E2E/live smoke와 나머지 비시세 마일스톤이다.
- 2026-03-08: 국내 시간외 REST 2차는 `market` 표면에 `kis market overtime-fluctuation`과 `kis market overtime-volume`만 추가한다. `overtime_asking_price_krx`와 `overtime_ccnl_krx`는 REST가 아니라 WebSocket 항목이므로 다시 넣지 않는다.
- 2026-03-08: 국내 시간외 REST 2차(`overtime_fluctuation`, `overtime_volume`)를 완료했다. 두 명령은 추가 필터 없이 `kis market overtime-fluctuation`과 `kis market overtime-volume`만 제공하고, screen code/시장범위/정렬·등락 구분은 공식 샘플 기본값(`20234` + 상승률, `20235` + 거래량, `0000`)으로 고정한다.
- 2026-03-08: WebSocket 표면 확대는 `정규장 호가/체결 1차`와 `다중 구독 UX`로 분리한다. 먼저 공식 실시간 TR `H0STASP0`/`H0STCNT0`를 기존 count-limited collect 모델에 얹고, 다중 구독 UX는 후속으로 남긴다.
- 2026-03-08: WebSocket 다중 구독 UX도 다시 `동일 spec 다중 종목`과 `mixed spec/mixed stream`으로 분리한다. 먼저 같은 TR에 여러 종목을 붙이는 수준까지만 구현하고, 서로 다른 stream을 한 번에 섞는 입력 surface는 후속으로 남긴다.
- 2026-03-08: `mixed spec` 단계에서도 진짜 동시 multiplex collector를 먼저 만들지 않는다. 우선은 approval key 재사용 + 기존 collector 조합으로 mixed input surface를 제공하고, 필요 시 이후에 동시 수집 모델을 검토한다.
- 2026-03-08: E2E 통합 테스트는 실제 자격증명 의존성이 있으므로 `ignored smoke harness`부터 추가한다. 기본 `make test`에는 포함하지 않고, opt-in 실행 경로와 필요한 환경 변수만 문서화한다.
- 2026-03-08: WebSocket 파이프/LLM 전달용 출력은 기존 batch 계약과 분리한다. `ws` 전용 `--stream`은 성공 출력만 NDJSON으로 즉시 flush하고, 기존 batch `text/json` 계약은 유지한다. `--stream`과 전역 JSON 모드는 섞지 않는다.
- 2026-03-24: WebSocket NDJSON 스트리밍 출력 1차를 완료했다. `kis ws ask|ccnl|overtime-ask|overtime-ccnl|collect --stream`이 row 단위 NDJSON을 stdout으로 출력하고 각 라인마다 flush한다. `--stream`은 `--json`/`--output json`과 함께 쓰지 못하며, 기존 batch text/json 출력 계약은 유지한다.
- 2026-03-24: 국내 `chart daily`는 `output` 단일 payload와 `output1`/`output2` payload를 모두 수용한다. `output1` summary는 무시하고 `output2` rows만 파싱한다.
- 2026-03-24: opt-in virtual smoke 범위를 `config`, `price`, `price --daily`, `chart daily` 성공 경로와 `quote ask`/`market holiday`/`info search` 구조화 실패 경로까지 확장했다.
- 2026-04-03: virtual live smoke에서 `info news`/`info opinion`의 `OPSQ0002`, `info detail --exchange NAS`의 해외 `search-info` 호출 실패를 재현했다. 이들 명령은 fallback 없이 KIS 응답/실패를 그대로 surface하고 opt-in blocker smoke에 포함한다.
- 2026-03-08: WebSocket 표면 확대 1차를 완료했다. `kis ws ask <symbol>`와 `kis ws ccnl <symbol>`를 추가하고, 기존 `kis ws overtime-ask|overtime-ccnl`의 approval-key + count-limited collect + 기본 재연결 모델을 그대로 재사용한다. 다중 구독 UX는 이번 단계에 넣지 않는다.
- 2026-03-08: WebSocket 다중 구독 UX 1차를 완료했다. `kis ws ask|ccnl|overtime-ask|overtime-ccnl <symbol>...`가 같은 spec 안에서 여러 종목을 순차 수집할 수 있고, approval key는 한 번만 발급해 종목별 수집에 재사용한다. `--count`는 종목별 메시지 개수로 해석한다.
- 2026-03-08: WebSocket 다중 구독 UX 2차를 완료했다. 전용 `kis ws collect KIND:SYMBOL...` surface로 `ask|ccnl|overtime-ask|overtime-ccnl`를 섞어 요청할 수 있고, 수집은 요청별 순차 처리지만 approval key는 1회만 발급해 재사용한다. `--count`는 요청별 메시지 개수다.
- 2026-03-08: WebSocket 실시간 시세 표면 확대 backlog는 이번 단계로 닫는다. 후속으로 동시 multiplex 모델을 검토하더라도 별도 성능/UX 마일스톤으로 취급한다.
- 2026-03-08: 다음 `ranking` 슬라이스는 기존 `market` 표면에 자연스럽게 들어가는 최소 범위로 제한한다. 우선 후보는 `trade-vol`과 `market-cap`이며, `price_fluct`/`new_highlow`/`volume_surge`는 후속으로 남긴다.
- 2026-03-08: 해외 시세/시장정보 2차 잔여분의 `chart` 슬라이스를 완료했다. `kis chart daily|time ... --exchange <quote-exchange>`가 해외 종목 차트로 라우팅된다. 이번 단계는 해외 종목 일별/분별 차트만 포함하며, 해외 시간차트는 기본 1페이지(`NREC=120`) 조회로 제한한다.
- 2026-03-08: 남은 해외 시세/시장정보 2차 잔여분은 `chart -> search/info -> ranking` 순서의 단계형 슬라이스로 진행한다. `chart` 계열을 완료했고 다음 단계는 `search/info`다. 리더 에이전트가 구현을 소유하고 오케스트레이터가 범위/검증/인수인계를 관리한다.
- 2026-03-08: 작업 컨텍스트 문서 canonical 경로를 `.context/TASKS.md`, `.context/STEERING.md`로 이동한다. 기존 `.claude/`는 협업 문서 기준 경로로 사용하지 않는다.
- 2026-03-08: 저장소 작업 완료 기준은 `AGENTS.md`의 DoD를 따른다. 작업은 `main`에서 분기한 브랜치에서 시작하고, 논리적 단계별 커밋 후 원격 push와 PR 작성까지를 기본 후속 절차로 명시한다.
- 2026-03-08: 개발 체크 표준화 마일스톤에서는 `rust-toolchain.toml`로 Rust 채널과 `clippy`/`rustfmt` 컴포넌트를 고정하고, 로컬/CI/훅은 `Makefile`을 공통 진입점으로 사용한다. 다만 PR CI의 release build 검증은 별도 step으로 유지한다.
- 2026-03-08: repo hooks는 `.githooks/`에 커밋하고 `make hooks-install`로 `core.hooksPath`를 설정한다. 문서에는 POSIX shell + `make` 기준 워크플로우(macOS/Linux, Windows는 Git Bash/WSL)를 명시한다.
- 2026-03-07: `release-build.yml`은 5개 자산(`linux amd64`, `linux arm64`, `macOS arm64`, `Windows x64`, `Windows arm64`)을 matrix로 빌드한다. Linux/macOS는 `tar.gz`, Windows는 `zip`으로 패키징하고, release job은 두 형식 모두에 대한 `checksums.txt`를 생성한다.
- 2026-04-03: 릴리즈 워크플로우는 `v*.*.*` 태그 push 또는 수동 실행을 모두 지원한다. 태그 릴리즈 후 `HOMEBREW_TAP_TOKEN`이 설정되어 있으면 `azyu/homebrew-tap`의 `kis.rb` formula를 release asset checksum 기준으로 자동 갱신한다. 초기 formula 검증은 `kis --help` 기반으로 유지한다.
- 2026-03-07: GitHub Actions는 `bb-cli`와 같은 최소 패턴을 유지하되, `release-build.yml`은 수동 `release_tag` 입력으로 platform matrix 자산과 `checksums.txt`를 GitHub Release에 업로드한다. Homebrew tap/formula 자동화는 이 마일스톤에 포함하지 않는다.
- 2026-03-07: Rust workspace를 `kis-core` + `kis-cli` 2-crate 구조로 단순화하고, 기존 `kis-api` crate는 `kis-core` 내부 모듈(`api_client`, `domestic`, `overseas`)로 흡수했다. 현재 구조 기준 문서는 `docs/SPEC.md`로 유지한다.
- 2026-03-07: `kis order reserve-cancel`은 검증된 TR ID가 있는 미국 예약취소만 지원한다. `balance reserve-orders`는 기존대로 `us|asia` 조회를 유지하고, Asia 예약취소는 TR ID가 확인되기 전까지 CLI에서 노출하지 않는다.
- 2026-03-07: 기본 설정 파일 경로를 `~/.config/kis/config.yaml`로 전환하고, 기존 `~/.kis/config.yaml` fallback은 두지 않는다. 이번 변경 범위는 설정 파일 경로에 한정하고 토큰 캐시는 유지한다.
- 2026-03-07: README는 공개 사용자 문서 기준으로 Rust CLI(`kis`)만 설명하고, Go reference 관련 내용은 제거한다.
- 2026-03-07: GitHub 공개 준비를 위해 기존 히스토리를 단일 루트 커밋으로 재작성한 뒤 `origin`에 초기 push 하기로 결정했다.
- 2026-03-05: AGENTS.md + CLAUDE.md(symlink), .claude/TASKS.md, .claude/STEERING.md 체계 확립
- 2026-03-05: 6개 skill 설치 (golang-architect, golang-cli-cobra-viper, go-testing-code-review, api-design, api-security-hardening, conventional-commit)
- 2026-03-05: Go 프로젝트 초기화 완료 - Go module/Cobra CLI(root/price/order/balance/config), Viper config(~/.kis/config.yaml), .gitignore를 구성했다.
- 2026-03-05: Domain 에이전트 - client.APIClient 인터페이스 정의, 국내주식 현재가/일별시세 API 구현, 해외주식 현재가 API 구현, price CLI 명령어 연동 (stubClient 사용)
- 2026-03-05: Core 에이전트 - OAuth TokenManager 구현 (토큰 발급/캐싱/만료관리), KISClient 구현 (공통헤더/Rate Limiting/Hashkey), stubClient 교체 완료
- 2026-03-06: Rust 병행 마이그레이션 시작 - 동일 저장소 내 `rust/` workspace 추가, 1차 범위는 `clap` 기반 CLI parity에 한정
- 2026-03-06: `ratatui`는 설치만 완료하고 1차 구현에서는 제외, 필요 시 `kis tui` 서브커맨드로 후속 도입
- 2026-03-06: Rust 1차 구현 완료 - `kis-rs` 바이너리, `kis-core`/`kis-api`/`kis-cli` workspace, Go 회귀 테스트 및 Rust 테스트 통과
- 2026-03-06: 실측 NAS AAPL 시세 payload는 `name/open/high/low` 없이 `rsym/base/last/diff/rate/tvol/tamt/ordy`만 포함될 수 있어, 해외 시세 파서는 `rsym`과 `-` 기본값으로 보정한다
- 2026-03-06: 실측 국내 현재가 payload는 `hts_kor_isnm` 없이 반환될 수 있어, 국내 현재가 파서는 종목코드 fallback을 사용한다
- 2026-03-06: 실측 일별체결 payload는 `output1`/`output2` 구조를 사용하므로, Rust 구현은 `output1` 목록만 파싱하고 `output2` 요약은 무시한다
- 2026-03-06: 모의투자 `inquire-psbl-sell`은 실측 기준 `rt_cd=1`, `msg_cd=OPSQ0002`를 반환해 성공 payload 대신 KIS 오류를 surface 한다
- 2026-03-06: `kis-rs`는 전역 `--json` 플래그로 성공 응답을 구조화된 JSON으로 출력하며, 에러 출력은 기존 텍스트 경로를 유지한다
- 2026-03-06: 협업 문서의 에이전트/스킬 설명은 Rust 우선 기준으로 정리하고, Go 관련 skill은 legacy reference 용도로만 남긴다
- 2026-03-06: Rust 국내 읽기 API parity 단계에서는 `quote/chart/market/finance/info`만 추가하고, 실 API 검증은 읽기 전용 endpoint에 한정한다
- 2026-03-06: Rust 국내 읽기 API parity를 완료했다. `kis-rs`는 `quote/chart/market/finance/info`를 포함해 Go CLI의 국내 읽기 명령 표면을 모두 지원한다
- 2026-03-06: 실측 재무 API payload는 Go reference보다 최신 shape를 사용할 수 있다. `balance-sheet`/`income-statement`/`financial-ratio`는 `stac_yymm` 기반이며, 재무비율은 `roe_val`, `bsop_prfi_inrt`, `ntin_inrt`, `grs` 키를 alias로 파싱한다
- 2026-03-06: 다음 마일스톤은 해외 API 확장으로 전환한다. 권장 순서는 `선행 기반 -> 해외 주문 1차 -> 해외 잔고/체결 1차 -> 해외 주문 2차 -> 시간외 REST -> WebSocket`이다.
- 2026-03-06: 해외 API에서는 주문용 거래소 코드와 시세용 거래소 코드를 분리하고, 주문 TR ID는 거래소/실전/모의 환경에 따라 resolver로 결정한다.
- 2026-03-06: `kis-core`/`kis-api`는 `tr_cont`를 읽을 수 있는 공통 응답 타입을 추가했다. 기존 body-only 호출 경로는 유지하고, pagination이 필요한 해외 잔고/체결 구현은 새 응답 타입을 사용한다.
- 2026-03-06: `kis-rs order`는 `--exchange` 플래그가 있으면 해외 주문으로 라우팅한다. 해외 주문은 주문용 거래소 코드만 받으며, `--market`은 지원하지 않고 `--price`를 명시적으로 요구한다.
- 2026-03-06: 해외 주문 1차(`order`, `order-rvsecncl`)를 Rust domain/CLI에 추가했고, `cargo test --manifest-path rust/Cargo.toml` 기준으로 전체 workspace 검증을 통과했다.
- 2026-03-06: 해외 잔고/체결 1차는 `rust/kis-api/src/overseas/balance.rs`로 묶었다. `inquire_balance`, `inquire_present_balance`, `inquire_paymt_stdr_balance`, `inquire_ccnl`, `inquire_nccs`를 추가했고, 필요한 필드만 partial struct로 파싱한다.
- 2026-03-06: pagination이 필요한 해외 잔고/체결 API는 응답 `tr_cont`뿐 아니라 다음 요청 헤더 `tr_cont: N`도 전송하도록 `get_json_response_with_tr_cont` 경로를 추가했다.
- 2026-03-06: `kis-rs balance`는 해외 전용 서브커맨드 `overseas`, `present`, `settlement`, `ovrs-executions`, `open-orders`를 지원한다. 텍스트 출력은 output1 표와 output2/output3 요약으로 최소 구성한다.
- 2026-03-06: 해외 잔고/체결 1차와 CLI 확장을 완료했고, `cargo test --manifest-path rust/Cargo.toml` 기준으로 전체 workspace 검증을 통과했다.
- 2026-03-06: 해외 주문 2차로 `order-resv`, `daytime-order`, `daytime-order-rvsecncl`를 추가했다. 예약주문 응답은 일반 주문과 다른 shape(`ODNO`, `RSVN_ORD_RCIT_DT`, `OVRS_RSVN_ODNO`)를 별도 타입으로 파싱한다.
- 2026-03-06: `kis-rs order`는 해외 주문에 한해 `--reserve`와 `--daytime` 모드를 지원한다. 두 플래그는 동시에 금지하고, `--daytime`은 미국 거래소(`NASD`, `NYSE`, `AMEX`) + 실전 환경에서만 허용한다.
- 2026-03-06: 해외 주문 2차까지 포함해 `cargo test --manifest-path rust/Cargo.toml` 기준으로 전체 workspace 검증을 통과했다.
- 2026-03-06: 남은 API audit 결과, 다음 우선순위는 국내 시간외 REST 1차와 해외 `inquire_psamount`다. `overtime_ccnl_krx`는 REST가 아니라 WebSocket 구독 항목으로 재분류했다.
- 2026-03-07: 코드 리뷰 확정 이슈(`rate_limit`, token cache failure handling, explicit config path validation, overseas balance pagination, Korean display width/render smoke`)를 기능 확장보다 먼저 정리한다.
- 2026-03-07: 리뷰 반영 안정화를 완료했다. `kis-core`는 concurrent-safe rate limiting, best-effort token cache, explicit config path fail-fast를 적용했고, `kis-api`는 pagination-aware wrapper dispatch와 해외 현재/결제잔고 `CTX_AREA_*` 연속조회를 보정했으며, `kis-cli`는 `dirs::home_dir()` fallback, `unicode-width` 기반 렌더링, 실제 바이너리 smoke test를 추가했다.
- 2026-03-07: Go 제거 audit 결과, Rust CLI/API는 현재 Go 사용자 표면을 대체했지만 저장소 운영 문서와 설치 경로가 아직 Go reference 유지 전제를 갖고 있어 `cmd/`/`internal/` 삭제는 별도 정리 마일스톤으로 분리한다.
- 2026-03-07: Rust CLI 계약 1차로 `--config`와 `--env`를 실제 global 플래그로 고정했다. 이제 `kis-rs config --config ...`와 `kis-rs price 005930 --env real` 형태를 파서/바이너리 smoke test로 보장한다.
- 2026-03-07: Rust CLI 공식 진입점을 `kis`로 승격했다. `rust/kis-cli` 바이너리 이름, clap 앱 이름, smoke test 런타임 참조를 `kis` 기준으로 전환했고 `rust/target/debug/kis` 산출물과 `cargo test --manifest-path rust/Cargo.toml -p kis-cli` 통과를 확인했다.
- 2026-03-07: Go reference 제거 마일스톤으로 `cmd/`, `internal/`, 루트 `go.mod`/`go.sum`을 삭제하고 협업 문서를 Rust-only 기준으로 정리한다.
- 2026-03-07: agent-friendly CLI 계약 1차는 별도 `agent` 서브커맨드 없이 기존 `kis` 표면을 유지한 채 `--output text|json`, 공통 JSON success/error envelope, `--quiet`, 주문 `--dry-run`을 추가하는 범위로 진행한다.
- 2026-03-07: agent-friendly CLI 계약 1차 구현을 완료했다. JSON 모드는 성공/실패 모두 `{ok, command, data|error}` envelope를 stdout으로 출력하고, 주문 계열은 `--dry-run`으로 endpoint/TR ID/request payload를 검증할 수 있으며 관련 parser/runtime/smoke test를 `cargo test --manifest-path rust/Cargo.toml -p kis-cli`로 고정했다.
- 2026-03-07: 설치된 `rust-cli` skill 문서는 base guidance를 유지하되, `kis`에서 검증된 output-mode resolution, machine-readable error contract, secret redaction, side-effecting command `--dry-run` 패턴을 선택적 일반 규칙으로 반영한다. `rust-cli-kis-style`과 reference 문서는 현재 `--output`/JSON error envelope 계약에 맞춰 동기화한다.
- 2026-03-07: 국내 시간외 REST 1차(`inquire_overtime_price`, `inquire_overtime_asking_price`)와 해외 계좌 조회 2차(`inquire_psamount`, `inquire_period_profit`, `inquire_period_trans`, `inquire_algo_ccnl`, `order_resv_list`, `order_resv_ccnl`)를 Rust domain/CLI에 추가했다.
- 2026-03-07: `kis ws` 1차로 `/oauth2/Approval` 발급과 국내 시간외 실시간 호가/체결 (`H0STOAA0`, `H0STOUP0`) 수집을 추가했다. 현재 CLI는 count-limited collect + 기본 재연결 + best-effort unsubscribe 모델을 사용한다.
- 2026-03-08: 해외 시세/시장정보 2차는 한 번에 `chart/search/ranking`까지 넓히지 않고 1차 슬라이스(`dailyprice`, `inquire-asking-price`, `inquire-ccnl`)로 분할한다. 이번 단계는 기존 `price`/`quote` 표면만 확장하고, 해외 차트/검색/랭킹은 후속 태스크로 유지한다.
- 2026-03-08: 해외 시세/시장정보 2차 1차 슬라이스를 완료했다. `kis price --exchange ... --daily`, `kis quote ask --exchange ...`, `kis quote ccnl --exchange ...`가 Rust domain/CLI에 추가됐고, 검증은 `cargo test --manifest-path rust/Cargo.toml -p kis-core` 및 `-p kis-cli` 기준으로 통과했다.
