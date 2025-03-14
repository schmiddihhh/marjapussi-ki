// use std::{collections::{BTreeSet, HashMap, HashSet}, hash::Hash};

// use serde_json;

// use marjapussi::game::{cards::{Card, Suit}, gameevent::{ActionType, GameAction}, Game};

// type EqSetID = u8;
// type Owner = u8;

// #[derive(Debug)]
// struct EqSet {
//     id: EqSetID,
//     owner: Owner,
//     cards: BTreeSet<Card>,
//     prev: Option<EqSetID>,
//     next: Option<EqSetID>
// }

// struct EqChecker {
//     card_to_id: HashMap<String, EqSetID>,
//     id_to_eq_set: HashMap<EqSetID, EqSet>
// }

// impl EqChecker {
//     fn reduce_legal_actions(&self, legal_actions: Vec<GameAction>, trick: Vec<Card>, trump: Option<Suit>) -> Vec<GameAction> {
//         /*
//             Sorts out redundant cardplay moves.
//          */
        
//         let mut reduced_legal_actions = vec![];
//         let mut already_matched: HashSet<EqSetID> = HashSet::new();

//         for action in legal_actions {

//             // check if "action" is a cardplay action
//             // if so, check if it is redundant and drop or keep the action accordingly
//             // else, keep the action
//             if let ActionType::CardPlayed(card) = &action.action_type {

//                 // get the EqualitySet corresponding to the card
//                 let eq_set_id = self.card_to_id
//                     .get(&serde_json::to_string(card).unwrap())
//                     .expect("Card could not be mapped to an EqSetID");
                
//                 // check if an action from this EqSet was already added to the reduced_legal_actions
//                 if already_matched.insert(*eq_set_id) {
//                     // this EqSet was not matched yet
//                     let unmatched_eq_set = self.id_to_eq_set
//                         .get(eq_set_id)
//                         .expect("EqSetID could not me mapped to an EqSet");
//                     // if the cards in the EqSet are standing, play the highest one
//                     // else, play the lowest one
//                     if self.standing_in_suit(unmatched_eq_set) {

//                     };
//                 }


//                 reduced_legal_actions.push(action);
//             } else {
//                 // this action is not a cardplay action, thus cannot be sorted out
//                 reduced_legal_actions.push(action);
//             }
//         }

//         unimplemented!()
//     }

//     fn standing(&self, eq_set: &EqSet, trick: Vec<Card>, trump: Option<Suit>) -> bool {
//         /*
//             Checks if the cards in the EqSet would safely win the current trick.
//          */

//         let own_suit = eq_set.cards.first().unwrap().suit;
//         let standing_in_suit = if let Some(next) = eq_set.next {
//             if self.id_to_eq_set.get(&next).unwrap().cards.first().unwrap().suit == own_suit {
//                 // higher EqSet in the same suit exists
//                 false
//             } else {
//                 // no higher EqSet in the same suit
//                 true
//             }
//         } else {
//             // no higher EqSet
//             true
//         };
//         if let Some(trump_suit) = trump {
//             if own_suit == trump_suit {
//                 true
//             } else {
//                 false
//             }
//         } else {
            
//         }
//     }
// }


































// // #[derive(Debug)]
// // struct EqCheckerOld {
// //     card_to_eq_set: HashMap<String, Weak<RefCell<EqSet>>>,
// //     eq_sets: Vec<EqSet>
// // }

// // impl EqCheckerOld {

// //     fn get_eq_set(&self, card: Card) -> EqSetID {
// //         /*
// //             Returns the ID of the EqSet the card is in.
// //          */
// //         self.card_to_eq_set
// //             .get(&serde_json::to_string(&card).unwrap())
// //             .unwrap()
// //             .upgrade()
// //             .unwrap()
// //             .borrow()
// //             .id
// //     }
    
// //     fn remove_card(&mut self, card: Card) {
// //         /*
// //             Removes the given card from the EqCheckerOld.
// //          */

// //         // remove the card from the [card -> eq_set] translator
// //         // and get the eq_set the card is in
// //         let remaining_eq_set_rc = self.card_to_eq_set
// //             .remove(&serde_json::to_string(&card).unwrap())
// //             .expect("Card that should be removed had no EqSet assigned to it")
// //             .upgrade()
// //             .expect("Card that should be removed had an invalid Pointer to an EqSet assigned to it");

// //         let mut remaining_eq_set = remaining_eq_set_rc.borrow_mut();

// //         // remove the card from the eq_set
// //         remaining_eq_set
// //             .cards
// //             .take(&card)
// //             .expect("Card that should be removed was not in its EqSet");

// //         if remaining_eq_set.cards.is_empty() {
// //             self.remove_eq_set(Rc::clone(&remaining_eq_set_rc));
// //         }
// //     }
    
// //     fn remove_eq_set(&mut self, eq_set_rc: Rc<RefCell<EqSet>>) {
// //         /*
// //             Removes an EqSet from the EqCheckerOld.
// //             If the neighbors of the removed set have the same owner, they are merged.
// //             Else, they are linked to each other.
// //          */

// //         // get a mutable borrow of the EqSet
// //         let mut eq_set = eq_set_rc.borrow_mut();
    
// //         // get references to the previous and next EqSet
// //         // it the references are present, check if they are still valid
// //         let prev_option = eq_set
// //             .prev
// //             .and_then(|prev| {
// //                 Some(prev
// //                         .upgrade()
// //                         .expect("Pointer to previous EqSet was present but invalid"))
// //             });
// //         let next_option = eq_set
// //             .next
// //             .and_then(|next| {
// //                 Some(next
// //                         .upgrade()
// //                         .expect("Pointer to next EqSet was present but invalid"))
// //             });
        
// //         // remove the EqSet and link prev and next as required
// //         match (prev_option, next_option) {
// //             (Some(prev_rc), Some(next_rc)) => {
// //                 if prev_rc.borrow().owner == next_rc.borrow().owner {
// //                     // merge the next EqSet into the previous
// //                     let mut merged = prev_rc.borrow_mut();
// //                     let next = next_rc.borrow();
// //                     merged.cards.extend(next.cards.clone());
        
// //                     // make all cards from "next" point to the merged EqSet
// //                     for card in &next.cards {
// //                         self.card_to_eq_set.insert(
// //                             serde_json::to_string(&card).unwrap(), 
// //                             Rc::downgrade(&prev_rc));
// //                     }
        
// //                     // update the "next" pointer of the merged set
// //                     // if "next" is present: update its "prev" pointer
// //                     merged.next = next.next.clone();
// //                 } else {
// //                     // link 
// //                 }
// //             }
// //             _ => {
// //                 // Just remove the EqSet
// //                 if let Some(prev) = EqSet.prev.as_ref().and_then(|w| w.upgrade()) {
// //                     prev.borrow_mut().next = EqSet.next.clone();
// //                 }
// //                 if let Some(next) = &EqSet.next {
// //                     next.borrow_mut().prev = EqSet.prev.clone();
// //                 }
// //             }
// //         }
    
// //         self.EqSets.retain(|s| !Rc::ptr_eq(s, &EqSet_rc));
// //     }
// // }