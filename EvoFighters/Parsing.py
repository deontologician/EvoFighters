'''Handles parsing of streams of integers into instructions for the EvoFighters'''

MAX_THINKING_STEPS = 200

#This is some fancy metaclass hackery that allows enums that support the len()
#function
MetaEnum = type('MetaEnum', (type,), {'__len__': lambda self: self.clslen()})
def enum(*sequential, **named):
    strings = type('StringReprs', (object,), dict(zip(sequential, sequential)))()
    enums = dict(zip(sequential, range(len(sequential))), **named)
    enums['str'] = strings
    enums['clslen'] = classmethod(lambda cls: len(sequential) + len(named))
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
    def __init__(self, msg, icount, skipped):
        Exception.__init__(self, msg)
        self.icount = icount
        self.skipped = skipped

class CantCountError(Exception):
    pass

def parse_condition(dna_iter, count = (0,0)):
    '''Parses the dna generator (Must not be the dna list!) into a tree
    structure of battle conditionals. `dna_iter` is the stream to parse from,
    and count is the total symbols read so far.'''
    #get the condition type symbol
    cond_type, count0 = get_next_valid(Condition, dna_iter, count)

    if cond_type == Condition.always:
        act, count1 = parse_action(dna_iter, count0)
        return (Condition.str.always, act), count1
    elif cond_type == Condition.enemy_has:
        item, count1 = get_next_valid(Item, dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return (Condition.str.enemy_has, item, thenAct, elseAct), count3
    elif cond_type == Condition.me_has:
        item, count1 = get_next_valid(Item, dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return (Condition.str.me_has, item, thenAct, elseAct), count3
    elif cond_type == Condition.enemy_energy:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return (Condition.str.enemy_energy, comparison, thenAct, elseAct), count3
    elif cond_type == Condition.me_energy:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return (Condition.str.me_energy, comparison, thenAct, elseAct), count3
    elif cond_type == Condition.enemy_signal:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return (Condition.str.enemy_signal, comparison, thenAct, elseAct), count3
    elif cond_type == Condition.me_signal:
        comparison, count1 = parse_comparison(dna_iter, count0)
        thenAct, count2 = parse_action(dna_iter, count1)
        elseAct, count3 = parse_action(dna_iter, count2)
        return (Condition.str.me_signal, comparison, thenAct, elseAct), count3
    elif cond_type == Condition.enemy_last_act:
        enemyAct, count1 = parse_action(dna_iter, count0, nosub = True)
        thenAct, count2   = parse_action(dna_iter, count1)
        elseAct, count3   = parse_action(dna_iter, count2)
        return (Condition.str.enemy_last_act, enemyAct, thenAct, elseAct), count3
    elif cond_type == Condition.me_last_act:
        myAct, count1 = parse_action(dna_iter, count0, nosub = True)
        thenAct, count2   = parse_action(dna_iter, count1)
        elseAct, count3   = parse_action(dna_iter, count2)
        return (Condition.str.me_last_act, myAct, thenAct, elseAct), count3
    else:
        raise ParseError("Condition didn't match: {}".format(cond_type))
        

def parse_action(dna_iter, count, nosub = False):
    '''Parses an Action'''
    act_typ, count0 = get_next_valid(Action, dna_iter, count,
                                     mini = 1 if nosub else 0)
    if act_typ == Action.attack:
        attack, count1 = get_next_valid(Damage, dna_iter, count0)
        return (Action.str.attack, attack), count1
    elif act_typ == Action.defend:
        defense, count1 = get_next_valid(Damage, dna_iter, count0)
        return (Action.str.defend, defense), count1
    elif act_typ == Action.use:
        return (Action.str.use,), count0
    elif act_typ == Action.take:
        return (Action.str.take,), count0
    elif act_typ == Action.signal:
        signal, count1 = get_next_valid(Signal, dna_iter, count0)
        return (Action.str.signal, signal), count1
    elif act_typ == Action.wait:
        return (Action.str.wait,), count0
    elif act_typ == Action.subcondition:
        condition, count1 = parse_condition(dna_iter, count0)
        return (Action.str.subcondition, condition), count1
    else:
        raise ParseError("Action didn't match: {}".format(act_typ))

def parse_comparison(dna_iter, (count, skipped)):
    '''Parses a Comparison'''
    comp_typ, (count0, skipped0) = get_next_valid(Compare, dna_iter,
                                                  (count, skipped))
    if comp_typ == Compare.inrange:
        arg1 = dna_iter.next()
        arg2 = dna_iter.next()
        return (Compare.str.inrange, arg1, arg2), (count0 + 2, skipped0)
    if comp_typ == Compare.lessthan:
        arg1 = dna_iter.next()
        return (Compare.str.lessthan, arg1), (count0 + 1, skipped0)
    if comp_typ == Compare.greaterthan:
        arg1 = dna_iter.next()
        return (Compare.str.greaterthan, arg1), (count0 + 1, skipped0)
    if comp_typ == Compare.equalto:
        arg1 = dna_iter.next()
        return (Compare.str.equalto, arg1), (count0 + 1, skipped0)
    if comp_typ == Compare.notequalto:
        arg1 = dna_iter.next()
        return (Compare.str.notequalto, arg1), (count0 + 1, skipped0)
    else:
        raise ParseError("Compare didn't match")

def get_next_valid(typ, dna_iter, (icount, skipped), mini = 0, maxi = 10):
    '''Gets the next valid integer in the range allowed by the given type. Adds
    on to the `count` passed in. mini and maxi allow one to restrict the range
    further than `typ` does'''
    next_val = dna_iter.next()
    icount += 1
    minimum, maximum = max(0, mini), min(len(typ), maxi)
    # we want to return a count 1 less than the number required, since we dont
    # want to penalize for required parser symbols. Therefore if the while
    # condition succeeds the first time through, count is not incremented
    while not( minimum <= next_val < maximum ):
        next_val = dna_iter.next()
        skipped += 1
        if icount + skipped > MAX_THINKING_STEPS:
            raise TooMuchThinkingError('Thought too much :/', icount, skipped)
    return next_val, (icount, skipped)

def ind(i):
    return ' '*i

def explain_plan(tree):
    return cond_repr(tree)

def _cond_helper(thenTree, elseTree, indent = 0):
    p = ':\n{thenTree}\n{i}else:\n{elseTree}'\
        .format(i = ind(indent), thenTree = act_repr(thenTree, indent + 4),
                elseTree = act_repr(elseTree, indent + 4))
    return p
                                                          

def cond_repr(tree, indent = 0):
    cond_typ = tree[0]
    if cond_typ == 'always':
        return "{i}Always:\n{}".format(act_repr(tree[1], indent + 4),
                                          i = ind(indent) )
    elif cond_typ == Condition.str.enemy_has:
        return "{i}if the enemy has {}{}".format(item_repr(tree[1]), 
                                           _cond_helper(tree[2],tree[3],indent),
                                           i = ind(indent))
    elif cond_typ == Condition.str.me_has:
        return "{i}if I have {}{}".format(item_repr(tree[1]),
                                          _cond_helper(tree[2], tree[3],indent),
                                          i = ind(indent))
    elif cond_typ == Condition.str.enemy_energy:
        return "{i}if the enemy has an energy that is {}{}"\
            .format(comp_repr(tree[1]), _cond_helper(tree[2],tree[3], indent),
                    i = ind(indent))
    elif cond_typ == Condition.str.me_energy:
        return "{i}if I have an energy that is {}{}"\
            .format(comp_repr(tree[1]), _cond_helper(tree[2],tree[3], indent),
                    i = ind(indent))
    elif cond_typ == Condition.str.enemy_signal:
        return "{i}if the enemy's signal is {}{}"\
            .format(comp_repr(tree[1], sig_repr),
                    _cond_helper(tree[2],tree[3], indent),
                    i = ind(indent))
    elif cond_typ == Condition.str.me_signal:
        return "{i}if my signal is {}{}"\
            .format(comp_repr(tree[1], sig_repr),
                    _cond_helper(tree[2],tree[3], indent),
                    i = ind(indent))
    elif cond_typ == Condition.str.enemy_last_act:
        return "{i}if the enemy's last action was {}{}"\
            .format(act_repr(tree[1]),
                    _cond_helper(tree[2],tree[3], indent),
                    i = ind(indent))
    elif cond_typ == Condition.str.me_last_act:
        return "{i}if my last action was {}{}"\
            .format(act_repr(tree[1]),
                    _cond_helper(tree[2],tree[3], indent),
                    i = ind(indent))
    else:
        return '{i}Unknown Condition({})'.format(cond_typ, i = ind(indent))

def comp_repr(tree, formatter = lambda x:x):
    comp_typ = tree[0]
    if comp_typ == Compare.str.inrange:
        low = min(tree[1], tree[2])
        high = max(tree[1], tree[2])
        return 'from {} to {}'.format(formatter(low), formatter(high))
    elif comp_typ == Compare.str.lessthan:
        return 'less than {}'.format(formatter(tree[1]))
    elif comp_typ == Compare.str.greaterthan:
        return 'greater than {}'.format(formatter(tree[1]))
    elif comp_typ == Compare.str.equalto:
        return '{}'.format(formatter(tree[1]))
    elif comp_typ == Compare.str.notequalto:
        return 'not {}'.format(formatter(tree[1]))
    else:
        return 'Unknown Comparison ({})'.format(comp_typ)


def item_repr(item):
    if item == Item.food:
        return "a regular food"
    elif item == Item.ice_food:
        return "an ice food"
    elif item == Item.fire_food:
        return "a fire food"
    elif item == Item.electric_food:
        return "an electric food"
    else:
        return "an unknown item({})".format(item)

def act_repr(tree, indent = 0):
    act_typ = tree[0]
    if act_typ == Action.str.subcondition:
        return cond_repr(tree[1], indent)
    elif act_typ == Action.str.take:
        return '{}take enemy item'.format(ind(indent))
    elif act_typ == Action.str.attack:
        return '{}attack with {}'.format(ind(indent), dmg_repr(tree[1]))
    elif act_typ == Action.str.defend:
        return '{}defend against {}'.format(ind(indent), dmg_repr(tree[1]))
    elif act_typ == Action.str.use:
        return '{}use top inventory item'.format(ind(indent))
    elif act_typ == Action.str.signal:
        return '{}signal {}'.format(ind(indent), sig_repr(tree[1]))
    elif act_typ == Action.str.wait:
        return '{}wait'.format(ind(indent))
    else:
        return '{}Unknown action ({})'.format(ind(indent), tree[1])
    
def dmg_repr(damage):
    if damage == Damage.fire:
        return 'Fire'
    elif damage == Damage.ice:
        return 'Ice'
    elif damage == Damage.electricity:
        return 'Electricity'
    else:
        return 'Unknown Damage Type({})'.format(damage)


def sig_repr(signal):
    if signal == Signal.red:
        return 'Red'
    elif signal == Signal.yellow:
        return 'Yellow'
    elif signal == Signal.blue:
        return 'Blue'
    elif signal == Signal.purple:
        return 'Purple'
    elif signal == Signal.orange:
        return 'Orange'
    elif signal == Signal.green:
        return 'Green'
    else:
        return 'Signal({})'.format(signal)

