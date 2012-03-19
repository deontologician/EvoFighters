'''Handles parsing of streams of integers into instructions for the
EvoFighters'''

from itertools import cycle
from collections import namedtuple

MAX_THINKING_STEPS = 200

class Enum(dict):
    '''Simple subclass that allows getting codes as attributes'''
    def __getattr__(self, attr):
        return self[attr]

COND = Enum(always         = 0,
            in_range       = 1,
            less_than      = 2,
            greater_than   = 3,
            equal_to       = 4,
            not_equal_to   = 5,
            me_last_act    = 6,
            target_last_act = 7,
            )

VAL = Enum(literal = 0, # a number straight from the dna
           random  = 1, # a randomly selected number
           me      = 2, # one of my attributes (1 arg)
           target   = 3, # one of my enemies attributes (1 arg)
           )

ACT = Enum(subcondition = 0, # indicates a subconditional
           attack       = 1, # attacks target with specified attack type (1 arg)
           defend       = 2, # defends with specified defense type (1 arg)
           signal       = 3, # sets a flag on the creature (1 arg)
           use          = 4, # uses the top item in the inventory
           take         = 5, # takes top item in enemies inventory
           wait         = 6, # do nothing
           flee         = 7, # escape the fight
           mate         = 8, # attempt to mate with target
           )

ATTR = Enum(energy       = 0, # energy level
            signal       = 1, # current signal
            generation   = 2, # generation number
            kills        = 3,  # how many targets killed
            survived     = 4, # how many encounters survived
            num_children = 5, # how many children
            top_item     = 6, # value of top inventory item
            )

ITEM = Enum(food           = 0,
            good_food      = 1, 
            better_food    = 2, 
            excellent_food = 3,
            )

SIG = Enum(red    = 0,
           yellow = 1,
           blue   = 2,
           purple = 3,
           orange = 4,
           green  = 5,
           )

DMG = Enum(fire        = 0,
           ice         = 1,
           electricity = 2,
           )

class ParseError(Exception):
    'Thrown when a parse goes wrong'
    pass


class TooMuchThinkingError(Exception):
    'Thrown when a creatures takes too many steps to produce a tree'
    def __init__(self, msg, icount, skipped):
        Exception.__init__(self, msg)
        self.icount = icount
        self.skipped = skipped

# these are used for internal state of the parser and should bot be modified by
# the clients of the module
_icount = 0
_skipped = 0
_dna_iter = None

Thought = namedtuple('Thought', 'tree icount skipped')

class Parser(object):
    '''Handles parsing of a dna iterator and returning a parse tree which
    represents a creature's tthought process in making encounter decisions'''
    def __init__(self, dna):
        self._icount = 0
        self._skipped = 0
        self._dna = dna
        self._dna_iter = cycle(dna)

    def next(self):
        '''Parses a dna iterator (Must not be the dna list!) into a tree
        structure that represents a creature's thought process in making
        encounter decisions'''
        self._icount = 0
        self._skipped = 0
        return Thought(tree = self._cond, 
                       icount = self._icount,
                       skipped = self._skipped)

    @property
    def _cond(self):
        '''Parses a conditional node'''
        #get the condition type symbol
        cond_typ = self._get_next_valid(COND)
        
        if cond_typ == COND.always:
            return (COND.always,
                    self._act())
        elif cond_typ == COND.in_range:
            return (COND.in_range,
                    self._val, 
                    self._val, 
                    self._val,
                    self._act(), 
                    self._act())
        elif COND.less_than <= cond_typ <= COND.not_equal_to:
            return (cond_typ, 
                    self._val, 
                    self._val,
                    self._act(), 
                    self._act())
        elif cond_typ in [COND.me_last_act, COND.target_last_act]:
            return(cond_typ, 
                   self._act(nosub = True),
                   self._act(),
                   self._act())
        else:
            raise ParseError("Condition didn't match: {}".format(cond_typ))
    
    @property
    def _val(self):
        '''Parses a value node'''
        val_typ = self._get_next_valid(VAL)
        
        if val_typ == VAL.literal:
            self._icount += 1
            return (VAL.literal, 
                    next(self._dna_iter))
        elif val_typ == VAL.random:
            return (VAL.random,)
        elif val_typ in [VAL.me, VAL.target]:
            return (val_typ,
                    self._get_next_valid(ATTR))

    def _act(self, nosub = False):
        '''Parses an action node'''
        act_typ = self._get_next_valid(ACT, minimum = 1 if nosub else 0)

        if act_typ == ACT.subcondition:
            return (ACT.subcondition, 
                    self._cond)
        elif act_typ in [ACT.attack, ACT.defend]:
            return (act_typ,
                    self._get_next_valid(DMG))
        elif act_typ == ACT.signal:
            return (ACT.signal,
                    self._get_next_valid(SIG))
        elif ACT.use <= act_typ <= ACT.mate:
            return (act_typ,)
        else:
            raise ParseError("Action didn't match: {}".format(act_typ))

    def _get_next_valid(self, typ, minimum = 0):
        '''Gets the next valid integer in the range allowed by the given
        type. Adds on to the `count` passed in. mini and maxi allow one to
        restrict the range further than `typ` does'''
        next_val = next(self._dna_iter)
        self._icount += 1
        # we want to return a count 1 less than the number required, since we
        # dont want to penalize for required parser symbols. Therefore if the
        # while condition succeeds the first time through, count is not
        # incremented
        while not( minimum <= next_val < len(typ) ):
            next_val = next(self._dna_iter)
            self._skipped += 1
            if self._icount + self._skipped > MAX_THINKING_STEPS:
                raise TooMuchThinkingError('Thought too much :/', 
                                           self._icount, self._skipped)
        return next_val



