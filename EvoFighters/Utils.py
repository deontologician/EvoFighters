'''Various odds and ends that are used throughout the other EvoFighters
modules'''

from __future__ import print_function

from cStringIO import StringIO
import os, sys

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


def _print_helper(*args, **kwargs):
    if 'prefix' in kwargs:
        prefix = kwargs['prefix']
        del kwargs['prefix']
    else:
        prefix = ''
    tmp = StringIO()
    print(*args, file = tmp, **kwargs)
    b =  tmp.getvalue().splitlines()
    bp = '\n'.join(['{} {}'.format(prefix, line) for line in b])
    print(bp)

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

if __name__ == '__main__':
    import doctest
    doctest.testmod()
