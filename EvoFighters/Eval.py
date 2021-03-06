'''This module handles evaluating the parse trees that Parsing creates'''

from Parsing import COND, ACT, ATTR, VAL
from Utils import dmg_repr, sig_repr
import operator as Op
from random import randint

sd = None  # Set by Arena

class PerformableAction(object):
    '''Represents a concrete, comparable action that a creature intends to carry
    out'''
    def __init__(self, typ, arg):
        self.typ = typ
        self.arg = arg

    def __eq__(self, other):
        return self.typ == other.typ and self.arg == other.arg

    def __repr__(self):
        return str(self)

    def __str__(self):
        if self.typ == ACT['attack']:
            return "attack with damage type: {}".format(dmg_repr(self.arg))
        elif self.typ == ACT['defend']:
            return "defend against damage type: {}".format(dmg_repr(self.arg))
        elif self.typ == ACT['signal']:
            return "signal with the color {0}".format(sig_repr(self.arg))
        elif self.typ == ACT['use']:
            return "use an item in his inventory"
        elif self.typ == ACT['take']:
            return "take something from target"
        elif self.typ == ACT['wait']:
            return "wait"
        elif self.typ == ACT['flee']:
            return "flee the encounter"
        elif self.typ == ACT['mate']:
            return "mate with target"
        else:
            return "do unknown action: ({}, {})".format(self.typ, self.arg)


class InvalidInstructionError(Exception):
    '''Thrown whenever an invalid instruction is evaluated'''
    pass


def evaluate(me, tree):
    '''Eval takes information from the creature and a thought and returns an
    action to perform'''
    cond_typ = tree[0]
    if cond_typ == COND['always']:
        return eval_act(me, tree[1])
    elif cond_typ == COND['in_range']:
        val1 = get_val(me, tree[1])
        val2 = get_val(me, tree[2])
        check_val = get_val(me, tree[3])
        if min(val1, val2) <= check_val <= max(val1, val2):
            sd.print3('{val_repr} was between {} and {}', val1, val2, val_repr = tree[3])
            return eval_act(me, tree[4])
        else:
            sd.print3('{val_repr} was not between {} and {}', val1, val2, val_repr = tree[1])
            return eval_act(me, tree[5])
    elif COND['less_than'] <= cond_typ <= COND['not_equal_to']:
        if cond_typ == COND['less_than']:
            op_str = '<'
            op = Op.lt
        elif cond_typ == COND['greater_than']:
            op_str = '>'
            op = Op.gt
        elif cond_typ == COND['equal_to']:
            op_str = '=='
            op = Op.eq
        elif cond_typ == COND['not_equal_to']:
            op_str = '!='
            op = Op.ne
        val1 = get_val(me, tree[1])
        val2 = get_val(me, tree[2])
        if op(val1, val2):
            sd.print3('{val_repr} was {} {val_repr2}', op_str, val_repr = tree[1],
                   val_repr2 = tree[2])
            return eval_act(me, tree[3])
        else:
            sd.print3('{val_repr} was not {} {val_repr2}', op_str, val_repr = tree[1], 
                   val_repr2 = tree[2])
            return eval_act(me, tree[4])
    elif cond_typ in [COND['me_last_act'], COND['target_last_act']]:
        if cond_typ == COND['me_last_act']:
            actor = me
        else:
            actor = me.target
        act1 = eval_act(me, tree[1])
        if act1 == actor.last_action:
            sd.print3('{who.name}\'s last action was {act_repr}', who = actor, 
                   act_repr = tree[1])
            return eval_act(me, tree[2])
        else:
            sd.print3('{who.name}\'s last action was not {act_repr} ({fst} vs. {snd})',
                   who = actor, act_repr = tree[1], fst = act1, 
                   snd = actor.last_action)
            return eval_act(me, tree[3])
    else:
        raise InvalidInstructionError("Couldn't understand condition: {0}".
                                      format(cond_typ))

def get_val(me, tree):
    '''Evaluates a VAL node in a thought tree'''
    val_typ = tree[0]
    if val_typ == VAL['literal']:
        return tree[1]
    elif val_typ == VAL['random']:
        ret = randint(-1, 9)
        return ret
    elif val_typ == VAL['me']:
        return get_attr(me, tree[1])
    elif val_typ == VAL['target']:
        return get_attr(me.target, tree[1])

def get_attr(who, attr_typ):
    '''Returns the value of the attribute on "who" '''
    if attr_typ == ATTR['energy']:
        return who.energy
    elif attr_typ == ATTR['signal']:
        return who.signal
    elif attr_typ == ATTR['generation']:
        return who.generation
    elif attr_typ == ATTR['kills']:
        return who.kills
    elif attr_typ == ATTR['survived']:
        return who.survived
    elif attr_typ == ATTR['num_children']:
        return who.num_children
    elif attr_typ == ATTR['top_item']:
        return who.top_item if who.has_items else -1

def eval_act(me, tree):
    '''Returns an action suitable for performing (PerformableAction)'''
    act_typ = tree[0]
    if act_typ in (ACT['attack'], ACT['defend'], ACT['signal']):
        return PerformableAction(act_typ, tree[1])
    elif act_typ in (ACT['use'], ACT['take'],
                     ACT['wait'], ACT['flee'], ACT['mate']):
        return PerformableAction(act_typ, None)
    elif act_typ == ACT['subcondition']:
        return evaluate(me, tree[1])
    else:
        raise InvalidInstructionError("Didn't understand action: {0}"\
                                      .format(act_typ))

if __name__ == '__main__':
    from Parsing import Parser, TooMuchThinkingError
    from Creatures import Creature
    last_action = PerformableAction(ACT['wait'], None)
    for _ in xrange(1000):
        a = Creature()
        b = Creature()
        a.target = b
        a.last_action = last_action
        try:
            p = Parser(a.dna)
            last_action = evaluate(a, next(p).tree)
            str(last_action)
        except TooMuchThinkingError:
            continue
    
