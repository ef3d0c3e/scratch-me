//! Cosmetics: app themes and scratch tools. Some are sold in the shop, some
//! only come from Golden Tickets. Everyone owns the defaults.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Midnight,
    Sunset,
    Mint,
    /// Golden Ticket only
    Neon,
    /// Golden Ticket only
    Royal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    Coin,
    Key,
    Eraser,
    /// Golden Ticket only
    Laser,
}

impl Theme {
    pub const ALL: [Theme; 5] = [Theme::Midnight, Theme::Sunset, Theme::Mint, Theme::Neon, Theme::Royal];
    pub const DEFAULT: Theme = Theme::Midnight;

    pub fn key(self) -> &'static str {
        match self {
            Theme::Midnight => "midnight",
            Theme::Sunset => "sunset",
            Theme::Mint => "mint",
            Theme::Neon => "neon",
            Theme::Royal => "royal",
        }
    }

    pub fn parse(key: &str) -> Option<Theme> {
        Theme::ALL.into_iter().find(|t| t.key() == key)
    }

    /// Shop price, None when it is not sold
    pub fn price(self) -> Option<i64> {
        match self {
            Theme::Sunset | Theme::Mint => Some(1500),
            Theme::Midnight | Theme::Neon | Theme::Royal => None,
        }
    }
}

impl Tool {
    pub const ALL: [Tool; 4] = [Tool::Coin, Tool::Key, Tool::Eraser, Tool::Laser];
    pub const DEFAULT: Tool = Tool::Coin;

    pub fn key(self) -> &'static str {
        match self {
            Tool::Coin => "coin",
            Tool::Key => "key",
            Tool::Eraser => "eraser",
            Tool::Laser => "laser",
        }
    }

    pub fn parse(key: &str) -> Option<Tool> {
        Tool::ALL.into_iter().find(|t| t.key() == key)
    }

    pub fn price(self) -> Option<i64> {
        match self {
            Tool::Key | Tool::Eraser => Some(1200),
            Tool::Coin | Tool::Laser => None,
        }
    }
}

/// One cosmetic, as stored: (kind, item)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Cosmetic {
    Theme(Theme),
    Tool(Tool),
}

impl Cosmetic {
    pub fn kind(self) -> &'static str {
        match self {
            Cosmetic::Theme(_) => "theme",
            Cosmetic::Tool(_) => "tool",
        }
    }

    pub fn item(self) -> &'static str {
        match self {
            Cosmetic::Theme(t) => t.key(),
            Cosmetic::Tool(t) => t.key(),
        }
    }

    pub fn parse(kind: &str, item: &str) -> Option<Cosmetic> {
        match kind {
            "theme" => Theme::parse(item).map(Cosmetic::Theme),
            "tool" => Tool::parse(item).map(Cosmetic::Tool),
            _ => None,
        }
    }

    pub fn is_default(self) -> bool {
        matches!(self, Cosmetic::Theme(Theme::DEFAULT) | Cosmetic::Tool(Tool::DEFAULT))
    }

    pub fn price(self) -> Option<i64> {
        match self {
            Cosmetic::Theme(t) => t.price(),
            Cosmetic::Tool(t) => t.price(),
        }
    }

    /// Everything a Golden Ticket can give: every cosmetic but the defaults
    pub fn prizes() -> Vec<Cosmetic> {
        Theme::ALL
            .into_iter()
            .map(Cosmetic::Theme)
            .chain(Tool::ALL.into_iter().map(Cosmetic::Tool))
            .filter(|c| !c.is_default())
            .collect()
    }
}

/// Coins paid instead of a cosmetic the player already owns
pub const DUPLICATE_COINS: i64 = 800;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip() {
        for t in Theme::ALL {
            assert_eq!(Cosmetic::parse("theme", t.key()), Some(Cosmetic::Theme(t)));
        }
        for t in Tool::ALL {
            assert_eq!(Cosmetic::parse("tool", t.key()), Some(Cosmetic::Tool(t)));
        }
        assert_eq!(Cosmetic::parse("hat", "coin"), None);
    }

    #[test]
    fn json() {
        let c = Cosmetic::Theme(Theme::Neon);
        assert_eq!(serde_json::to_value(c).unwrap(), serde_json::json!({ "kind": "theme", "id": "neon" }));
    }

    #[test]
    fn defaults_are_free_and_never_prizes() {
        assert!(Cosmetic::prizes().iter().all(|c| !c.is_default()));
        assert_eq!(Cosmetic::Theme(Theme::DEFAULT).price(), None);
        // Some cosmetics only come from Golden Tickets
        assert!(Cosmetic::prizes().iter().any(|c| c.price().is_none()));
    }
}
