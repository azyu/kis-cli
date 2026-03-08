use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    pub fn is_json(self) -> bool {
        matches!(self, Self::Json)
    }
}

#[derive(Debug, Parser)]
#[command(
    name = "kis",
    about = "KIS Open API CLI - 한국투자증권 터미널 도구",
    long_about = "kis는 한국투자증권(KIS) Open API를 활용한 CLI 도구입니다.\n국내/해외 주식 시세 조회, 주문, 잔고 확인 등을 터미널에서 수행할 수 있습니다."
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        help = "config file (default: ~/.config/kis/config.yaml)"
    )]
    pub config: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        help = "environment: real or virtual (overrides config)"
    )]
    pub env: Option<String>,

    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = OutputFormat::Text,
        help = "output format: text or json"
    )]
    pub output: OutputFormat,

    #[arg(long, global = true, help = "print successful command output as JSON")]
    pub json: bool,

    #[arg(
        long,
        global = true,
        help = "suppress extra text in text output (ignored for JSON output)"
    )]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            self.output
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "시세 조회 - 국내/해외 주식 현재가 조회")]
    Price(PriceArgs),
    #[command(about = "시세 상세 - 호가, 체결, 투자자, 회원사 조회")]
    Quote(QuoteArgs),
    #[command(about = "차트 데이터 - 일별/분별 차트, 지수 차트")]
    Chart(ChartArgs),
    #[command(about = "주문 - 국내/해외 주식 매수/매도 주문")]
    Order(OrderArgs),
    #[command(about = "잔고 조회 - 계좌 잔고 및 보유 종목 조회")]
    Balance(BalanceArgs),
    #[command(about = "시장 현황 - 거래량순위, 휴장일 조회")]
    Market(MarketArgs),
    #[command(about = "재무 데이터 - 재무상태표, 손익계산서, 재무비율")]
    Finance(FinanceArgs),
    #[command(about = "기업 정보 - 배당, 뉴스, 투자의견, 종목검색")]
    Info(InfoArgs),
    #[command(about = "WebSocket - 접속키 발급 및 실시간 시세")]
    Ws(WsArgs),
    #[command(about = "설정 관리 - 설정 파일 조회 및 수정")]
    Config,
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Price(_) => "price",
            Self::Quote(_) => "quote",
            Self::Chart(_) => "chart",
            Self::Order(_) => "order",
            Self::Balance(_) => "balance",
            Self::Market(_) => "market",
            Self::Finance(_) => "finance",
            Self::Info(_) => "info",
            Self::Ws(_) => "ws",
            Self::Config => "config",
        }
    }
}

#[derive(Debug, Args)]
pub struct PriceArgs {
    #[arg(help = "종목코드 또는 티커")]
    pub symbol: String,

    #[arg(
        short = 'x',
        long,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: Option<String>,

    #[arg(long, help = "국내주식 일별시세 조회")]
    pub daily: bool,

    #[arg(long, default_value = "D", help = "일별시세 기간 (D:일, W:주, M:월)")]
    pub period: String,
}

#[derive(Debug, Args)]
pub struct QuoteArgs {
    #[command(subcommand)]
    pub command: QuoteCommand,
}

#[derive(Debug, Subcommand)]
pub enum QuoteCommand {
    #[command(about = "호가 조회 (매수/매도 호가잔량)")]
    Ask(ExchangeSymbolArgs),
    #[command(about = "시간외 현재가 조회")]
    OvertimePrice(SymbolArgs),
    #[command(about = "시간외 호가 조회")]
    OvertimeAsk(SymbolArgs),
    #[command(about = "체결 조회 (최근 거래 내역)")]
    Ccnl(ExchangeSymbolArgs),
    #[command(about = "투자자별 매매동향")]
    Investor(SymbolArgs),
    #[command(about = "회원사별 매매동향")]
    Member(SymbolArgs),
}

#[derive(Debug, Args)]
pub struct ChartArgs {
    #[command(subcommand)]
    pub command: ChartCommand,
}

#[derive(Debug, Subcommand)]
pub enum ChartCommand {
    #[command(about = "일별 종목 차트 (OHLCV)")]
    Daily(ChartDailyArgs),
    #[command(about = "분별 종목 차트")]
    Time(ChartTimeArgs),
    #[command(about = "일별 지수 차트 (0001:KOSPI, 1001:KOSDAQ)")]
    Index(ChartIndexArgs),
    #[command(about = "지수 현재가 (0001:KOSPI, 1001:KOSDAQ)")]
    IndexPrice(IndexCodeArgs),
}

#[derive(Debug, Args)]
pub struct ChartDailyArgs {
    #[arg(help = "종목코드")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: Option<String>,

    #[arg(long, help = "시작일 (YYYYMMDD)")]
    pub start: Option<String>,

    #[arg(long, help = "종료일 (YYYYMMDD)")]
    pub end: Option<String>,

    #[arg(long, default_value = "D", help = "기간 (D:일, W:주, M:월)")]
    pub period: String,
}

