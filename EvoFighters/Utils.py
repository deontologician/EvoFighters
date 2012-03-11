'''Various odds and ends that are used throughout the other EvoFighters
modules'''

from __future__ import print_function

import cStringIO as StringIO
import sys

__all__ = ['print1', 'print2', 'print3', 'set_verbosity', 'get_verbosity', 'progress_bar']

_verbosity = 0

def set_verbosity(lvl):
    global _verbosity
    _verbosity = lvl

def get_verbosity(lvl):
    return _verbosity

def print1(*args, **kwargs):
    if _verbosity >= 1:
        _print_helper(*args, prefix='***',**kwargs)

def print2(*args, **kwargs):
    if _verbosity >= 1:
        _print_helper(*args, prefix='**', **kwargs)

def print3(*args, **kwargs):
    if _verbosity >= 1:
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


def progress_bar(total, title = 'Doing {} fights', width = 80 ):
    '''Generator to create a progress bar with pipes'''
    print(title.format(total))
    rounds_per_pipe = width / float(total)
    printed = 0
    for i in xrange(0, total - 1):
        pipes_to_add = int(i * rounds_per_pipe) - printed
        print('|' * pipes_to_add, end = '')
        sys.stdout.flush()
        printed += pipes_to_add
        quit = yield
        if quit:
            break
    print('')
    sys.stdout.flush()
    yield

if __name__ == '__main__':
    import doctest
    doctest.testmod()
