"""The Arena and how the fighters are to mess with each other"""
from __future__ import print_function

import random as rand
from random import randint
from itertools import izip
from contextlib import contextmanager
from collections import namedtuple, defaultdict
import cPickle as pickle
import sys, os.path

from Parsing import ACT
from Utils import print1, print2, print3, progress_bar, get_verbosity, \
    set_verbosity
from Creatures import Creature, try_to_mate

OPTIMAL_GEN_SIZE = 500
    
SAVE_FILENAME = 'evofighters.save'


def encounter(p1, p2):
    '''Carries out an encounter between two creatures'''
    # these numbers were very carefully tuned to pretty much never go less than
    # 10 rounds
    max_rounds = abs(int(rand.gauss(200, 30)))
    children = []
    print1('Max rounds: {}'.format(max_rounds))
    for rounds, (p1act, c1), (p2act, c2) in izip(xrange(max_rounds),
                                                 p1.decision_generator(),
                                                 p2.decision_generator()):
        print2('Round {}'.format(rounds))
        if c1 > c2:
            print3('{0.name} is going first'.format(p2))
            child = do_round(p1, p1act, p2, p2act)
        else:
            print3('{0.name} is going first'.format(p1))
            child = do_round(p2, p2act, p1, p1act)
        if child is not None:
            children.append(child)
        if p1.dead or p2.dead:
            break

    def _victory(winner, loser):
        print1(winner.name, 'has killed', loser.name)
        winner.inv.extend(loser.inv)
        winner.energy = min(40, winner.energy + randint(1, 6))
        winner.survived += 1
        winner.kills += 1
    if p2.dead and p1.alive:
        _victory(p1, p2)
    elif p1.dead and p2.alive:
        _victory(p2, p1)
    elif p1.dead and p2.dead:
        print1('Both {0.name} and {1.name} have died.'.format(p1, p2))
    else:
        p1.survived += 1
        p2.survived += 1
    return children

def do_round(p1, p1_act, p2, p2_act):
    '''Handles carrying out the decided actions for a single round'''
    # convenient short-hands to make code more readable
    ATTACKING = 1
    DEFENDING = 2
    MATING    = 3
    OTHER     = 4
    # defaults to OTHER if the key is not present
    act_kind = defaultdict(lambda: OTHER,
                           {ACT.attack : ATTACKING,
                            ACT.defend : DEFENDING,
                            ACT.mate   : MATING})
    # c2h = chance to hit
    M = namedtuple('Multipliers', 'mate_chance p1_c2h p2_c2h '\
                                  'dmg1_mult dmg2_mult p1_share p2_share')
        
    damage_matrix = {
        (ATTACKING, ATTACKING)  :   M(  0, 75, 75, 50, 50,  0,  0),
        (ATTACKING, DEFENDING)  :   M(  0, 25, 25, 25, 25,  0,  0),
        (ATTACKING, MATING)     :   M( 50, 50,  0, 75,  0, 70, 30),
        (ATTACKING, OTHER)      :   M(  0,100,  0,100,  0,  0,  0),
        (DEFENDING, DEFENDING)  :   M(  0,  0,  0,  0,  0,  0,  0),
        (DEFENDING, MATING)     :   M( 25,  0,  0,  0,  0, 70, 30),
        (DEFENDING, OTHER)      :   M(  0,  0,  0,  0,  0,  0,  0),
        (MATING,    MATING)     :   M(100,  0,  0,  0,  0, 50, 50),
        (MATING,    OTHER)      :   M( 75,  0,  0,  0,  0,  0,100),
        (OTHER,     OTHER)      :   M(  0,  0,  0,  0,  0,  0,  0),
        # the rest of these are duplicates of the above with swapped order
        (DEFENDING, ATTACKING)  :   M(  0, 25, 25, 25, 25,  0,  0),
        (MATING,    ATTACKING)  :   M( 50,  0, 50,  0, 75, 30, 70),
        (MATING,    DEFENDING)  :   M( 25,  0,  0,  0,  0, 30, 70),
        (OTHER,     ATTACKING)  :   M(  0,  0,100,  0,100,  0,  0),
        (OTHER,     DEFENDING)  :   M(  0,  0,  0,  0,  0,  0,  0),
        (OTHER,     MATING)     :   M( 75,  0,  0,  0,  0,100,  0),
        }
    mults = damage_matrix[(act_kind[p1_act.typ], act_kind[p2_act.typ])]
    def damage_fun(chance, mult):
        '''Takes a "chance to hit" and a "damage multiplier" and returns
        damage'''
        if randint(1,100) <= chance:
            return randint(1, int(round(((mult/100.0) * 6))))
        else:
            return 0
    p1_dmg = damage_fun(mults.p1_c2h, mults.dmg1_mult)
    p2_dmg = damage_fun(mults.p2_c2h, mults.dmg2_mult)
    # TODO: take into account damage type!
    if p1_dmg > 0:
        print1('p1 takes', p1_dmg, 'damage')
        p1.energy -= p1_dmg
    if p2_dmg > 0:
        print1('p2 takes', p2_dmg, 'damage.')
        p2.energy -= p2_dmg
    # we reverse the order of p1, p2 when calling try_to_mate because paying
    # costs first in mating is worse, and in this function p1 is preferred in
    # actions that happen to both creatures in order. Conceivably, p2 could die
    # without p1 paying any cost at all, even if p2 initiated mating against
    # p1's will
    child = try_to_mate(mults.mate_chance, p2, mults.p2_share, 
                                           p1, mults.p1_share)
    if child:
        print1(p1.name, 'and', p2.name, 'have a child named', child.name)
    if act_kind[p1_act.typ] == OTHER:
        p1.carryout(p1_act)
    if act_kind[p2_act.typ] == OTHER:
        p2.carryout(p2_act)
    return child


