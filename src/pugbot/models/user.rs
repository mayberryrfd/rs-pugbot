use crate::models::user_rating::UserRating;
use crate::schema::users;
use diesel::dsl::Eq;
use diesel::pg::Pg;
use diesel::prelude::{Insertable, Queryable};
use diesel::ExpressionMethods;
use serenity::model::id::UserId;
use serenity::model::user::User as SerenityUser;

#[derive(Identifiable, PartialEq, Debug)]
#[primary_key(user_id)]
#[table_name = "users"]
pub struct DiscordUser {
  pub user_id: i32,
  pub database_id: Option<i32>,
  pub discord_user_id: UserId,
  pub bot: bool,
  pub discriminator: u16,
  pub name: String,
  pub avatar: Option<String>,
}

impl<'a> Insertable<users::table> for &'a SerenityUser {
  type Values = <(
    Eq<users::bot, bool>,
    Eq<users::discriminator, i32>,
    Eq<users::name, &'a String>,
    Eq<users::discord_user_id, i32>,
  ) as Insertable<users::table>>::Values;

  fn values(self) -> Self::Values {
    (
      users::bot.eq(self.bot),
      users::discriminator.eq(self.discriminator as i32),
      users::name.eq(&self.name),
      users::discord_user_id.eq(self.id.0 as i32),
    )
      .values()
  }
}

impl From<DiscordUser> for SerenityUser {
  fn from(discord_user: DiscordUser) -> SerenityUser {
    SerenityUser {
      id: discord_user.discord_user_id,
      bot: discord_user.bot,
      discriminator: discord_user.discriminator,
      name: discord_user.name,
      avatar: discord_user.avatar,
    }
  }
}

impl Queryable<users::SqlType, Pg> for DiscordUser {
  type Row = (i32, bool, i32, String, i32);

  fn build((id, bot, discriminator, name, discord_user_id): Self::Row) -> Self {
    DiscordUser {
      database_id: Some(id),
      bot,
      name,
      discriminator: discriminator as u16,
      avatar: None,
      discord_user_id: UserId(discord_user_id as u64),
    }
  }
}

impl From<DiscordUser> for UserRating {
  fn from(record: DiscordUser) -> UserRating {
    UserRating {
      id: None,
      user_id: record.database_id.unwrap(),
      rating: None,
      deviation: None,
      volatility: None,
      game_mode_id: 0,
    }
  }
}
