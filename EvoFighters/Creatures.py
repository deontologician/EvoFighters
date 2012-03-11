'''Contains Creature class and all genetic related functionality'''

from collections import namedtuple
from random import randint
from itertools import cycle, takewhile
from math import floor

import cPickle as pickle
import Parsing as P
import struct
import random as rand

from Eval import PerformableAction, evaluate
from Utils import print1, print2, print3

#this is just temporarily here because thinking penalty is going away
thinking_penalty = 20.0 # higher = more thinking allowed
# need to move this into a config file
mutation_rate = 0.05 # higher = more mutations

CreatureTuple = namedtuple('CreatureTuple', 
                           'dna inv energy age survived won used skipped')
class Creature(object):
    def __init__(self, dna = None):
        if dna is None:
            self.dna = [randint(-1,9) for _ in xrange(0,50)]
        else:
            self.dna = dna
        self.dna_cycler = cycle(self.dna)
        self.inv = []
        self.energy = 40
        self.target = None
        self.age = 0
        self.signal = -1
        self.fights_survived = 0
        self.fights_won = 0
        self.instructions_used = 0
        self.instructions_skipped = 0
        self.last_action = PerformableAction('wait', None)

    def __str__(self):
        return "<]Creature {0.name}[>".format(self)
    
    def __repr__(self):
        return \
'''[]{0.name:=^76}[]
DNA: {0.fullname}
Inventory: {inv}
Energy: {0.energy}
Age: {0.age}
Survived: {0.fights_survived}
Won: {0.fights_won}
Instructions used/skipped: {0.instructions_used}/{0.instructions_skipped}
[]{equals}[]'''.format(self, inv = ','.join([str(i) for i in self.inv]),
                       equals = '='*76)
    
    @property
    def pickled(self):
        '''A pickled form of this creature that can be used to reconstruct him
        later with the static method :func: `from_pickle`'''
        c = CreatureTuple(self.dna, self.inv, self.energy, self.age, 
                          self.fights_survived, self.fights_won, 
                          self.instructions_used, self.instructions_skipped)
        return pickle.dumps(c, 2)

    @staticmethod
    def from_pickle(pickled):
        '''Returns a Creature from a pickled creature'''
        c = pickle.loads(pickled)
        nc = Creature(c.dna)
        nc.inv = c.inv
        nc.energy = c.energy
        nc.age = c.age
        nc.fights_survived = c.survived
        nc.fights_won = c.won
        nc.instructions_used = c.used
        nc.instructions_skipped = c.skipped
        return nc

    @property
    def copy(self):
        return Creature.from_pickle(self.pickled)

    @property
    def fullname(self):
        return ''.join(map(lambda x: str(x) if x != -1 else '|', self.dna))

    @property
    def name(self):
        'A simple short name that is probably unique'
        def sum_and_encode(gene):
            return struct.pack('b',sum(gene) % 256 - 128)\
                  .encode('base_64').rstrip('=\n')
        return ''.join([ sum_and_encode(gene) for gene in \
                            takewhile(lambda x:x, gene_primer(self.dna))])

    @property
    def dead(self):
        return self.energy <= 0

    @property
    def alive(self):
        return self.energy > 0

    @property
    def next_action(self):
        '''Reads dna to decide next course of action. Outputs verbiage'''
        try: 
            thought_process, (icount, skipped) = P.parse_condition(self.dna_cycler)
            print3("{0.name}'s thought process:".format(self))
            print3(P.explain_plan(thought_process))
            print3('which required', icount, 'instructions.',
                   'and', skipped, 'instructions skipped over')
            self.instructions_used += icount
            self.instructions_skipped += skipped
        except P.TooMuchThinkingError:
            self.energy -= 5
            return PerformableAction('wait', None)
            print1(self.name,'got caught thinking too much!')
            print2(self.name,'loses 5 life!')
        energy_loss = int(floor(skipped / thinking_penalty))
        if energy_loss > 0:
            print1(self.name,'lost', energy_loss, 'energy due to thinking')
            self.energy -= energy_loss
        decision = evaluate(self, thought_process)
        print2(self.name, 'decided to', decision)
        return decision

    def reset_cycle(self):
        self.dna_cycler = cycle(self.dna)
        
    def use(self):
        'Does something with inventory items'
        if self.inv:
            item = self.inv.pop()
            if 0 <= item <= len(P.Item):
                mult = item + 1
            else:
                mult = 0
            energy_gain = min(40 - self.energy, self.dna_cycler.next() * mult)
            print2(self.name, 'gains', energy_gain, 'life from', P.item_repr(item))
            self.energy += energy_gain


def gene_primer(dna):
    '''Breaks a dna list into chunks by the terminator -1.'''
    chunk = []
    #dna_iter = iter(dna)
    for i in iter(dna):
        chunk.append(i)
        if i == -1:
            yield chunk
            chunk = []
    if chunk:
        yield chunk
        chunk = []
    while True:
        #just keep yielding empty chunks rather than raising StopIteration
        yield chunk

def mate(p1, p2):
    '''Takes in two creatures, splices their dna together randomly by chunks,
    possibly mutates it, then spits out a new creature. Mutation rate is the
    chance that a mutation will occur'''
    # chunkify the dna
    dna1_primer = gene_primer(p1.dna)
    dna2_primer = gene_primer(p2.dna)
    dna3 = []
    while True:
        gene1 = dna1_primer.next()
        gene2 = dna2_primer.next()
        if gene1 == [] and gene2 == []:
            break
        gene3 = rand.choice([gene1, gene2])
        dna3.append(gene3)
    if rand.uniform(0,1) < mutation_rate:
        if randint(1,4) == 1:
            transpose(dna3)
        if randint(1,4) > 1: # yes both branches can happen
            index = randint(0, len(dna3) - 1)
            print2('mutating gene', index)
            dna3[index] = mutate(dna3[index])
    return Creature([base for gene in dna3 for base in gene])

def transpose(genome):
    length = len(genome)
    i1 = randint(0, length - 1)
    i2 = randint(0, length - 1)
    print2('swapped gene', i1, 'and', i2)
    genome[i1], genome[i2] = genome[i2], genome[i1]

def mutate(gene):
    '''Does a mutation on a gene in various different ways'''
    def _invert(x):
        x.reverse()
        print2('reversed gene')
        return x
    def _delete(x):
        print2('deleted gene')
        return []
    def _insert(x):
        val = randint(-1,9)
        index = randint(0, len(x) - 1)
        x.insert(index, val)
        print2('inserted {} at {}'.format(val, index))
        return x
    def _duplicate(x):
        x.extend(x)
        print2('doubled')
        return x
    def _point(x):
        val = int(round(rand.gauss(0,1)))
        index = randint(0, len(x) - 1)
        x[index] += val
        print2('changed', index, 'by', val)
        return x
    def _swap(x):
        i1 = randint(0, len(x) - 1)
        i2 = randint(0, len(x) - 1)
        x[i1], x[i2] = x[i2], x[i1]
        print2('swapped bases {} and {}'.format(i1,i2))
        return x
    if not gene:
        print3('Mutated an empty gene!')
        return gene
    return rand.choice([_invert,
                        _delete,
                        _insert,
                        _duplicate,
                        _point,
                        _swap,
                        ])(list(gene))
