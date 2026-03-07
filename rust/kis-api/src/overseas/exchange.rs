use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

pub const VALID_ORDER_EXCHANGES: &[&str] = &[
    "NASD", "NYSE", "AMEX", "SEHK", "SHAA", "SZAA", "TKSE", "HASE", "VNSE",
];

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum OrderExchange {
    Nasd,
    Nyse,
    Amex,
    Sehk,
    Shaa,
    Szaa,
    Tkse,
    Hase,
    Vnse,
}

impl OrderExchange {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_uppercase().as_str() {
            "NASD" => Ok(Self::Nasd),
            "NYSE" => Ok(Self::Nyse),
            "AMEX" => Ok(Self::Amex),
            "SEHK" => Ok(Self::Sehk),
            "SHAA" => Ok(Self::Shaa),
            "SZAA" => Ok(Self::Szaa),
            "TKSE" => Ok(Self::Tkse),
            "HASE" => Ok(Self::Hase),
            "VNSE" => Ok(Self::Vnse),
            other => bail!(
                "invalid overseas order exchange {other:?}; valid codes: {}",
                VALID_ORDER_EXCHANGES.join(", ")
            ),
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::Nasd => "NASD",
            Self::Nyse => "NYSE",
            Self::Amex => "AMEX",
            Self::Sehk => "SEHK",
            Self::Shaa => "SHAA",
            Self::Szaa => "SZAA",
            Self::Tkse => "TKSE",
            Self::Hase => "HASE",
            Self::Vnse => "VNSE",
        }
    }

    pub fn is_us(self) -> bool {
        matches!(self, Self::Nasd | Self::Nyse | Self::Amex)
    }

    pub fn buy_tr_id(self, is_virtual: bool) -> &'static str {
        match (self, is_virtual) {
            (Self::Nasd | Self::Nyse | Self::Amex, false) => "TTTT1002U",
            (Self::Nasd | Self::Nyse | Self::Amex, true) => "VTTT1002U",
            (Self::Sehk, false) => "TTTS1002U",
            (Self::Sehk, true) => "VTTS1002U",
            (Self::Shaa, false) => "TTTS0202U",
            (Self::Shaa, true) => "VTTS0202U",
            (Self::Szaa, false) => "TTTS0305U",
            (Self::Szaa, true) => "VTTS0305U",
            (Self::Tkse, false) => "TTTS0308U",
            (Self::Tkse, true) => "VTTS0308U",
            (Self::Hase | Self::Vnse, false) => "TTTS0311U",
            (Self::Hase | Self::Vnse, true) => "VTTS0311U",
        }
    }

    pub fn sell_tr_id(self, is_virtual: bool) -> &'static str {
        match (self, is_virtual) {
            (Self::Nasd | Self::Nyse | Self::Amex, false) => "TTTT1006U",
            (Self::Nasd | Self::Nyse | Self::Amex, true) => "VTTT1006U",
            (Self::Sehk, false) => "TTTS1001U",
            (Self::Sehk, true) => "VTTS1001U",
            (Self::Shaa, false) => "TTTS1005U",
            (Self::Shaa, true) => "VTTS1005U",
            (Self::Szaa, false) => "TTTS0304U",
            (Self::Szaa, true) => "VTTS0304U",
            (Self::Tkse, false) => "TTTS0307U",
            (Self::Tkse, true) => "VTTS0307U",
            (Self::Hase | Self::Vnse, false) => "TTTS0310U",
            (Self::Hase | Self::Vnse, true) => "VTTS0310U",
        }
    }
}
