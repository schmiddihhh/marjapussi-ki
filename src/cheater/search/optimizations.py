from __future__ import annotations

def standing(card) -> bool:
    # returns if the card is standing
    # TODO: implement
    pass

class EqSet:
    def __init__(self, id: int, owner: int, prev: EqSet, next: EqSet, cards: list[str]):
        self.id = id
        self.owner = owner
        self.prev = prev
        self.next = next
        self.cards = cards
    def highest(self) -> str:
        # returns the highest card in the set
        # TODO: implement
        pass
    def lowest(self) -> str:
        # returns the lowest card in the set
        # TODO: implement
        pass

class EqChecker:
    def __init__(self, cards: set[EqSet], cards_to_set: dict[int, EqSet]):
        self.cards = cards
        self.cards_to_set = cards_to_set

    def reduce_legal_actions(self, legal_actions: list[str]) -> list[str]:

        reduced_legal_actions = []
        set_already_checked = {eq_set.id: False for eq_set in self.cards}

        for action in legal_actions:
            matched_set = self.cards_to_set[action]

            if not set_already_checked[matched_set.id]:
                if len(matched_set.cards) == 1:
                    reduced_legal_actions.append(matched_set.cards[0])
                else:
                    if standing(matched_set.cards[0]):
                        reduced_legal_actions.append(matched_set.highest())
                    else:
                        reduced_legal_actions.append(matched_set.lowest())

                set_already_checked[matched_set.id] = True

        return reduced_legal_actions