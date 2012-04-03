"""The Arena and how the fighters are to mess with each other"""
from __future__ import print_function

import random as rand
from random import randint
from itertools import izip
from contextlib import contextmanager
import operator as op
import cPickle as pickle
import sys, os.path, time, cmd

from Parsing import ACT
from Utils import print1, print2, print3, progress_bar, get_verbosity, \
    set_verbosity, term_width
from Creatures import Creature, Feeder, try_to_mate

OPTIMAL_GEN_SIZE = 1000
    
SAVE_FILENAME = 'evofighters.save'
SAVE_PERIOD = 90.0


def encounter(p1, p2):
    '''Carries out an encounter between two creatures'''
    # these numbers were very carefully tuned to pretty much never go less than
    # 10 rounds
    max_rounds = abs(int(rand.gauss(200, 30)))
    children = []
    print1('Max rounds: {}', max_rounds)
    for rounds, (p1act, c1), (p2act, c2) in izip(xrange(max_rounds),
                                                 p1.decision_generator(),
                                                 p2.decision_generator()):
        print2('Round {}', rounds)
        try:
            if c1 > c2:
                print3('{0.name} is going first', p2)
                child = do_round(p1, p1act, p2, p2act)
            else:
                print3('{0.name} is going first', p1)
                child = do_round(p2, p2act, p1, p1act)
        except FightOver as fo:
            print3('The fight ended before it timed out')
            if fo.child is not None:
                children.append(fo.child)
            break
        if child is not None:
            children.append(child)
        if p1.dead or p2.dead:
            break
        p1.last_action = p1act
        p2.last_action = p2act
    else:
        # if the rounds timed out, penalty
        penalty = randint(1,5)
        print1('Time is up!, both combatants take {} damage', penalty)
        p1.energy -= penalty
        p2.energy -= penalty
    def _victory(winner, loser):
        print1('{.name} has killed {.name}', winner, loser)
        winner.add_item(loser.pop_item())
        winner.energy = min(40, winner.energy + randint(1, 6))
        winner.survived += 1
        winner.kills += 1
        winner.last_action = Creature.wait_action
    if p2.dead and p1.alive:
        _victory(p1, p2)
    elif p1.dead and p2.alive:
        _victory(p2, p1)
    elif p1.dead and p2.dead:
        print1('Both {0.name} and {1.name} have died.', p1, p2)
    else:
        p1.survived += 1
        p2.survived += 1
        p1.last_action = Creature.wait_action
        p2.last_action = Creature.wait_action
    return children

