mod search_state;
mod bidding;

use search_state::AlphaBetaGameState;
use bidding::BiddingInfos;
use super::ai::MarjapussiCheater;
use crate::alpha_beta::alpha_beta_search;
use marjapussi::game::Game;
use marjapussi::game::gameevent::{ActionType, GameAction};
use marjapussi::game::gamestate::GamePhase;
use marjapussi::game::player::PlaceAtTable;
use std::collections::HashMap;

static mut COUNT_TREES: u32 = 0;
static mut COUNT_NODES: u32 = 0;
static mut COUNT_CHILDREN: u32 = 0;
static mut COUNT_NODES_PER_CHILDREN: [i32; 20] = [0; 20];


pub struct CheaterV1 {
    pub name: String,
    pub position: PlaceAtTable,
    to_communicate: Vec<BiddingInfos>,
    search_depth: u32,
    knowledge: HashMap<String, String>
}


impl CheaterV1 {
    pub fn new(name: &str, position: u8, search_depth: u32) -> Self {
        CheaterV1 {
            name: String::from(name),
            position: PlaceAtTable(position),
            to_communicate: vec![],
            search_depth,
            knowledge: HashMap::new()
        }
    }

    fn bid(&mut self, game: Game, legal_actions: Vec<GameAction>) -> GameAction {
    
        let next_bidding_step = bidding::next_bidding_step(
            &game.state.player_at_turn,
            &game.state.player_at_turn().cards,
            &game.state.partner().cards,
            &game.state.bidding_history,
            &mut self.knowledge,
            &mut self.to_communicate);
        let desired_action = match next_bidding_step {
            0 => ActionType::StopBidding,
            step => ActionType::NewBid(step)
        };
        legal_actions
            .into_iter()
            .find(|action| {
                action.action_type == desired_action
            }).expect("Wanted to bid but the desired step was not in the legal_actions")
    }
}

impl MarjapussiCheater for CheaterV1 {
    fn select_action(&mut self, game: Game) -> GameAction {

        // get the legal actions
        // remove all UndoRequests, since they are irrelevant here and create infinite paths in the search tree
        let legal_actions = game.legal_actions()
            .into_iter()
            .filter(|action| {
                action.action_type != ActionType::UndoRequest
            })
            .collect::<Vec<GameAction>>();

        // make sure that there are any legal actions
        let first_action = legal_actions.get(0).expect("Player was asked to choose an action, but there are no legal actions");
        
        // make sure that we are the correct player to choose
        if self.position != first_action.player {
            // should not occur
            // currently, there is a bug in the framework so the "start" action always has player number 0
            println!("\nALAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARM (player in action and player choosing an action mismatch)");
        }
        if self.position != game.state.player_at_turn {
            println!("\nALAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAARM (player at turn and player choosing an action mismatch)");
        }

        // choose an action
        // if there is only one option, we need no further evaluation
        if legal_actions.len() == 1 {
            legal_actions.into_iter().last().unwrap()
        } else {
            // act according to the current game phase
            match game.state.phase {
                GamePhase::Bidding => self.bid(game, legal_actions),
                // in case the game phase is GamePhase::Raising, we will sort out all raising actions within the search and just play a card
                GamePhase::StartTrick | GamePhase::Trick | GamePhase::Raising => {
                    unsafe {
                        if COUNT_TREES == 1 {
                            print_avg_tree_size();
                        }
                        COUNT_TREES += 1;
                    }
                    alpha_beta_search(AlphaBetaGameState::new(self.position.clone(), game), Some(self.search_depth)).0
                },
                _ => legal_actions.into_iter().nth(0).expect("Player was asked to choose an action, but there are no legal actions")
            }
        }
    }
}


pub unsafe fn print_avg_tree_size() {
    let avg_nodes_per_tree = f64::from(COUNT_NODES) / f64::from(COUNT_TREES);
    let avg_children_per_node = f64::from(COUNT_CHILDREN) / f64::from(COUNT_NODES);
    let mut avg_nodes_per_children = [0.0; 20];
    for (index, num) in COUNT_NODES_PER_CHILDREN.iter().enumerate() {
        avg_nodes_per_children[index] += f64::from(*num) / f64::from(COUNT_NODES)
    }
    println!("avg nodes per tree: {}", avg_nodes_per_tree);
    println!("avg children per node: {}", avg_children_per_node);
    println!("fraction of nodes with n children:");
    for (index, num) in avg_nodes_per_children.iter().enumerate() {
        println!("  {} -> {}", index, num);
    }
}