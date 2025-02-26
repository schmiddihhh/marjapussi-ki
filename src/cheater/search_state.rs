use crate::alpha_beta::State;

use std::vec;

use marjapussi::game::Game;
use marjapussi::game::gameevent::{ActionType, AnswerType, GameAction, GameCallback};
use marjapussi::game::gamestate::FinishedTrick;
use marjapussi::game::player::PlaceAtTable;
use marjapussi::game::points::{points_pair, Points};

// a node in the search tree
pub struct AlphaBetaGameState {
    pub owning_player: PlaceAtTable,
    pub game: Game,
    pub depth: i32,
    pub debugging_depth: i32
}

impl AlphaBetaGameState {
    fn legal_moves_unordered(&self) -> Vec<GameAction> {
        // UndoRequests are useless for our tree search and create infinite paths
        // "useless questions" are questions that don't change the trump and therefore lead to evaluating the same subtree multiple times
        self.game.legal_actions()
            .into_iter()
            .filter(|action| {
                action.action_type != ActionType::UndoRequest && 
                !matches!(action.action_type, ActionType::NewBid(_)) &&     // this sorts out all raising actions
                !self.useless_question(action)
            })
            .collect()
    }
    fn order_moves(&self, mut moves: Vec<GameAction>, do_sort: bool) -> Vec<GameAction> {
        // println!("before ordering: {:?}", moves);
        if do_sort {
            // println!("sorting");
            moves.sort_by_key(|move_| {
                let game_after_move = self.apply_move(move_);
                game_after_move.legal_moves_unordered().len()
            });
            // println!("done sorting");
            // println!("after ordering: {:?}", moves);
        }
        moves
    }
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

        // debug prints to follow the process in the search tree
        if self.depth <= self.debugging_depth {
            for _ in 0..self.depth {
                print!(" ");
            }
            println!("{}", self.depth);
        }

        // get the legal moves
        let legal_moves = self.legal_moves_unordered();
        // order the legal moves according to a heuristic

        // the current implementation of move ordering slows the execution down, probably too much overhead and bad heuristic
        // thus, ordering is deactivated
        self.order_moves(legal_moves, false)
    }

    fn apply_move(&self, next_move: &GameAction) -> Self {
        AlphaBetaGameState {
            owning_player: self.owning_player.clone(),
            game: self.game.apply_action(next_move.clone()).unwrap(),
            depth: self.depth + 1,
            debugging_depth: self.debugging_depth
        }
    }

    fn is_leaf(&self) -> bool {
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
        // -> this check is deactivated to enable a limited search depth by evaluating inner nodes
        // if self.game.state.phase != GamePhase::Ended {
        //     panic!("Tried to evaluate an unfinished game");
        // }
        
        // this will hold the points each player made
        let mut players_points = [Points(0); 4];

        // calculate the points from winning tricks
        let mut players_tricks: [Vec<FinishedTrick>; 4] = [vec![], vec![], vec![], vec![]];
        for trick in &self.game.state.all_tricks {
            players_tricks[trick.winner.0 as usize].push(trick.clone());
            players_points[trick.winner.0 as usize] += trick.points;
        }

        // points from winning the last trick (+20)
        let last_trick = self.game.state.all_tricks.last().unwrap();
        players_points[last_trick.winner.0 as usize] += Points(20);

        // calculate the points from announcing pairs
        // also get the playing party
        let mut playing_party = None;
        for event in &self.game.all_events {
            if ActionType::NewBid(self.game.state.value.0) == event.last_action.action_type {
                playing_party = Some(event.last_action.player.0 % 2);
            }
            if let Some(GameCallback::NewTrump(suit)) = event.callback {
                players_points[event.last_action.player.clone().0 as usize] +=
                    points_pair(suit);
            }
        }

        // calculate the points each party reached
        let own_party_points = players_points[self.owning_player.0 as usize].0
            + players_points[self.owning_player.partner().0 as usize].0;
        let opponent_party_points = players_points[self.owning_player.next().0 as usize].0
            + players_points[self.owning_player.next().partner().0 as usize].0;
        let game_value = &self.game.state.value.0;

        // find out if this was a schwarz game
        let tricks_party_zero = players_tricks[0].len() + players_tricks[2].len();
        let schwarz_game = tricks_party_zero == 0 || tricks_party_zero == 9;

        // final point difference between the own party and the opponent party
        let own_party = self.owning_player.0 % 2;
        let opponent_party = (own_party + 1) % 2;
        if playing_party == None {

            // if nobody played: every party gets the points it reached
            own_party_points - opponent_party_points

        } else if playing_party == Some(own_party) {
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
        } else if playing_party == Some(opponent_party) {
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