def do_round(p1, p1_act, p2, p2_act):
    '''Handles carrying out the decided actions for a single round'''
    # convenient short-hands to make code more readable
    ATTACKING = 1
    DEFENDING = 2
    MATING    = 3
    OTHER     = 4
    # defaults to OTHER if the key is not present
    act_kind = {ACT['attack'] : ATTACKING,
                ACT['defend'] : DEFENDING,
                ACT['mate']   : MATING,
                ACT['signal'] : OTHER,
                ACT['use']    : OTHER,
                ACT['take']   : OTHER,
                ACT['wait']   : OTHER,
                ACT['flee']   : OTHER }

    # c2h = chance to hit
    #  Below indexes: 0=mate_chance, 1=p1_c2h, 2=p2_c2h, 3=dmg1_mult,
    #                 4=dmg2_mult,   5=p1_share,     6=p2_share
        
    damage_matrix = {           #      0   1   2   3   4   5   6
        (ATTACKING, ATTACKING)  :   (  0, 75, 75, 50, 50,  0,  0),
        (ATTACKING, DEFENDING)  :   (  0, 25, 25, 25, 25,  0,  0),
        (ATTACKING, MATING)     :   ( 50, 50,  0, 75,  0, 70, 30),
        (ATTACKING, OTHER)      :   (  0,100,  0,100,  0,  0,  0),
        (DEFENDING, DEFENDING)  :   (  0,  0,  0,  0,  0,  0,  0),
        (DEFENDING, MATING)     :   ( 25,  0,  0,  0,  0, 70, 30),
        (DEFENDING, OTHER)      :   (  0,  0,  0,  0,  0,  0,  0),
        (MATING,    MATING)     :   (100,  0,  0,  0,  0, 50, 50),
        (MATING,    OTHER)      :   ( 75,  0,  0,  0,  0,  0,100),
        (OTHER,     OTHER)      :   (  0,  0,  0,  0,  0,  0,  0),
        # the rest of these are duplicates of the above with swapped order
        (DEFENDING, ATTACKING)  :   (  0, 25, 25, 25, 25,  0,  0),
        (MATING,    ATTACKING)  :   ( 50,  0, 50,  0, 75, 30, 70),
        (MATING,    DEFENDING)  :   ( 25,  0,  0,  0,  0, 30, 70),
        (OTHER,     ATTACKING)  :   (  0,  0,100,  0,100,  0,  0),
        (OTHER,     DEFENDING)  :   (  0,  0,  0,  0,  0,  0,  0),
        (OTHER,     MATING)     :   ( 75,  0,  0,  0,  0,100,  0),
        }
    mults = damage_matrix[(act_kind[p1_act.typ], act_kind[p2_act.typ])]
    def damage_fun(chance, mult):
        '''Takes a "chance to hit" and a "damage multiplier" and returns
        damage'''
        if randint(1,100) <= chance:
            return randint(1, int(round(((mult/100.0) * 6))))
        else:
            return 0
    p1_dmg = damage_fun(mults[1], mults[3])
    p2_dmg = damage_fun(mults[2], mults[4])
    # TODO: take into account damage type!
    if p1_dmg > 0:
        print1('{.name} takes {} damage', p2, p1_dmg)
        p2.energy -= p1_dmg
    if p2_dmg > 0:
        print1('{.name} takes {} damage', p1, p2_dmg)
        p1.energy -= p2_dmg
    # we reverse the order of p1, p2 when calling try_to_mate because paying
    # costs first in mating is worse, and in this function p1 is preferred in
    # actions that happen to both creatures in order. Conceivably, p2 could die
    # without p1 paying any cost at all, even if p2 initiated mating against
    # p1's will
    child = try_to_mate(mults[0], p2, mults[6], p1, mults[5])
    if child:
        print1('{.name} and {.name} have a child named {.name}', p1, p2, child)
        if not child.dna:
            print1('But it was stillborn as it has no dna.')
            child = None
    try:
        if act_kind[p1_act.typ] == OTHER:
            p1.carryout(p1_act)
        if act_kind[p2_act.typ] == OTHER:
            p2.carryout(p2_act)
    except StopIteration:
        raise FightOver(child)
    print3('{0.name} has {0.energy} life left', p1)
    print3('{0.name} has {0.energy} life left', p2)
    return child

class FightOver(StopIteration):
    '''Thrown when a fight is over'''
    def __init__(self, child):
        self.child = child

def maxencounters(creatures):
    '''Number of encounters required for a given population based on size'''
    return round((len(creatures) ** 3) / (OPTIMAL_GEN_SIZE * 1000.0))
       
@contextmanager
def random_encounter(sd, copy = False):
    '''A context manager that handles selecting two random creatures from the
    creature list, setting them as targets of each other, and then yielding to
    the actual encounter code.'''
    c_len = len(sd.creatures)
    if c_len < 2:
        raise RuntimeError('Not enough creatures.')
    
    p1_i = rand.randint(0, c_len - 1)
    p2_i = rand.randint(0, c_len + sd.feeder_count - 1)
    while p1_i == p2_i:
        p2_i = rand.randint(0, c_len + sd.feeder_count - 1)
    p1 = sd.creatures[p1_i]
    if p2_i < c_len:
        p2 = sd.creatures[p2_i]
    else:
        p2 = Feeder()
    if copy:
        p1 = p1.copy
        p2 = p2.copy
        p1.energy = 40
        p2.energy = 40
    p1.target = p2
    p2.target = p1
    try:
        yield p1, p2
    finally:
        p1.target = None
        p2.target = None
        if not (p1.alive or copy):
            del sd.creatures[p1_i]
        if not (p2.alive or copy):
            if p2.is_feeder:
                sd.feeder_count -= 1
            else:
                del sd.creatures[p2_i]

