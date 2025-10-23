use bigdecimal::BigDecimal;

pub trait BigDecimalExt {
    fn to_u128(self) -> u128;
}

impl BigDecimalExt for BigDecimal {
    fn to_u128(self) -> u128 {
        self.to_string()
            .parse()
            .expect("Invalid BigDecimal to u128 conversion")
    }
}
