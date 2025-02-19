use std::cmp::max;
use std::cmp::min;


// this trait specifies the requirements for a node in the search tree
pub trait State<M> {
    fn legal_moves(&self) -> Vec<M>;
    fn apply_move(&self, next_move: &M) -> Self;
    fn is_leaf(&self) -> bool;
    fn is_maximizing(&self) -> bool;
    fn evaluate(&self) -> i32;
}


// this function finds the best move to make in the current game state
// a recursive implementation of alpha-beta tree search is used within this function
pub fn alpha_beta_search<M, S>(start_state: S) -> M
where M: Copy, S: State<M> {

    // the search has to start at a state where the maximizing player is at turn
    assert!(start_state.is_maximizing(), 
            "Alpha-Beta search can not be started at a state where the opponent (minimizing player) is at play");

    // if the game is over, we cannot find the best move (because there are no valid moves)
    assert!(!start_state.is_leaf(),
            "The search has to start at a node that is not a leaf node");

    // initialize alpha and beta
    let mut alpha: i32 = i32::MIN;
    let beta: i32 = i32::MAX;

    // init values for the best move search
    let mut best_move = None;
    let mut max_eval: i32 = i32::MIN;

    // beta is (theoretically) infinite, so no beta cutoffs will happen here
    // we will simply choose the move with the highest evaluation
    for next_move in start_state.legal_moves().into_iter() {
        let next_state = start_state.apply_move(&next_move);
        let eval = recursive_minimax(&next_state, alpha, beta);
        if eval > max_eval {
            max_eval = eval;
            best_move = Some(next_move);
        }
        alpha = max(alpha, eval);
    }
    best_move.expect("There are no legal moves for the player")
}


// recursive implementation of alpha-beta search (called by alpha_beta_search)
fn recursive_minimax<M, S>(start_node: &S, alpha: i32, beta: i32) -> i32
where S: State<M> {
    if start_node.is_leaf() {
        return start_node.evaluate();
    } else if start_node.is_maximizing() {
        let mut alpha = alpha;
        let mut max_eval: i32 = i32::MIN;
        for next_move in start_node.legal_moves() {
            let next_state = start_node.apply_move(&next_move);
            let eval = recursive_minimax(&next_state, alpha, beta);
            max_eval = max(max_eval, eval);
            alpha = max(alpha, eval);
            if beta <= alpha {
                break;
            }
        }
        return max_eval;
    } else {
        let mut min_eval: i32 = i32::MAX;
        for next_move in start_node.legal_moves() {
            let mut beta = beta;
            let next_state = start_node.apply_move(&next_move);
            let eval = recursive_minimax(&next_state, alpha, beta);
            min_eval = min(min_eval, eval);
            beta = min(beta, eval);
            if beta <= alpha {
                break;
            }
        }
        return min_eval;
    }
}

#[cfg(test)]
mod tests {
    /*
    This test uses an example search tree to test if the correct move is chosen and if the correct prunings occur.
     */

    use super::*;

    // every test state should have a left and a right child (except the leaf states, which have none)
    // therefore, the only allowed moves are "Left" and "Right"
    #[derive(Copy, Clone)]
    enum Move {
        Left,
        Right
    }

    struct TestState {
        depth: i32,
        id: i32
    }

    impl State<Move> for TestState {
        fn legal_moves(&self) -> Vec<Move> {
            assert!(! self.is_leaf(),
                    "Tried to generate legal moves for a leaf node");
            println!("Generating moves for ({}, {})", self.depth, self.id);
            vec![Move::Left, Move::Right]
        }

        fn apply_move(&self, next_move: &Move) -> TestState {
            assert!(! self.is_leaf(),
                    "Tried to generate child for a leaf node");
            
            // find the depth and ID for the child node
            let new_depth: i32 = self.depth + 1;
            let new_id: i32;
            if matches!(next_move, Move::Left) {
                new_id = self.id * 2;
            } else {
                new_id = self.id * 2 + 1;
            }
            
            println!("({}, {}) -> ({}, {})", self.depth, self.id, new_depth, new_id);

            // generate the child node
            TestState{ depth: new_depth, id: new_id }
        }
        
        fn is_leaf(&self) -> bool {
            assert!(self.depth <= 4,
                    "Found a node with illegaly high depth (> 4)");
            self.depth == 4
        }
        
        fn is_maximizing(&self) -> bool {
            self.depth % 2 == 0
        }
        
        fn evaluate(&self) -> i32 {
            // should only be called on leaf nodes
            assert!(self.is_leaf(),
                    "Evaluation called on a non-leaf node");
            let value = match self.id {
                0 => 8,
                1 => 5,
                2 => 6,
                3 => -4,
                4 => 3,
                5 => 8,
                6 => 4,
                7 => -6,
                8 => 1,
                9 => i32::MIN,
                10 => 5,
                11 => 2,
                12 => i32::MIN,
                13 => i32::MIN,
                14 => i32::MIN,
                15 => i32::MIN,
                _ => i32::MAX
            };
            assert!(value != i32::MIN,
                    "Evaluated a node that should be pruned");
            assert!(value != i32::MAX,
                    "Evaluated a node that shouldn't exist");
            println!("({}, {}) = {}", self.depth, self.id, value);
            value
        }
    }

    #[test]
    fn test_alpha_beta() {
        let start_state = TestState { depth: 0, id: 0 };
        let chosen_move = alpha_beta_search(start_state);
        assert!(matches!(chosen_move, Move::Left));
    }
}