def simulate(sd):
    time_till_save = progress_bar('{:4} creatures, {:4} feeders, {} total '
                                  'encounters',
                                  lambda: len(sd.creatures),
                                  lambda: sd.feeder_count,
                                  lambda: sd.num_encounters)
    timestamp = updatetime = time.time()
    try:
        while True:
            new_time = time.time()
            if len(sd.creatures) < 2:
                raise RuntimeError('Not enough creatures')
            if new_time - timestamp > SAVE_PERIOD:
                print('\nCurrently', len(sd.creatures), 'creatures alive.')
                sd.save()
                timestamp = time.time()
                print()            
            if  new_time - updatetime > 0.033333: #update 30 times per second
                time_till_save.send((time.time() - timestamp) / SAVE_PERIOD)
                updatetime = time.time()
            total_beings = len(sd.creatures) + sd.feeder_count
            if total_beings < OPTIMAL_GEN_SIZE:
                sd.feeder_count += 1

            with random_encounter(sd) as (p1, p2):
                print1('{.name} encounters {.name} in the wild', p1, p2)
                sd.creatures.extend(encounter(p1, p2))
                if not (p1.is_feeder or p2.is_feeder):
                    sd.num_encounters += 1
                    
            
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
    def __init__(self, creatures, feeder_count, num_encounters, count, filename = None):
        self.creatures = creatures
        self.feeder_count = feeder_count
        self.num_encounters = num_encounters
        self.count = count
        self.filename = filename or SAVE_FILENAME

    def save(self):
        '''Saves a generation to a file, with the generation number for starting
        up again'''
        print('Saving progress to file.')
        with open(SAVE_FILENAME, 'w') as savefile:
            pickle.dump(self, file = savefile, protocol = 2)
        print('Finished saving.')

def load(savefile):
    '''Loads savedata from `savefile`'''
    sd = pickle.load(savefile)
    Creature.count = sd.count
    return sd
      
def do_random_encounter(sd):
    '''Runs a fight between two random creatures at the current verbosity'''
    with random_encounter(sd, copy = True) as (p1, p2):
        print(repr(p1))
        print(repr(p2))
        print1('{0.name} is fighting {1.name}', p1, p2)
        encounter(p1, p2)


class EvoCmd(cmd.Cmd):
    '''Command line processor for EvoFighters'''

    prompt = 'EvoFighters >>>> '

    def __init__(self, sd):
        cmd.Cmd.__init__(self)
        self.sd = sd
    
    @property
    def intro(self):
        tw = term_width()
        if tw >= 90:
            return \
"""
'||''''|  '||'  '|'  ..|''||   '||''''|  ||          '||        .                          
 ||  .     '|.  .'  .|'    ||   ||  .   ...    ... .  || ..   .||.    ....  ... ..   ....  
 ||''|      ||  |   ||      ||  ||''|    ||   || ||   ||' ||   ||   .|...||  ||' '' ||. '  
 ||          |||    '|.     ||  ||       ||    |''    ||  ||   ||   ||       ||     . '|.. 
.||.....|     |      ''|...|'  .||.     .||.  '||||. .||. ||.  '|.'  '|...' .||.    |'..|' 
                                             .|....'                                       
"""
        elif tw >= 79 :
            return \
"""
 ________                ________  _         __       _                         
|_   __  |              |_   __  |(_)       [  |     / |_                      
  | |_ \_|_   __   .--.   | |_ \_|__   .--./)| |--. `| |-'.---.  _ .--.  .--.  
  |  _| _[ \ [  ]/ .'`\ \ |  _|  [  | / /'`\;| .-. | | | / /__\\[ `/'`\]( (`\]
 _| |__/ |\ \/ / | \__. |_| |_    | | \ \._//| | | | | |,| \__., | |     `'.'. 
|________| \__/   '.__.'|_____|  [___].',__`[___]|__]\__/ '.__.'[___]   [\__) )
                                     ( ( __))
"""
        elif tw >= 51:
            return \
