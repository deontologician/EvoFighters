"""The Arena and how the fighters are to mess with each other"""
from __future__ import print_function

import random as rand
from random import randint
from itertools import count
from math import ceil
import cPickle as pickle
import sys, os.path

import Parsing as P
from Utils import print1, print2, progress_bar, get_verbosity, set_verbosity
from Creatures import Creature, mate

mate_mult = 1.5
optimal_generation_size = 500.0

def fight(p1, p2):
    p1.target = p2
    p2.target = p1
    rounds = 0
    while randint(0, 200) != 200:
        rounds += 1
        print2('Round {}'.format(rounds))
        p1act = p1.last_action = p1.next_action
        p2act = p2.last_action = p2.next_action
        if p1act.typ == P.Action.str.attack or p2act.typ == P.Action.str.attack:
            attacking(p1, p1act, p2, p2act)
        else:
            carryout(p1, p1act, p2)
            carryout(p2, p2act, p1)
        if p2.dead and p1.alive:
            print1(p1.name, 'has won.')
            p1.inv.extend(p2.inv)
            p1.energy += min(40 - p2.energy, randint(1,6))
            p1.target = None
            p1.fights_survived += 1
            p1.fights_won += 1
            return
        elif p1.dead and p2.alive:
            print1(p2.name, 'has won')
            p2.inv.extend(p1.inv)
            p2.energy += min(40 - p2.energy, randint(1,6))
            p2.target = None
            p2.fights_survived += 1
            p2.fights_won += 1
            return
        elif p1.dead and p2.dead:
            print1('Both {0.name} and {1.name} have died.'.format(p1, p2))
            p1.target = None # garbage collection
            p2.target = None
            return
        p1, p2 = p2, p1
    print1('Both combatants survived after {} rounds'.format(rounds))
    p1.fights_survived += 1
    p2.fights_survived += 1
    p1.target = None
    p2.target = None

def carryout(p1, act, p2):
    '''p1 acts, possibly on p2'''
    # take an item from the other's inventory
    if act.typ == P.Action.str.take:
        if p2.inv:
            item = p2.inv.pop()
            print1("{0.name} takes {1} from {2.name}"\
                       .format(p1, P.item_repr(item), p2))
            p1.inv.append(item)
        else:
            print2("{0.name} tries to take an item from {1.name}, "\
                       "but there's nothing to take.".format(p1, p2))
    #using an item
    elif act.typ == P.Action.str.use:
        if p1.inv:
            print1(p1.name, 'uses', P.item_repr(p1.inv[-1]))
            p1.use()
        else:
            print2(p1.name, "tries to use an item, but doesn't have one")
    #signalling
    elif act.typ == P.Action.str.signal:
        print1(p1.name, 'signals with color', P.sig_repr(act.arg))
        p1.signal = act.arg
    # waiting 
    elif act.typ == P.Action.str.wait:
        print2(p1.name, 'waits')
    # defending with no corresponding attack
    elif act.typ == P.Action.str.defend:
        print2(p1.name, 'defends, but no one is attacking')
    else:
        print1(p1.name, 'did', act.typ, 'with magnitude:', act.arg)
        assert False
       

