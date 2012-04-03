'''Contains Creature class and all genetic related functionality'''

from random import randint

import Parsing as P
import random as rand
import cPickle as pickle

from Parsing import ACT, ITEM, SIG, COND, MAX_THINKING_STEPS
from Eval import PerformableAction, evaluate
from Utils import print1, print2, print3

# need to move this into a config file
mutation_rate = 0.10 # higher = more mutations
# cost in energy of mating. May be taken out of items in inventory
MATING_COST = 40

class Creature(object):
    '''Represents a creature'''
    # There will be a lot of these creatures, so we'll use slots for memory
    # efficiency
    __slots__ = ('dna', '_inv', 'energy', 'target', 'generation', 'num_children',
                 'signal', 'survived', 'kills', 'instr_used', 'instr_skipped', 
                 'last_action', 'name', 'is_feeder')

    wait_action = PerformableAction(ACT['wait'], None)
    count = 0
    
    def __init__(self, dna = None):
        if dna is None:
            self.dna = (COND['always'], ACT['mate'], COND['always'], ACT['flee'])
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
Instructions used/skipped: {0.instr_used}/{0.instr_skipped}
[]{bar}[]'''.format(self, 
                    inv = ','.join([str(i) for i in self._inv]),
                    bar = ''.center(76, '='))
    
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
            return xs
        return ''.join([stringify(x) for x in self.dna])

    def add_item(self, item):
        if item is not None and len(self._inv) + 1 <= 4:
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
                print3("{0.name}'s thought process: \n{thought}", self,
                       thought = thought.tree)
                print3('which required {0.icount} instructions and {0.skipped} '
                       'instructions skipped over', thought)
                self.instr_used += thought.icount
                self.instr_skipped += thought.skipped
            except P.TooMuchThinkingError as tmt:
                self.instr_used += tmt.icount
                self.instr_skipped += tmt.skipped
                penalty = randint(1,5)
                print1('{.name} got caught thinking too much and lost {} life',
                       self, penalty)
                self.energy -= penalty
                yield PerformableAction(ACT['wait'], None), \
                    tmt.icount + tmt.skipped
                continue
            decision = evaluate(self, thought.tree)
            print2('{.name} decided to {}', self, decision)
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
            print1('{.name} signals with color {sig_repr}', self, sig_repr = act.arg)
            self.signal = act.arg
        #using an item
        elif act.typ == ACT['use']:
            if self.has_items:
                print1('{.name} uses {item_repr}', self, item_repr = self.top_item)
                self.use()
            else:
                print2("{.name} tries to use an item, but doesn't have one", self)

        # take an item from the other's inventory
        elif act.typ == ACT['take']:
            if self.target.has_items:
                item = self.target.pop_item()
                print1("{0.name} takes {item_repr} from {1.name}", self, self.target,
                       item_repr = item)
                self.add_item(item)
            else:
                print2("{0.name} tries to take an item from {1.name}, "\
                           "but there's nothing to take.", self, self.target)
        # waiting
        elif act.typ == ACT['wait']:
            print2('{.name} waits', self)
        # defending with no corresponding attack
        elif act.typ == ACT['defend']:
            print2('{.name} defends', self)
        elif act.typ == ACT['flee']:
            enemy_roll = randint(0, 100) * (self.target.energy / 40.0)
            my_roll = randint(0, 100) * (self.energy / 40.0)
            if enemy_roll < my_roll:
                print1('{.name} flees the encounter!', self)
                raise StopIteration()
            else:
                print1('{.name} tries to flee, but {.name} prevents it', self, self.target)
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
            print2('{.name} gains {} life from {item_repr}', self, energy_gain,
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
            cls._instance.signal = -1
            cls._instance.survived = 0
            cls._instance.kills = 0
            cls._instance.instr_used = 0
            cls._instance.instr_skipped = 0
            cls._instance.last_action = Creature.wait_action
            cls._instance.is_feeder = True
            cls._instance.signal = SIG['green']
            cls._instance.name = 'Feeder'
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
            self.instr_skipped += MAX_THINKING_STEPS + 1
            yield PerformableAction(ACT['wait'], None), MAX_THINKING_STEPS + 1
        yield StopIteration()

    @property
    def dead(self):
        '''Also dies if inventory is raided'''
        return self.energy <= 0 or not self.has_items

    def carryout(self, act):
        '''Never do anything'''
        print2('Feeder does nothing')
        pass

def gene_primer(dna):
    '''Breaks a dna list into chunks by the terminator -1.'''
    chunk = []
    for i in dna:
        chunk.append(i)
        if i < 0:
            yield chunk
            chunk = []
    if chunk:
        yield chunk


def try_to_mate(mating_chance, first_mate, fm_share, second_mate, sm_share):
    '''Takes a chance of mating, two creatures to mate, and the relative
    proportion of costs each creature must pay, mates two creatures to create a
    third.'''
    if randint(1,100) > mating_chance or first_mate.dead or second_mate.dead:
        return None
    if first_mate.is_feeder or second_mate.is_feeder:
        print1('{.name} tried to mate with {.name}!', first_mate, second_mate)
        if first_mate.is_feeder:
            first_mate.energy = 0
        if second_mate.is_feeder:
            second_mate.energy = 0
        return None
    print2('Attempting to mate')
    
    def pay_cost(p, share):
        cost = int(round(MATING_COST * (share / 100.0)))
        while cost > 0:
            if p.has_items:
                item = p.pop_item()
                cost -= item + 1
            else:
                p.energy -= cost
                break

    pay_cost(first_mate, fm_share)
    if first_mate.dead:
        print1('{.name} died in the process of mating', first_mate)
        return None
    pay_cost(second_mate, sm_share)
    if second_mate.dead:
        print1('{.name} died in the process of mating', second_mate)
        return None
    return mate(first_mate, second_mate)


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
    if rand.uniform(0, 1) < mutation_rate:
        mutate(child_genes)
    child = Creature(tuple([base for gene in child_genes for base in gene]))
    child.generation = max(p1.generation, p2.generation) + 1
    p1.num_children += 1
    p2.num_children += 1
    return child


def mutate(dna):
    '''Mutates the dna on either the genome or gene level'''
    if randint(0, 2) == 0:
        genome_level_mutation(dna)
    else: # both branches can happen
        index = randint(0, len(dna) - 1)
        print2('mutating gene {}', index)
        dna[index] = gene_level_mutation(dna[index])

def genome_level_mutation(dna):
    '''Mutate the dna on a meta-gene level'''
    def _swap(genome):
        'Swap two genes'
        length = len(genome)
        i1 = randint(0, length - 1)
        i2 = randint(0, length - 1)
        print2('swapped gene {} and {}', i1, i2)
        genome[i1], genome[i2] = genome[i2], genome[i1]
    def _delete(genome):
        'Delete a gene'
        index = randint(0, len(genome) - 1)
        print2('Deleted gene {}', index)
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
        print2('inserted {} at {}', val, index)
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
        print2('changed {} by {}', index, val)
        return x
    def _swap(x):
        'Swap two bases'
        i1 = randint(0, len(x) - 1)
        i2 = randint(0, len(x) - 1)
        x[i1], x[i2] = x[i2], x[i1]
        print2('swapped bases {} and {}', i1, i2)
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
