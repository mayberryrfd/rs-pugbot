use crate::consume_message;
use crate::models::game::Game;
use crate::traits::has_members::HasMembers;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::user::User;

#[command]
pub fn remove(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
  let mut data = ctx.data.lock();
  let mut game = data.get_mut::<Game>().unwrap();
  remove_member(game, msg, true);
}

pub fn remove_member(
  game: &mut Game,
  msg: &Message,
  send_embed: bool,
) -> Vec<User> {
  let author = msg.author.clone();
  if let Some(embed) = game.draft_pool.remove_member(author) {
    if send_embed {
      consume_message(msg, embed)
    }
  }
  game.draft_pool.members()
}

#[cfg(test)]
mod tests {
  use serde;
  use serde_json;
  use serenity;

  use self::serde::de::Deserialize;
  use self::serde_json::Value;
  use crate::models::draft_pool::DraftPool;
  use crate::models::game::{Game, Phases};
  use crate::{commands, struct_from_json};
  use serenity::model::channel::Message;
  use serenity::model::id::UserId;
  use serenity::model::user::User;
  use std::fs::File;

  fn gen_test_user(id: Option<UserId>) -> User {
    User {
      id: match id {
        Some(user_id) => user_id,
        None => UserId(210),
      },
      avatar: Some("abc".to_string()),
      bot: false,
      discriminator: 1432,
      name: "TestUser".to_string(),
    }
  }

  #[test]
  fn test_remove_member() {
    let message = struct_from_json!(Message, "message");
    let game = &mut Game::new(
      vec![],
      DraftPool::new(vec![gen_test_user(Some(message.author.id))], 12),
      1,
      Vec::new(),
      2,
      6,
    );
    assert_eq!(game.phase, Some(Phases::PlayerRegistration));
    let members = commands::remove::remove_member(game, &message, false);
    assert_eq!(members.len(), 0);
  }
}