def maxencounters(creatures):
    '''Number of encounters required for a given population based on size'''
    return round((len(creatures) ** 3) / (OPTIMAL_GEN_SIZE * 1000.0))
       
@contextmanager
def random_encounter(creatures, copy = False):
    '''A context manager that handles selecting two random creatures from the
    creature list, setting them as targets of each other, and then yielding to
    the actual encounter code.'''
    if len(creatures) < 2:
        raise RuntimeError('Not enough creatures.')
    p1_index = randint(0, len(creatures) - 1)
    p2_index = randint(0, len(creatures) - 1)
    while p1_index == p2_index:
        p2_index = randint(0, len(creatures) - 1)
    p1 = creatures[p1_index]
    p2 = creatures[p2_index]
    if copy:
        p1 = p1.copy
        p2 = p2.copy
    p1.target = p2
    p2.target = p1
    try:
        yield p1, p2
    finally:
        p1.target = None
        p2.target = None
        if p1.dead and not copy:
            creatures.remove(p1)
        if p2.dead and not copy:
            creatures.remove(p2)

def simulate(sd):
    time_to_save = progress_bar()
    try:
        while True:
            if len(sd.creatures) < 2:
                raise RuntimeError('Not enough creatures')
            sd.num_encounters += 1
            if sd.num_encounters % 500 == 0:
                print('Currently', len(sd.creatures), 'creatures alive.')
                sd.save()
            time_to_save.send((sd.num_encounters % 500) / 500.0)
            with random_encounter(sd.creatures) as (p1, p2):
                print1(p1.name, 'encounters', p2.name, 'in the wild')
                sd.creatures.extend(encounter(p1, p2))
            
    except KeyboardInterrupt:
        print('\nOk, let me just save real quick...')
    finally:
        sd.save()
        if len(sd.creatures) < 2:
            print('You need at least two creatures in your population to have '\
                  'an encounter. Unfortunately, this means the end for your ' \
                  'population.')
            if len(sd.creatures) == 1:
                print('Here is the last of its kind:')
                print(repr(sd.creatures.pop()))
        


class SaveData(object):
    def __init__(self, creatures, num_encounters, filename = None):
        self.creatures = creatures
        self.num_encounters = num_encounters
        self.filename = filename or SAVE_FILENAME

    def save(self):
        '''Saves a generation to a file, with the generation number for starting
        up again'''
        print('Saving progress to file.')
        with open(SAVE_FILENAME, 'w') as savefile:
            pickle.dump(self, file = savefile, protocol = 2)
        print('Finished saving.')
    
    def generate_snapshot(self):
        pass

