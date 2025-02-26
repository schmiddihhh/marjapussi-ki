mod search_state;
mod bidding;

use search_state::AlphaBetaGameState;
use bidding::BiddingInfos;

use super::ai::MarjapussiCheater;

use crate::alpha_beta::alpha_beta_search;

use marjapussi::game::Game;
use marjapussi::game::gameevent::{ActionType, GameAction};
use marjapussi::game::cards::{Value, Suit, pairs, halves, Card};
use marjapussi::game::gamestate::GamePhase;
use marjapussi::game::player::PlaceAtTable;

pub struct CheaterV1 {
    name: String,
    position: PlaceAtTable,
    to_communicate: Vec<BiddingInfos>
}

impl CheaterV1 {
    pub fn new(name: &str, position: u8) -> Self {
        CheaterV1 {
            name: String::from(name),
            position: PlaceAtTable(position),
            to_communicate: vec![]
        }
    }

    fn assess_own_hand(&mut self,  game: &Game) {
        /* 
        get all information that is relevant for the bidding phase and gather it in a vector
        */ 

        let own_cards = &game.state.player_at_turn().cards;
        let mut to_communicate = vec![];

        // find out if we have an ace
        if own_cards.iter().any(|card| {card.value == Value::Ace}) {
            to_communicate.push(BiddingInfos::Ace);
        }

        // find out if we have pairs
        let own_pairs = pairs(own_cards.clone());
        // we assume that the pairs function sorts the pairs ascendingly
        assert!(own_pairs.is_sorted());
        own_pairs
            .iter()
            .rev()
            .for_each(|suit| {
                to_communicate.push(
                    match suit {
                        Suit::Red => BiddingInfos::BigPair,
                        Suit::Bells => BiddingInfos::BigPair,
                        Suit::Acorns => BiddingInfos::SmallPair,
                        Suit::Green => BiddingInfos::SmallPair
                    }
                )
            });

        // find out how many single halves we have
        let count_single_halves = halves(own_cards.clone())
            .iter()
            .filter(|suit| {!own_pairs.contains(suit)})
            .count();
        if count_single_halves >= 3 {
            to_communicate.push(BiddingInfos::Halves3_4);
        } else if count_single_halves == 2 {
            to_communicate.push(BiddingInfos::Halves2);
        }

        // save the communication info
        // first info to communicate should be the last element (we will use pop to get the information), thus the reverse
        to_communicate.reverse();
        self.to_communicate = to_communicate;
    }

    fn player_announced_ace(&self, game: &Game, player: PlaceAtTable) -> bool {

        println!("  checking if player {} announced an ace", player.0);

        // find out if the partner announced an ace
        let result = vec!((ActionType::NewBid(115), PlaceAtTable(0)))
                    .into_iter()
                    .chain(game.state.bidding_history.clone())
                    .collect::<Vec<(ActionType, PlaceAtTable)>>()   // at this point, we have a Vec with all bidding steps, including the imaginary first step of 115
                    .windows(4)
                    .map(|window| {
                        let step = match (&window[0].0, &window[1].0, &window[2].0, &window[3].0) {
                            (_, _, _, ActionType::StopBidding) => 0,
                            (_, ActionType::NewBid(bid1), ActionType::StopBidding, ActionType::NewBid(bid2)) => bid2 - bid1,
                            (ActionType::NewBid(bid1), ActionType::StopBidding, ActionType::StopBidding, ActionType::NewBid(bid2)) => bid2 - bid1,
                            (_, _, ActionType::NewBid(bid1), ActionType::NewBid(bid2)) => bid2 - bid1,
                            _ => panic!("Don't know how to handle this bidding history: {:?}", game.state.bidding_history)
                        };
                        (step, &window[3].1)
                    })
                    .any(|(step, player_num)| {
                        *player_num == player && step == 5
                    });
        match result {
            true => println!("  he did"),
            false => println!("  he didn't")
        };
        result
    }

    fn bid(&mut self, game: Game, legal_actions: Vec<GameAction>) -> GameAction {
        
        // if this is our first bidding action, we have to 
        // find out what information we want to share in our bidding
        if game.state.bidding_history.len() < 4 {
            self.assess_own_hand(&game);
        }

        let next_bidding_step = self.next_bidding_step(game);
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

    fn next_bidding_step(&mut self, game: Game) -> i32 {
        while let Some(next_info) = self.to_communicate.pop() {

            println!("  next bidding step: {:?}", next_info);
            // skip bidding steps under certain conditions
            if next_info == BiddingInfos::Ace && self.player_announced_ace(&game, game.state.player_at_turn().partner_place()) {
                // don't announce an ace if your partner already did
                continue;
            } else if next_info == BiddingInfos::Halves2 
                      && !self.player_announced_ace(&game, game.state.player_at_turn().partner_place())
                      && !self.player_announced_ace(&game, game.state.player_at_turn().place_at_table.clone()) {
                // don't announce two halves if no ace was announced in the party yet
                continue;
            }
            
            let step = match next_info {
                BiddingInfos::Ace => 5,
                BiddingInfos::BigPair => 15,
                BiddingInfos::SmallPair => 10,
                BiddingInfos::Halves3_4 => 10,
                BiddingInfos::Halves2 => 5,
            };
            let current_value = game.state.value.0;
            let mut next_value = current_value + step;
            
            if current_value < 140 && next_value >= 140 {
                next_value += 5;
            }
            
            if next_value > 420 {
                println!(
                    "  folding since I can't exceed the game limit, remaining steps were {:?}",
                    std::iter::once(&next_info).chain(self.to_communicate.iter()).collect::<Vec<_>>()
                );
                return 0;
            }
            
            if next_value < 140 {
                println!("  bidding {} for {:?} while staying under 140", next_value, next_info);
                return next_value;
            } else {
                let cards_together: Vec<Card> = game.state.player_at_turn().cards.clone()
                    .into_iter()
                    .chain(game.state.partner().cards.clone())
                    .collect();
                let have_secure_pair = !pairs(cards_together).is_empty();
                if have_secure_pair {
                    println!("  bidding {} for {:?} while being sure that we have a pair", next_value, next_info);
                    return next_value;
                } else {
                    println!("  not going over 140 for {:?} since I am not sure if we have a pair; trying next bidding step", next_info);
                }
            }
        }
        
        println!("  folding since there is nothing to communicate");
        0
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
                GamePhase::StartTrick | GamePhase::Trick | GamePhase::Raising => alpha_beta_search(AlphaBetaGameState{owning_player: self.position.clone(), game, depth: 0, debugging_depth: -1}, Some(12)).0,
                _ => legal_actions.into_iter().nth(0).expect("Player was asked to choose an action, but there are no legal actions")
            }
        }
        
        
    }
}