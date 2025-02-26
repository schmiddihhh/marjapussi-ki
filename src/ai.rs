use marjapussi::game::Game;
use marjapussi::game::gameevent::GameEvent;
use marjapussi::game::gameevent::GameAction;

pub trait MarjapussiAI {
    fn observe_event(&self, event: GameEvent);
    fn select_action(&self, legal_actions: Vec<GameAction>) -> GameAction;
}

pub trait MarjapussiCheater {
    fn select_action(&mut self, gamestate: Game) -> GameAction;
}