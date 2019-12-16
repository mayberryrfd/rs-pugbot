use crate::models::user::DiscordUser as User;
use crate::schema::*;
use bigdecimal::BigDecimal;

#[table_name = "user_ratings"]
#[derive(Debug, Insertable, Queryable, Associations)]
#[belongs_to(User, GameMode)]
pub struct UserRating {
  pub id: Option<i32>,
  pub user_id: i32,
  pub rating: Option<BigDecimal>,
  pub deviation: Option<BigDecimal>,
  pub volatility: Option<BigDecimal>,
  pub game_mode_id: i32,
}
