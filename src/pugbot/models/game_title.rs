use crate::schema::*;
use diesel::dsl::Eq;
use diesel::pg::Pg;
use diesel::prelude::{Insertable, Queryable};
use diesel::ExpressionMethods;

#[primary_key(game_title_id)]
#[table_name = "game_titles"]
#[derive(Debug, Associations, Identifiable)]
pub struct GameTitle {
  pub game_title_id: i32,
  pub game_name: String,
}

impl<'a> Insertable<game_titles::table> for &'a GameTitle {
  type Values = <(Eq<game_titles::game_name, &'a String>,) as Insertable<
    game_titles::table,
  >>::Values;

  fn values(self) -> Self::Values {
    (game_titles::game_name.eq(&self.game_name),).values()
  }
}

impl Queryable<game_titles::SqlType, Pg> for GameTitle {
  type Row = (i32, String);

  fn build((game_title_id, game_name): Self::Row) -> Self {
    GameTitle {
      game_title_id,
      game_name,
    }
  }
}
