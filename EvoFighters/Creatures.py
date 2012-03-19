'''Contains Creature class and all genetic related functionality'''

from random import randint
from itertools import takewhile

import Parsing as P
import struct
import random as rand
import cPickle as pickle

from Parsing import ACT, ITEM
from Eval import PerformableAction, evaluate
from Utils import print1, print2, print3

# need to move this into a config file
mutation_rate = 0.05 # higher = more mutations

class Creature(object):
    '''Represents a creature'''
    # There will be a lot of these creatures, so we'll use slots for memory
    # efficiency
    __slots__ = ('dna', 'inv', 'energy', 'target', 'generation', 'num_children',
                 'signal', 'survived', 'kills', 'instr_used', 'instr_skipped', 
                 'last_action', 'name')
    
    def __init__(self, dna = None):
        if dna is None:
            self.dna = [randint(-1, 9) for _ in xrange(0, 50)]
        else:
            self.dna = dna
        self.inv = []
        self.energy = 40
        self.target = None
        self.generation = 0
        self.num_children = 0
        self.signal = -1
        self.survived = 0
        self.kills = 0
        self.instr_used = 0
        self.instr_skipped = 0
        self.last_action = PerformableAction('wait', None)
        self.name = self._get_name()

    def __str__(self):
        return "<]Creature {0.name}[>".format(self)
    
    def __repr__(self):
        return \
'''[]{0.name:=^76}[]
DNA: {0.fullname}
Inventory: {inv}
Energy: {0.energy}
Generation: {0.generation}
Children: {0.num_children}
Survived: {0.survived}
Kills: {0.kills}
Instructions used/skipped: {0.instr_used}/{0.instr_skipped}
[]{equals}[]'''.format(self, inv = ','.join([str(i) for i in self.inv]),
                       equals = '='*76)
    
    @property
    def copy(self):
        '''Performs a value copy of this creature'''
        return pickle.loads(pickle.dumps(self, 2))

    @property
    def fullname(self):
        '''A compact view of a creature's dna. Not necessarily a unique
        representation of a dna (can't distinguish adjacent 1 digit numbers from
        a single 2 digit number) but should be sufficient disambiguation for
        most purposes'''
        def stringify(x):
            '''helper for fullname'''
            xs = str(x) if x != -1 else '|'
            if len(xs) > 1:
                xs = '({xs})'.format(xs = xs)
        return ''.join([stringify(x) for x in self.dna])

    def _get_name(self):
        'A simple short name that is probably unique'
        def sum_and_encode(gene):
            'Helper for _get_name'
            return struct.pack('b', sum(gene) % 256 - 128)\
                  .encode('base_64').rstrip('=\n')
        return ''.join([ sum_and_encode(gene) for gene in \
                            takewhile(lambda x:x, gene_primer(self.dna))])

    @property
    def dead(self):
        '''Whether the creature is dead'''
        return self.energy <= 0

    @property
    def alive(self):
        '''Whether the creature is alive, defined as "not self.dead"'''
        return not self.dead

    def decision_generator(self):
        '''Reads dna to decide next course of action. Outputs verbiage'''
        parser = P.Parser(self.dna)
        while self.alive:
            try: 
                thought = next(parser)
                print3("{0.name}'s thought process:".format(self))
                print3(P.show_thought(thought.tree))
                print3('which required', thought.icount, 'instructions.',
                       'and', thought.skipped, 'instructions skipped over')
                self.instr_used += thought.icount
                self.instr_skipped += thought.skipped
            except P.TooMuchThinkingError as tmt:
                print1(self.name, 'got caught thinking too much!')
                self.instr_used += tmt.icount
                self.instr_skipped += tmt.skipped
                yield PerformableAction(ACT.wait, None), tmt.icount + tmt.skipped
                continue
            decision = evaluate(self, thought.tree)
            print2(self.name, 'decided to', decision)
            yield decision, thought.icount + thought.skipped
        raise CreatureDied()

    def carryout(self, act):
        '''Carries out any actions that unlike mating and fighting, don't depend
        on what the target's current action is. Nothing will be done if the
        creature is dead.'''
        if self.dead:
            return
        #signalling
        if act.typ == ACT.signal:
            print1(self.name, 'signals with color', P.sig_repr(act.arg))
            self.signal = act.arg
        #using an item
        elif act.typ == ACT.use:
            if self.inv:
                print1(self.name, 'uses', P.item_repr(self.inv[-1]))
                self.use()
            else:
                print2(self.name, "tries to use an item, but doesn't have one")

        # take an item from the other's inventory
        elif act.typ == ACT.take:
            if self.target.inv:
                item = self.target.inv.pop()
                print1("{0.name} takes {1} from {2.name}"\
                           .format(self, P.item_repr(item), self.target))
                self.inv.append(item)
            else:
                print2("{0.name} tries to take an item from {1.name}, "\
                           "but there's nothing to take.".format(self,
                                                                 self.target))

        # waiting 
        elif act.typ == ACT.wait:
            print2(self.name, 'waits')
        # defending with no corresponding attack
        elif act.typ == ACT.defend:
            print2(self.name, 'defends, but no one is attacking')
        elif act.typ == ACT.flee:
            enemy_roll = randint(0, 100) * (self.target.energy / 40.0)
            my_roll = randint(0, 100) * (self.energy / 40.0)
            if enemy_roll < my_roll:
                print1(self.name, 'flees the encounter!')
                raise CreatureFlees()
            else:
                print1(self.name, 'tries to flee, but', self.target.name, 
                       'prevents it')
        elif act.typ == ACT.mate:
            print1(self.name, 'attempts to mate with', self.target.name)
        else:
            raise RuntimeError("{0.name} did {1.typ} with magnitude {1.arg}"\
                                   .format(self, act))
        self.last_action = act

    def use(self):
        'Uses the top inventory item'
        if self.inv:
            item = self.inv.pop()
            if 0 <= item <= len(ITEM):
                mult = item + 1
            else:
                mult = 0
            energy_gain = 3 * mult
            print2(self.name, '{.name} gains {} life from {}'\
                       .format(self, energy_gain, P.item_repr(item)))
            self.energy += energy_gain