#[derive(Debug, Args)]
pub struct ChartTimeArgs {
    #[arg(help = "종목코드")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: Option<String>,

    #[arg(long, default_value = "1", help = "분봉 단위 (1, 5, 10, 15, 30, 60)")]
    pub unit: String,
}

#[derive(Debug, Args)]
pub struct ChartIndexArgs {
    #[arg(help = "지수코드")]
    pub index: String,

    #[arg(long, help = "시작일 (YYYYMMDD)")]
    pub start: Option<String>,

    #[arg(long, help = "종료일 (YYYYMMDD)")]
    pub end: Option<String>,

    #[arg(long, default_value = "D", help = "기간 (D:일, W:주, M:월)")]
    pub period: String,
}

#[derive(Debug, Args)]
pub struct IndexCodeArgs {
    #[arg(help = "지수코드")]
    pub index: String,
}

#[derive(Debug, Args)]
pub struct OrderArgs {
    #[command(subcommand)]
    pub command: OrderCommand,
}

#[derive(Debug, Subcommand)]
pub enum OrderCommand {
    #[command(about = "매수 주문")]
    Buy(PlaceOrderArgs),
    #[command(about = "매도 주문")]
    Sell(PlaceOrderArgs),
    #[command(about = "주문 정정")]
    Modify(ModifyOrderArgs),
    #[command(about = "주문 취소")]
    Cancel(CancelOrderArgs),
    #[command(about = "해외 예약주문 취소")]
    ReserveCancel(ReserveCancelOrderArgs),
}

#[derive(Debug, Args)]
pub struct PlaceOrderArgs {
    #[arg(long, required = true, help = "종목코드 (필수)")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: Option<String>,

    #[arg(long, required = true, help = "주문수량 (필수)")]
    pub qty: String,

    #[arg(long, help = "주문단가 (지정가)")]
    pub price: Option<String>,

    #[arg(long, help = "시장가 주문")]
    pub market: bool,

    #[arg(long, conflicts_with = "daytime", help = "해외 예약주문으로 라우팅")]
    pub reserve: bool,

    #[arg(
        long,
        conflicts_with = "reserve",
        help = "해외 미국 주간주문으로 라우팅"
    )]
    pub daytime: bool,

    #[arg(
        long = "order-type",
        default_value = "00",
        help = "주문구분 (00:지정가, 01:시장가)"
    )]
    pub order_type: String,

    #[arg(long, help = "실제 주문 대신 요청 metadata만 출력")]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct ModifyOrderArgs {
    #[arg(long = "order-no", required = true, help = "원주문번호 (필수)")]
    pub order_no: String,

    #[arg(long, help = "종목코드 (해외 주문 정정 시 필수)")]
    pub stock: Option<String>,

    #[arg(
        short = 'x',
        long,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: Option<String>,

    #[arg(long = "org-no", help = "주문조직번호")]
    pub org_no: Option<String>,

    #[arg(long, default_value = "0", help = "정정수량 (0=전량)")]
    pub qty: String,

    #[arg(long, required = true, help = "정정단가 (필수)")]
    pub price: String,

    #[arg(long, help = "해외 미국 주간정정으로 라우팅")]
    pub daytime: bool,

    #[arg(long = "order-type", default_value = "00", help = "주문구분")]
    pub order_type: String,

    #[arg(long, help = "실제 주문 대신 요청 metadata만 출력")]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct CancelOrderArgs {
    #[arg(long = "order-no", required = true, help = "원주문번호 (필수)")]
    pub order_no: String,

    #[arg(long, help = "종목코드 (해외 주문 취소 시 필수)")]
    pub stock: Option<String>,

    #[arg(
        short = 'x',
        long,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: Option<String>,

    #[arg(long = "org-no", help = "주문조직번호")]
    pub org_no: Option<String>,

    #[arg(long, default_value = "0", help = "취소수량 (0=전량)")]
    pub qty: String,

    #[arg(long, help = "해외 미국 주간취소로 라우팅")]
    pub daytime: bool,

    #[arg(long, help = "실제 주문 대신 요청 metadata만 출력")]
    pub dry_run: bool,
}

#[derive(Debug, Args)]
pub struct ReserveCancelOrderArgs {
    #[arg(
        long,
        value_enum,
        default_value_t = ReservationCancelRegion::Us,
        help = "예약취소 지역 구분"
    )]
    pub region: ReservationCancelRegion,

    #[arg(
        long = "receipt-date",
        required = true,
        help = "예약주문 접수일자 (YYYYMMDD)"
    )]
    pub receipt_date: String,

    #[arg(
        long = "reservation-order-no",
        required = true,
        help = "해외 예약주문번호"
    )]
    pub reservation_order_no: String,
}

#[derive(Debug, Args)]
pub struct BalanceArgs {
    #[command(subcommand)]
    pub command: Option<BalanceCommand>,
}

