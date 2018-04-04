#!/usr/bin/env python2
'''Creates a family tree from the currently living generation'''

from SaveData import SaveData
import pygraphviz as pgv
import sys
from collections import Counter
import argparse

def join(creatures, dead):
    for c in creatures:
        dead[c.name] = c
    return dead

def segment(alive, dead):
    candidates = {alive[0].name : alive[0]}#{ c.name : c for c in alive }
    combined = join(alive, dead)
    vetted = {}
    layer = 0
    while candidates:
        print >>sys.stderr, 'Got {} candidates to work through'.format(len(candidates))
        counter = Counter(c.fullname for c in candidates.values())
        print 'Gene summary for layer {} ({} creatures):'.format(layer, len(candidates))
        for val, count in counter.most_common():
            print count, 'x', val
        print '============'
        old_candidates = candidates.copy()
        candidates = {}
        for c in old_candidates.values():
            if c.parents:
                p1, p2 = c.parents
                if p1 not in vetted and p1 not in old_candidates and p1 in combined:
                    candidates[p1] = combined[p1]
                if p2 not in vetted and p2 not in old_candidates and p2 in combined:
                    candidates[p2] = combined[p2]
            vetted[c.name] = c
        layer += 1
    return vetted

def create_graph(vetted):
    print 'Creating graph...'
    G = pgv.AGraph(directed = True)
    total = len(vetted)
    last_percent = 0
    for i, c in enumerate(vetted.values()):
        percent = int((float(i) / total) * 100)
        if percent > last_percent:
            last_percent = percent
            print '\r{:3}%'.format(percent),
            sys.stdout.flush()
        if c.parents is not None:
            p1, p2 = c.parents
            G.add_edge(p1, c.name)
            G.add_edge(p2, c.name)
        else:
            G.add_edge('First Generation', c.name)
    return G


def layout_and_draw(G):
    print 'Laying out...'
    G.layout(prog = 'dot')
    print 'Drawing...'
    G.draw('family_tree.svg')

def parse_args(argv):
    parser = argparse.ArgumentParser(description = 'A script to do various analysis')

def main(argv):
    #args = parse_args(argv)
    if len(argv) > 1:
        filename = argv[1]
    else:
        filename = 'evofighters.save'
    print 'Opening {}...'.format(filename)
    with open(filename, 'r') as fd:
        sd = SaveData.loadfrom(fd)
    print 'Segmenting based on survivors...'
    vetted = segment(sd.creatures, sd.dead)
    sd = None

    G = create_graph(vetted)
    vetted = None

    layout_and_draw(G)

if __name__ == '__main__':
    main(sys.argv)

