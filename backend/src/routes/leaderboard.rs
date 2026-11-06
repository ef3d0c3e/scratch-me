//! The leaderboard: members by coins held, and by level (experience).
//! Guests are not ranked; they can look.

use actix_web::{get, web};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::CurrentUser;
use crate::error::ApiResult;
use crate::progress::levels;
use crate::schema::users;

/// Rows shown
pub const TOP: i64 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Board {
    Coins,
    Level,
}

#[derive(Deserialize)]
pub struct Query {
    #[serde(default = "coins")]
    by: Board,
}

fn coins() -> Board {
    Board::Coins
}

#[derive(Debug, Serialize)]
pub struct Row {
    pub rank: i64,
    pub username: String,
    pub level: i32,
    pub title: Option<String>,
    /// Coins, or experience on the level board
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct Mine {
    pub rank: i64,
    pub value: i64,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardDto {
    pub by: Board,
    pub rows: Vec<Row>,
    /// The viewer's place, top 50 or not; None for a guest
    pub me: Option<Mine>,
}

/// Ranks a list sorted best first: equal values share a rank, the next one
/// skips (1, 2, 2, 4)
pub fn ranks(values: &[i64]) -> Vec<i64> {
    let mut out = Vec::with_capacity(values.len());
    for (i, v) in values.iter().enumerate() {
        let rank = if i > 0 && values[i - 1] == *v { out[i - 1] } else { i as i64 + 1 };
        out.push(rank);
    }
    out
}

#[get("/leaderboard")]
pub async fn leaderboard(
    state: web::Data<AppState>,
    user: CurrentUser,
    query: web::Query<Query>,
) -> ApiResult<web::Json<LeaderboardDto>> {
    let by = query.into_inner().by;
    let mut conn = state.pool.get().await?;
    let members = users::table.filter(users::is_guest.eq(false));
    type Top = (String, i64, Option<String>, i64);
    let fields = (users::username, users::xp, users::title);
    let top: Vec<Top> = match by {
        Board::Coins => {
            members
                .order((users::coins.desc(), users::id))
                .limit(TOP)
                .select((fields.0, fields.1, fields.2, users::coins))
                .load(&mut conn)
                .await?
        }
        Board::Level => {
            members
                .order((users::xp.desc(), users::id))
                .limit(TOP)
                .select((fields.0, fields.1, fields.2, users::xp))
                .load(&mut conn)
                .await?
        }
    };
    let values: Vec<i64> = top.iter().map(|r| r.3).collect();
    let rows = top
        .into_iter()
        .zip(ranks(&values))
        .map(|((username, xp, title, value), rank)| Row { rank, username, level: levels::level_for(xp), title, value })
        .collect();
    // The viewer's rank: one more than the members ahead of them
    let me = if user.is_guest {
        None
    } else {
        let (coins, xp): (i64, i64) =
            users::table.find(user.id).select((users::coins, users::xp)).first(&mut conn).await?;
        let (value, ahead): (i64, i64) = match by {
            Board::Coins => (coins, members.filter(users::coins.gt(coins)).count().get_result(&mut conn).await?),
            Board::Level => (xp, members.filter(users::xp.gt(xp)).count().get_result(&mut conn).await?),
        };
        Some(Mine { rank: ahead + 1, value })
    };
    Ok(web::Json(LeaderboardDto { by, rows, me }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ties_share_a_rank() {
        assert_eq!(ranks(&[]), Vec::<i64>::new());
        assert_eq!(ranks(&[900, 500, 500, 100]), [1, 2, 2, 4]);
        assert_eq!(ranks(&[7, 7, 7]), [1, 1, 1]);
    }
}