#[derive(Debug, Subcommand)]
pub enum BalanceCommand {
    #[command(about = "매수가능금액 조회")]
    PsblBuy(PossibleBuyArgs),
    #[command(about = "매도가능수량 조회")]
    PsblSell(PossibleSellArgs),
    #[command(about = "해외주식 기간손익 조회")]
    PeriodProfit(OverseasPeriodProfitArgs),
    #[command(about = "해외주식 일별거래내역 조회")]
    PeriodTrans(OverseasPeriodTransArgs),
    #[command(about = "해외주식 지정가체결내역 조회")]
    AlgoExecutions(OverseasAlgoExecutionsArgs),
    #[command(about = "해외주식 예약주문 조회")]
    ReserveOrders(ReservationOrderListArgs),
    #[command(about = "일별체결내역 조회")]
    Executions(ExecutionArgs),
    #[command(about = "해외주식 잔고 조회")]
    Overseas(OverseasBalanceArgs),
    #[command(about = "해외주식 체결기준현재잔고 조회")]
    Present(PresentBalanceArgs),
    #[command(about = "해외주식 결제기준잔고 조회")]
    Settlement(SettlementBalanceArgs),
    #[command(about = "해외주식 주문체결내역 조회")]
    OvrsExecutions(OverseasExecutionArgs),
    #[command(about = "해외주식 미체결내역 조회")]
    OpenOrders(OpenOrdersArgs),
}

#[derive(Debug, Args)]
pub struct PossibleBuyArgs {
    #[arg(help = "종목코드")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: Option<String>,

    #[arg(long, default_value = "0", help = "주문단가")]
    pub price: String,

    #[arg(
        long = "order-type",
        default_value = "01",
        help = "주문구분 (00:지정가, 01:시장가)"
    )]
    pub order_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReservationRegion {
    Us,
    Asia,
}

impl ReservationRegion {
    pub fn code(self) -> &'static str {
        match self {
            Self::Us => "us",
            Self::Asia => "asia",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ReservationCancelRegion {
    Us,
}

impl ReservationCancelRegion {
    pub fn code(self) -> &'static str {
        match self {
            Self::Us => "us",
        }
    }
}

#[derive(Debug, Args)]
pub struct OverseasPeriodProfitArgs {
    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 거래소 코드 (예: NASD, SEHK, SHAA, TKSE, HASE)"
    )]
    pub exchange: String,

    #[arg(long, required = true, help = "통화코드 (USD, HKD, CNY, JPY, VND)")]
    pub currency: String,

    #[arg(long, help = "국가코드 (기본: 공란)")]
    pub country: Option<String>,

    #[arg(long, default_value = "", help = "종목코드 (기본: 전체)")]
    pub stock: String,

    #[arg(long, required = true, help = "조회시작일 (YYYYMMDD)")]
    pub start: String,

    #[arg(long, required = true, help = "조회종료일 (YYYYMMDD)")]
    pub end: String,

    #[arg(
        long = "currency-type",
        default_value = "01",
        help = "원화외화구분코드"
    )]
    pub currency_type: String,
}

#[derive(Debug, Args)]
pub struct OverseasPeriodTransArgs {
    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 거래소 코드 (예: NAS, HKS, SHS, TSE)"
    )]
    pub exchange: String,

    #[arg(long, required = true, help = "조회시작일 (YYYYMMDD)")]
    pub start: String,

    #[arg(long, required = true, help = "조회종료일 (YYYYMMDD)")]
    pub end: String,

    #[arg(long, default_value = "", help = "종목코드 (기본: 전체)")]
    pub stock: String,

    #[arg(
        long = "side",
        default_value = "00",
        help = "매도매수구분 (00:전체, 01:매도, 02:매수)"
    )]
    pub side: String,

    #[arg(long = "loan-type", default_value = "", help = "대출구분코드")]
    pub loan_type: String,
}

#[derive(Debug, Args)]
pub struct OverseasAlgoExecutionsArgs {
    #[arg(long = "date", default_value = "", help = "주문일자 (YYYYMMDD)")]
    pub date: String,

    #[arg(long = "org-no", default_value = "", help = "주문채번지점번호")]
    pub org_no: String,

    #[arg(long = "order-no", default_value = "", help = "주문번호")]
    pub order_no: String,

    #[arg(long = "totalize", default_value = "", help = "집계포함여부")]
    pub totalize: String,
}

#[derive(Debug, Args)]
pub struct ReservationOrderListArgs {
    #[arg(long, value_enum, default_value_t = ReservationRegion::Us, help = "예약주문 지역 구분")]
    pub region: ReservationRegion,

    #[arg(long, required = true, help = "조회시작일 (YYYYMMDD)")]
    pub start: String,

    #[arg(long, required = true, help = "조회종료일 (YYYYMMDD)")]
    pub end: String,

    #[arg(long = "inquiry", default_value = "00", help = "조회구분코드")]
    pub inquiry: String,

    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: String,

    #[arg(long = "product-type", default_value = "", help = "상품유형코드")]
    pub product_type: String,
}

#[derive(Debug, Args)]
pub struct PossibleSellArgs {
    #[arg(help = "종목코드")]
    pub stock: String,
}

