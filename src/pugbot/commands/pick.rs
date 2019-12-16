use crate::consume_message;
use crate::models::game::{Game, Phases};

use crate::commands::error_embed;
use crate::traits::has_members::HasMembers;
use crate::traits::phased::Phased;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

#[command]
pub fn pick(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
  let user_index = args.single::<usize>().unwrap();
  let mut data = ctx.data.lock();
  let game = data.get_mut::<Game>().unwrap();
  draft_player(game, msg, true, user_index)?;
}

pub fn draft_player(
  game: &mut Game,
  msg: &Message,
  send_embed: bool,
  user_index: usize,
) -> Result<(), &'static str> {
  if game.phase != Some(Phases::PlayerDrafting) && send_embed {
    let err = "We're not drafting right now!";
    consume_message(msg, error_embed(err));
    return Err(err);
  }

  if let Some(user) = game.draft_pool.pop_available_player(&user_index) {
    let next_id = game.turn_taker.next().unwrap();
    game.teams[next_id as usize].add_member(user);
  } else {
    let err =
      "The user selected for drafting has been drafted or is otherwise invalid";
    if send_embed {
      consume_message(msg, error_embed(err));
    }
    return Err(err);
  }

  game.next_phase();

  if game.phase == Some(Phases::MapSelection) && send_embed {
    consume_message(msg, game.drafting_complete_embed(165, 255, 241).unwrap());
    consume_message(msg, game.map_selection_embed(164, 255, 241).unwrap());
  }
  Ok(())
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
  use serde;
  use serde_json;
  use serenity;

  use self::serde::de::Deserialize;
  use self::serde_json::Value;
  use crate::models::draft_pool::DraftPool;
  use crate::models::game::{Game, Phases};
  use crate::traits::phased::Phased;
  use crate::{commands, struct_from_json};
  use serenity::model::channel::Message;
  use serenity::model::user::User;
  use std::fs::File;

  #[test]
  fn test_pick_player() {
    let authors: Vec<User> = struct_from_json!(Vec, "authors");
    let (team_count, team_size) = (2, (authors.len() / 2) as u32);
    // Choosing 2 teams of 5 here since there are 10 authors in authors.json
    let game = &mut Game::new(
      vec![],
      DraftPool::new(authors, team_count * team_size),
      1,
      Vec::new(),
      team_count,
      team_size,
    );
    game.next_phase();
    // Check that captain-selection has finished before proceeding as this is a
    // precondition of the test.
    assert_eq!(game.select_captains(), Ok(()));

    // Make a random selection from available players
    let pool = game.draft_pool.available_players.clone();

    if let Some(key) = pool.keys().next() {
      if let Some(_user) = game.draft_pool.available_players.get(key) {
        let message = struct_from_json!(Message, "message");
        // Drafting a single player works as expected?
        assert_eq!(
          commands::pick::draft_player(game, &message, false, *key),
          Ok(())
        );
      }
    }

    // There should be as many teams as specified.
    assert_eq!(game.teams.len() as u32, team_count);
    // The order in which teams choose seems to be non-deterministic, so instead
    // of checking that team 1 has x members and team 2 has x+1 members, just
    // test their member counts equal 3.
    let member_count = &game
      .teams
      .iter()
      .fold(0, |acc, team| acc + team.members.len());
    assert_eq!(*member_count as u32, 3);
  }

  #[test]
  fn test_full_teams() {
    let authors: Vec<User> = struct_from_json!(Vec, "authors");
    let message = struct_from_json!(Message, "message");
    let (team_count, team_size) = (2, (authors.len() / 2) as u32);
    let game = &mut Game::new(
      vec![],
      DraftPool::new(authors, team_count * team_size),
      1,
      Vec::new(),
      team_count,
      team_size,
    );
    game.next_phase();
    game.select_captains();

    let player_pool = game.draft_pool.available_players.clone();
    for (key, _) in player_pool.iter() {
      commands::pick::draft_player(game, &message, false, *key);
    }
    // available_players should be empty. Each drafted player is popped out of
    // the available_players pool.
    assert_eq!(game.draft_pool.available_players.len(), 0);
    // Since all players were drafted and teams are now full, the game should
    // proceed to the next phase.
    assert_eq!(game.phase, Some(Phases::MapSelection));
    let member_count = &game
      .teams
      .iter()
      .fold(0, |acc, team| acc + team.members.len());
    // Final post-condition: The sum of the counts of each team's roster should
    // equal the max size of the draft pool.
    assert_eq!(*member_count as u32, team_count * team_size);
  }
}
