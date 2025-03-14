mod optimizations;

use crate::alpha_beta::State;

use std::collections::HashMap;
use std::vec;

use marjapussi::game::cards::Card;
use marjapussi::game::gamestate::GamePhase;
use marjapussi::game::player::PlaceAtTable;
use marjapussi::game::{legal_actions, Game};
use marjapussi::game::gameevent::{ActionType, AnswerType, GameAction, GameCallback};
use marjapussi::game::points::{points_pair, Points};


// a node in the search tree
pub struct AlphaBetaGameState {
    owning_player: PlaceAtTable,
    game: Game,
    remaining_cards: Vec<Card>,
    points_per_party: [i32; 2],
    tricks_per_party: [i8; 2],
    playing_party: Option<u8>
}

impl AlphaBetaGameState {
    pub fn new(owning_player: PlaceAtTable, game: Game) -> Self {
        /*
            Creates a new AlphaBetaGameState from a marjapussi game.
        */

        // this function is only valid in the raising and cardplay phase, not while bidding or passing
        match game.state.phase {
            GamePhase::WaitingForStart | 
            GamePhase::Bidding | 
            GamePhase::PassingForth | 
            GamePhase::PassingBack | 
            GamePhase::Ended => panic!("AlphaBetaGameState can only be created from a game that is past the passing phase."),
            _ => ()
        }

        // join all players' cards into one vector and sort them ascendingly
        let mut remaining_cards = vec![];
        for player in &game.state.players {
            remaining_cards.extend(player.cards.clone());
        }
        remaining_cards.sort();
        
        // get the current points in the game
        // this will hold the points of each player
        let mut players_points = [Points(0); 4];
        // here we store how many tricks each player got
        let mut players_tricks: [i8; 4] = [0, 0, 0, 0];
        // calculate the points from winning tricks and count the tricks per player
        for trick in &game.state.all_tricks {
            players_tricks[trick.winner.0 as usize] += 1;
            players_points[trick.winner.0 as usize] += trick.points;
        }
        // points from winning the last trick (+20)
        if game.state.all_tricks.len() == 9 {
            let last_trick = game.state.all_tricks.last().unwrap();
            players_points[last_trick.winner.0 as usize] += Points(20);
        }
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
        // add the player points together to get the party points
        let points_per_party = [
            players_points[0].0 + players_points[2].0,
            players_points[1].0 + players_points[3].0
        ];
        // find out how many tricks each party got
        let tricks_per_party = [
            players_tricks[0] + players_tricks[2],
            players_tricks[1] + players_tricks[3]
        ];

        // create the AlphaBetaGameState
        AlphaBetaGameState {
            owning_player,
            game,
            remaining_cards,
            points_per_party,
            tricks_per_party,
            playing_party
        }
    }


    fn legal_moves_unordered(&self) -> Vec<GameAction> {

        // we sort out some irrelevant moves:
        // UndoRequests are useless for our tree search and create infinite paths
        // we will just never raise at this point, so all raising actions can be sorted out
        // "useless questions" are questions that don't change the trump and therefore lead to evaluating the same subtree multiple times
        let legal_actions: Vec<GameAction> = self.game.legal_actions()
            .into_iter()
            .filter(|action| {
                action.action_type != ActionType::UndoRequest && 
                !matches!(action.action_type, ActionType::NewBid(_)) &&
                !self.useless_question(action)
            })
            .collect();

        unsafe {
            super::COUNT_NODES_PER_CHILDREN[legal_actions.len() as usize] += 1;
        }

        legal_actions
        
        // // if two cards have no other cards between them, they are equally strong in cardplay
        // // if such equivalences exist, we sort out all of these equivalent cards except the weakest one
        // // for now, this is only applied to low cards, since they all give the same amount of points (0)

        // // first, we have to separate cardplay moves from all other moves
        // let mut legal_cardplay_actions = vec![];
        // let mut legal_other_actions = vec![];
        // legal_actions
        //     .into_iter()
        //     .for_each(|action| {
        //         if matches!(action.action_type, ActionType::CardPlayed(_)) {
        //             legal_cardplay_actions.push(action);
        //         } else {
        //             legal_other_actions.push(action);
        //         }
        //     });

        // // if there are no legal cardplay actions, we can skip this
        // // else, we will now sort out equivalent moves
        // if legal_cardplay_actions.len() > 0 {

        //     // the legal cardplay actions have to be sorted ascendingly, since the remaining cards are too
        //     legal_cardplay_actions
        //         .sort_by_key(|action| {
        //             if let ActionType::CardPlayed(card) = action.action_type {
        //                 action
        //             } else {
        //                 panic!("action should be a cardplay action, but something else was given")
        //             }
        //         });

        //     // iterate through the remaining cards and drop equivalent actions
        //     let mut iter_cardplay = legal_cardplay_actions.iter();
        //     let mut current_action = iter_cardplay.next().unwrap();
        //     let mut last_matched = false;
        //     for current_remaining in &self.remaining_cards {
        //         if matches!(current_action.action_type, ActionType::CardPlayed()) {
        //             if last_matched == true {

        //             }
        //         }
        //     }
        // }


        // let mut legal_actions_cut = legal_cardplay_actions;
        // legal_actions_cut.extend(legal_other_actions);
        // legal_actions_cut
    }


