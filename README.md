# kis-cli

한국투자증권(KIS) Open API용 Rust CLI입니다. 공식 바이너리 이름은 `kis`이며, 터미널에서 국내/해외 주식 시세 조회, 주문, 잔고 확인, 재무/기업 정보 조회를 수행할 수 있습니다.

## 설치

Rust toolchain이 준비되어 있다면 release 바이너리를 바로 빌드할 수 있습니다.

```bash
cargo build --manifest-path rust/Cargo.toml -p kis-cli --release --bin kis
install -m 755 rust/target/release/kis ~/.local/bin/kis
kis --help
```

설치 없이 바로 실행하려면:

```bash
cargo run --manifest-path rust/Cargo.toml -p kis-cli --bin kis -- config
```

## 설정

### 1. KIS Open API 앱키 발급

[한국투자증권 API 포털](https://apiportal.koreainvestment.com)에서 앱키와 시크릿키를 발급받습니다.

### 2. 설정 파일 작성

```bash
mkdir -p ~/.config/kis
```

`~/.config/kis/config.yaml`:

```yaml
app_key: "발급받은 앱키"
app_secret: "발급받은 시크릿키"
account_no: "12345678"
account_prod: "01"
environment: "virtual"
```

필드 설명:

- `app_key`: KIS Open API 앱키
- `app_secret`: KIS Open API 시크릿키
- `account_no`: 계좌번호 앞 8자리
- `account_prod`: 계좌상품코드 뒤 2자리, 일반적으로 `01`
- `environment`: `virtual`(모의투자) 또는 `real`(실전투자)

### 3. 환경변수 대안

```bash
export KIS_APP_KEY="앱키"
export KIS_APP_SECRET="시크릿키"
export KIS_ACCOUNT_NO="12345678"
export KIS_ACCOUNT_PROD="01"
export KIS_ENVIRONMENT="virtual"
```

### 4. 설정 확인

```bash
kis config
kis --config ~/.config/kis/config.yaml config
```

## 사용 예시

### 시세 조회

```bash
kis price 005930
kis price 005930 --daily
kis price --exchange NAS AAPL
kis price --exchange NYS MSFT
```

### 시세 상세 / 차트

```bash
kis quote ask 005930
kis quote overtime-price 005930
kis quote overtime-ask 005930
kis quote ccnl 005930
kis chart daily 005930 --start 20260101 --end 20260306
kis chart time 005930 --unit 5
kis chart index-price 0001
```

### 주문

```bash
# 국내 주문
kis order buy --stock 005930 --qty 1 --price 70000
kis order sell --stock 005930 --qty 1 --market
kis order modify --order-no 0000123456 --price 71000
kis order cancel --order-no 0000123456

# 해외 주문
kis order buy --exchange NASD --stock AAPL --qty 1 --price 200
kis order buy --exchange NASD --stock AAPL --qty 1 --price 200 --reserve
kis order buy --exchange NASD --stock AAPL --qty 1 --price 200 --daytime --env real
kis order reserve-cancel --region us --receipt-date 20260307 --reservation-order-no 0030008244
```

### 잔고 / 체결

```bash
kis balance
kis balance psbl-buy 005930 --price 70000
kis balance psbl-buy QQQ --exchange NASD --price 1.4
kis balance psbl-sell 005930
kis balance executions --start 20260301 --end 20260306

kis balance overseas --exchange NASD --currency USD
kis balance present --currency-type 02 --country 000 --market 00 --inquiry 00
kis balance settlement --date 20260307 --currency-type 01 --inquiry 00
kis balance ovrs-executions --start 20260301 --end 20260306 --exchange NASD
kis balance open-orders --exchange NASD
kis balance period-profit --exchange NASD --currency USD --start 20260301 --end 20260307
kis balance period-trans --exchange NAS --start 20260301 --end 20260307
kis balance algo-executions --date 20260307
kis balance reserve-orders --region us --start 20260301 --end 20260307 --exchange NASD
```

### WebSocket

```bash
kis ws approval
kis ws overtime-ask 005930 --count 1
kis ws overtime-ccnl 005930 --count 3
```

### 재무 / 기업정보 / 시장현황

```bash
kis finance bs 005930
kis finance is 005930
kis finance ratio 005930 --div 1

kis info dividend 005930
kis info news 005930
kis info opinion 005930
kis info search 삼성

kis market volume
kis market holiday 20260306
```

### JSON 출력

```bash
kis --json config
kis --output json config
kis --json price 005930
kis --json balance overseas --exchange NASD --currency USD
```

JSON 모드에서는 성공/실패 모두 공통 envelope를 사용합니다.

```json
{
  "ok": true,
  "command": "price",
  "data": {
    "...": "..."
  }
}
```

실패 시에는 `error.kind`가 `validation`, `api`, `config`, `runtime` 중 하나로 출력됩니다.

### 자동화 / 에이전트 사용

```bash
kis --output json config
kis order buy --stock 005930 --qty 1 --market --dry-run --output json
kis config --quiet
```

- 자동화에서는 `--output json`을 권장합니다.
- JSON 모드에서는 성공/실패 모두 stdout으로만 출력됩니다.
- 주문 자동화 전에는 `--dry-run`으로 endpoint, TR ID, 요청 payload를 먼저 확인할 수 있습니다.
- `--quiet`은 text 모드에서만 추가 문구를 제거합니다.

## 지원 표면

- `price`: 국내 현재가/일별시세, 해외 현재가
- `quote`: 호가, 시간외 현재가/호가, 체결, 투자자, 회원사
- `chart`: 일별 차트, 분별 차트, 지수 차트, 지수 현재가
- `order`: 국내 매수/매도/정정/취소, 해외 매수/매도/정정/취소, 해외 예약주문, 예약취소, 미국 주간주문/정정/취소
- `balance`: 국내 잔고/매수가능/매도가능/일별체결, 해외 잔고/체결기준현재잔고/결제기준잔고/주문체결/미체결, 매수가능금액, 기간손익/기간거래, 지정가체결, 예약주문 조회
- `market`: 거래량 순위, 휴장일
- `finance`: 재무상태표, 손익계산서, 재무비율
- `info`: 배당정보, 뉴스, 투자의견, 종목검색
- `ws`: approval key 발급, 국내 시간외 실시간 호가/체결
- `config`: 현재 설정 출력

모의투자(`virtual`)에서는 일부 국내 읽기 API가 KIS 측 제한으로 `404` 또는 `EGW2004`를 반환할 수 있습니다.

## 글로벌 플래그

| 플래그 | 설명 |
|--------|------|
| `--config` | 설정 파일 경로 (기본: `~/.config/kis/config.yaml`) |
| `--env` | 환경 전환: `real` 또는 `virtual` |
| `--output` | 출력 형식: `text` 또는 `json` |
| `--json` | `--output json` alias |
| `--quiet` | text 모드에서 추가 문구 억제 |

## 거래소 코드 주의

- 시세 조회(`price`)는 시세용 거래소 코드(`NAS`, `NYS`, `AMS`, `TSE`, `HKS`, `SHS`, `SZS`, `HSX`, `HNX`)를 사용합니다.
- 해외 주문/잔고는 주문용 거래소 코드(`NASD`, `NYSE`, `AMEX`, `SEHK`, `SHAA`, `SZAA`, `TKSE`, `HASE`, `VNSE`)를 사용합니다.
- `--daytime`은 미국 거래소(`NASD`, `NYSE`, `AMEX`) + `real` 환경에서만 허용됩니다.

## 프로젝트 구조

```text
kis-cli/
├── rust/
│   ├── kis-core/         # 설정, 인증, 공통 HTTP 클라이언트
│   ├── kis-api/          # 국내/해외 도메인 API
│   └── kis-cli/          # clap 기반 CLI 엔트리포인트
└── docs/
    └── reference.md      # KIS API 레퍼런스
```

## 보안

- 토큰 캐시 파일 권한은 `0600`으로 관리합니다.
- 앱키와 시크릿키는 로그에 출력하지 않습니다.
- 인증 파일은 `.gitignore`에 포함해야 합니다.

## 테스트

```bash
cargo test --manifest-path rust/Cargo.toml
cargo test --manifest-path rust/Cargo.toml -p kis-cli
```

- `kis-core`: 설정, 인증, HTTP 클라이언트
- `kis-api`: 국내/해외 도메인 API 파싱 및 요청 경로
- `kis-cli`: `clap` 파서, 렌더링, 실행 경로 smoke test

## 라이선스

MIT
