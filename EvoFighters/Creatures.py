'''Contains Creature class and all genetic related functionality'''

from random import randint

import random as rand
import cPickle as pickle
from array import array

import Parsing as P
from EvoFighters.Parsing import ACT, ITEM, SIG, COND
from EvoFighters.Eval import PerformableAction, evaluate
from EvoFighters.Utils import inv_repr, dna_repr


sd = None  # Set by Arena once the savedata is created

class Creature(object):
    '''Represents a creature'''
    # There will be a lot of these creatures, so we'll use slots for memory
    # efficiency
    __slots__ = ('dna', '_inv', 'energy', 'target', 'generation', 'num_children',
                 'signal', 'survived', 'kills', 'instr_used', 'instr_skipped', 
                 'last_action', 'name', 'is_feeder', 'eaten', 'parents')

    wait_action = PerformableAction(ACT['wait'], None)
    count = 0
    
    def __init__(self, dna = None, parents = None):
        if dna is None:
            self.dna = array('b', [COND['always'], ACT['mate'],
                                   COND['always'], ACT['flee']])
        else:
            self.dna = dna
        self._inv = []
        self.energy = 40
        self.target = None
        self.generation = 0
        self.num_children = 0
        self.signal = -1
        self.survived = 0
        self.kills = 0
        self.instr_used = 0
        self.instr_skipped = 0
        self.last_action = Creature.wait_action
        self.is_feeder = False
        self.eaten = 0
        self.parents = parents
        self.name = Creature.count
        Creature.count += 1

    def __str__(self):
        return "<]Creature {0.name}[>".format(self)
    
    def __repr__(self):
        return '''\
[]{0.name:=^76}[]
DNA: {0.fullname}
Inventory: {inv}
Energy: {0.energy}
Generation: {0.generation}
Children: {0.num_children}
Survived: {0.survived}
Kills: {0.kills}
Eaten: {0.eaten}
Parents: {0.parents}
Instructions used/skipped: {0.instr_used}/{0.instr_skipped}
[]{bar}[]'''.format(self, 
                    inv = inv_repr(self._inv),
                    bar = ''.center(76, '='))
    
    @property
    def copy(self):
        '''Performs a value copy of this creature'''
        return pickle.loads(pickle.dumps(self, 2))

    @property
    def fullname(self):
        '''A compact view of a creature's dna.'''
        return dna_repr(self.dna)

    def add_item(self, item):
        if item is not None and len(self._inv) + 1 <= sd.settings.max_inv_size:
            self._inv.append(item)

    def pop_item(self):
        ''
        if self._inv:
            return self._inv.pop()
        else:
            return None

    @property
    def has_items(self):
        'Whether creature has any items'
        return bool(self._inv)
    
    @property
    def top_item(self):
        'What the top item is. Will throw an exception if no items'
        return self._inv[-1]

    @property
    def dead(self):
        '''Whether the creature is dead'''
        return self.energy <= 0 or not self.dna

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
                sd.print3("{0.name}'s thought process: \n{thought}", self,
                       thought = thought.tree)
                sd.print3('which required {0.icount} instructions and {0.skipped} '
                       'instructions skipped over', thought)
                self.instr_used += thought.icount
                self.instr_skipped += thought.skipped
            except P.TooMuchThinkingError as tmt:
                sd.print1('{.name} was paralyzed by analysis and died', self)
                self.energy = 0
                yield Creature.wait_action, tmt.icount + tmt.skipped
                continue
            decision = evaluate(self, thought.tree)
            sd.print2('{.name} decided to {}', self, decision)
            yield decision, thought.icount + thought.skipped
        raise StopIteration()

    def carryout(self, act):
        '''Carries out any actions that unlike mating and fighting, don't depend
        on what the target's current action is. Nothing will be done if the
        creature is dead. Return value is whether the fight should end.'''
        if self.dead:
            return
        #signalling
        elif act.typ == ACT['signal']:
            sd.print1('{.name} signals with color {sig_repr}', self, sig_repr = act.arg)
            self.signal = act.arg
        #using an item
        elif act.typ == ACT['use']:
            if self.has_items:
                sd.print1('{.name} uses {item_repr}', self, item_repr = self.top_item)
                self.use()
            else:
                sd.print2("{.name} tries to use an item, but doesn't have one", self)

        # take an item from the other's inventory
        elif act.typ == ACT['take']:
            if self.target.has_items:
                item = self.target.pop_item()
                sd.print1("{0.name} takes {item_repr} from {1.name}", self, self.target,
                       item_repr = item)
                self.add_item(item)
            else:
                sd.print2("{0.name} tries to take an item from {1.name}, "\
                           "but there's nothing to take.", self, self.target)
        # waiting
        elif act.typ == ACT['wait']:
            sd.print2('{.name} waits', self)
        # defending with no corresponding attack
        elif act.typ == ACT['defend']:
            sd.print2('{.name} defends', self)
        elif act.typ == ACT['flee']:
            enemy_roll = randint(0, 100) * (self.target.energy / 40.0)
            my_roll = randint(0, 100) * (self.energy / 40.0)
            dmg = randint(0,3)
            if enemy_roll < my_roll:
                sd.print1('{.name} flees the encounter and takes {} damage', self, dmg)
                self.energy -= dmg
                raise StopIteration()
            else:
                sd.print1('{.name} tries to flee, but {.name} prevents it', self, self.target)
        else:
            raise RuntimeError("{0.name} did {1.typ} with magnitude {1.arg}"\
                                   .format(self, act))

    def use(self):
        'Uses the top inventory item'
        item = self.pop_item()
        if item:
            if 0 <= item <= len(ITEM):
                mult = item + 1
            else:
                mult = 0
            energy_gain = 3 * mult
            sd.print2('{.name} gains {} life from {item_repr}', self, energy_gain,
                   item_repr = item)
            self.energy += energy_gain


