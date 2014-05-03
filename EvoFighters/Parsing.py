'''Handles parsing of streams of integers into instructions for the
EvoFighters'''

from collections import namedtuple 

sd = None  # set by Arenas


COND = dict(always         = 0,
            in_range       = 1,
            less_than      = 2,
            greater_than   = 3,
            equal_to       = 4,
            not_equal_to   = 5,
            me_last_act    = 6,
            target_last_act = 7,
            )

VAL = dict(literal = 0, # a number straight from the dna
           random  = 1, # a randomly selected number
           me      = 2, # one of my attributes (1 arg)
           target   = 3, # one of my enemies attributes (1 arg)
           )

ACT = dict(subcondition = 0, # indicates a subconditional
           attack       = 1, # attacks target with specified attack type (1 arg)
           mate         = 2, # attempt to mate with target
           defend       = 3, # defends with specified defense type (1 arg)
           use          = 4, # uses the top item in the inventory
           signal       = 5, # sets a flag on the creature (1 arg)
           take         = 6, # takes top item in enemies inventory
           wait         = 7, # do nothing
           flee         = 8, # escape the fight
           )

ATTR = dict(energy       = 0, # energy level
            signal       = 1, # current signal
            generation   = 2, # generation number
            kills        = 3,  # how many targets killed
            survived     = 4, # how many encounters survived
            num_children = 5, # how many children
            top_item     = 6, # value of top inventory item
            )

ITEM = dict(food           = 0,
            good_food      = 1, 
            better_food    = 2, 
            excellent_food = 3,
            )

SIG = dict(red    = 0,
           yellow = 1,
           blue   = 2,
           purple = 3,
           orange = 4,
           green  = 5,
           )

DMG = dict(fire        = 0,
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


Thought = namedtuple('Thought', 'tree icount skipped')

class Parser(object):
    '''Handles parsing from dna and returning a parse tree which represents a
    creature's tthought process in making encounter decisions'''

    WAIT_THOUGHT = (COND['always'], (ACT['wait'],))

    def __init__(self, dna):
        self._icount = 0
        self._progress = 0
        self._skipped = 0
        self._dna = dna
        self._len = len(dna)
        self._depth = 0

    def next(self):
        '''Parses a dna iterator (Must not be the dna list!) into a tree
        structure that represents a creature's thought process in making
        encounter decisions'''
        self._icount = 0
        self._skipped = 0
        self._depth = 0
        return Thought(tree = self._cond(), 
                       icount = self._icount,
                       skipped = self._skipped)

    def _cond(self):
        '''Parses a conditional node'''
        #get the condition type symbol
        cond_typ = self._get_next_valid(COND)
        
        if cond_typ == COND['always']:
            return (COND['always'],
                    self._act())
        elif cond_typ == COND['in_range']:
            return (COND['in_range'],
                    self._val, 
                    self._val, 
                    self._val,
                    self._act(), 
                    self._act())
        elif COND['less_than'] <= cond_typ <= COND['not_equal_to']:
            return (cond_typ, 
                    self._val, 
                    self._val,
                    self._act(), 
                    self._act())
        elif cond_typ in [COND['me_last_act'], COND['target_last_act']]:
            return(cond_typ, 
                   self._act(nosub = True),
                   self._act(),
                   self._act())
    
    @property
    def _val(self):
        '''Parses a value node'''
        val_typ = self._get_next_valid(VAL)
        
        if val_typ == VAL['literal']:
            self._icount += 1
            return (VAL['literal'], 
                    self._dna[self._progress % self._len])
        elif val_typ == VAL['random']:
            return (VAL['random'],)
        elif val_typ in [VAL['me'], VAL['target']]:
            return (val_typ,
                    self._get_next_valid(ATTR))

    def _act(self, nosub = False):
        '''Parses an action node'''
        if self._depth > sd.settings.max_tree_depth:
            raise TooMuchThinkingError('Recursion depth exceeded',
                                       icount=0,
                                       skipped=sd.settings.max_thinking_steps)
        act_typ = self._get_next_valid(ACT, minimum = 1 if nosub else 0)

        if act_typ == ACT['subcondition']:
            # you're thinking, "increment a variable before a function call and
            # decrement it afterward? Why not use the call stack?!" Well, I'll
            # tell you why not: I don't want to thread a depth argument through
            # all of the other parser method calls. That's why this is a class!
            self._depth += 1
            retval = (ACT['subcondition'], 
                      self._cond())
            self._depth -= 1
            return retval
        elif act_typ in [ACT['attack'], ACT['defend']]:
            return (act_typ,
                    self._get_next_valid(DMG))
        elif act_typ == ACT['signal']:
            return (ACT['signal'],
                    self._get_next_valid(SIG))
        elif act_typ in (ACT['use'], ACT['take'], ACT['mate'],
                         ACT['wait'], ACT['flee']):
            return (act_typ,)

    def _get_next_valid(self, typ, minimum = 0):
        '''Gets the next valid integer in the range allowed by the given
        type. Adds on to the `count` passed in. mini and maxi allow one to
        restrict the range further than `typ` does'''
        next_val = self._dna[self._progress % self._len]
        self._icount += 1
        self._progress += 1
        # we want to return a count 1 less than the number required, since we
        # dont want to penalize for required parser symbols. Therefore if the
        # while condition succeeds the first time through, count is not
        # incremented
        while not( minimum <= next_val < len(typ) ):
            next_val = self._dna[self._progress % self._len]
            self._skipped += 1
            if self._icount + self._skipped > sd.settings.max_thinking_steps:
                raise TooMuchThinkingError('Recursion depth exceeded',
                                           icount = 0,
                                           skipped=sd.settings.max_thinking_steps)
        return next_val




if __name__ == '__main__':
    from random import randint
    from EvoFighters.Utils import show_thought
    for _ in xrange(500):
        p = Parser([randint(-1, 9) for _ in xrange(50)])
        show_thought(next(p).tree)
