use std::usize;

use marjapussi::game::gameevent::{ActionType, GameCallback};
use marjapussi::game::gameinfo::GameFinishedInfo;
use marjapussi::game::player;
use marjapussi::game::points::{points_pair, Points};
use marjapussi::game::Game;
use marjapussi::game::gamestate::{FinishedTrick, GamePhase};

use crate::ai::MarjapussiCheater;
use crate::cheater::CheaterV1;

pub fn four_cheaters() {
    let game_name = String::from("Cheater Game");
    let player_names = [String::from("Player 1"), String::from("Player 2"), String::from("Player 3"), String::from("Player 4")];
    let mut players: Vec<CheaterV1> = player_names.iter().enumerate().map(|(place, name)| {CheaterV1::new(name, place.try_into().unwrap())}).collect();
    let mut game = Game::new(game_name, player_names, None);
    let mut player_at_turn;

    // print the start cards of each player
    println!("\nStart cards:");
    for player in &game.state.players {
        println!("{}: {:?}", player.place_at_table.0, player.cards);
    }

    // iterate through the game step for step
    while game.state.phase != GamePhase::Ended {

        // find out which player is at turn
        player_at_turn = game.state.player_at_turn().place_at_table.0;

        // print the current state of the game
        match game.state.phase {
            GamePhase::StartTrick => println!("\nPlayer {} starting new trick", player_at_turn),
            GamePhase::Bidding => println!("\nPlayer {}: thinking about the next bidding step", player_at_turn),
            GamePhase::Raising => println!("\nRaising or starting first trick"),
            _ => ()
        }
        // print the player's cards
        match game.state.phase {
            GamePhase::WaitingForStart => (),
            GamePhase::Bidding => println!("  cards: {:?}", game.state.player_at_place(player::PlaceAtTable(player_at_turn)).cards),
            _ => println!("{}: cards: {:?}", player_at_turn, game.state.player_at_place(player::PlaceAtTable(player_at_turn)).cards)
        }

        // let the player choose an action
        let chosen_action = players[usize::from(player_at_turn)].select_action(game.clone());

        // print the chosen action
        match game.state.phase {
            GamePhase::WaitingForStart => (),
            GamePhase::Bidding => println!("  {:?}", chosen_action.action_type),
            _ => println!("{:?}", chosen_action.action_type)
        }
        
        // apply the chosen action to the game
        game.apply_action_mut(chosen_action.clone());

        // if a trick was just finished: print the winner
        if let Some(last_trick) = game.state.all_tricks.last() {
            if chosen_action.action_type == ActionType::CardPlayed(last_trick.cards.last().unwrap().clone()) {
                println!("Trick goes to {}", last_trick.winner.0);
            }
        }
    }

    // print the final info
    println!("\nFinal info:");
    let final_info = GameFinishedInfo::from(game.clone());
    print_evaluation(&game, &final_info);
}

fn print_evaluation(game: &Game, game_finished_info: &GameFinishedInfo) {
    /*
        Mainly copied from super::cheater::search_state::AlphaBetaGameState::evaluate
    */

    // this will hold the points each player made
    let mut players_points = [Points(0); 4];

    // calculate the points from winning tricks
    let mut players_tricks: [Vec<FinishedTrick>; 4] = [vec![], vec![], vec![], vec![]];
    for trick in &game.state.all_tricks {
        players_tricks[trick.winner.0 as usize].push(trick.clone());
        players_points[trick.winner.0 as usize] += trick.points;
    }

    // points from winning the last trick (+20)
    let last_trick = game.state.all_tricks.last().unwrap();
    players_points[last_trick.winner.0 as usize] += Points(20);

    // calculate the points from announcing pairs
    // also get the playing party
    let mut playing_party = None;
    for event in &game.all_events {
        if ActionType::NewBid(game.state.value.0) == event.last_action.action_type {
            playing_party = Some(event.last_action.player.0 % 2);
        }
        if let Some(GameCallback::NewTrump(suit)) = event.callback {
            players_points[event.last_action.player.clone().0 as usize] +=
                points_pair(suit);
        }
    }

    // calculate the points each party reached
    let party_points: [i32; 2] = [
        players_points[0].0 + players_points[2].0,
        players_points[1].0 + players_points[3].0
    ];
    let game_value = game.state.value.0;

    // find out if this was a schwarz game
    let tricks_party_zero = players_tricks[0].len() + players_tricks[2].len();
    let schwarz_game = tricks_party_zero == 0 || tricks_party_zero == 9;

    // print info
    if schwarz_game {
        println!("- schwarz game");
    }
    if let Some(playing) = playing_party {
        let non_playing = (playing + 1) % 2;
        println!("- playing party ({}): {}/{} points", playing, party_points[playing as usize], game_value);
        println!("- non-playing party ({}): {} points", non_playing, party_points[non_playing as usize]);
        println!("- playing party {}", if game_finished_info.won.unwrap() {"won"} else {"lost"})
    } else {
        println!("- no playing party");
        for i in 0..2 {
            println!("- party ({}): {} points", i, party_points[i as usize])
        }
    }
}