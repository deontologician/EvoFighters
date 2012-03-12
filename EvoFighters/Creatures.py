'''Contains Creature class and all genetic related functionality'''

from random import randint
from itertools import cycle, takewhile

import Parsing as P
import struct
import random as rand
import cPickle as pickle

from Eval import PerformableAction, evaluate
from Utils import print1, print2, print3

# need to move this into a config file
mutation_rate = 0.05 # higher = more mutations

class Creature(object):
    def __init__(self, dna = None):
        if dna is None:
            self.dna = [randint(-1,9) for _ in xrange(0,50)]
        else:
            self.dna = dna
        self.inv = []
        self.energy = 40
        self.target = None
        self.age = 0
        self.signal = -1
        self.survived = 0
        self.won = 0
        self.instr_used = 0
        self.instr_skipped = 0
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
Survived: {0.survived}
Won: {0.won}
Instructions used/skipped: {0.instr_used}/{0.instr_skipped}
[]{equals}[]'''.format(self, inv = ','.join([str(i) for i in self.inv]),
                       equals = '='*76)
    
    @property
    def copy(self):
        return pickle.loads(pickle.dumps(self,2))

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

    def decision_generator(self):
        '''Reads dna to decide next course of action. Outputs verbiage'''
        dna_cycle = cycle(self.dna)
        while self.alive:
            try: 
                thought_process, (icount, skipped) = P.parse_condition(dna_cycle)
                print3("{0.name}'s thought process:".format(self))
                print3(P.explain_plan(thought_process))
                print3('which required', icount, 'instructions.',
                       'and', skipped, 'instructions skipped over')
                self.instr_used += icount
                self.instr_skipped += skipped
            except P.TooMuchThinkingError as tmt:
                print1(self.name,'got caught thinking too much!')
                self.instr_used = tmt.icount
                self.instr_skipped += tmt.skipped
                yield PerformableAction('wait', None), tmt.icount + tmt.skipped
                continue
            decision = evaluate(self, thought_process)
            print2(self.name, 'decided to', decision)
            yield decision, icount + skipped
        raise CreatureDied()

    def carryout(self, act):
        '''Carries out an action, possibly on the current target'''
        # take an item from the other's inventory
        if act.typ == P.Action.str.take:
            if self.target.inv:
                item = self.target.inv.pop()
                print1("{0.name} takes {1} from {2.name}"\
                           .format(self, P.item_repr(item), self.target))
                self.inv.append(item)
            else:
                print2("{0.name} tries to take an item from {1.name}, "\
                           "but there's nothing to take.".format(self,
                                                                 self.target))
        #using an item
        elif act.typ == P.Action.str.use:
            if self.inv:
                print1(self.name, 'uses', P.item_repr(self.inv[-1]))
                self.use()
            else:
                print2(self.name, "tries to use an item, but doesn't have one")
        #signalling
        elif act.typ == P.Action.str.signal:
            print1(self.name, 'signals with color', P.sig_repr(act.arg))
            self.signal = act.arg
        # waiting 
        elif act.typ == P.Action.str.wait:
            print2(self.name, 'waits')
        # defending with no corresponding attack
        elif act.typ == P.Action.str.defend:
            print2(self.name, 'defends, but no one is attacking')
        else:
            print1(self.name, 'did', act.typ, 'with magnitude:', act.arg)
            assert False
        self.last_action = act

    def use(self):
        'Uses the top inventory item'
        if self.inv:
            item = self.inv.pop()
            if 0 <= item <= len(P.Item):
                mult = item + 1
            else:
                mult = 0
            energy_gain = 3 * mult
            print2(self.name, 'gains', energy_gain, 'life from', P.item_repr(item))
            self.energy += energy_gain

class CreatureDied(StopIteration):
    '''A semantic way to stop iterating because your creature is no longer
    alive'''
    pass

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
