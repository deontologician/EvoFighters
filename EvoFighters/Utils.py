'''Various odds and ends that are used throughout the other EvoFighters
modules'''

from __future__ import print_function

import os, sys
from Parsing import ITEM, ATTR, COND, VAL, ACT, DMG, SIG
from collections import Counter

_verbosity = 0

def set_verbosity(lvl):
    global _verbosity
    _verbosity = lvl

def get_verbosity():
    return _verbosity

def print1(*args, **kwargs):
    if _verbosity >= 1:
        _print_helper(*args, prefix='***',**kwargs)

def print2(*args, **kwargs):
    if _verbosity >= 2:
        _print_helper(*args, prefix='**', **kwargs)

def print3(*args, **kwargs):
    if _verbosity >= 3:
        _print_helper(*args, prefix='*', **kwargs)


def _print_helper(fmt, *args, **kwargs):
    if 'prefix' in kwargs:
        prefix = kwargs['prefix']
        del kwargs['prefix']
    else:
        prefix = ''
    if 'thought' in kwargs:
        kwargs['thought'] = show_thought(kwargs['thought'])
    if 'sig_repr' in kwargs:
        kwargs['sig_repr'] = sig_repr(kwargs['sig_repr'])
    if 'item_repr' in kwargs:
        kwargs['item_repr'] = item_repr(kwargs['item_repr'])
    if 'val_repr' in kwargs:
        kwargs['val_repr'] = val_repr(kwargs['val_repr'])
    if 'val_repr2' in kwargs:
        kwargs['val_repr2'] = val_repr(kwargs['val_repr2'])
    if 'act_repr' in kwargs:
        kwargs['act_repr'] = act_repr(kwargs['act_repr'])

    formatted = fmt.format(*args, **kwargs)
    lines = ["{} {}".format(prefix, line) for line in formatted.splitlines()]
    print('\n'.join(lines))




def term_width():
    '''A (probably not super portable) way to get the terminal width. Try to
    cache this value at reasonable points since it is slow to pop open a
    subprocess every time you want this value'''
    return int(os.popen('stty size', 'r').read().split()[1])

def progress_bar(fmt_str, *args, **kwargs):
    r'''Generator to create a pipish progress bar. `progress` is a float from
    0.0 to 1.0 representing the progress intended to be represented. It uses \r
    to overwrite the line it is printed on, so always print a newline before
    calling this function'''
    width = term_width()
    move_up_1 = '\033[1A'
    def _prog_gen(width = width):
        print()
        while True:
            progress = yield
            total_bars = (width - 5) # don't have super narrow terminals!
            num_bars = int(round(total_bars * progress))
            _kwargs = {k : v() for k,v in kwargs.iteritems()}
            _args = [x() for x in args]
            msg = fmt_str.format(*_args, **_kwargs)
            print('{moveup}\r{prog:3.0f}% {bars}\n{msg}'.format(moveup = move_up_1,
                                                        prog = progress * 100,
                                                        bars = '|' * num_bars
                                                        , msg = msg), end = '')
            sys.stdout.flush()
    # this is to get the generator initialized
    val = _prog_gen()
    next(val)
    return val




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

    if cond_typ == COND['always']:
        act = act_repr(tree[1])
        return "Always:\n{}".format(indent(act))
    elif cond_typ == COND['in_range']:
        rng_min = val_repr(tree[1])
        rng_max = val_repr(tree[2])
        match = val_repr(tree[3])
        cond_str = '{} is in the range {} to {}'.format(match, rng_min, rng_max)
        then_str = act_repr(tree[4])
        else_str = act_repr(tree[5])
        return branch_repr(cond_str, then_str, else_str)
    elif COND['less_than'] <= cond_typ <= COND['not_equal_to']:
        val1 = val_repr(tree[1])
        val2 = val_repr(tree[2])
        if cond_typ == COND['less_than']:
            relation = 'is less than'
        elif cond_typ == COND['greater_than']:
            relation = 'is greater than'
        elif cond_typ == COND['equal_to']:
            relation = 'is equal to'
        elif cond_typ == COND['not_equal_to']:
            relation = 'is not equal to'
        else:
            raise NotImplementedError('This cant happen! {}'.format(cond_typ))
        cond_str = '{} {} {}'.format(val1, relation, val2)
        then_str = act_repr(tree[3])
        else_str = act_repr(tree[4])
        return branch_repr(cond_str, then_str, else_str)
    elif cond_typ in [COND['me_last_act'], COND['target_last_act']]:
        matchact = act_repr(tree[1])
        if cond_typ == COND['me_last_act']:
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
    
    if val_typ == VAL['literal']:
        return str(tree[1])
    elif val_typ == VAL['random']:
        return 'a random number'
    elif val_typ in [VAL['me'], VAL['target']]:
        who = 'my' if val_typ == VAL['me'] else "the target's"
        attr = attr_repr(tree[1])
        return '{} {}'.format(who, attr)
    else:
        return 'Unknown Value({})'.format(val_typ)

