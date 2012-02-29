"""The Arena and how the fighters are to mess with each other"""

import random as rand
from random import randint
from itertools import count, cycle, izip, takewhile
from collections import namedtuple
from math import ceil, floor
from base64 import b64encode
from hashlib import md5

import Eval as E
from Parsing import parse_condition, TooMuchThinkingError
import Parsing as P

mutation_rate = 0.05 # higher = more mutations
thinking_penalty = 20.0 # higher = more thinking allowed

PerformableAction = namedtuple('PerformableAction', 'typ arg')

def fight(p1, p2):
    p1.target = p2
    p2.target = p1
    while randint(0, 200) != 200:
        p1act = p1.next_action
        p2act = p2.next_action
        carryout(p1, p1act, p2, p2act)
        if p2.dead and p1.alive:
            p1.inv.extend(p2.inv)
            p2.energy += randint(1,6)
            p1.target = None
            p1.fights_survived += 1
            return
        elif p1.dead and p2.alive:
            p2.inv.extend(p1.inv)
            p2.energy += randint(1,6)
            p2.target = None
            p2.fights_survived += 1
            return
    p1.fights_survived += 1
    p2.fights_survived += 1
    p1.target = None
    p2.target = None

def carryout(p1, p1_act, p2, p2_act):
    '''p1 and p2 simulataneously inflict their predetermined actions on one
    another'''
    #fight
    if p1_act.typ == 'attack' or p2_act.typ== 'attack':
        attacking(p1, p1_act, p2, p2_act)
    #defending does nothing if no one is attacking (other than waste time)
    # take an item from the other's inventory
    if p1_act.typ == 'take':
        try:
            p1.inv.append(p2.inv.pop())
        except:
            pass
    if p2_act.typ == 'take':
        try:
            p1.inv.append(p1.inv.pop())
        except:
            pass
    #using an item
    if p1_act.typ == 'use':
        p1.use()
    if p2_act.typ == 'use':
        p2.use()
    #signalling
    if p1_act.typ == 'signal':
        p1.signal = p1_act.arg
    if p2_act.typ == 'signal':
        p2.signal = p2_act.arg
    # waiting does nothing, so we don't need to consider it
    

def attacking(p1, p1_act, p2, p2_act):
    '''Handles attacking and defending. Call this only if either p1 or p2 is
    attacking'''
    p1_att = p1_act.typ == 'attack'
    p2_att = p2_act.typ == 'attack'
    p1_def = p2_act.typ == 'defend'
    p2_def = p2_act.typ == 'defend'
    if p1_att:
        if p2_att:
            p1.energy -= randint(3,6)
            p2.energy -= randint(3,6)
        elif p2_def:
            p2.energy -= randint(2,5) * damage_mult[p1_act.arg][p2_act.arg]
        else:
            p2.energy -= randint(1,4)
    elif p2_att:
        if p1_def:
            p1.energy -= randint(2,5) * damage_mult[p2_act.arg][p1_act.arg]
        else: #p1 cannot be attacking, we already dealt with that
            p1.energy -= randint(1,4)
    else:
        # this function should only be called when either p1 or p2 are attacking
        assert False

