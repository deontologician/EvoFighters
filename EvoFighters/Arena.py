"""The Arena and how the fighters are to mess with each other"""
from __future__ import print_function

import random as rand
from random import randint
from itertools import count, izip
from contextlib import contextmanager
from collections import namedtuple
import cPickle as pickle
import sys, os.path

from Parsing import ACT, ITEM
from Utils import print1, print2, print3, progress_bar, get_verbosity, \
    set_verbosity
from Creatures import Creature, CreatureFlees, mate

mate_mult = 1.5
optimal_generation_size = 500.0

def encounter(p1, p2):
    '''Carries out an encounter between two creatures'''
    # these numbers were very carefully tuned to pretty much never go less than
    # 10 rounds
    max_rounds = abs(int(rand.gauss(200, 30)))
    print1('Max rounds: {}'.format(max_rounds))
    try:
        for rounds, (p1act, c1), (p2act, c2) in izip(xrange(max_rounds),
                                                     p1.decision_generator(),
                                                     p2.decision_generator()):
            print2('Round {}'.format(rounds))
            if p1act.typ == ACT.attack or p2act.typ == ACT.attack:
                if c1 > c2:
                    attacking(p2, p2act, p1, p1act)
                else:
                    attacking(p1, p1act, p2, p2act)
            else:
                if c1 > c2:
                    print3('{0.name} is going first'.format(p2))
                    p2.carryout(p2act)
                    p1.carryout(p1act)
                else:
                    print3('{0.name} is going first'.format(p1))
                    p1.carryout(p1act)
                    p2.carryout(p2act)
    except CreatureFlees:
        pass
    if p2.dead and p1.alive:
        print1(p1.name, 'has won.')
        p1.inv.extend(p2.inv)
        p1.energy = min(40, p1.energy + randint(1, 6))
        p1.survived += 1
        p1.kills += 1
        return
    elif p1.dead and p2.alive:
        print1(p2.name, 'has won')
        p2.inv.extend(p1.inv) #looting the corpse
        p2.energy = min(40, p2.energy + randint(1, 6))
        p2.survived += 1
        p2.kills += 1
        return
    elif p1.dead and p2.dead:
        print1('Both {0.name} and {1.name} have died.'.format(p1, p2))
        return
    p1.survived += 1
    p2.survived += 1


def attacking(p1, p1_act, p2, p2_act):
    '''Handles attacking and defending. Call this only if either p1 or p2 is
    attacking'''
    p1_att = p1_act.typ == ACT.attack
    p2_att = p2_act.typ == ACT.attack
    p1_def = p1_act.typ == ACT.defend
    p2_def = p2_act.typ == ACT.defend
    if p1_att:
        if p2_att:
            print1('Both fighters are attacking')
            p1_dmg = randint(3, 6)
            p1.energy -= p1_dmg
            print1(p1.name, 'takes', p1_dmg, 'and is down to', p1.energy,
                   'energy')
            p2_dmg = randint(3, 6)
            p2.energy -= p2_dmg
            print1(p2.name, 'takes', p2_dmg, 'and is down to', p2.energy,
                   'energy')
        elif p2_def:
            print1(p1.name, 'is attacking and', p2.name, 'is defending')
            p2_dmg = randint(2, 5) * damage_mult[p1_act.arg][p2_act.arg]
            p2.energy -= p2_dmg
            if p2_dmg < 0:
                print1(p2.name, 'heals', -p2_dmg, 'energy. Up to:', p2.energy)
            else:
                print1(p2.name, 'takes', p2_dmg, 'damage. Down to:', p2.energy)
            p2.energy = min(40, p2.energy)
        else:
            print1(p1.name, 'is attacking, but', p2.name, 'is not concerned.')
            p2_dmg = randint(1, 4)
            p2.carryout(p2_act)
            p2.energy -= p2_dmg
            print1(p2.name, 'takes', p2_dmg, 'damage. Down to:', p2.energy)
    elif p2_att:
        if p1_def:
            print1(p2.name, 'is attacking and', p1.name, 'is defending')
            p1_dmg = randint(2, 5) * damage_mult[p2_act.arg][p1_act.arg]
            p1.energy -= p1_dmg
            if p1_dmg < 0:
                print1(p1.name, 'heals', -p1_dmg, 'energy. Up to:', p1.energy)
            else:
                print1(p1.name, 'takes', p1_dmg, 'damage. Down to:', p1.energy)
            p1.energy = min(40, p1.energy)
                
        else: #p1 cannot be attacking, we already dealt with that
            print1(p2.name, 'is attacking, but', p1.name, 'is not concerned.')
            dmg_to_p1 = randint(1, 4)
            p1.carryout(p1_act)
            p1.energy -= dmg_to_p1
            print1(p1.name, 'takes', dmg_to_p1, 'damage. Down to:', p1.energy)
    else:
        # this function should only be called when either p1 or p2 are attacking
        #print1(p1_act.typ, ACT.attack, p2_act.typ,
        #ACT.attack)
        assert False

      
def clear_dead(creatures):
    '''Return a new list with all dead creatures removed'''
    return [creature for creature in creatures if not creature.dead]


def feeding_time(creatures):
    '''Gives random amounts of food to the creatures randomly'''
    jitter = int(optimal_generation_size * 0.10)
    for _ in xrange(0, optimal_generation_size + randint(-jitter, jitter)):
        creatures[randint(0, len(creatures) - 1)]\
            .inv.append(randint(0, len(ITEM) - 1))