def indent(val):
    '''Indent a string by 4 spaces per line'''
    return '\n'.join(['    {}'.format(val) for val in val.splitlines()])

def branch_repr(condition, then_str, else_str):
    """Prints an if then else branch"""
    return '''\
if {condition}:
{then_clause}
else:
{else_clause}\
'''.format(condition = condition,
           then_clause = indent(then_str),
           else_clause = indent(else_str))
                                                          
def show_thought(tree):
    '''Show a thought as a pretty printed string'''
    return cond_repr(tree)

def cond_repr(tree):
    '''Creates a string from a condition tree node'''
    cond_typ = tree[0]

    if cond_typ == COND.always:
        act = act_repr(tree[1])
        return "Always:\n{}".format(indent(act))
    elif cond_typ == COND.in_range:
        rng_min = val_repr(tree[1])
        rng_max = val_repr(tree[2])
        match = val_repr(tree[3])
        cond_str = '{} is in the range {} to {}'.format(match, rng_min, rng_max)
        then_str = act_repr(tree[4])
        else_str = act_repr(tree[5])
        return branch_repr(cond_str, then_str, else_str)
    elif COND.less_than <= cond_typ <= COND.not_equal_to:
        val1 = val_repr(tree[1])
        val2 = val_repr(tree[2])
        if cond_typ == COND.less_than:
            relation = 'is less than'
        elif cond_typ == COND.greater_than:
            relation = 'is greater than'
        elif cond_typ == COND.equal_to:
            relation = 'is equal to'
        elif cond_typ == COND.not_equal_to:
            relation = 'is not equal to'
        else:
            raise NotImplementedError('This cant happen! {}'.format(cond_typ))
        cond_str = '{} {} {}'.format(val1, relation, val2)
        then_str = act_repr(tree[3])
        else_str = act_repr(tree[4])
        return branch_repr(cond_str, then_str, else_str)
    elif cond_typ in [COND.me_last_act, COND.target_last_act]:
        matchact = act_repr(tree[1])
        if cond_typ == COND.me_last_act:
            what = 'my last action was'
        else:
            what = "my target's last action was"
        cond_str = '{} "{}"'.format(what, matchact)
        then_str = act_repr(tree[2])
        else_str = act_repr(tree[3])
        return branch_repr(cond_str, then_str, else_str)
    else:
        return 'Unknown Condition({})'.format(cond_typ)


def val_repr(tree):
    '''Creates a string from a value tree node'''
    val_typ = tree[0]
    
    if val_typ == VAL.literal:
        return str(tree[1])
    elif val_typ == VAL.random:
        return 'a random number'
    elif val_typ in [VAL.me, VAL.target]:
        who = 'my' if val_typ == VAL.me else "the target's"
        attr = attr_repr(tree[1])
        return '{} {}'.format(who, attr)
    else:
        return 'Unknown Value({})'.format(val_typ)

def act_repr(tree):
    '''Creates a string from an action tree node'''
    act_typ = tree[0]
    
    if act_typ == ACT.subcondition:
        return cond_repr(tree[1])
    elif act_typ in [ACT.attack, ACT.defend]:
        dmg_str = dmg_repr(tree[1])
        if act_typ == ACT.attack:
            return 'attack with {}'.format(dmg_str)
        else:
            return 'defend with {}'.format(dmg_str)
    elif act_typ == ACT.signal:
        sig = sig_repr(tree[1])
        return 'signal {}'.format(sig)
    elif act_typ == ACT.use:
        return 'use the top inventory item'
    elif act_typ == ACT.take:
        return "take the target's top inventory item"
    elif act_typ == ACT.wait:
        return 'wait'
    elif act_typ == ACT.flee:
        return 'flee from the encounter'
    elif act_typ == ACT.mate:
        return 'attempt to mate with the target'
    else:
        return 'Unknown Action({})'.format(act_typ)


def attr_repr(attr):
    '''Creates a string from an attribute code'''
    if attr == ATTR.energy:
        return 'energy level'
    elif attr == ATTR.signal:
        return 'signal'
    elif attr == ATTR.generation:
        return 'generation'
    elif attr == ATTR.kills:
        return 'kills'
    elif attr == ATTR.survived:
        return 'number of encounters survived'
    elif attr == ATTR.num_children:
        return 'number of children'
    elif attr == ATTR.top_item:
        return 'top inventory item'
    else:
        return 'Unknown attribute({})'.format(attr)


def item_repr(item):
    '''Creates a string from an item code'''
    if item == ITEM.food:
        return "a piece of food"
    elif item == ITEM.good_food:
        return "an good food"
    elif item == ITEM.better_food:
        return "a better food"
    elif item == ITEM.excellent_food:
        return "an excellent food"
    else:
        return "Unknown Item({})".format(item)
 
def dmg_repr(damage):
    '''Creates a string from a damage code'''
    if damage == DMG.fire:
        return 'Fire'
    elif damage == DMG.ice:
        return 'Ice'
    elif damage == DMG.electricity:
        return 'Electricity'
    else:
        return 'Unknown Damage Type({})'.format(damage)


def sig_repr(signal):
    '''Creates a string from a signal code'''
    if signal == SIG.red:
        return 'Red'
    elif signal == SIG.yellow:
        return 'Yellow'
    elif signal == SIG.blue:
        return 'Blue'
    elif signal == SIG.purple:
        return 'Purple'
    elif signal == SIG.orange:
        return 'Orange'
    elif signal == SIG.green:
        return 'Green'
    else:
        return 'Unknown Signal({})'.format(signal)


if __name__ == '__main__':
    from random import randint
    for _ in xrange(500):
        p = Parser([randint(-1, 9) for _ in xrange(50)])
        show_thought(next(p).tree)