#[derive(Debug, Args)]
pub struct ExecutionArgs {
    #[arg(long, help = "조회시작일 (YYYYMMDD)")]
    pub start: Option<String>,

    #[arg(long, help = "조회종료일 (YYYYMMDD)")]
    pub end: Option<String>,
}

#[derive(Debug, Args)]
pub struct OverseasBalanceArgs {
    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: String,

    #[arg(long, required = true, help = "거래통화코드 (USD, HKD, CNY, JPY, VND)")]
    pub currency: String,
}

#[derive(Debug, Args)]
pub struct PresentBalanceArgs {
    #[arg(
        long = "currency-type",
        default_value = "02",
        help = "원화외화구분코드"
    )]
    pub currency_type: String,

    #[arg(long = "country", default_value = "000", help = "국가코드")]
    pub country: String,

    #[arg(long = "market", default_value = "00", help = "거래시장코드")]
    pub market: String,

    #[arg(long = "inquiry", default_value = "00", help = "조회구분코드")]
    pub inquiry: String,
}

#[derive(Debug, Args)]
pub struct SettlementBalanceArgs {
    #[arg(long = "date", required = true, help = "기준일자 (YYYYMMDD)")]
    pub date: String,

    #[arg(
        long = "currency-type",
        default_value = "01",
        help = "원화외화구분코드"
    )]
    pub currency_type: String,

    #[arg(long = "inquiry", default_value = "00", help = "조회구분코드")]
    pub inquiry: String,
}

#[derive(Debug, Args)]
pub struct OverseasExecutionArgs {
    #[arg(long, required = true, help = "조회시작일 (YYYYMMDD)")]
    pub start: String,

    #[arg(long, required = true, help = "조회종료일 (YYYYMMDD)")]
    pub end: String,

    #[arg(long, default_value = "%", help = "종목코드 (%: 전체)")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        default_value = "NASD",
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: String,

    #[arg(long = "side", default_value = "00", help = "매도매수구분")]
    pub side: String,

    #[arg(long = "status", default_value = "00", help = "체결미체결구분")]
    pub status: String,

    #[arg(long = "sort", default_value = "DS", help = "정렬순서")]
    pub sort: String,
}

#[derive(Debug, Args)]
pub struct OpenOrdersArgs {
    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 주문 거래소 코드 (NASD, NYSE, AMEX, SEHK, SHAA, SZAA, TKSE, HASE, VNSE)"
    )]
    pub exchange: String,

    #[arg(long = "sort", default_value = "DS", help = "정렬순서")]
    pub sort: String,
}

#[derive(Debug, Args)]
pub struct MarketArgs {
    #[command(subcommand)]
    pub command: MarketCommand,
}

#[derive(Debug, Subcommand)]
pub enum MarketCommand {
    #[command(about = "거래량 순위 조회")]
    Volume(MarketVolumeArgs),
    #[command(about = "해외 시가총액 순위 조회")]
    Cap(OverseasMarketArgs),
    #[command(about = "해외 급등락 순위 조회")]
    PriceFluct(OverseasMarketArgs),
    #[command(about = "해외 신고가/신저가 순위 조회")]
    NewHighlow(OverseasMarketArgs),
    #[command(about = "해외 거래량 급증 순위 조회")]
    VolumeSurge(OverseasMarketArgs),
    #[command(about = "휴장일 조회")]
    Holiday(HolidayArgs),
}

#[derive(Debug, Args)]
pub struct MarketVolumeArgs {
    #[arg(
        short = 'x',
        long,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: Option<String>,
}

#[derive(Debug, Args)]
pub struct OverseasMarketArgs {
    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: String,
}

#[derive(Debug, Args)]
pub struct HolidayArgs {
    #[arg(help = "기준일YYYYMMDD")]
    pub date: String,
}

#[derive(Debug, Args)]
pub struct FinanceArgs {
    #[command(subcommand)]
    pub command: FinanceCommand,
}

#[derive(Debug, Subcommand)]
pub enum FinanceCommand {
    #[command(about = "재무상태표 (Balance Sheet)")]
    Bs(FinanceSymbolArgs),
    #[command(about = "손익계산서 (Income Statement)")]
    Is(FinanceSymbolArgs),
    #[command(about = "재무비율 (PER, PBR, ROE 등)")]
    Ratio(FinanceSymbolArgs),
}

#[derive(Debug, Args)]
pub struct FinanceSymbolArgs {
    #[arg(help = "종목코드")]
    pub stock: String,

    #[arg(long, default_value = "0", help = "구분 (0:연간, 1:분기)")]
    pub div: String,
}

#[derive(Debug, Args)]
pub struct InfoArgs {
    #[command(subcommand)]
    pub command: InfoCommand,
}

#[derive(Debug, Subcommand)]
pub enum InfoCommand {
    #[command(about = "배당정보 조회")]
    Dividend(SymbolArgs),
    #[command(about = "뉴스 조회")]
    News(SymbolArgs),
    #[command(about = "투자의견 조회")]
    Opinion(SymbolArgs),
    #[command(about = "종목검색")]
    Search(SearchArgs),
    #[command(about = "해외주식 상품기본정보 조회")]
    Detail(OverseasInfoDetailArgs),
}