class Creature(object):
    def __init__(self, dna = None):
        if dna is None:
            self.dna = [randint(-1,9) for _ in xrange(0,25)]
        else:
            self.dna = dna
        self.dna_cycler = cycle(self.dna)
        self.inv = []
        self.energy = 40
        self.target = None
        self.age = 0
        self.signal = -1
        self.fights_survived = 0
        self.instructions_read = 0
        self.last_action = PerformableAction('wait', None)

    def __repr__(self):
        return "<]Creature {0.name}[>".format(self)

    @property
    def fullname(self):
        return ''.join(map(lambda x: str(x) if x != -1 else '|', self.dna))

    @property
    def name(self):
        'A simple short name that creates a summary value for each gene with xor'
        _xor = lambda y: reduce(lambda x,acc: x ^ acc, y, 0)
        return '|'.join([b64encode(str(_xor(gene))) for gene in \
                             takewhile(lambda x:x, gene_primer(self.dna))])

    @property
    def dead(self):
        return self.energy <= 0

    @property
    def alive(self):
        return self.energy > 0

    @property
    def next_action(self):
        '''Reads dna to decide next course of action'''
        try:
            thought_process, instructions = parse_condition(self.dna_cycler)
        except TooMuchThinkingError:
            self.energy -= 5
            return PerformableAction('wait', None)
        self.energy -= int(floor(instructions / thinking_penalty))
        return E.evaluate(self, thought_process)
    
    def use(self):
        'Does something with inventory items'
        if self.inv:
            item = self.inv.pop()
            if 0 <= item <= len(P.Item):
                mult = item + 1
            else:
                mult = 0
            self.energy += self.dna_cycler.next() * mult

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
            print 'mutating gene', index
            dna3[index] = mutate(dna3[index])
    return Creature([base for gene in dna3 for base in gene])

def transpose(genome):
    length = len(genome)
    i1 = randint(0, length - 1)
    i2 = randint(0, length - 1)
    print 'swapped gene',i1,'and',i2
    genome[i1], genome[i2] = genome[i2], genome[i1]

def mutate(gene):
    '''Does a mutation on a gene in various different ways'''
    def _invert(x):
        x.reverse()
        print 'reversed gene'
        return x
    def _delete(x):
        print 'deleted gene'
        return []
    def _insert(x):
        val = randint(-1,9)
        index = randint(0, len(x) - 1)
        x.insert(index, val)
        print 'inserted',val,'at',index
        return x
    def _duplicate(x):
        x.extend(x)
        print 'doubled'
        return x
    def _point(x):
        val = int(round(rand.gauss(0,1)))
        index = randint(0, len(x) - 1)
        x[index] += val
        print 'changed',index,'by',val
        return x
    def _swap(x):
        i1 = randint(0, len(x) - 1)
        i2 = randint(0, len(x) - 1)
        x[i1], x[i2] = x[i2], x[i1]
        print 'swapped bases', i1, 'and', i2
        return x
    if not gene:
        print 'mutated an empty gene!'
        return gene
    return rand.choice([_invert,
                        _delete,
                        _insert,
                        _duplicate,
                        _point,
                        _swap,
                        ])(list(gene))
        
        

def clear_dead(creatures):
    '''Return a new list with all dead creatures removed'''
    return [creature for creature in creatures if not creature.dead]

def famine(creatures):
    '''Damages everyone by a random amount'''
    avg_energy = int(ceil(sum(map(lambda c: c.energy, creatures)) / \
                            float(len(creatures))))
    def _subenergy(creature, value):
        creature.energy -= value
        return creature
    return clear_dead([_subenergy(x, randint(0, avg_energy)) for x in creatures])

def mating_season(creatures):
    '''Mates random creatures together and creates offspring'''
    max_creature = len(creatures) - 1
    for _ in xrange(0, int(0.25 * max_creature)):
        mate1 = creatures[randint(0, max_creature)]
        mate2 = creatures[randint(0, max_creature)]
        offspring = mate(mate1, mate2)
        print '{off} is the offspring of {m1} and {m2}'.format(off = offspring.name,
                                                               m1 = mate1.name,
                                                               m2 = mate2.name)
        creatures.append(offspring)
    # offspring = [mate(creatures[randint(0, max_creature)],
    #                   creatures[randint(0, max_creature)]) \
    #                  for _ in xrange(0, int(0.25 * max_creature))]
    #creatures.extend(offspring)
    return creatures


def feeding_time(creatures):
    '''Gives random amounts of food to all of the creatures'''
    for x in xrange(0, int(len(creatures) * 1.5)):
        creatures[randint(0,len(creatures))].inv.append(item['food'])






damage_mult = [[ 0,  1, -1],
               [-1,  0,  1],
               [ 1, -1,  0]]

if __name__ == '__main__':
    print "OK let's do this!"