class CreatureDied(StopIteration):
    '''A semantic way to stop iterating because your creature is no longer
    alive'''
    pass

class CreatureFlees(StopIteration):
    '''A semantic way to stop iterating because a creature has successfully fled
    the encounter'''
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
    baby_genes = []
    while True:
        gene1 = next(dna1_primer)
        gene2 = next(dna2_primer)
        if gene1 == [] and gene2 == []:
            break
        gene3 = rand.choice([gene1, gene2])
        baby_genes.append(gene3)
    if rand.uniform(0, 1) < mutation_rate:
        mutate(baby_genes)
    baby = Creature([base for gene in baby_genes for base in gene])
    baby.generation = max(p1.generation, p2.generation) + 1
    p1.num_children += 1
    p2.num_children += 1
    #pay for mating
    if p1.inv:
        p1.inv.pop()
    else:
        p1.energy -= 2
    if p2.inv:
        p2.inv.pop()
    else:
        p2.energy -= 2
    return baby
def mutate(dna):
    '''Mutates the dna on either the genome or gene level'''
    if randint(0, 2) == 0:
        genome_level_mutation(dna)
    else: # both branches can happen
        index = randint(0, len(dna) - 1)
        print2('mutating gene', index)
        dna[index] = gene_level_mutation(dna[index])

def genome_level_mutation(dna):
    '''Mutate the dna on a meta-gene level'''
    def _swap(genome):
        'Swap two genes'
        length = len(genome)
        i1 = randint(0, length - 1)
        i2 = randint(0, length - 1)
        print2('swapped gene', i1, 'and', i2)
        genome[i1], genome[i2] = genome[i2], genome[i1]
    def _delete(genome):
        'Delete a gene'
        index = randint(0, len(genome) - 1)
        print2('Deleted gene {}'.format(index))
        del dna[index]

    rand.choice([_swap, _delete])(dna)

def gene_level_mutation(gene):
    '''Does a mutation on a gene in various different ways'''
    def _invert(x):
        'Reverse the order of the bases in a gene'
        x.reverse()
        print2('reversed gene')
        return x
    def _delete(_):
        'Delete a gene'
        print2('deleted gene')
        return []
    def _insert(x):
        'Insert an extra base in the gene'
        val = randint(-1, 9)
        index = randint(0, len(x) - 1)
        x.insert(index, val)
        print2('inserted {} at {}'.format(val, index))
        return x
    def _duplicate(x):
        'Double a gene'
        x.extend(x)
        print2('doubled')
        return x
    def _point(x):
        "Increment or decrement a base's value"
        val = int(round(rand.gauss(0, 1)))
        index = randint(0, len(x) - 1)
        x[index] += val
        print2('changed', index, 'by', val)
        return x
    def _swap(x):
        'Swap two bases'
        i1 = randint(0, len(x) - 1)
        i2 = randint(0, len(x) - 1)
        x[i1], x[i2] = x[i2], x[i1]
        print2('swapped bases {} and {}'.format(i1, i2))
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
