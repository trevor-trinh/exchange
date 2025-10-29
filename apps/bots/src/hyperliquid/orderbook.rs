use rust_decimal::Decimal;
use std::collections::BTreeMap;
use std::str::FromStr;

/// Price level in the orderbook
#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: Decimal,
}

/// Local orderbook state
#[derive(Debug, Clone)]
pub struct Orderbook {
    pub symbol: String,
    pub last_update_id: u64,
    pub bids: BTreeMap<Decimal, Decimal>, // price -> quantity (sorted descending)
    pub asks: BTreeMap<Decimal, Decimal>, // price -> quantity (sorted ascending)
}

impl Orderbook {
    pub fn new(symbol: String) -> Self {
        Self {
            symbol,
            last_update_id: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Update from Hyperliquid L2 data
    pub fn update_from_l2(&mut self, bids: Vec<super::types::L2Level>, asks: Vec<super::types::L2Level>) {
        self.bids.clear();
        self.asks.clear();

        for level in bids {
            if let (Ok(p), Ok(q)) = (Decimal::from_str(&level.px), Decimal::from_str(&level.sz)) {
                if q > Decimal::ZERO {
                    self.bids.insert(p, q);
                }
            }
        }

        for level in asks {
            if let (Ok(p), Ok(q)) = (Decimal::from_str(&level.px), Decimal::from_str(&level.sz)) {
                if q > Decimal::ZERO {
                    self.asks.insert(p, q);
                }
            }
        }
    }

    /// Get top N levels on each side
    pub fn get_top_levels(&self, depth: usize) -> (Vec<PriceLevel>, Vec<PriceLevel>) {
        let bids: Vec<PriceLevel> = self
            .bids
            .iter()
            .rev() // Highest prices first
            .take(depth)
            .map(|(price, quantity)| PriceLevel {
                price: *price,
                quantity: *quantity,
            })
            .collect();

        let asks: Vec<PriceLevel> = self
            .asks
            .iter() // Lowest prices first
            .take(depth)
            .map(|(price, quantity)| PriceLevel {
                price: *price,
                quantity: *quantity,
            })
            .collect();

        (bids, asks)
    }

    /// Get best bid and ask
    pub fn get_best_prices(&self) -> Option<(Decimal, Decimal)> {
        let best_bid = self.bids.iter().next_back().map(|(p, _)| *p)?;
        let best_ask = self.asks.iter().next().map(|(p, _)| *p)?;
        Some((best_bid, best_ask))
    }
}
