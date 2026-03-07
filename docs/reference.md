# KIS Open API Reference

한국투자증권 Open Trading API 레퍼런스 문서.
소스: [koreainvestment/open-trading-api](https://github.com/koreainvestment/open-trading-api)

---

## 1. API 개요

### 도메인

| 환경 | 프로토콜 | URL |
|------|----------|-----|
| 실전투자 (REST) | HTTPS | `https://openapi.koreainvestment.com:9443` |
| 실전투자 (WebSocket) | WS | `ws://ops.koreainvestment.com:21000` |
| 모의투자 (REST) | HTTPS | `https://openapivts.koreainvestment.com:29443` |
| 모의투자 (WebSocket) | WS | `ws://ops.koreainvestment.com:31000` |

### 기본 인증 흐름

1. 앱키/시크릿으로 **OAuth 토큰 발급** (`/oauth2/tokenP`)
2. 발급받은 `access_token`을 `Authorization: Bearer {token}` 헤더에 포함
3. WebSocket 사용 시 별도 **접속키 발급** (`/oauth2/Approval`)
4. 토큰 유효기간: **24시간** (86,400초)

### Rate Limiting

- 실전투자: 요청 간 **0.05초** 간격 권장
- 모의투자: 요청 간 **0.5초** 간격 권장
- WebSocket: 최대 **40개** 동시 구독

---

## 2. 인증 (OAuth)

### 2.1 토큰 발급

```
POST /oauth2/tokenP
Content-Type: application/json
```

**Request Body:**
```json
{
  "grant_type": "client_credentials",
  "appkey": "{앱키}",
  "appsecret": "{시크릿키}"
}
```

**Response:**
```json
{
  "access_token": "eyJ0eXAi...",
  "access_token_token_expired": "2024-01-01 12:00:00",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

### 2.2 WebSocket 접속키 발급

```
POST /oauth2/Approval
Content-Type: application/json
```

**Request Body:**
```json
{
  "grant_type": "client_credentials",
  "appkey": "{앱키}",
  "secretkey": "{시크릿키}"
}
```

**Response:**
```json
{
  "approval_key": "..."
}
```

### 2.3 Hashkey 발급

POST 요청 시 body 데이터의 무결성 검증을 위한 해시키.

```
POST /uapi/hashkey
Content-Type: application/json
```

---

## 3. 공통 요청/응답 구조

### 3.1 REST API 공통 헤더

```http
Content-Type: application/json
Accept: text/plain
charset: UTF-8
User-Agent: {사용자 에이전트}
authorization: Bearer {access_token}
appkey: {앱키}
appsecret: {시크릿키}
tr_id: {거래ID}
custtype: P
tr_cont: {연속조회 키}
```

| 헤더 | 설명 |
|------|------|
| `tr_id` | 거래 ID - API별 고유 식별자 (예: `FHKST01010100`) |
| `custtype` | 고객 타입 (`P`: 개인, `B`: 법인) |
| `tr_cont` | 연속조회 (`""`: 최초, `"N"`: 다음 페이지) |

### 3.2 WebSocket 헤더

```json
{
  "content-type": "utf-8",
  "approval_key": "{접속키}"
}
```

### 3.3 TR ID 체계

TR ID는 API 엔드포인트별 고유 식별자로, 실전/모의 환경에 따라 접두어가 다르다.

| 패턴 | 의미 |
|------|------|
| `T***` / `F***` / `H***` | 실전투자 |
| `V***` | 모의투자 |

예시:
- 국내주식 매수: 실전 `TTTC0012U` / 모의 `VTTC0012U`
- 해외주식 매수(미국): 실전 `TTTT1002U` / 모의 `VTTT1002U`

### 3.4 공통 응답 구조

```json
{
  "rt_cd": "0",
  "msg_cd": "MCA00000",
  "msg1": "정상처리 되었습니다.",
  "output": { ... }
}
```

| 필드 | 설명 |
|------|------|
| `rt_cd` | 응답 코드 (`"0"`: 성공) |
| `msg_cd` | 메시지 코드 |
| `msg1` | 메시지 내용 |
| `output` | 응답 데이터 (API별 상이) |

---

## 4. API 카테고리별 엔드포인트 목록

### 4.1 국내주식 (156 API)

#### 시세 조회

| API | URL Path | TR ID | Method |
|-----|----------|-------|--------|
| 주식현재가 시세 | `/uapi/domestic-stock/v1/quotations/inquire-price` | `FHKST01010100` | GET |
| 주식현재가 체결 | `/uapi/domestic-stock/v1/quotations/inquire-ccnl` | - | GET |
| 주식현재가 일별 | `/uapi/domestic-stock/v1/quotations/inquire-daily-price` | - | GET |
| 주식현재가 호가/예상체결 | `/uapi/domestic-stock/v1/quotations/inquire-asking-price` | - | GET |
| 주식현재가 투자자 | `/uapi/domestic-stock/v1/quotations/inquire-investor` | - | GET |
| 주식현재가 회원사 | `/uapi/domestic-stock/v1/quotations/inquire-member` | - | GET |
| 지수현재가 | `/uapi/domestic-stock/v1/quotations/inquire-index-price` | - | GET |
| ELW현재가 | `/uapi/domestic-stock/v1/quotations/inquire-elw-price` | - | GET |

주요 시세 조회 파라미터:

| 파라미터 | 설명 |
|----------|------|
| `FID_COND_MRKT_DIV_CODE` | 시장 구분 (`J`: KRX, `NX`: NXT, `UN`: 통합) |
| `FID_INPUT_ISCD` | 종목코드 (6자리, ETN은 Q접두어) |

#### 차트 데이터

| API | 디렉토리명 |
|-----|-----------|
| 일별 종목 차트 | `inquire_daily_itemchartprice` |
| 분별 종목 차트 | `inquire_time_itemchartprice` |
| 일별 지수 차트 | `inquire_daily_indexchartprice` |
| 분별 지수 차트 | `inquire_time_indexchartprice` |
| 시간별 일별 차트 | `inquire_time_dailychartprice` |

#### 주문/계좌

| API | URL Path | TR ID (실전) | TR ID (모의) | Method |
|-----|----------|-------------|-------------|--------|
| 주식주문(현금)-매수 | `/uapi/domestic-stock/v1/trading/order-cash` | `TTTC0012U` | `VTTC0012U` | POST |
| 주식주문(현금)-매도 | `/uapi/domestic-stock/v1/trading/order-cash` | `TTTC0011U` | `VTTC0011U` | POST |
| 주식주문(신용) | `/uapi/domestic-stock/v1/trading/order-credit` | - | - | POST |
| 주식주문(예약) | `/uapi/domestic-stock/v1/trading/order-resv` | - | - | POST |
| 주식주문 정정/취소 | `/uapi/domestic-stock/v1/trading/order-rvsecncl` | - | - | POST |

주문 요청 Body:

| 파라미터 | 설명 |
|----------|------|
| `CANO` | 계좌번호 (앞 8자리) |
| `ACNT_PRDT_CD` | 계좌상품코드 (뒤 2자리) |
| `PDNO` | 종목코드 |
| `ORD_DVSN` | 주문구분 (`00`: 지정가, `01`: 시장가 등) |
| `ORD_QTY` | 주문수량 |
| `ORD_UNPR` | 주문단가 |
| `EXCG_ID_DVSN_CD` | 거래소 구분 (`KRX`, `SOR` 등) |
| `SLL_TYPE` | 매도유형 (`01`/`02`/`05`, 매도 시) |
| `CNDT_PRIC` | 조건가 (STOP주문 시) |

#### 잔고/손익 조회

- `inquire_balance` - 잔고조회
- `inquire_account_balance` - 계좌잔고
- `inquire_balance_rlz_pl` - 실현손익
- `credit_balance` - 신용잔고
- `inquire_psbl_order` - 매수가능조회
- `inquire_psbl_sell` - 매도가능조회
- `inquire_ccnl` - 체결조회
- `inquire_daily_ccld` - 일별체결조회
- `after_hour_balance` - 시간외잔고

#### 시장 현황

- `market_status_krx` / `market_status_nxt` / `market_status_total` - 시장 현황
- `ccnl_krx` / `ccnl_nxt` / `ccnl_total` - 체결
- `asking_price_krx` / `asking_price_nxt` / `asking_price_total` - 호가
- `fluctuation` - 등락률
- `volume_rank` - 거래량순위
- `volume_power` - 체결강도
- `market_cap` - 시가총액
- `market_value` - 시장가치
- `market_time` - 시장 시간

#### 프로그램/기관 매매

- `program_trade_krx` / `program_trade_nxt` / `program_trade_total`
- `investor_program_trade_today`
- `comp_program_trade_daily` / `comp_program_trade_today`
- `foreign_institution_total`
- `inquire_investor_daily_by_market` / `inquire_investor_time_by_market`

#### 시간외 거래

- `inquire_overtime_price` - 시간외 시세
- `inquire_overtime_asking_price` - 시간외 호가
- `overtime_asking_price_krx` - 시간외 호가 (KRX)
- `overtime_ccnl_krx` - 시간외 체결
- `overtime_fluctuation` - 시간외 등락
- `overtime_volume` - 시간외 거래량

#### 재무 데이터

- `finance_balance_sheet` - 재무상태표
- `finance_income_statement` - 손익계산서
- `finance_ratio` - 재무비율
- `finance_financial_ratio` / `finance_growth_ratio` / `finance_profit_ratio` / `finance_stability_ratio` / `finance_other_major_ratios`

#### 기업 공시/정보

- `ksdinfo_dividend` - 배당
- `ksdinfo_bonus_issue` - 무상증자
- `ksdinfo_merger_split` - 합병/분할
- `ksdinfo_cap_dcrs` - 감자
- `ksdinfo_rev_split` - 액면병합
- `ksdinfo_forfeit` - 실권
- `ksdinfo_pub_offer` - 공모
- `news_title` - 뉴스
- `estimate_perform` - 실적추정
- `invest_opinion` - 투자의견
- `search_info` / `search_stock_info` - 종목검색

#### 기타

- `disparity` - 이격도
- `chk_holiday` - 휴장일 조회
- `hts_top_view` - HTS 인기종목
- `quote_balance` - 시세잔량
- `pension_inquire_balance` - 연금잔고
- `daily_credit_balance` - 일별신용잔고

### 4.2 해외주식 (50 API)

#### 시세 조회

| API | URL Path | TR ID | Method |
|-----|----------|-------|--------|
| 해외주식 현재가 | `/uapi/overseas-price/v1/quotations/price` | `HHDFS00000300` | GET |
| 해외주식 기간별시세 | `/uapi/overseas-price/v1/quotations/dailyprice` | - | GET |
| 해외주식 호가 | `/uapi/overseas-price/v1/quotations/inquire-asking-price` | - | GET |
| 해외주식 체결 | `/uapi/overseas-price/v1/quotations/inquire-ccnl` | - | GET |
| 해외주식 일별차트 | `/uapi/overseas-price/v1/quotations/inquire-daily-chartprice` | - | GET |
| 해외주식 분별차트 | `/uapi/overseas-price/v1/quotations/inquire-time-itemchartprice` | - | GET |
| 해외지수 분별차트 | `/uapi/overseas-price/v1/quotations/inquire-time-indexchartprice` | - | GET |

시세 조회 파라미터:

| 파라미터 | 설명 |
|----------|------|
| `AUTH` | 사용자 인증 |
| `EXCD` | 거래소코드 (예: `NAS`, `NYS`, `AMS`) |
| `SYMB` | 종목 심볼 (예: `AAPL`) |

#### 주문

| API | URL Path | Method |
|-----|----------|--------|
| 해외주식 주문 | `/uapi/overseas-stock/v1/trading/order` | POST |
| 해외주식 정정/취소 | `/uapi/overseas-stock/v1/trading/order-rvsecncl` | POST |
| 해외주식 예약주문 | `/uapi/overseas-stock/v1/trading/order-resv` | POST |
| 해외주식 주간주문 | `/uapi/overseas-stock/v1/trading/daytime-order` | POST |
| 해외주식 주간정정/취소 | `/uapi/overseas-stock/v1/trading/daytime-order-rvsecncl` | POST |

해외주식 주문 TR ID (거래소별):

| 거래소 | 코드 | 매수 (실전/모의) | 매도 (실전/모의) |
|--------|------|-----------------|-----------------|
| 나스닥 | NASD | `TTTT1002U` / `VTTT1002U` | `TTTT1006U` / `VTTT1006U` |
| 뉴욕 | NYSE | `TTTT1002U` / `VTTT1002U` | `TTTT1006U` / `VTTT1006U` |
| 아멕스 | AMEX | `TTTT1002U` / `VTTT1002U` | `TTTT1006U` / `VTTT1006U` |
| 홍콩 | SEHK | `TTTS1002U` / `VTTS1002U` | `TTTS1001U` / `VTTS1001U` |
| 상해 | SHAA | `TTTS0202U` / `VTTS0202U` | `TTTS1005U` / `VTTS1005U` |
| 심천 | SZAA | `TTTS0305U` / `VTTS0305U` | `TTTS0304U` / `VTTS0304U` |
| 도쿄 | TKSE | `TTTS0308U` / `VTTS0308U` | `TTTS0307U` / `VTTS0307U` |
| 하노이 | HASE | `TTTS0311U` / `VTTS0311U` | `TTTS0310U` / `VTTS0310U` |
| 호치민 | VNSE | `TTTS0311U` / `VTTS0311U` | `TTTS0310U` / `VTTS0310U` |

#### 잔고/계좌

- `inquire_balance` - 잔고조회
- `inquire_present_balance` - 현재잔고
- `inquire_paymt_stdr_balance` - 결제기준잔고
- `inquire_ccnl` - 체결조회
- `inquire_nccs` - 미체결조회
- `inquire_period_profit` - 기간손익
- `inquire_period_trans` - 기간거래내역
- `inquire_psamount` - 매수가능금액
- `inquire_algo_ccnl` - 알고리즘 체결

#### 시장 정보

- `inquire_search` - 종목검색
- `search_info` - 종목정보
- `market_cap` - 시가총액
- `price_detail` - 상세시세
- `price_fluct` - 등락률
- `updown_rate` - 상승/하락률
- `new_highlow` - 신고/신저
- `volume_power` / `volume_surge` - 거래량
- `trade_growth` / `trade_pbmn` / `trade_turnover` / `trade_vol` - 매매동향
- `industry_price` / `industry_theme` - 업종/테마
- `news_title` / `brknews_title` - 뉴스
- `countries_holiday` - 국가별 휴장일
- `foreign_margin` - 외국인 신용
- `colable_by_company` - 담보대출

#### 기타

- `algo_ordno` - 알고리즘 주문번호
- `order_resv_ccnl` / `order_resv_list` - 예약주문 체결/목록
- `ccnl_notice` - 체결통보
- `delayed_asking_price_asia` - 아시아 지연호가
- `delayed_ccnl` - 지연체결
- `period_rights` / `rights_by_ice` - 권리/배당
- `quot_inquire_ccnl` - 시세체결

### 4.3 국내채권 (18 API)

| API | 디렉토리명 | 설명 |
|-----|-----------|------|
| 채권시세 | `inquire_price` | 현재가 조회 |
| 채권일별시세 | `inquire_daily_price` | 일별 시세 |
| 채권호가 | `inquire_asking_price` / `bond_asking_price` | 호가 조회 |
| 채권체결 | `inquire_ccnl` / `bond_ccnl` | 체결 조회 |
| 채권지수체결 | `bond_index_ccnl` | 지수 체결 |
| 채권차트 | `inquire_daily_itemchartprice` | 일별 차트 |
| 채권매수 | `buy` | 매수 주문 |
| 채권매도 | `sell` | 매도 주문 |
| 주문정정/취소 | `order_rvsecncl` | 정정/취소 |
| 잔고조회 | `inquire_balance` | 잔고 |
| 체결조회 | `inquire_daily_ccld` | 일별체결 |
| 주문가능조회 | `inquire_psbl_order` | 주문가능 |
| 정정취소가능 | `inquire_psbl_rvsecncl` | 정정취소가능 |
| 평균단가 | `avg_unit` | 평균단가 |
| 채권정보검색 | `search_bond_info` | 종목검색 |
| 발행정보 | `issue_info` | 발행정보 |

### 4.4 국내선물옵션 (43 API)

#### 시세/실시간

- `inquire_price` - 시세조회
- `inquire_asking_price` - 호가
- `index_futures_realtime_quote` / `index_futures_realtime_conclusion` - 지수선물 실시간
- `index_option_realtime_quote` / `index_option_realtime_conclusion` - 지수옵션 실시간
- `commodity_futures_realtime_quote` / `commodity_futures_realtime_conclusion` - 상품선물 실시간
- `stock_futures_realtime_quote` / `stock_futures_realtime_conclusion` - 주식선물 실시간
- `stock_option_asking_price` / `stock_option_ccnl` - 주식옵션

#### 차트

- `inquire_daily_fuopchartprice` - 일별 차트
- `inquire_time_fuopchartprice` - 분별 차트

#### 주문/계좌

- `order` - 주문
- `order_rvsecncl` - 정정/취소
- `inquire_balance` - 잔고
- `inquire_balance_settlement_pl` / `inquire_balance_valuation_pl` - 손익
- `inquire_ccnl` / `inquire_ccnl_bstime` - 체결
- `inquire_deposit` - 예수금
- `inquire_psbl_order` - 주문가능
- `inquire_daily_amount_fee` - 일별수수료

#### 야간거래 (KRX 야간)

- `inquire_ngt_balance` / `inquire_ngt_ccnl` / `inquire_psbl_ngt_order`
- `krx_ngt_futures_asking_price` / `krx_ngt_futures_ccnl` / `krx_ngt_futures_ccnl_notice`
- `krx_ngt_option_asking_price` / `krx_ngt_option_ccnl` / `krx_ngt_option_exp_ccnl` / `krx_ngt_option_notice`
- `ngt_margin_detail`

#### 기타

- `display_board_callput` / `display_board_futures` / `display_board_option_list` / `display_board_top`
- `exp_price_trend` - 만기가격추이
- `futures_exp_ccnl` / `option_exp_ccnl` - 만기체결
- `fuopt_ccnl_notice` - 체결통보

### 4.5 해외선물옵션 (35 API)

#### 시세

- `inquire_price` - 현재가
- `inquire_asking_price` - 호가
- `opt_price` / `opt_asking_price` / `opt_detail` - 옵션시세
- `ccnl` / `daily_ccnl` / `tick_ccnl` / `weekly_ccnl` / `monthly_ccnl` - 체결
- `opt_daily_ccnl` / `opt_tick_ccnl` / `opt_weekly_ccnl` / `opt_monthly_ccnl` - 옵션체결
- `stock_detail` / `search_contract_detail` / `search_opt_detail` - 종목정보

#### 차트

- `inquire_time_futurechartprice` - 선물차트
- `inquire_time_optchartprice` - 옵션차트

#### 주문/계좌

- `order` - 주문
- `order_rvsecncl` - 정정/취소
- `inquire_ccld` / `inquire_daily_ccld` / `inquire_daily_order` / `inquire_period_ccld` / `inquire_period_trans` - 체결/거래
- `inquire_deposit` - 예수금
- `inquire_psamount` - 주문가능
- `inquire_unpd` - 미결제
- `margin_detail` - 증거금
- `investor_unpd_trend` - 투자자미결제추이
- `market_time` - 시장시간

#### 기타

- `asking_price` - 호가 (실시간)
- `ccnl_notice` / `order_notice` - 체결통보/주문통보

### 4.6 ETF/ETN (6 API)

| API | 디렉토리명 | 설명 |
|-----|-----------|------|
| ETF 시세 | `inquire_price` | 현재가 조회 |
| ETF 구성종목 | `inquire_component_stock_price` | 구성종목 시세 |
| ETF NAV 추이 | `etf_nav_trend` | NAV 추이 |
| NAV 비교 일별 | `nav_comparison_daily_trend` | NAV 비교 (일별) |
| NAV 비교 분별 | `nav_comparison_time_trend` | NAV 비교 (분별) |
| NAV 비교 추이 | `nav_comparison_trend` | NAV 비교 추이 |

### 4.7 ELW (24 API)

- `elw_asking_price` / `elw_ccnl` / `elw_exp_ccnl` - 호가/체결/만기체결
- `sensitivity` / `sensitivity_trend_ccnl` / `sensitivity_trend_daily` - 민감도
- `indicator` / `indicator_trend_ccnl` / `indicator_trend_daily` / `indicator_trend_minute` - 지표
- `volatility_trend_ccnl` / `volatility_trend_daily` / `volatility_trend_minute` / `volatility_trend_tick` - 변동성
- `lp_trade_trend` - LP매매추이
- `compare_stocks` - 비교종목
- `cond_search` - 조건검색
- `expiration_stocks` - 만기종목
- `newly_listed` - 신규상장
- `quick_change` - 급변종목
- `udrl_asset_list` / `udrl_asset_price` - 기초자산
- `updown_rate` - 등락률
- `volume_rank` - 거래량순위

---

## 5. 거래소 코드

### 국내 시장 구분 코드 (`FID_COND_MRKT_DIV_CODE`)

| 코드 | 시장 |
|------|------|
| `J` | KRX (유가증권/코스닥) |
| `NX` | NXT (넥스트) |
| `UN` | 통합 |

### 국내 거래소 구분 코드 (`EXCG_ID_DVSN_CD`)

| 코드 | 설명 |
|------|------|
| `KRX` | 한국거래소 |
| `SOR` | Smart Order Routing |

### 해외 거래소 코드 (주문용)

| 코드 | 거래소 | 국가 |
|------|--------|------|
| `NASD` | 나스닥 | 미국 |
| `NYSE` | 뉴욕증권거래소 | 미국 |
| `AMEX` | 아메리칸증권거래소 | 미국 |
| `SEHK` | 홍콩거래소 | 홍콩 |
| `SHAA` | 상해거래소 | 중국 |
| `SZAA` | 심천거래소 | 중국 |
| `TKSE` | 도쿄거래소 | 일본 |
| `HASE` | 하노이거래소 | 베트남 |
| `VNSE` | 호치민거래소 | 베트남 |

### 해외 거래소 코드 (시세용)

| 코드 | 거래소 |
|------|--------|
| `NAS` | 나스닥 |
| `NYS` | 뉴욕 |
| `AMS` | 아멕스 |
| `HKS` | 홍콩 |
| `SHS` | 상해 |
| `SZS` | 심천 |
| `TSE` | 도쿄 |
| `HNX` | 하노이 |
| `HSX` | 호치민 |

---

## 6. 주요 TR ID 매핑 테이블

### 국내주식

| 기능 | TR ID (실전) | TR ID (모의) |
|------|-------------|-------------|
| 주식 매수 (현금) | `TTTC0012U` | `VTTC0012U` |
| 주식 매도 (현금) | `TTTC0011U` | `VTTC0011U` |
| 현재가 시세 | `FHKST01010100` | `FHKST01010100` |

### 해외주식 주문

| 거래소 | 매수 (실전) | 매수 (모의) | 매도 (실전) | 매도 (모의) |
|--------|-----------|-----------|-----------|-----------|
| 미국 (NASD/NYSE/AMEX) | `TTTT1002U` | `VTTT1002U` | `TTTT1006U` | `VTTT1006U` |
| 홍콩 (SEHK) | `TTTS1002U` | `VTTS1002U` | `TTTS1001U` | `VTTS1001U` |
| 상해 (SHAA) | `TTTS0202U` | `VTTS0202U` | `TTTS1005U` | `VTTS1005U` |
| 심천 (SZAA) | `TTTS0305U` | `VTTS0305U` | `TTTS0304U` | `VTTS0304U` |
| 도쿄 (TKSE) | `TTTS0308U` | `VTTS0308U` | `TTTS0307U` | `VTTS0307U` |
| 베트남 (HASE/VNSE) | `TTTS0311U` | `VTTS0311U` | `TTTS0310U` | `VTTS0310U` |

### 해외주식 시세

| 기능 | TR ID |
|------|-------|
| 해외주식 현재가 | `HHDFS00000300` |

---

## 7. 설정 파일 구조 (`kis_devlp.yaml`)

```yaml
# API 인증 정보
my_app: ""          # 실전 앱키
my_sec: ""          # 실전 시크릿키
paper_app: ""       # 모의 앱키
paper_sec: ""       # 모의 시크릿키

# HTS ID
my_htsid: ""        # HTS 사용자 ID

# 계좌 정보 (앞 8자리)
my_acct_stock: ""   # 실전 주식 계좌
my_acct_future: ""  # 실전 선물 계좌
my_paper_stock: ""  # 모의 주식 계좌
my_paper_future: "" # 모의 선물 계좌

# 계좌 상품 코드 (뒤 2자리)
my_prod: "01"       # 01:종합, 03:선물옵션, 08:해외선물, 22:개인연금, 29:퇴직연금

# API 엔드포인트
prod: "https://openapi.koreainvestment.com:9443"
vps: "https://openapivts.koreainvestment.com:29443"
ops: "ws://ops.koreainvestment.com:21000"
vops: "ws://ops.koreainvestment.com:31000"

# 토큰 (자동 갱신)
my_token: ""

# User-Agent
my_agent: ""
```

### 계좌 상품 코드 (`my_prod`)

| 코드 | 설명 |
|------|------|
| `01` | 종합계좌 |
| `03` | 국내선물옵션 |
| `08` | 해외선물옵션 |
| `22` | 개인연금 |
| `29` | 퇴직연금 |

### 토큰 저장 경로

- 디렉토리: `~/.KIS/config/`
- 파일명: 일별 패턴 (매일 갱신)

---

## 참고 링크

- GitHub: https://github.com/koreainvestment/open-trading-api
- API 포털: https://apiportal.koreainvestment.com
- KIS Developers: https://apiportal.koreainvestment.com/apiservice