#[derive(Debug, Args)]
pub struct SymbolArgs {
    #[arg(help = "종목코드")]
    pub stock: String,
}

#[derive(Debug, Args)]
pub struct ExchangeSymbolArgs {
    #[arg(help = "종목코드 또는 티커")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: Option<String>,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    #[arg(help = "키워드")]
    pub keyword: String,
}

#[derive(Debug, Args)]
pub struct OverseasInfoDetailArgs {
    #[arg(help = "종목코드 또는 티커")]
    pub stock: String,

    #[arg(
        short = 'x',
        long,
        required = true,
        help = "해외 거래소 코드 (NAS, NYS, AMS, TSE, HKS, SHS, SZS, HSX, HNX)"
    )]
    pub exchange: String,
}

#[derive(Debug, Args)]
pub struct WsArgs {
    #[command(subcommand)]
    pub command: WsCommand,
}

#[derive(Debug, Subcommand)]
pub enum WsCommand {
    #[command(about = "WebSocket approval key 발급")]
    Approval,
    #[command(about = "국내 시간외 실시간호가 구독")]
    OvertimeAsk(WsStreamArgs),
    #[command(about = "국내 시간외 실시간체결가 구독")]
    OvertimeCcnl(WsStreamArgs),
}

#[derive(Debug, Args)]
pub struct WsStreamArgs {
    #[arg(help = "종목코드")]
    pub stock: String,

    #[arg(long, default_value_t = 1, help = "수집할 메시지 개수")]
    pub count: usize,