def attacking(p1, p1_act, p2, p2_act):
    '''Handles attacking and defending. Call this only if either p1 or p2 is
    attacking'''
    p1_att = p1_act.typ == P.Action.str.attack
    p2_att = p2_act.typ == P.Action.str.attack
    p1_def = p1_act.typ == P.Action.str.defend
    p2_def = p2_act.typ == P.Action.str.defend
    if p1_att:
        if p2_att:
            print1('Both fighters are attacking')
            p1_dmg = randint(3,6)
            p1.energy -= p1_dmg
            print1(p1.name, 'takes', p1_dmg, 'and is down to', p1.energy, 'energy')
            p2_dmg = randint(3,6)
            p2.energy -= p2_dmg
            print1(p2.name, 'takes', p2_dmg, 'and is down to', p2.energy, 'energy')
        elif p2_def:
            print1(p1.name, 'is attacking and', p2.name, 'is defending')
            p2_dmg = randint(2,5) * damage_mult[p1_act.arg][p2_act.arg]
            p2.energy -= p2_dmg
            if p2_dmg < 0:
                print1(p2.name, 'heals', -p2_dmg, 'energy. Up to:', p2.energy)
            else:
                print1(p2.name, 'takes', p2_dmg, 'damage. Down to:', p2.energy)
            p2.energy = min(40, p2.energy)
        else:
            print1(p1.name, 'is attacking, but',p2.name,'is not concerned.')
            p2_dmg = randint(1,4)
            carryout(p2, p2_act, p1)
            p2.energy -= p2_dmg
            print1(p2.name, 'takes', p2_dmg, 'damage. Down to:', p2.energy)
    elif p2_att:
        if p1_def:
            print1(p2.name, 'is attacking and', p1.name, 'is defending')
            p1_dmg = randint(2,5) * damage_mult[p2_act.arg][p1_act.arg]
            p1.energy -= p1_dmg
            if p1_dmg < 0:
                print1(p1.name, 'heals', -p1_dmg, 'energy. Up to:', p1.energy)
            else:
                print1(p1.name, 'takes', p1_dmg, 'damage. Down to:', p1.energy)
            p1.energy = min(40, p1.energy)
                
        else: #p1 cannot be attacking, we already dealt with that
            print1(p2.name, 'is attacking, but', p1.name, 'is not concerned.')
            p1_dmg = randint(1,4)
            carryout(p1, p1_act, p2)
            p1.energy -= p1_dmg
            print1(p1.name, 'takes',p1_dmg, 'damage. Down to:', p1.energy)
    else:
        # this function should only be called when either p1 or p2 are attacking
        #print1(p1_act.typ, P.Action.str.attack, p2_act.typ, P.Action.str.attack)
        assert False

      
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

def feeding_time(creatures):
    '''Gives random amounts of food to the creatures randomly'''
    jitter = int(optimal_generation_size * 0.10)
    for _ in xrange(0, optimal_generation_size + randint(-jitter, jitter)):
        creatures[randint(0, len(creatures) - 1)].inv.append(randint(0, len(P.Item) - 1))


def mating_phase(creatures, gen_nbr, progress):
    print('Mating now')
    if len(creatures) < 10:
        print('Dangerously low population. Maximizing variation!')
        # mate everyone to everyone, including themselves
        creatures.extend([mate(a,b) for a in creatures for b in creatures])
    else:
        maxmatings = randint(0, int(len(creatures) * mate_mult * (1.0 - progress)))
        mate_progress = progress_bar(maxmatings, 'Doing {} matings...')
        for i in xrange(0, maxmatings):
            try:
                next(mate_progress)
                a = randint(0, len(creatures) - 1)
                b = randint(0, len(creatures) - 1)
                while a == b:
                    a = randint(0, len(creatures) - 1)
                am, bm = creatures[a], creatures[b]
                creatures.append(mate(am, bm))
            except (KeyboardInterrupt, EOFError):
                mate_progress.send(True) # quit progress bar
                raise NotDoneError(i, maxmatings)
    print('Creatures after repopulating: {}'.format(len(creatures)))


def fighting_phase(creatures, gen_nbr, progress):
    print('Fighting now')
    fight_mult = (len(creatures) ** 2) / (optimal_generation_size * 1000.0)
    print('Fight multiplier is {}'.format(fight_mult))
    maxfights = int(len(creatures) * fight_mult * (1.0 - progress))
    fight_progress = progress_bar(maxfights, 'Doing {} fights...')
    for i in xrange(0, maxfights):
        try:
            next(fight_progress)
            if len(creatures) == 1:
                print('Your last dude is doomed to extinction! ',
                      'Here is your forever alone creature for posterity:')
                print(repr(creatures[0]))
                return
            a = randint(0, len(creatures) - 1)
            b = randint(0, len(creatures) - 1)
            while a == b: # don't fight yourself!
                a = randint(0, len(creatures) - 1)
            af, bf = creatures[a], creatures[b]
            fight(creatures[a], creatures[b])
            if af.dead:
                creatures.remove(af)
            if bf.dead:
                creatures.remove(bf)
            if not creatures:
                print('All creatures died! This is improbable!')
                return
        except (KeyboardInterrupt, EOFError):
            fight_progress.send(True) # quit progress bar
            raise NotDoneError(i, maxfights)
    print('Creatures left: {}'.format(len(creatures)))
    for creature in creatures:
        creature.age += 1

class NotDoneError(Exception):
    def __init__(self, current, total):
        self.progress = float(current) / total