    // fn order_moves(&self, mut moves: Vec<GameAction>, do_sort: bool) -> Vec<GameAction> {
    //     // println!("before ordering: {:?}", moves);
    //     if do_sort {
    //         // println!("sorting");
    //         moves.sort_by_key(|move_| {
    //             let game_after_move = self.apply_move(move_);
    //             game_after_move.legal_moves_unordered().len()
    //         });
    //         // println!("done sorting");
    //         // println!("after ordering: {:?}", moves);
    //     }
    //     moves
    // }


    fn useless_question(&self, action: &GameAction) -> bool {
        // only questions can be useless questions
        matches!(action.action_type, ActionType::Question(_)) && {
            // this section checks if the question would change the trump
            let game_after_question = self.game.apply_action(action.clone()).unwrap();
            let answers = game_after_question.legal_actions();
            if answers.len() == 1 {
                // check if the answer of the partner would change the trump
                let game_after_answer = game_after_question.apply_action(answers[0].clone()).unwrap();
                let last_event = game_after_answer.all_events.last().unwrap();
                let last_action = &last_event.last_action.action_type;
                let last_callback_option = &last_event.callback;

                if matches!(last_action, ActionType::Answer(AnswerType::YesPair(_))) ||
                    matches!(last_action, ActionType::Answer(AnswerType::YesHalf(_))) && matches!(last_callback_option, Some(GameCallback::NewTrump(_))) {
                    // the trump would be changed -> the question is not useless
                    false
                } else {
                    // this is a useless question
                    true
                }
            } else {
                // if there are multiple answers, the partner has multiple pairs he can announce -> no pruning should occur
                false
            }
        }
    }
}

impl State<GameAction> for AlphaBetaGameState {
    fn legal_moves(&self) -> Vec<GameAction> {

        // // debug prints to follow the process in the search tree
        // if self.depth <= self.debugging_depth {
        //     for _ in 0..self.depth {
        //         print!(" ");
        //     }
        //     println!("{}", self.depth);
        // }

        // get the legal moves
        let legal_moves = self.legal_moves_unordered();
        unsafe {
            // execution is serial, so at most one AlphaBetaGameState will execute this at a time
            super::COUNT_CHILDREN += legal_moves.len() as u64;
        }
        legal_moves
        // order the legal moves according to a heuristic

        // the current implementation of move ordering slows the execution down, probably too much overhead and bad heuristic
        // thus, ordering is deactivated
        // self.order_moves(legal_moves, false)
    }

