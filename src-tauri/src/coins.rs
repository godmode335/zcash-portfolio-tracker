#[derive(Debug, Clone, Copy)]
pub struct Coin {
    pub id: &'static str,
    pub symbol: &'static str,
    pub name: &'static str,
    pub coingecko_id: &'static str,
}

static COINS: &[Coin] = &[
    Coin { id: "zcash", symbol: "ZEC", name: "Zcash", coingecko_id: "zcash" },
];

pub fn all() -> &'static [Coin] { COINS }

pub fn by_id(id: &str) -> Option<&'static Coin> {
    COINS.iter().find(|c| c.id == id)
}

pub fn coingecko_ids() -> Vec<&'static str> {
    COINS.iter().map(|c| c.coingecko_id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_only_zcash() {
        assert_eq!(all().len(), 1);
        assert_eq!(all()[0].id, "zcash");
    }

    #[test]
    fn lookup_zcash_works() {
        assert_eq!(by_id("zcash").unwrap().symbol, "ZEC");
        assert!(by_id("monero").is_none());
    }

    #[test]
    fn coingecko_ids_is_zcash() {
        assert_eq!(coingecko_ids(), vec!["zcash"]);
    }
}
