use models::draft_pool::DraftPool;
use models::map::Map as GameMap;
use models::team::Team;
use rand::{thread_rng, Rng};
use serenity::model::channel::Embed;
use serenity::model::id::UserId;
use serenity::utils::Colour;
use std::collections::HashMap;
use std::iter::Cycle;
use std::ops::Range;
use team_id_range;
use traits::phased::Phased;
use typemap::Key;

pub struct Game {
  pub active_map: Option<GameMap>,
  pub eligible_voter_ids: Vec<UserId>,
  pub map_choices: Vec<GameMap>,
  pub map_votes: HashMap<i32, i32>,
  pub draft_pool: DraftPool,
  pub game_mode_id: i32,
  pub phase: Option<Phases>,
  pub teams: Option<Vec<Team>>,
  pub turn_number: usize,
  pub turn_taker: Cycle<Range<usize>>,
}

#[derive(PartialEq, Debug)]
pub enum Phases {
  PlayerRegistration,
  CaptainSelection,
  PlayerDrafting,
  MapSelection,
  ResultRecording,
}

#[derive(PartialEq)]
pub enum Outcome {
  Win,
  Loss,
  Draw,
}

impl Game {
  pub fn new(
    teams: Option<Vec<Team>>,
    draft_pool: DraftPool,
    mode_id: i32,
    map_choices: Vec<GameMap>,
  ) -> Game {
    let members = draft_pool.members.clone();

    Game {
      active_map: None,
      draft_pool: draft_pool,
      eligible_voter_ids: members.iter().map(|m| m.id).collect(),
      game_mode_id: mode_id,
      map_choices: map_choices,
      map_votes: [(1, 0), (2, 0), (3, 0), (4, 0), (5, 0)]
        .iter()
        .cloned()
        .collect(),
      phase: Some(Phases::PlayerRegistration),
      teams: teams,
      turn_number: 1,
      turn_taker: team_id_range().cycle(),
    }
  }

  pub fn next_team_to_draft(&mut self) -> Option<Team> {
    let next_team = self.turn_taker.next().unwrap();
    self.team_by_id(next_team)
  }

  pub fn team_by_id(&self, id: usize) -> Option<Team> {
    match self.teams {
      Some(ref teams) => teams.iter().find(|t| t.id == id).cloned(),
      None => None,
    }
  }

  pub fn select_captains(&mut self) -> Result<(), &'static str> {
    if self.phase != Some(Phases::CaptainSelection) {
      return Err("We aren't picking captains, yet!");
    }

    let mut rng = thread_rng();
    let teams: Vec<Team> = team_id_range()
      .map(|i| {
        let pool = self.draft_pool.available_players.clone();
        let keys: Vec<&usize> = pool.keys().collect();
        let random_key: &usize = rng.choose(&[keys]).unwrap().first().unwrap();

        if let Some(user) = self.draft_pool.pop_available_player(random_key) {
          Some(Team {
            id: i,
            captain: Some(user.clone()),
            members: vec![user],
            glicko2_ratings: vec![],
          })
        } else {
          None
        }
      })
      .filter(|t| t.is_some())
      .map(|t| t.unwrap())
      .collect();

    self.teams = Some(teams.clone());
    self.next_phase();
    Ok(())
  }

  pub fn drafting_complete_embed(
    &mut self,
    r: u8,
    g: u8,
    b: u8,
  ) -> Option<Embed> {
    let roster: Vec<String> = self
      .teams
      .clone()
      .unwrap()
      .iter()
      .map(|team| {
        let member_names: Vec<String> =
          team.members.iter().map(|user| user.clone().name).collect();
        format!("Team {} roster:\n{}", team.id, member_names.join("\n"))
      })
      .collect();

    if self.phase == Some(Phases::PlayerDrafting) {
      self.next_phase();
    }

    Some(Embed {
      author: None,
      colour: Colour::from_rgb(r, g, b),
      description: Some(roster.join("\n---\n")),
      footer: None,
      fields: Vec::new(),
      image: None,
      kind: "rich".to_string(),
      provider: None,
      thumbnail: None,
      timestamp: None,
      title: Some(String::from("Drafting has been completed!")),
      url: None,
      video: None,
    })
  }

  pub fn register_vote(&mut self, user_id: UserId) {
    self.eligible_voter_ids.retain(|&id| id != user_id);
  }

  pub fn map_winner_embed(&self, r: u8, g: u8, b: u8) -> Option<Embed> {
    let map_name = &self.active_map;
    Some(Embed {
      author: None,
      colour: Colour::from_rgb(r, g, b),
      description: Some(format!("The winning map is {:?}!", map_name)),
      footer: None,
      fields: Vec::new(),
      image: None,
      kind: "rich".to_string(),
      provider: None,
      thumbnail: None,
      timestamp: None,
      title: Some(format!("The winning map is {:?}!", map_name)),
      url: None,
      video: None,
    })
  }

  pub fn map_selection_embed(&self, r: u8, g: u8, b: u8) -> Option<Embed> {
    let maps: Vec<String> = self.map_choices.iter().enumerate().fold(
      vec![String::from(
        "Typing `~mv <#>` will register your map vote (You must be on a team to vote)",
      )],
      |mut acc, (index, map)| {
        acc.push(format!("{} -> {}", index + 1, map.map_name));
        acc
      },
    );
    Some(Embed {
      author: None,
      colour: Colour::from_rgb(r, g, b),
      description: Some(maps.join("\n")),
      footer: None,
      fields: Vec::new(),
      image: None,
      kind: "rich".to_string(),
      provider: None,
      thumbnail: None,
      timestamp: None,
      title: Some("Time to pick a map!".to_string()),
      url: None,
      video: None,
    })
  }
}

impl Phased for Game {
  fn next_phase(&mut self) {
    self.phase = match self.phase {
      Some(Phases::PlayerRegistration) => {
        self.draft_pool.generate_available_players();
        self.draft_pool.members = Vec::new();
        Some(Phases::CaptainSelection)
      }
      Some(Phases::CaptainSelection) => {
        self.select_captains();
        Some(Phases::PlayerDrafting)
      }
      Some(Phases::PlayerDrafting) => {
        self.turn_number = 1;
        self.turn_taker = team_id_range().cycle();
        Some(Phases::MapSelection)
      }
      Some(Phases::MapSelection) => {
        let mut winning_map_index: i32 = 0;
        let mut winning_vote_amount: i32 = 0;
        for (key, val) in self.map_votes.iter() {
          if *val > winning_vote_amount {
            winning_map_index = *key;
            winning_vote_amount = *val;
          }
        }
        let choice = &self.map_choices[winning_map_index as usize - 1];
        self.active_map = Some(GameMap {
          game_title_id: choice.game_title_id,
          map_name: choice.map_name.clone(),
        });
        Some(Phases::ResultRecording)
      }
      _ => None,
    };
  }

  fn previous_phase(&mut self) {
    self.phase = match self.phase {
      Some(Phases::CaptainSelection) => Some(Phases::PlayerRegistration),
      Some(Phases::PlayerDrafting) => Some(Phases::CaptainSelection),
      Some(Phases::MapSelection) => Some(Phases::PlayerDrafting),
      Some(Phases::ResultRecording) => Some(Phases::MapSelection),
      _ => None,
    };
  }

  fn reset_phase(&mut self) {
    self.phase = Some(Phases::PlayerRegistration);
  }
}

impl Key for Game {
  type Value = Game;
}
