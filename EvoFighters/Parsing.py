'''Handles parsing of streams of integers into instructions for the EvoFighters'''

MAX_THINKING_STEPS = 200

#This is some fancy metaclass hackery that allows enums that support the len() function
MetaEnum = type('MetaEnum', (type,), {'__len__': lambda self: self.clslen()})
def enum(*sequential, **named):
    enums = dict(zip(sequential, range(len(sequential))), **named)
    enums['clslen'] = classmethod(lambda cls: len(sequential))
    return MetaEnum('Enum', (object,), enums)

Action = enum('subcondition', # indicates a subconditional
              'take', # takes top item in enemies inventory
              'attack', # attacks enemy with specified attack type (1 arg)
              'defend', # defends with specified defense type (1 arg)
              'use', # uses the top item in the inventory
              'signal', # sets a flag on the creature (1 arg)
              'wait', # do nothing
              )

Compare = enum('inrange',
               'lessthan',
               'greaterthan',
               'equalto',
               'notequalto',
               )

Condition = enum('always',
                 'enemy_has',
                 'me_has',
                 'enemy_energy',
                 'me_energy',
                 'enemy_signal',
                 'me_signal',
                 'enemy_last_act',
                 'me_last_act',
                 )

Item = enum('food',
            'ice_food',
            'fire_food',
            'electric_food')

Signal = enum('red',
              'yellow',
              'blue',
              'purple',
              'orange',
              'green'
              )

Damage = enum('fire',
              'ice',
              'electricity',
              )

class ParseError(Exception):
    pass


class TooMuchThinkingError(Exception):
    pass


def parse_condition(dna_iter, count = 0):
    '''Parses the dna generator (Must not be the dna list!) into a tree
    structure of battle conditionals. `dna_iter` is the stream to parse from,
    and count is the total symbols read so far.'''
    #get the condition type symbol
    cond_type, count0 = get_next_valid(Condition, dna_iter, count)

    if cond_type == Condition.always:
        act, count1 = parse_action(dna_iter, count0)
        return ('always', act), count1
    elif cond_type == Condition.enemy_has:
        item, count1 = get_next_valid(Item, dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return ('enemy_has', item, thenAct, elseAct), count3
    elif cond_type == Condition.me_has:
        item, count1 = get_next_valid(Item, dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return ('me_has', item, thenAct, elseAct), count3
    elif cond_type == Condition.enemy_energy:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return ('enemy_energy', comparison, thenAct, elseAct), count3
    elif cond_type == Condition.me_energy:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return ('me_energy', comparison, thenAct, elseAct), count3
    elif cond_type == Condition.enemy_signal:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return ('enemy_signal', comparison, thenAct, elseAct), count3
    elif cond_type == Condition.me_signal:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return ('me_signal', comparison, thenAct, elseAct), count3
    elif cond_type == Condition.enemy_last_act:
        enemyAct, count1 = parse_action(dna_iter, count0, nosub = True)
        thenAct, count2   = parse_action(dna_iter, count1)
        elseAct, count3   = parse_action(dna_iter, count2)
        return ('enemy_last_act', enemyAct, thenAct, elseAct), count3
    elif cond_type == Condition.me_last_act:
        myAct, count1 = parse_action(dna_iter, count0, nosub = True)
        thenAct, count2   = parse_action(dna_iter, count1)
        elseAct, count3   = parse_action(dna_iter, count2)
        return ('me_last_act', myAct, thenAct, elseAct), count3
    else:
        raise ParseError("Condition didn't match")
        

def parse_action(dna_iter, count, nosub = False):
    '''Parses an Action'''
    act_typ, count0 = get_next_valid(Action, dna_iter, count,
                                     mini = 1 if nosub else 0)
    if act_typ == Action.attack:
        attack, count1 = get_next_valid(Damage, dna_iter, count0)
        return ('attack', attack), count1
    elif act_typ == Action.defend:
        defense, count1 = get_next_valid(Damage, dna_iter, count0)
        return ('defend', defense), count1
    elif act_typ == Action.use:
        return ('use',), count0
    elif act_typ == Action.take:
        return ('take',), count0
    elif act_typ == Action.signal:
        signal, count1 = get_next_valid(Signal, dna_iter, count0)
        return ('signal', signal), count1
    elif act_typ == Action.wait:
        return ('wait',), count0
    elif act_typ == Action.subcondition:
        condition, count1 = parse_condition(dna_iter, count0)
        return ('subcondition', condition), count1
    else:
        raise ParseError("Action didn't match")

def parse_comparison(dna_iter, count):
    '''Parses a Comparison'''
    comp_typ, count0 = get_next_valid(Compare, dna_iter, count)
    if comp_typ == Compare.inrange:
        arg1 = dna_iter.next()
        arg2 = dna_iter.next()
        count1 = count0 + 2
        return ('inrange', arg1, arg2), count1
    if comp_typ == Compare.lessthan:
        arg1 = dna_iter.next()
        count1 = count + 1
        return ('lessthan', arg1), count1
    if comp_typ == Compare.greaterthan:
        arg1 = dna_iter.next()
        count1 = count + 1
        return ('greaterthan', arg1), count1
    if comp_typ == Compare.equalto:
        arg1 = dna_iter.next()
        count1 = count + 1
        return ('equalto', arg1), count1
    if comp_typ == Compare.notequalto:
        arg1 = dna_iter.next()
        count1 = count + 1
        return ('notequalto', arg1), count1
    else:
        raise ParseError("Compare didn't match")

def get_next_valid(typ, dna_iter, count, mini = 0, maxi = 10):
    '''Gets the next valid integer in the range allowed by the given type. Adds
    on to the `count` passed in. mini and maxi allow one to restrict the range
    further than `typ` does'''
    next_val = dna_iter.next()
    count += 1
    minimum, maximum = max(0, mini), min(len(typ), maxi)
    while not( minimum <= next_val < maximum ):
        next_val = dna_iter.next()
        count += 1
        if count > MAX_THINKING_STEPS:
            raise TooMuchThinkingError('Thought too much :/')
    return next_val, count