    #[arg(
        long = "timeout-secs",
        default_value_t = 30,
        help = "연결/수신 타임아웃(초)"
    )]
    pub timeout_secs: u64,

    #[arg(long = "reconnects", default_value_t = 2, help = "재연결 시도 횟수")]
    pub reconnects: usize,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{
        BalanceArgs, BalanceCommand, ChartCommand, Cli, Command, FinanceCommand, InfoCommand,
        MarketCommand, OrderCommand, OutputFormat, QuoteCommand, ReservationCancelRegion,
        ReservationRegion, WsCommand,
    };

    #[test]
    fn parses_price_command_with_exchange() {
        let cli = Cli::try_parse_from(["kis", "price", "--exchange", "NAS", "AAPL"]).unwrap();
        let Command::Price(args) = cli.command else {
            panic!("expected price command");
        };

        assert_eq!(args.exchange.as_deref(), Some("NAS"));
        assert_eq!(args.symbol, "AAPL");
        assert!(!args.daily);
    }

    #[test]
    fn parses_overseas_daily_price_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "price",
            "--exchange",
            "NAS",
            "AAPL",
            "--daily",
            "--period",
            "W",
        ])
        .unwrap();
        let Command::Price(args) = cli.command else {
            panic!("expected price command");
        };

        assert_eq!(args.exchange.as_deref(), Some("NAS"));
        assert_eq!(args.symbol, "AAPL");
        assert!(args.daily);
        assert_eq!(args.period, "W");
    }

    #[test]
    fn parses_order_buy_command() {
        let cli = Cli::try_parse_from([
            "kis", "order", "buy", "--stock", "005930", "--qty", "1", "--market",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::Buy(args) = order.command else {
            panic!("expected buy command");
        };

        assert_eq!(args.stock, "005930");
        assert_eq!(args.qty, "1");
        assert!(args.market);
        assert_eq!(args.exchange, None);
    }

    #[test]
    fn parses_overseas_order_buy_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "buy",
            "--exchange",
            "NASD",
            "--stock",
            "AAPL",
            "--qty",
            "1",
            "--price",
            "145.00",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::Buy(args) = order.command else {
            panic!("expected buy command");
        };

        assert_eq!(args.exchange.as_deref(), Some("NASD"));
        assert_eq!(args.stock, "AAPL");
        assert_eq!(args.price.as_deref(), Some("145.00"));
        assert!(!args.market);
    }

    #[test]
    fn parses_overseas_order_buy_with_reserve_flag() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "buy",
            "--exchange",
            "NASD",
            "--stock",
            "AAPL",
            "--qty",
            "1",
            "--price",
            "145.00",
            "--reserve",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::Buy(args) = order.command else {
            panic!("expected buy command");
        };

        assert!(args.reserve);
        assert!(!args.daytime);
        assert!(!args.dry_run);
    }

    #[test]
    fn parses_overseas_order_modify_with_daytime_flag() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "modify",
            "--exchange",
            "NYSE",
            "--stock",
            "BA",
            "--order-no",
            "30135009",
            "--qty",
            "1",
            "--price",
            "226.00",
            "--daytime",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::Modify(args) = order.command else {
            panic!("expected modify command");
        };

        assert!(args.daytime);
    }

    #[test]
    fn parses_order_buy_with_dry_run_flag() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "buy",
            "--stock",
            "005930",
            "--qty",
            "1",
            "--market",
            "--dry-run",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::Buy(args) = order.command else {
            panic!("expected buy command");
        };

        assert!(args.dry_run);
    }

    #[test]
    fn rejects_overseas_order_buy_with_conflicting_modes() {
        let err = Cli::try_parse_from([
            "kis",
            "order",
            "buy",
            "--exchange",
            "NASD",
            "--stock",
            "AAPL",
            "--qty",
            "1",
            "--price",
            "145.00",
            "--reserve",
            "--daytime",
        ])
        .unwrap_err();

        assert!(err.to_string().contains("--reserve"));
        assert!(err.to_string().contains("--daytime"));
    }

    #[test]
    fn parses_overseas_order_modify_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "modify",
            "--exchange",
            "NYSE",
            "--stock",
            "BA",
            "--order-no",
            "30135009",
            "--qty",
            "1",
            "--price",
            "226.00",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::Modify(args) = order.command else {
            panic!("expected modify command");
        };

        assert_eq!(args.exchange.as_deref(), Some("NYSE"));
        assert_eq!(args.stock.as_deref(), Some("BA"));
        assert_eq!(args.order_no, "30135009");
        assert_eq!(args.price, "226.00");
    }

    #[test]
    fn parses_balance_subcommand() {
        let cli = Cli::try_parse_from(["kis", "balance", "psbl-buy", "005930"]).unwrap();
        let Command::Balance(BalanceArgs { command }) = cli.command else {
            panic!("expected balance command");
        };
        let Some(BalanceCommand::PsblBuy(args)) = command else {
            panic!("expected balance psbl-buy");
        };

        assert_eq!(args.stock, "005930");
        assert_eq!(args.order_type, "01");
        assert_eq!(args.price, "0");
    }

    #[test]
    fn parses_overseas_balance_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "balance",
            "overseas",
            "--exchange",
            "NASD",
            "--currency",
            "USD",
        ])
        .unwrap();
        let Command::Balance(BalanceArgs { command }) = cli.command else {
            panic!("expected balance command");
        };
        let Some(BalanceCommand::Overseas(args)) = command else {
            panic!("expected balance overseas");
        };

        assert_eq!(args.exchange, "NASD");
        assert_eq!(args.currency, "USD");
    }

    #[test]
    fn parses_overseas_present_balance_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "balance",
            "present",
            "--currency-type",
            "01",
            "--country",
            "840",
            "--market",
            "01",
            "--inquiry",
            "01",
        ])
        .unwrap();
        let Command::Balance(BalanceArgs { command }) = cli.command else {
            panic!("expected balance command");
        };
        let Some(BalanceCommand::Present(args)) = command else {
            panic!("expected balance present");
        };

        assert_eq!(args.currency_type, "01");
        assert_eq!(args.country, "840");
        assert_eq!(args.market, "01");
        assert_eq!(args.inquiry, "01");
    }

    #[test]
    fn parses_global_json_flag_after_subcommand() {
        let cli = Cli::try_parse_from(["kis", "price", "005930", "--json"]).unwrap();
        let Command::Price(ref args) = cli.command else {
            panic!("expected price command");
        };

        assert_eq!(args.symbol, "005930");
        assert!(cli.json);
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn parses_global_output_flag_after_subcommand() {
        let cli = Cli::try_parse_from(["kis", "price", "005930", "--output", "json"]).unwrap();
        let Command::Price(ref args) = cli.command else {
            panic!("expected price command");
        };

        assert_eq!(args.symbol, "005930");
        assert_eq!(cli.output, OutputFormat::Json);
        assert_eq!(cli.output_format(), OutputFormat::Json);
    }

    #[test]
    fn parses_global_env_flag_after_subcommand() {
        let cli = Cli::try_parse_from(["kis", "price", "005930", "--env", "real"]).unwrap();
        let Command::Price(args) = cli.command else {
            panic!("expected price command");
        };

        assert_eq!(args.symbol, "005930");
        assert_eq!(cli.env.as_deref(), Some("real"));
    }

    #[test]
    fn parses_global_config_flag_after_subcommand() {
        let cli =
            Cli::try_parse_from(["kis", "config", "--config", "/tmp/kis/config.yaml"]).unwrap();

        assert!(matches!(cli.command, Command::Config));
        assert_eq!(
            cli.config.as_deref(),
            Some(std::path::Path::new("/tmp/kis/config.yaml"))
        );
    }

    #[test]
    fn parses_global_quiet_flag_after_subcommand() {
        let cli = Cli::try_parse_from(["kis", "config", "--quiet"]).unwrap();

        assert!(matches!(cli.command, Command::Config));
        assert!(cli.quiet);
    }

    #[test]
    fn parses_quote_ask_command() {
        let cli = Cli::try_parse_from(["kis", "quote", "ask", "005930"]).unwrap();
        let Command::Quote(args) = cli.command else {
            panic!("expected quote command");
        };
        let QuoteCommand::Ask(args) = args.command else {
            panic!("expected quote ask command");
        };

        assert_eq!(args.stock, "005930");
    }

    #[test]
    fn parses_overseas_quote_ask_command() {
        let cli =
            Cli::try_parse_from(["kis", "quote", "ask", "AAPL", "--exchange", "NAS"]).unwrap();
        let Command::Quote(args) = cli.command else {
            panic!("expected quote command");
        };
        let QuoteCommand::Ask(args) = args.command else {
            panic!("expected quote ask command");
        };

        assert_eq!(args.stock, "AAPL");
        assert_eq!(args.exchange.as_deref(), Some("NAS"));
    }

    #[test]
    fn parses_quote_overtime_price_command() {
        let cli = Cli::try_parse_from(["kis", "quote", "overtime-price", "005930"]).unwrap();
        let Command::Quote(args) = cli.command else {
            panic!("expected quote command");
        };
        let QuoteCommand::OvertimePrice(args) = args.command else {
            panic!("expected quote overtime-price command");
        };

        assert_eq!(args.stock, "005930");
    }

    #[test]
    fn parses_overseas_quote_ccnl_command() {
        let cli =
            Cli::try_parse_from(["kis", "quote", "ccnl", "AAPL", "--exchange", "NAS"]).unwrap();
        let Command::Quote(args) = cli.command else {
            panic!("expected quote command");
        };
        let QuoteCommand::Ccnl(args) = args.command else {
            panic!("expected quote ccnl command");
        };

        assert_eq!(args.stock, "AAPL");
        assert_eq!(args.exchange.as_deref(), Some("NAS"));
    }

    #[test]
    fn parses_overseas_possible_buy_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "balance",
            "psbl-buy",
            "QQQ",
            "--exchange",
            "NASD",
            "--price",
            "1.4",
        ])
        .unwrap();
        let Command::Balance(BalanceArgs { command }) = cli.command else {
            panic!("expected balance command");
        };
        let Some(BalanceCommand::PsblBuy(args)) = command else {
            panic!("expected balance psbl-buy");
        };

        assert_eq!(args.stock, "QQQ");
        assert_eq!(args.exchange.as_deref(), Some("NASD"));
        assert_eq!(args.price, "1.4");
    }

    #[test]
    fn parses_reserve_cancel_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "reserve-cancel",
            "--region",
            "us",
            "--receipt-date",
            "20260307",
            "--reservation-order-no",
            "0030008244",
        ])
        .unwrap();

        let Command::Order(order) = cli.command else {
            panic!("expected order command");
        };
        let OrderCommand::ReserveCancel(args) = order.command else {
            panic!("expected reserve-cancel command");
        };

        assert_eq!(args.region, ReservationCancelRegion::Us);
        assert_eq!(args.receipt_date, "20260307");
        assert_eq!(args.reservation_order_no, "0030008244");
    }

    #[test]
    fn rejects_asia_region_for_reserve_cancel_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "order",
            "reserve-cancel",
            "--region",
            "asia",
            "--receipt-date",
            "20260307",
            "--reservation-order-no",
            "0030008244",
        ]);

        assert!(cli.is_err());
    }

    #[test]
    fn parses_asia_region_for_reserve_orders_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "balance",
            "reserve-orders",
            "--region",
            "asia",
            "--start",
            "20260301",
            "--end",
            "20260307",
            "--exchange",
            "TKSE",
        ])
        .unwrap();

        let Command::Balance(BalanceArgs { command }) = cli.command else {
            panic!("expected balance command");
        };
        let Some(BalanceCommand::ReserveOrders(args)) = command else {
            panic!("expected reserve-orders command");
        };

        assert_eq!(args.region, ReservationRegion::Asia);
        assert_eq!(args.exchange, "TKSE");
    }

    #[test]
    fn parses_ws_overtime_ask_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "ws",
            "overtime-ask",
            "005930",
            "--count",
            "2",
            "--timeout-secs",
            "10",
        ])
        .unwrap();

        let Command::Ws(args) = cli.command else {
            panic!("expected ws command");
        };
        let WsCommand::OvertimeAsk(args) = args.command else {
            panic!("expected ws overtime-ask command");
        };

        assert_eq!(args.stock, "005930");
        assert_eq!(args.count, 2);
        assert_eq!(args.timeout_secs, 10);
    }

    #[test]
    fn parses_chart_daily_command() {
        let cli = Cli::try_parse_from([
            "kis", "chart", "daily", "005930", "--start", "20260101", "--end", "20260306",
            "--period", "W",
        ])
        .unwrap();
        let Command::Chart(args) = cli.command else {
            panic!("expected chart command");
        };
        let ChartCommand::Daily(args) = args.command else {
            panic!("expected chart daily command");
        };

        assert_eq!(args.stock, "005930");
        assert_eq!(args.start.as_deref(), Some("20260101"));
        assert_eq!(args.end.as_deref(), Some("20260306"));
        assert_eq!(args.period, "W");
    }

    #[test]
    fn parses_overseas_chart_daily_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "chart",
            "daily",
            "AAPL",
            "--exchange",
            "NAS",
            "--start",
            "20260301",
            "--end",
            "20260306",
        ])
        .unwrap();
        let Command::Chart(args) = cli.command else {
            panic!("expected chart command");
        };
        let ChartCommand::Daily(args) = args.command else {
            panic!("expected chart daily command");
        };

        assert_eq!(args.stock, "AAPL");
        assert_eq!(args.exchange.as_deref(), Some("NAS"));
    }

    #[test]
    fn parses_overseas_chart_time_command() {
        let cli = Cli::try_parse_from([
            "kis",
            "chart",
            "time",
            "AAPL",
            "--exchange",
            "NAS",
            "--unit",
            "5",
        ])
        .unwrap();
        let Command::Chart(args) = cli.command else {
            panic!("expected chart command");
        };
        let ChartCommand::Time(args) = args.command else {
            panic!("expected chart time command");
        };

        assert_eq!(args.stock, "AAPL");
        assert_eq!(args.exchange.as_deref(), Some("NAS"));
        assert_eq!(args.unit, "5");
    }

    #[test]
    fn parses_chart_index_price_command() {
        let cli = Cli::try_parse_from(["kis", "chart", "index-price", "0001"]).unwrap();
        let Command::Chart(args) = cli.command else {
            panic!("expected chart command");
        };
        let ChartCommand::IndexPrice(args) = args.command else {
            panic!("expected chart index-price command");
        };

        assert_eq!(args.index, "0001");
    }

    #[test]
    fn parses_market_holiday_command() {
        let cli = Cli::try_parse_from(["kis", "market", "holiday", "20260306"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::Holiday(args) = args.command else {
            panic!("expected market holiday command");
        };

        assert_eq!(args.date, "20260306");
    }

    #[test]
    fn parses_domestic_market_volume_command() {
        let cli = Cli::try_parse_from(["kis", "market", "volume"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::Volume(args) = args.command else {
            panic!("expected market volume command");
        };

        assert_eq!(args.exchange, None);
    }

    #[test]
    fn parses_overseas_market_volume_command() {
        let cli = Cli::try_parse_from(["kis", "market", "volume", "--exchange", "NAS"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::Volume(args) = args.command else {
            panic!("expected market volume command");
        };

        assert_eq!(args.exchange.as_deref(), Some("NAS"));
    }

    #[test]
    fn parses_overseas_market_cap_command() {
        let cli = Cli::try_parse_from(["kis", "market", "cap", "--exchange", "NAS"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::Cap(args) = args.command else {
            panic!("expected market cap command");
        };

        assert_eq!(args.exchange, "NAS");
    }

    #[test]
    fn parses_overseas_market_price_fluct_command() {
        let cli =
            Cli::try_parse_from(["kis", "market", "price-fluct", "--exchange", "NAS"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::PriceFluct(args) = args.command else {
            panic!("expected market price-fluct command");
        };

        assert_eq!(args.exchange, "NAS");
    }

    #[test]
    fn parses_overseas_market_new_highlow_command() {
        let cli =
            Cli::try_parse_from(["kis", "market", "new-highlow", "--exchange", "NAS"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::NewHighlow(args) = args.command else {
            panic!("expected market new-highlow command");
        };

        assert_eq!(args.exchange, "NAS");
    }

    #[test]
    fn parses_overseas_market_volume_surge_command() {
        let cli =
            Cli::try_parse_from(["kis", "market", "volume-surge", "--exchange", "NAS"]).unwrap();
        let Command::Market(args) = cli.command else {
            panic!("expected market command");
        };
        let MarketCommand::VolumeSurge(args) = args.command else {
            panic!("expected market volume-surge command");
        };

        assert_eq!(args.exchange, "NAS");
    }

    #[test]
    fn parses_finance_ratio_command() {
        let cli = Cli::try_parse_from(["kis", "finance", "ratio", "005930", "--div", "1"]).unwrap();
        let Command::Finance(args) = cli.command else {
            panic!("expected finance command");
        };
        let FinanceCommand::Ratio(args) = args.command else {
            panic!("expected finance ratio command");
        };

        assert_eq!(args.stock, "005930");
        assert_eq!(args.div, "1");
    }

    #[test]
    fn parses_info_search_command() {
        let cli = Cli::try_parse_from(["kis", "info", "search", "삼성"]).unwrap();
        let Command::Info(args) = cli.command else {
            panic!("expected info command");
        };
        let InfoCommand::Search(args) = args.command else {
            panic!("expected info search command");
        };

        assert_eq!(args.keyword, "삼성");
    }

    #[test]
    fn parses_overseas_info_detail_command() {
        let cli =
            Cli::try_parse_from(["kis", "info", "detail", "AAPL", "--exchange", "NAS"]).unwrap();
        let Command::Info(args) = cli.command else {
            panic!("expected info command");
        };
        let InfoCommand::Detail(args) = args.command else {
            panic!("expected info detail command");
        };

        assert_eq!(args.stock, "AAPL");
        assert_eq!(args.exchange, "NAS");
    }
}
