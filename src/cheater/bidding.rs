use std::collections::HashMap;

use marjapussi::game::{cards::{halves, pairs, Card, Suit, Value}, gameevent::ActionType, player::PlaceAtTable};

#[derive(PartialEq, Debug)]
pub enum BiddingInfos {
    Ace,
    BigPair,
    SmallPair,
    Halves3_4,
    Halves2
}


pub fn next_bidding_step(own_position: &PlaceAtTable,
                         hand_cards: &Vec<Card>,
                         partner_cards: &Vec<Card>,
                         bidding_history: &Vec<(ActionType, PlaceAtTable)>, 
                         knowledge: &mut HashMap<String, String>, 
                         to_communicate: &mut Vec<BiddingInfos>) -> i32 {
    /*
        This function derives the next bidding step from the hand cards, the bidding history and previously derived information.
     */
        
    // if this is our first bidding action, we have to 
    // find out what information we want to share in our bidding
    if bidding_history.len() < 4 {
        to_communicate.extend(assess_hand(hand_cards));
    }
    
    let partner_position = PlaceAtTable((own_position.0 + 2) % 4);

    // get the next information we want to share
    while let Some(next_info) = to_communicate.pop() {

        println!("  next bidding step: {:?}", next_info);
        // skip bidding steps under certain conditions
        if next_info == BiddingInfos::Ace && player_announced_ace(bidding_history, &partner_position) {
            // don't announce an ace if your partner already did
            continue;
        } else if next_info == BiddingInfos::Halves2 
                  && !player_announced_ace(bidding_history, &partner_position)
                  && !player_announced_ace(bidding_history, &own_position) {
            // don't announce two halves if no ace was announced in the party yet
            continue;
        }
        
        // get the corresponding step for the information
        let step = match next_info {
            BiddingInfos::Ace => 5,
            BiddingInfos::BigPair => 15,
            BiddingInfos::SmallPair => 10,
            BiddingInfos::Halves3_4 => 10,
            BiddingInfos::Halves2 => 5,
        };

        // get the current game value (last bidding step) and the new value after bidding
        let last_bidding_action = bidding_history
            .iter()
            .rev()
            .find(|(action, _)| {
                matches!(action, ActionType::NewBid(_))
            })
            .unwrap_or(&(ActionType::NewBid(115), PlaceAtTable(u8::MAX))).0
            .clone();
        let current_value = if let ActionType::NewBid(value) = last_bidding_action {
            value
        } else {
            panic!()
        };
        let mut next_value = current_value + step;
        
        if current_value < 140 && next_value >= 140 {
            next_value += 5;
        }
        
        if next_value > 420 {
            println!(
                "  folding since I can't exceed the game limit, remaining steps were {:?}",
                std::iter::once(&next_info).chain(to_communicate.iter()).collect::<Vec<_>>()
            );
            return 0;
        }
        
        if next_value < 140 {
            println!("  bidding {} for {:?} while staying under 140", next_value, next_info);
            return next_value;
        } else {
            let cards_together: Vec<Card> = hand_cards.clone()
                .into_iter()
                .chain(partner_cards.clone())
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


pub fn assess_hand(cards: &Vec<Card>) -> Vec<BiddingInfos> {
    /* 
    get all information that is relevant for the bidding phase and gather it in a vector
    */ 

    let mut to_communicate = vec![];

    // find out if we have an ace
    if cards.iter().any(|card| {card.value == Value::Ace}) {
        to_communicate.push(BiddingInfos::Ace);
    }

    // find out if we have pairs
    let own_pairs = pairs(cards.clone());
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
    let count_single_halves = halves(cards.clone())
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

    to_communicate
}


pub fn player_announced_ace(bidding_history: &Vec<(ActionType, PlaceAtTable)>, player: &PlaceAtTable) -> bool {
    /*
        !!!! Not working properly!
     */

    println!("  checking if player {} announced an ace", player.0);

    // find out if the partner announced an ace
    let result = vec!((ActionType::NewBid(115), PlaceAtTable(0)))
                .into_iter()
                .chain(bidding_history.clone())
                .collect::<Vec<(ActionType, PlaceAtTable)>>()   // at this point, we have a Vec with all bidding steps, including the imaginary first step of 115
                .windows(4)
                .map(|window| {
                    let step = match (&window[0].0, &window[1].0, &window[2].0, &window[3].0) {
                        (_, _, _, ActionType::StopBidding) => 0,
                        (_, ActionType::NewBid(bid1), ActionType::StopBidding, ActionType::NewBid(bid2)) => bid2 - bid1,
                        (ActionType::NewBid(bid1), ActionType::StopBidding, ActionType::StopBidding, ActionType::NewBid(bid2)) => bid2 - bid1,
                        (_, _, ActionType::NewBid(bid1), ActionType::NewBid(bid2)) => bid2 - bid1,
                        _ => panic!("Don't know how to handle this bidding history: {:?}", bidding_history)
                    };
                    (step, &window[3].1)
                })
                .any(|(step, player_num)| {
                    *player_num == *player && step == 5
                });
    match result {
        true => println!("  he did"),
        false => println!("  he didn't")
    };
    result
}