def generationer(creatures, gen_nbr, phase, progress):
    '''Runs the generation calculation'''
    for gen_nbr in count(gen_nbr):
        try:
            print('Generation {}'.format(gen_nbr))
            if phase == 'fighting':
                print('Feeding time!')
                feeding_time(creatures)
                fighting_phase(creatures, gen_nbr, progress)
                phase, progress = 'mating', 0.0
                save(creatures, gen_nbr, phase, progress)
            if phase == 'mating':
                mating_phase(creatures, gen_nbr, progress)
                phase, progress = 'fighting', 0.0
                save(creatures, gen_nbr, phase, progress)
        except NotDoneError as nde:
            save(creatures, gen_nbr, phase, nde.progress)
            print('Was {0:.2f}% done with {1}'.format(nde.progress * 100,
                                                      phase))
            return creatures, gen_nbr, phase, nde.progress
            
        


damage_mult = [[ 0,  1, -1],
               [-1,  0,  1],
               [ 1, -1,  0]]

savefilename = 'evofighters.save'

def save(creatures, generation, phase, progress):
    '''Saves a generation to a file, with the generation number for starting up
    again'''
    print('Saving Generation to file...')
    with open(savefilename, 'w') as savefile:
        savefile.write(pickle.dumps(([i.pickled for i in creatures],
                                     generation,
                                     phase,
                                     progress)))
      
def random_fight(creatures):
    '''Runs a fight between two random creatures at the current verbosity'''
    a = randint(0, len(creatures) - 1)
    b = randint(0, len(creatures) - 1)
    while a == b:
        a = randint(0, len(creatures) - 1)
    af = creatures[a].copy
    bf = creatures[b].copy
    print(repr(af))
    print(repr(bf))
    print1('{0.name} is fighting {1.name}'.format(af, bf))
    fight(af, bf)

if __name__ == '__main__':
    if os.path.isfile(savefilename):
        with open(savefilename, 'r') as savefile:
            try:
                _creatures, gen_nbr, phase, progress = pickle.loads(savefile.read())
                creatures = [Creature.from_pickle(i) for i in _creatures]
            except:
                print('Invalid save file!', file=sys.stdin)
                sys.exit(1)

        print('Loaded an existing save file with {gen_size} creatures of '\
                  'generation {gen_nbr} in it who who are {progress:.2f}% done '\
                  'with {phase}'.format(gen_size = len(creatures), 
                                        gen_nbr = gen_nbr, 
                                        progress = progress * 100, 
                                        phase = phase))
    else:
        print('No save file found, creating a new generation!')
        creatures = [Creature() for _ in xrange(0, 100)]
        gen_nbr = 0
        phase = 'mating'
        progress = 0.0
        save(creatures, gen_nbr, phase, progress)
    
    while True:
        try:
            userin = raw_input('command> ')
        except (KeyboardInterrupt, EOFError):
            print('Bye!')
            break
        if userin == 'watch fight':
            random_fight(creatures)
        elif userin == 'exit':
            print('Bye!')
            break
        elif userin == 'simulate':
            creatures, gen_nbr, phase, progress = generationer(creatures,
                                                               gen_nbr, 
                                                               phase, 
                                                               progress)
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
            print(repr(rand.choice(creatures)))
        elif userin == 'show most wins':
            print(repr(max(creatures, key = lambda c: c.fights_won)))
        elif userin == 'show oldest':
            print(repr(max(creatures, key = lambda c: c.age)))
        elif userin == 'show survivalist':
            print(repr(max(creatures, key = lambda c: c.fights_survived)))
        elif userin == 'show most skillful':
            def _skill(c):
                if c.fights_survived > 0:
                    return (float(c.fights_won ** 2) / c.fights_survived)
                else:
                    return 0
            print(repr(max(creatures, key = _skill)))
        elif userin == 'show most items':
            print(repr(max(creatures, key = lambda c: len(c.inv))))
        elif userin.split()[0] == 'fight':
            fighter1, fighter2 = ('random',)*2
            if len(userin.split()) > 1:
                fighter1 = userin.split()[1]
            if len(userin.split()) > 2:
                fighter2 = userin.split()[2]
            getname = lambda name : lambda x: x.name == name
            if fighter1 == 'random':
                fighter1 = rand.choice(creatures).copy
            else:
                fighter1 = filter(getname(fighter1), creatures)[0].copy
            if fighter2 == 'random':
                fighter2 = rand.choice(creatures).copy
            else:
                fighter2 = filter(getname(fighter2), creatures)[0].copy
            fight(fighter1, fighter2)
        elif userin == 'gene survey':
            # split up dna by genes, throw in bucket and count them, then show
            # summary here
            pass
        

