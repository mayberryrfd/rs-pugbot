use crate::commands::error_embed;
use crate::consume_message;
use crate::models::game::{Game, Phases};
use crate::traits::phased::Phased;
use serenity::client::Context;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::channel::Message;

#[command]
pub fn mapvote(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
  let mut data = ctx.data.lock();
  let game = data.get_mut::<Game>().unwrap();
  map_vote(game, msg, true, args.single::<usize>()? as i32)?;
}

#[allow(unused_must_use)]
pub fn map_vote(
  game: &mut Game,
  msg: &Message,
  send_embed: bool,
  map_index: i32,
) -> Result<(), &'static str> {
  if game.phase != Some(Phases::MapSelection) {
    let err = "We're not picking maps right now!";

    if send_embed {
      consume_message(msg, error_embed(err));
    }

    return Err(err);
  }

  if !game.draft_pool.members.contains(&msg.author) && send_embed {
    match msg.author.direct_message(|m| m.content(
      "Sorry, but you're not allowed to map vote because you're not registered to play!"
    )) {
      Ok(_) => {
        msg.reply("You're welcome");
        Ok(())
      },
      Err(why) => {
        println!("Error sending message: {:?}", why);
        let err = "Had some kind of problem sending you a message.";
        msg.reply(err);
        consume_message(msg, error_embed(err));
        Err(err)
      }
    }
  } else {
    if let Some(vote_count) = game.map_votes.clone().get(&map_index) {
      game.map_votes.insert(map_index, vote_count + 1);
      game.register_vote(msg.author.id);
      game.next_phase();

      if game.phase == Some(Phases::ResultRecording) && send_embed {
        consume_message(msg, game.map_winner_embed(164, 255, 241).unwrap());
      }
      Ok(())
    } else {
      let err = "Invalid map selection.";

      if send_embed {
        consume_message(msg, error_embed(err));
      }

      Err(err)
    }
  }
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
  use crate::models::map::Map as GameMap;
  use crate::traits::phased::Phased;
  use crate::{commands, struct_from_json};
  use serenity::model::channel::Message;
  use serenity::model::user::User;
  use std::fs::File;

  #[test]
  fn test_register_vote() {
    let authors: Vec<User> = struct_from_json!(Vec, "authors");
    let maps: Vec<GameMap> = struct_from_json!(Vec, "maps");
    // Choosing 2 teams of 5 here since there are 10 authors in authors.json
    let (team_count, team_size) = (2, (authors.len() / 2) as u32);
    let game = &mut Game::new(
      vec![],
      DraftPool::new(authors, team_count * team_size),
      1,
      maps,
      team_count,
      team_size,
    );
    game.next_phase();
    game.select_captains();

    let player_pool = game.draft_pool.available_players.clone();
    let message = struct_from_json!(Message, "message");

    // Populate the draft pool.
    for (key, _) in player_pool.iter() {
      commands::pick::draft_player(game, &message, false, *key);
    }

    // This is the key of the game map we're voting for in this test.
    let candidate_map_idx = 1;
    let mut counter = 0;

    // We register a map vote for each player here.
    for _ in 0..(team_count * team_size) {
      // Precondition. We should be in the right phase every time.
      assert_eq!(game.phase, Some(Phases::MapSelection));
      // Precondition. The count of votes should be what we expect.
      assert_eq!(game.map_votes.get(&candidate_map_idx), Some(&counter));
      commands::mapvote::map_vote(game, &message, false, candidate_map_idx);
      // Postcondition. The count of votes for this particular map should be one
      // higher now.
      assert_eq!(game.map_votes.get(&candidate_map_idx), Some(&(counter + 1)));
      counter += 1;
    }

    let vote_counts: i32 = game
      .map_votes
      .values()
      .clone()
      .fold(0, |acc, val| acc + *val);
    // The total number of votes should now equal the total number of players.
    assert_eq!(vote_counts as u32, team_count * team_size);
    // The number of votes for our candidate should be all the votes. (No other
    // maps should have votes)
    assert_eq!(game.map_votes.get(&candidate_map_idx), Some(&vote_counts));
    // The game should advance to the next phase since all the votes have been
    // tallied.
    assert_eq!(game.phase, Some(Phases::ResultRecording));
  }
}