def act_repr(tree):
    '''Creates a string from an action tree node'''
    act_typ = tree[0]
    
    if act_typ == ACT['subcondition']:
        return cond_repr(tree[1])
    elif act_typ in [ACT['attack'], ACT['defend']]:
        dmg_str = dmg_repr(tree[1])
        if act_typ == ACT['attack']:
            return 'attack with {}'.format(dmg_str)
        else:
            return 'defend with {}'.format(dmg_str)
    elif act_typ == ACT['signal']:
        sig = sig_repr(tree[1])
        return 'signal {}'.format(sig)
    elif act_typ == ACT['use']:
        return 'use the top inventory item'
    elif act_typ == ACT['take']:
        return "take the target's top inventory item"
    elif act_typ == ACT['wait']:
        return 'wait'
    elif act_typ == ACT['flee']:
        return 'flee from the encounter'
    elif act_typ == ACT['mate']:
        return 'attempt to mate with the target'
    else:
        return 'Unknown Action({})'.format(act_typ)


def attr_repr(attr):
    '''Creates a string from an attribute code'''
    if attr == ATTR['energy']:
        return 'energy level'
    elif attr == ATTR['signal']:
        return 'signal'
    elif attr == ATTR['generation']:
        return 'generation'
    elif attr == ATTR['kills']:
        return 'kills'
    elif attr == ATTR['survived']:
        return 'number of encounters survived'
    elif attr == ATTR['num_children']:
        return 'number of children'
    elif attr == ATTR['top_item']:
        return 'top inventory item'
    else:
        return 'Unknown attribute({})'.format(attr)


def item_repr(item):
    '''Creates a string from an item code'''
    if item == ITEM['food']:
        return "a bread"
    elif item == ITEM['good_food']:
        return "a cheese"
    elif item == ITEM['better_food']:
        return "a fruit"
    elif item == ITEM['excellent_food']:
        return "a chocolate"
    else:
        return "Unknown Item({})".format(item)
 
def inv_repr(inv):
    'A string that represents an inventory succinctly'
    c = Counter(inv)
    fixup = lambda x: ' '.join(x.split()[1:])
    return ', '.join('{} {}'.format(c, fixup(item_repr(i))) for i,c in c.iteritems())


def dmg_repr(damage):
    '''Creates a string from a damage code'''
    if damage == DMG['fire']:
        return 'Fire'
    elif damage == DMG['ice']:
        return 'Ice'
    elif damage == DMG['electricity']:
        return 'Electricity'
    else:
        return 'Unknown Damage Type({})'.format(damage)


def sig_repr(signal):
    '''Creates a string from a signal code'''
    if signal == SIG['red']:
        return 'Red'
    elif signal == SIG['yellow']:
        return 'Yellow'
    elif signal == SIG['blue']:
        return 'Blue'
    elif signal == SIG['purple']:
        return 'Purple'
    elif signal == SIG['orange']:
        return 'Orange'
    elif signal == SIG['green']:
        return 'Green'
    else:
        return 'Unknown Signal({})'.format(signal)

def dna_repr(dna):
    'Represents DNA as blocks of color'
    pass
    
if __name__ == '__main__':
    import doctest
    doctest.testmod()