def load(savefile):
    '''Loads savedata from `savefile`'''
    return pickle.load(savefile)
      
def do_random_encounter(creatures):
    '''Runs a fight between two random creatures at the current verbosity'''
    with random_encounter(creatures, copy = True) as (p1, p2):
        print(repr(p1))
        print(repr(p2))
        print1('{0.name} is fighting {1.name}'.format(p1, p2))
        encounter(p1, p2)

if __name__ == '__main__':
    if os.path.isfile(SAVE_FILENAME):
        with open(SAVE_FILENAME, 'r') as savefile:
            try:
                sd = load(savefile)
            except Exception as e:
                print('Invalid save file!', e, file=sys.stdin)
                sys.exit(1)

        print('Loaded an existing save file with {gen_size} creatures with '\
              '{num_encounters} encounters under their belt'\
                  .format(gen_size = len(sd.creatures), 
                          num_encounters = sd.num_encounters))
    else:
        print('No save file found, creating a new generation!')
        sd = SaveData(creatures = [Creature() for _ in xrange(0, 100)],
                      num_encounters = 0,
                      filename = SAVE_FILENAME
                      )
        sd.save()
    
    while True:
        try:
            userin = raw_input('command> ')
        except (KeyboardInterrupt, EOFError):
            print('Bye!')
            break
        try:
            if userin == 'watch fight':
                do_random_encounter(sd.creatures)
            elif userin == 'num creatures':
                print(len(sd.creatures))
            elif userin == 'exit':
                print('\nBye!')
                break
            elif userin == 'simulate':
                simulate(sd)
            elif userin == 'v0':
                set_verbosity(0)
                print('Verbosity level is {}'.format(get_verbosity()))
            elif userin == 'v1':
                set_verbosity(1)
                print('Verbosity level is {}'.format(get_verbosity()))
            elif userin == 'v2':
                set_verbosity(2)
                print('Verbosity level is {}'.format(get_verbosity()))
            elif userin == 'v3':
                set_verbosity(3)
                print('Verbosity level is {}'.format(get_verbosity()))
            elif userin == 'echo verbosity':
                print('Verbosity level is {}'.format(get_verbosity()))
            elif userin == 'show random':
                print(repr(rand.choice(sd.creatures)))
            elif userin == 'show most wins':
                print(repr(max(sd.creatures, key = lambda c: c.kills)))
            elif userin == 'show oldest':
                print(repr(max(sd.creatures, key = lambda c: c.generation)))
            elif userin == 'show survivalist':
                print(repr(max(sd.creatures, key = lambda c: c.survived)))
            elif userin == 'show most skillful':
                def _skill(c):
                    'Determine skill number'
                    if c.survived > 0:
                        return (float(c.kills ** 2) / c.survived)
                    else:
                        return 0
                print(repr(max(sd.creatures, key = _skill)))
            elif userin == 'show most items':
                print(repr(max(sd.creatures, key = lambda c: len(c.inv))))
            elif userin.split()[0] == 'fight':
                fighter1, fighter2 = ('random',)*2
                if len(userin.split()) > 1:
                    fighter1 = userin.split()[1]
                if len(userin.split()) > 2:
                    fighter2 = userin.split()[2]
                getname = lambda name : lambda x: x.name == name
                if fighter1 == 'random':
                    fighter1 = rand.choice(sd.creatures).copy
                else:
                    fighter1 = [c for c in sd.creatures 
                                if c.name == fighter1][0].copy
                if fighter2 == 'random':
                    fighter2 = rand.choice(sd.creatures).copy
                else:
                    fighter2 = [c for c in sd.creatures if c.name == fighter2][0].copy
                encounter(fighter1, fighter2)
            elif userin == 'gene survey':
                # split up dna by genes, throw in bucket and count them, then show
                # summary here
                pass
            else:
                print('command not recognized.')
        except:
            import traceback
            traceback.print_exc()