"""
  __             ___                               
 /              /    /      /    /                 
(___       ___ (___    ___ (___ (___  ___  ___  ___
|     \  )|   )|    | |   )|   )|    |___)|   )|___
|__    \/ |__/ |    | |__/ |  / |__  |__  |     __/
                      __/                          
"""
        else:
            return 'EvoFighters (You may want to widen your terminal)'
    def default(self, line):
        print("Sorry, that isn't a recognized command")

    def doc_header(self):
        return 'Available commands:'
    def do_simulate(self, arg):
        simulate(self.sd)
        
    def do_show(self, arg):
        '''Shows various things'''
        args = arg.split()
        if args[0] in (c.name for c in self.sd.creatures):
            print(repr(next((c for c in self.sd.creatures if c.name == args[0]), None)))
        elif args[0] == 'verbosity':
            print('verbosity = {}'.format(get_verbosity()))
        elif args[0] == 'random':
            print(repr(rand.choice(self.sd.creatures)))
        elif args[0] == 'max':
            try:
                print(repr(max(self.sd.creatures,
                               key = op.attrgetter(args[1]))))
            except:
                print("Couldn't get the maximum of that")
        elif args[0] == 'min':
            try:
                print(repr(min(self.sd.creatures,
                               key = op.attrgetter(args[1]))))
            except:
                print("Couldn't get the minimum of that.")
        elif arg == 'most skillful':
            def _skill(c):
                'Determine skill number'
                if c.survived > 0:
                    return (float(c.kills ** 2) / c.survived)
                else:
                    return 0
            print(repr(max(self.sd.creatures, key = _skill)))
        else:
            print("Not sure what you want me to show you :(")

    def do_count(self, arg):
        '''Count either creatures or feeders'''
        if arg == 'creatures':
            num = len(self.sd.creatures)
            print('There are {} creatures'.format(num))
        elif arg == 'feeders':
            num = self.sd.feeder_count
            print('There are {} feeders.'.format(num))
        else:
            print("Not sure what we're counting here")

    def do_set(self, arg):
        '''Set a variable'''
        args = arg.split()
        if args[0] == 'verbosity':
            try:
                set_verbosity(int(args[1]))
            except ValueError:
                print("That isn't a valid verbosity level.")
                return
        else:
            print("I can't set that right now. Maybe soon")
        
    def do_fight(self, arg):
        '''Watch a fight between two creatures'''
        args = arg.split()
        if len(args) >= 1:
            fighter1 = next((c for c in self.sd.creatures 
                             if c.name == int(args[0])), None)
        else:
            fighter1 = rand.choice(self.sd.creatures)
        if len(args) >= 2:
            fighter2 = next((c for c in self.sd.creatures
                             if c.name == int(args[1])), None)
        else:
            fighter2 = rand.choice(self.sd.creatures)
        
        if fighter1 is None or fighter2 is None:
            print("Invalid fighter name")
            return
        
        do_random_encounter([fighter1, fighter2])

    def do_EOF(self, arg):
        raise KeyboardInterrupt

    def do_exit(self, arg):
        'Exit evofighters'
        raise KeyboardInterrupt


def main(argv):
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
        sd = SaveData(creatures = [Creature() 
                                   for i in xrange(0, OPTIMAL_GEN_SIZE)],
                      feeder_count = 0,
                      num_encounters = 0,
                      filename = SAVE_FILENAME,
                      count = OPTIMAL_GEN_SIZE
                      )
        sd.save()
    
    try:
        EvoCmd(sd).cmdloop()
    except KeyboardInterrupt:
        print('Bye')
        
if __name__ == '__main__':
    main(sys.argv)