def mating_phase(creatures, progress, children = None):
    '''Does mating phase (soon to be deprecated in favor of encounter based
    mating)'''
    print('Mating now')
    maxmatings = randint(0, int(len(creatures) * mate_mult * (1.0 - progress)))
    mate_progress = progress_bar()
    children = children or []
    print('Doing {} matings...'.format(maxmatings))
    try:
        for i in xrange(maxmatings):
            mate_progress.send(float(i) / maxmatings)
            with random_encounter(creatures) as (m1, m2):
                children.append(mate(m1, m2))
        mate_progress.send(1.0)
    except (KeyboardInterrupt, EOFError):
        mate_progress.send(True) # quit progress bar
        raise NotDoneError(float(i) / maxmatings, children)
    creatures.extend(children)
    print('Creatures after repopulating: {}'.format(len(creatures)))

def maxencounters(creatures):
    '''Number of encounters required for a given population based on size'''
    return round((len(creatures) ** 3) / (optimal_generation_size * 1000.0))
       
def encounter_phase(creatures, progress = 0.0):
    '''Calculates how many encounters need to be done and carries them out'''
    encounter_progress = progress_bar()
    total_encounters = 0
    print('Doing encounters...')
    try:
        while progress < 1.0:
            encounter_progress.send(progress)
            with random_encounter(creatures) as (a, b):
                encounter(a, b)
            total_encounters += 1
            progress += 1 / maxencounters(creatures)
        encounter_progress.send(1.0)
    except (KeyboardInterrupt, EOFError):
        raise NotDoneError(progress)
    finally:
        print() # clear the progress bar line
    print('Creatures left after {} encounters: {}'.format(total_encounters, 
                                                          len(creatures)))
    for creature in creatures:
        creature.generation += 1

class NotDoneError(Exception):
    'Thrown when a phase is not complete'
    def __init__(self, msg, progress, children = None):
        Exception.__init__(self, msg)
        self.progress = progress
        self.children = children if children else []

@contextmanager
def random_encounter(creatures, copy = False):
    '''A context manager that handles selecting two random creatures from the
    creature list, setting them as targets of each other, and then yielding to
    the actual encounter code.'''
    if len(creatures) <= 1:
        raise NotEnoughCreatures('Need at least two creatures to have an'\
                                     ' encounter')
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

class NotEnoughCreatures(ValueError):
    '''Raised when not enough creatures are in the creature list to perform an
    operation'''
    pass

def generationer(sd):
    '''Runs the generation calculation'''
    children = sd.children if sd.children else []
    for gen in count(sd.gen_nbr):
        try:
            print('Generation {}'.format(gen))
            if sd.phase == 'fighting':
                print('Feeding time!')
                feeding_time(sd.creatures)
                encounter_phase(sd.creatures, sd.progress)
                sd.phase, sd.progress = 'mating', 0.0
                save(sd)
            if sd.phase == 'mating':
                mating_phase(sd.creatures, sd.progress, children)
                sd.phase, sd.progress = 'fighting', 0.0
                save(sd)
        except NotDoneError as nde:
            sd = SaveData(sd.creatures, sd.gen_nbr, sd.phase,
                          nde.progress, nde.children)
            save(sd)
            print('Was {0:.2f}% done with {1}.'.format(nde.progress * 100,
                                                      sd.phase))
            if nde.children:
                print('{} children born so far'.format(len(nde.children)))
            return sd
            
        


damage_mult = [[ 0,  1, -1],
               [-1,  0,  1],
               [ 1, -1,  0]]

savefilename = 'evofighters.save'


SaveData = namedtuple('SaveData', 'creatures gen_nbr phase progress children')

def save(creatures, gen_nbr, phase, progress, children = None):
    '''Saves a generation to a file, with the generation number for starting up
    again'''
    print1('Saving Generation to file...')
    with open(savefilename, 'w') as savefile:
        save = SaveData(creatures = creatures, 
                        gen_nbr = gen_nbr,
                        phase = phase,
                        progress = progress,
                        children = children if children else [])
        pickle.dump(save, file = savefile, protocol = 2)

def load(savefile):
    '''Loads savedata from `savefile`'''
    savedata = pickle.load(savefile)
    return SaveData(*savedata)
      
def do_random_encounter(creatures):
    '''Runs a fight between two random creatures at the current verbosity'''
    with random_encounter(creatures, copy = True) as (p1, p2):
        print(repr(p1))
        print(repr(p2))
        print1('{0.name} is fighting {1.name}'.format(p1, p2))
        encounter(p1, p2)

if __name__ == '__main__':
    if os.path.isfile(savefilename):
        with open(savefilename, 'r') as savefile:
            try:
                sd = load(savefile)
            except:
                print('Invalid save file!', file=sys.stdin)
                sys.exit(1)

        print('Loaded an existing save file with {gen_size} creatures of '\
                  'generation {gen_nbr} in it who who are {progress:.2f}% done'\
                  ' with {phase}'.format(gen_size = len(sd.creatures), 
                                         gen_nbr = sd.gen_nbr, 
                                         progress = sd.progress * 100, 
                                         phase = sd.phase,
                                         children = sd.children))
    else:
        print('No save file found, creating a new generation!')
        sd = SaveData(creatures = [Creature() for _ in xrange(0, 100)],
                      gen_nbr = 0,
                      phase = 'mating',
                      progress = 0.0,
                      children = [],
                      )
        save(sd)
    
    while True:
        try:
            userin = raw_input('command> ')
        except (KeyboardInterrupt, EOFError):
            print('Bye!')
            break
        try:
            if userin == 'watch fight':
                random_encounter(sd.creatures)
            elif userin == 'exit':
                print('\nBye!')
                break
            elif userin == 'simulate':
                sd = generationer(sd)
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
        except:
            import traceback
            traceback.print_exc()

