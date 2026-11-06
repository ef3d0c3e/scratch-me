//! Cosmetics: app themes, scratch tools and ticket foils. Each comes from one
//! place: owned by everyone (the defaults), sold in the shop, won on Golden
//! Tickets, or earned with a trophy.

use serde::{Deserialize, Serialize};

/// Where a cosmetic comes from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Origin {
    Default,
    Shop {
        price: i64,
    },
    Golden,
    /// Earned with this achievement
    Trophy {
        achievement: &'static str,
    },
}

/// One kind of cosmetic: its items, their keys and where they come from
macro_rules! catalog {
    ($(#[$doc:meta])* $name:ident, default $default:ident, { $($item:ident => $key:literal, $origin:expr;)+ }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($item,)+
        }

        impl $name {
            pub const ALL: &[$name] = &[$($name::$item,)+];
            pub const DEFAULT: $name = $name::$default;

            pub fn key(self) -> &'static str {
                match self {
                    $($name::$item => $key,)+
                }
            }

            pub fn parse(key: &str) -> Option<$name> {
                $name::ALL.iter().copied().find(|t| t.key() == key)
            }

            pub fn origin(self) -> Origin {
                match self {
                    $($name::$item => $origin,)+
                }
            }
        }
    };
}

const fn shop(price: i64) -> Origin {
    Origin::Shop { price }
}

const fn trophy(achievement: &'static str) -> Origin {
    Origin::Trophy { achievement }
}

catalog!(
    /// The app's colours, and the foil's when the foil is the plain one
    Theme, default Midnight, {
        Midnight => "midnight", Origin::Default;
        Sunset => "sunset", shop(1500);
        Mint => "mint", shop(1500);
        Ocean => "ocean", shop(1500);
        Candy => "candy", shop(2000);
        Arcade => "arcade", shop(2500);
        Neon => "neon", Origin::Golden;
        Royal => "royal", Origin::Golden;
        Forest => "forest", Origin::Golden;
        Champion => "champion", trophy("level_10");
    }
);

catalog!(
    /// What the player scratches with
    Tool, default Coin, {
        Coin => "coin", Origin::Default;
        Card => "card", shop(1000);
        Key => "key", shop(1200);
        Eraser => "eraser", shop(1200);
        Paw => "paw", shop(1800);
        Laser => "laser", Origin::Golden;
        Wand => "wand", Origin::Golden;
        Shovel => "shovel", trophy("deep_digger");
    }
);

catalog!(
    /// The scratch layer of every ticket
    Foil, default Silver, {
        Silver => "silver", Origin::Default;
        Carbon => "carbon", shop(1500);
        Leopard => "leopard", shop(1800);
        Gold => "gold", shop(2000);
        Holo => "holo", shop(2500);
        Galaxy => "galaxy", Origin::Golden;
    }
);

/// One cosmetic, as stored: (kind, item)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Cosmetic {
    Theme(Theme),
    Tool(Tool),
    Foil(Foil),
}

impl Cosmetic {
    pub fn kind(self) -> &'static str {
        match self {
            Cosmetic::Theme(_) => "theme",
            Cosmetic::Tool(_) => "tool",
            Cosmetic::Foil(_) => "foil",
        }
    }

    pub fn item(self) -> &'static str {
        match self {
            Cosmetic::Theme(t) => t.key(),
            Cosmetic::Tool(t) => t.key(),
            Cosmetic::Foil(t) => t.key(),
        }
    }

    pub fn parse(kind: &str, item: &str) -> Option<Cosmetic> {
        match kind {
            "theme" => Theme::parse(item).map(Cosmetic::Theme),
            "tool" => Tool::parse(item).map(Cosmetic::Tool),
            "foil" => Foil::parse(item).map(Cosmetic::Foil),
            _ => None,
        }
    }

    pub fn origin(self) -> Origin {
        match self {
            Cosmetic::Theme(t) => t.origin(),
            Cosmetic::Tool(t) => t.origin(),
            Cosmetic::Foil(t) => t.origin(),
        }
    }

    pub fn is_default(self) -> bool {
        self.origin() == Origin::Default
    }

    /// Shop price, None when it is not sold
    pub fn price(self) -> Option<i64> {
        match self.origin() {
            Origin::Shop { price } => Some(price),
            _ => None,
        }
    }

    /// Every cosmetic, themes then tools then foils
    pub fn all() -> Vec<Cosmetic> {
        Theme::ALL
            .iter()
            .map(|t| Cosmetic::Theme(*t))
            .chain(Tool::ALL.iter().map(|t| Cosmetic::Tool(*t)))
            .chain(Foil::ALL.iter().map(|t| Cosmetic::Foil(*t)))
            .collect()
    }

    /// What a Golden Ticket can give: anything but the defaults and the trophies' own
    pub fn prizes() -> Vec<Cosmetic> {
        Cosmetic::all().into_iter().filter(|c| matches!(c.origin(), Origin::Shop { .. } | Origin::Golden)).collect()
    }
}

/// Coins paid instead of a cosmetic the player already owns
pub const DUPLICATE_COINS: i64 = 800;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip() {
        for c in Cosmetic::all() {
            assert_eq!(Cosmetic::parse(c.kind(), c.item()), Some(c));
        }
        assert_eq!(Cosmetic::parse("hat", "coin"), None);
    }

    #[test]
    fn json() {
        let c = Cosmetic::Foil(Foil::Holo);
        assert_eq!(serde_json::to_value(c).unwrap(), serde_json::json!({ "kind": "foil", "id": "holo" }));
    }

    #[test]
    fn one_default_per_kind_and_trophies_are_not_prizes() {
        for kind in ["theme", "tool", "foil"] {
            let defaults = Cosmetic::all().into_iter().filter(|c| c.kind() == kind && c.is_default()).count();
            assert_eq!(defaults, 1, "{kind}");
        }
        assert!(Cosmetic::prizes().iter().all(|c| !matches!(c.origin(), Origin::Default | Origin::Trophy { .. })));
        assert!(Cosmetic::prizes().iter().any(|c| c.origin() == Origin::Golden));
    }

    #[test]
    fn trophies_name_real_achievements() {
        let ids: Vec<&str> = crate::progress::achievements::all().iter().map(|a| a.id).collect();
        for c in Cosmetic::all() {
            if let Origin::Trophy { achievement } = c.origin() {
                assert!(ids.contains(&achievement), "{} needs {achievement}", c.item());
                let a = crate::progress::achievements::all().into_iter().find(|a| a.id == achievement).unwrap();
                assert_eq!(a.reward, crate::tickets::Prize::Cosmetic { item: c }, "{achievement} gives it");
            }
        }
    }
}
