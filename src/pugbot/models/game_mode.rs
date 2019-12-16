use crate::models::game_title::GameTitle;
use crate::schema::*;
use diesel::dsl::Eq;
use diesel::pg::Pg;
use diesel::prelude::{Insertable, Queryable};
use diesel::ExpressionMethods;

#[primary_key(game_mode_id)]
#[table_name = "game_modes"]
#[belongs_to(GameTitle)]
#[derive(Debug, Associations, Identifiable)]
#[primary_key(game_mode_id)]
pub struct GameMode {
  pub game_mode_id: i32,
  pub game_title_id: i32,
  pub mode_name: String,
  pub team_size: i32,
}

impl<'a> Insertable<game_modes::table> for &'a GameMode {
  type Values = <(
    Eq<game_modes::game_title_id, i32>,
    Eq<game_modes::mode_name, &'a String>,
    Eq<game_modes::team_size, i32>,
  ) as Insertable<game_modes::table>>::Values;

  fn values(self) -> Self::Values {
    (
      game_modes::game_title_id.eq(self.game_title_id),
      game_modes::mode_name.eq(&self.mode_name),
      game_modes::team_size.eq(self.team_size),
    )
      .values()
  }
}

impl Queryable<game_modes::SqlType, Pg> for GameMode {
  type Row = (i32, i32, String, i32);

  fn build(
    (game_mode_id, game_title_id, mode_name, team_size): Self::Row,
  ) -> Self {
    GameMode {
      game_mode_id,
      game_title_id,
      mode_name,
      team_size,
    }
  }
}