class Feeder(Creature):
    '''A pitiful subclass of creature, used only for eating by creatures.'''

    _instance = None #singleton instance

    def __new__(cls, *args, **kwargs):
        'Feeder is a singleton'
        if not cls._instance:
            cls._instance = super(Feeder, cls).__new__(cls, *args, **kwargs)
            cls._instance.dna = None
            cls._instance.target = None
            cls._instance.generation = 0
            cls._instance.num_children = 0
            cls._instance.survived = 0
            cls._instance.kills = 0
            cls._instance.instr_used = 0
            cls._instance.instr_skipped = 0
            cls._instance.last_action = Creature.wait_action
            cls._instance.is_feeder = True
            cls._instance.signal = SIG['green']
            cls._instance.name = 'Feeder'
            cls._instance.parents = None
            cls._instance.eaten = 0
        cls._instance.energy = 1
        cls._instance._inv = Feeder._getinv()
        return cls._instance
            
 
    def __init__(self):
        pass

    @staticmethod
    def _getinv():
        choices = [i for i in xrange(len(ITEM)) for _ in xrange(len(ITEM) - i)]
        return [rand.choice(choices)]

    def __str__(self):
        return '[|Feeder|]'

    def decision_generator(self):
        '''Dummy decision generator'''
        while self.alive:
            # always 'wait', and always think about it for more than the max
            # number of steps
            self.instr_used += 0
            self.instr_skipped += sd.settings.max_thinking_steps + 1
            yield (PerformableAction(ACT['wait'], None),
                   sd.settings.max_thinking_steps + 1)
        yield StopIteration()

    @property
    def dead(self):
        '''Also dies if inventory is raided'''
        return self.energy <= 0 or not self.has_items

    def carryout(self, act):
        '''Never do anything'''
        sd.print2('Feeder does nothing')
        pass

def gene_primer(dna):
    '''Breaks a dna list into chunks by the terminator -1.'''
    chunk = []
    for base in dna:
        chunk.append(base)
        if base == -1:
            yield chunk
            chunk = []
    if chunk:
        yield chunk


