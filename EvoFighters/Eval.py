'''This module handles evaluating the parse trees that Parsing creates'''

from collections import namedtuple

PerformableAction = namedtuple('PerformableAction', 'typ arg')

class InvalidInstructionError(Exception):
    pass

def evaluate(self, tree):
    '''Eval takes information from the creature and returns an action to perform'''
    instr = tree[0]
    if instr == 'always':
        return eval_act(self, tree[1])
    elif instr == 'enemy_has':
        if tree[1] in self.target.inv:
            return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'me_has':
        if tree[1] in self.inv:
            return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'enemy_energy':
        if eval_comp(self.target.energy, tree[1]):
            return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'me_energy':
        if eval_comp(self.energy, tree[1]):
            return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'enemy_signal':
        if eval_comp(self.target.signal, tree[1]):
            return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'me_signal':
        if eval_comp(self.signal, tree[1]):
            return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'enemy_last_act':
        if self.target.last_action.typ == tree[1][0]: # action type matches
            if tree[1][0] in ['attack', 'defend', 'signal']:
                if tree[1][1] == self.target.last_action.arg:
                    return eval_act(self, tree[2])
                else:
                    return eval_act(self, tree[3])
            else:
                # use, take, and wait all have no arguments and so match if
                # their type matches
                return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    elif instr == 'me_last_act':
        if self.last_action.typ == tree[1][0]: # action type matches
            if tree[1][0] in ['attack', 'defend', 'signal']:
                if tree[1][1] == self.last_action.arg:
                    return eval_act(self, tree[2])
                else:
                    return eval_act(self, tree[3])
            else:
                # use, take, and wait all have no arguments and so match if
                # their type matches
                return eval_act(self, tree[2])
        else:
            return eval_act(self, tree[3])
    else:
        raise InvalidInstructionError("Couldn't understand condition: {0}".
                                      format(instr))
        


def eval_comp(value, tree):
    '''Evaluates whether value matches the comparison represented by tree'''
    comp_typ = tree[0]
    if comp_typ == 'inrange':
        low = min(tree[1], tree[2])
        high = max(tree[1], tree[2])
        return low <= value <= high
    elif comp_typ == 'lessthan':
        return value < tree[1]
    elif comp_typ == 'greaterthan':
        return value > tree[1]
    elif comp_typ == 'equalto':
        return value == tree[1]
    elif comp_typ == 'notequalto':
        return value != tree[1]
    else:
        raise InvalidInstructionError("Couldn't understand comparison: {0}".
                                      format(comp_typ))

def eval_act(self, tree):
    '''Returns an action suitable for performing (PerformableAction)'''
    act_typ = tree[0]
    if act_typ in ['attack', 'defend', 'signal']:
        return PerformableAction(act_typ, tree[1])
    elif act_typ in ['use', 'take', 'wait']:
        return PerformableAction(act_typ, None)
    elif act_typ == 'subcondition':
        return evaluate(self, tree[1])
    else:
        raise InvalidInstructionError("Didn't understand action: {0}".
                                      format(act_typ))