    fn apply_move(&self, next_move: &GameAction) -> Self {

        // apply the move to the game
        let new_game = self.game.apply_action(next_move.clone()).unwrap();

        // if a card is played, remove this card from the remaining cards
        let resulting_event = new_game.all_events.last().unwrap();
        let mut new_remaining_cards = self.remaining_cards.clone();
        if let ActionType::CardPlayed(played_card) = resulting_event.last_action.action_type.clone() {
            new_remaining_cards.remove(new_remaining_cards.iter().position(|card| *card == played_card).unwrap());
        }

        // find out if the last action finished a trick
        // if yes: add the points and the achieved trick to the party
        let mut new_points = self.points_per_party;
        let mut new_tricks = self.tricks_per_party;
        if new_game.state.phase == GamePhase::StartTrick || new_game.state.phase == GamePhase::Ended {
            let last_trick = new_game.state.all_tricks.last().unwrap();
            let trick_winner_party = last_trick.winner.0 % 2;
            new_points[trick_winner_party as usize] += last_trick.points.0;
            new_tricks[trick_winner_party as usize] += 1;
            // extra 20 points for the last trick
            if new_game.state.phase == GamePhase::Ended {
                new_points[trick_winner_party as usize] += 20;
            }
        }

        // if a pair was just announced: calculate the points of this pair
        if let Some(GameCallback::NewTrump(announced_pair)) = resulting_event.callback {
            new_points[usize::from(resulting_event.last_action.player.0 % 2)] += points_pair(announced_pair).0
        }

        // create the resulting gamestate
        AlphaBetaGameState {
            owning_player: self.owning_player.clone(),
            game: new_game,
            remaining_cards: new_remaining_cards,
            points_per_party: new_points,
            tricks_per_party: new_tricks,
            playing_party: self.playing_party
        }
    }

    fn is_leaf(&self) -> bool {
        unsafe {
            // execution is serial, so at most one AlphaBetaGameState will execute this at a time
            super::COUNT_NODES += 1;
        }
        self.game.ended()
    }

    fn is_maximizing(&self) -> bool {
        self.game.state.player_at_turn.0 % 2 == self.owning_player.0 % 2
    }

    fn evaluate(&self) -> i32 {
        /* 
            Evaluate the outcome of the game by calculating the point difference between the own party and the opponent party.
        */

        // normally, this evaluation is only useful if the game is completed
        // a complete exploration of the search tree is infeasible with the current implementation
        // -> the following check is deactivated to enable a limited search depth by evaluating inner nodes
        // if self.game.state.phase != GamePhase::Ended {
        //     panic!("Tried to evaluate an unfinished game");
        // }


        // get the important values
        let own_party_points = self.points_per_party[(self.owning_player.0 % 2) as usize];
        let opponent_party_points = self.points_per_party[((self.owning_player.0 + 1) % 2) as usize];
        let game_value = &self.game.state.value.0;

        // find out if this was a schwarz game
        let tricks_party_zero = self.tricks_per_party[0];
        let schwarz_game = tricks_party_zero == 0 || tricks_party_zero == 9;

        // final point difference between the own party and the opponent party
        let own_party = self.owning_player.0 % 2;
        let opponent_party = (own_party + 1) % 2;
        if self.playing_party == None {

            // if nobody played: every party gets the points it reached
            own_party_points - opponent_party_points

        } else if self.playing_party == Some(own_party) {
            if !schwarz_game {
                if own_party_points >= *game_value {
                    // we played and won the game without playing schwarz
                    *game_value - opponent_party_points
                } else {
                    // we played the game, but lost it; no one played schwarz
                    - *game_value - opponent_party_points
                }
            } else {
                if opponent_party_points == 0 {
                    // we played the game and won schwarz
                    *game_value + 2 * *game_value
                } else {
                    // we played the game, but got played schwarz
                    - 2 * game_value - opponent_party_points
                }
            }
        } else if self.playing_party == Some(opponent_party) {
            if !schwarz_game {
                if opponent_party_points >= *game_value {
                    // the opponents played and won without playing schwarz
                    own_party_points - *game_value
                } else {
                    // the opponents played and lost, nobody played schwarz
                    own_party_points + *game_value
                }
            } else {
                if opponent_party_points == 0 {
                    // we played the opponents schwarz
                    *game_value + 2 * *game_value
                } else {
                    // the opponents played us schwarz
                    - 2 * game_value - *game_value
                }
            }
        } else {
            panic!("Invalid playing party")
        }
    }
}