def try_to_mate(sd, mating_chance, first_mate, fm_share, second_mate, sm_share):
    '''Takes a chance of mating, two creatures to mate, and the relative
    proportion of costs each creature must pay, mates two creatures to create a
    third.'''
    if randint(1,100) > mating_chance or first_mate.dead or second_mate.dead:
        return None
    if first_mate.is_feeder or second_mate.is_feeder:
        sd.print1('{.name} tried to mate with {.name}!', first_mate, second_mate)
        if first_mate.is_feeder:
            first_mate.energy = 0
        if second_mate.is_feeder:
            second_mate.energy = 0
        return None
    sd.print2('Attempting to mate')
    
    def pay_cost(p, share):
        cost = int(round(sd.settings.mating_cost * (share / 100.0)))
        while cost > 0:
            if p.has_items:
                item = p.pop_item()
                cost -= (item + 1) * 2
            else:
                sd.print1('{.name} ran out of items and failed to mate', p)
                return False
        return True
        
    if pay_cost(first_mate, fm_share) and pay_cost(second_mate, sm_share):
        return mate(first_mate, second_mate)
    else:
        return None


def mate(p1, p2):
    '''Takes in two creatures, splices their dna together randomly by chunks,
    possibly mutates it, then spits out a new creature. Mutation rate is the
    chance that a mutation will occur'''
    # chunkify the dna
    dna1_primer = gene_primer(p1.dna)
    dna2_primer = gene_primer(p2.dna)
    child_genes = []
    while True:
        gene1 = next(dna1_primer, [])
        gene2 = next(dna2_primer, [])
        if gene1 == [] and gene2 == []:
            break
        gene3 = rand.choice([gene1, gene2])
        child_genes.append(gene3)
    if rand.uniform(0, 1) < sd.settings.mutation_rate:
        mutate(child_genes)
    child = Creature(array('b', [base for gene in child_genes for base in gene]), 
                     parents = (p1.name, p2.name))
    child.generation = min(p1.generation, p2.generation) + 1
    p1.num_children += 1
    p2.num_children += 1
    return child


def mutate(dna):
    '''Mutates the dna on either the genome or gene level'''
    if randint(0, int(10000/sd.settings.mutation_rate)) == 0:
        genome_level_mutation(dna)
    else:
        index = randint(0, len(dna) - 1)
        sd.print2('mutating gene {}', index)
        dna[index] = gene_level_mutation(dna[index])

def genome_level_mutation(dna):
    '''Mutate the dna on a meta-gene level'''
    def _swap(genome):
        'Swap two genes'
        length = len(genome)
        i1 = randint(0, length - 1)
        i2 = randint(0, length - 1)
        sd.print2('swapped gene {} and {}', i1, i2)
        genome[i1], genome[i2] = genome[i2], genome[i1]
    def _double(genome):
        'Doubles a gene'
        i = randint(0, len(genome) - 1)
        gene = genome[i]
        genome.insert(i, gene)
    def _delete(genome):
        'Delete a gene'
        index = randint(0, len(genome) - 1)
        sd.print2('Deleted gene {}', index)
        del dna[index]

    rand.choice([_swap, _delete, _double])(dna)

def gene_level_mutation(gene):
    '''Does a mutation on a gene in various different ways'''
    def _invert(x):
        'Reverse the order of the bases in a gene'
        x.reverse()
        sd.print2('reversed gene')
        return x
    def _delete(_):
        'Delete a gene'
        sd.print2('deleted gene')
        return []
    def _insert(x):
        'Insert an extra base in the gene'
        val = randint(-1, 9)
        index = randint(0, len(x) - 1)
        x.insert(index, val)
        sd.print2('inserted {} at {}', val, index)
        return x
    def _point(x):
        "Increment or decrement a base's value"
        val = int(round(rand.gauss(0, 1)))
        index = randint(0, len(x) - 1)
        new_base = (x[index] + 1 + val) % 11 - 1
        sd.print2('changed {} from {} to {}', index, x[index], new_base)
        x[index] = new_base
        return x
    def _swap(x):
        'Swap two bases'
        i1 = randint(0, len(x) - 1)
        i2 = randint(0, len(x) - 1)
        x[i1], x[i2] = x[i2], x[i1]
        sd.print2('swapped bases {} and {}', i1, i2)
        return x
    if not gene:
        sd.print3('Mutated an empty gene!')
        return gene
    return rand.choice([_invert,
                        _delete,
                        _insert,
                        _point,
                        _swap,
                        ])(list(gene))
