use consume_message;
use models::game::{Game, Phases};
use queue_size;
use serenity::model::channel::Message;
use team_count;
use traits::has_members::HasMembers;
use traits::phased::Phased;

command!(pick(ctx, msg, args) {
  let user_index = args.single::<usize>().unwrap();
  let mut data = ctx.data.lock();
  let game = data.get_mut::<Game>().unwrap();
  pick_user(game, msg, user_index, true)
});

pub fn pick_user(
  game: &mut Game,
  msg: &Message,
  user_index: usize,
  send_embed: bool,
) {
  if game.phase != Some(Phases::PlayerDrafting) {
    panic!("We're not drafting right now!");
  }

  let user = game.draft_pool.pop_available_player(&user_index).unwrap();
  game.next_team_to_draft().add_member(user);

  let max_turns: u32 = queue_size() - team_count().unwrap();

  if game.turn_number == max_turns as usize {
    game.next_phase();

    if send_embed {
      consume_message(
        msg,
        game.drafting_complete_embed(165, 255, 241).unwrap(),
      );
      consume_message(msg, game.map_selection_embed(164, 255, 241).unwrap());
    }
  } else {
    game.turn_number += 1;
  }
}

#[cfg(test)]
mod tests {
  extern crate serde;
  extern crate serde_json;
  extern crate serenity;

  use self::serde::de::Deserialize;
  use self::serde_json::Value;
  use commands;
  use models::draft_pool::DraftPool;
  use models::game::{Game, Phases};
  use serenity::model::channel::Message;
  use serenity::model::id::UserId;
  use serenity::model::user::User;
  use std::fs::File;
  use traits::phased::Phased;

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

  macro_rules! p {
    ($s:ident, $filename:expr) => {{
      let f =
        File::open(concat!("./tests/resources/", $filename, ".json")).unwrap();
      let v = serde_json::from_reader::<File, Value>(f).unwrap();

      $s::deserialize(v).unwrap()
    }};
  }

  #[test]
  #[allow(unused_must_use)]
  fn test_pick_user() {
    let mut message = p!(Message, "message");
    let game = &mut Game::new(
      None,
      DraftPool::new(vec![gen_test_user(None)]),
      1,
      Vec::new(),
    );
    assert_eq!(game.phase, Some(Phases::PlayerRegistration));

    for i in 1..10 {
      println!("{}", i);
      message.author = gen_test_user(Some(UserId(i)));
      commands::add::update_members(game, &message, false);
    }
    assert_eq!(game.draft_pool.members.len(), 10);
    game.next_phase();
    game.select_captains();
    assert_eq!(game.phase, Some(Phases::PlayerDrafting));
    if let Some(ref teams) = game.teams {
      assert_eq!(teams.len(), 2);
      if let Some(ref first_captain) = teams[0].captain {}
    }
  }
}
