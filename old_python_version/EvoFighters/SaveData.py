import cPickle as pickle
import ConfigParser
import os
import re

from EvoFighters.Creatures import Creature
from EvoFighters.Utils import print_helper

    
_int_rgx = re.compile(r'\d+')
_float_rgx = re.compile(r'[\d\.]+')
_bool_rgx = re.compile(r'yes|no|true|false|on|off', flags=re.I)


def _make_bool(possibool):
    if possibool.lower() in ('on', 'true', 'yes'):
        return True
    elif possibool.lower() in ('off', 'false', 'no'):
        return False
    else:
        raise ValueError('Not Possibool!')


class Settings(object):
    '''Settings. By default loaded from the .settings file, but saved
    individually for each generation file'''

    __slots__ = ('max_pop_size', 'fps', 'save_filename', 'save_dir',
                 'config_dir', 'mutation_rate', 'mating_cost', 'max_inv_size',
                 'winner_life_bonus', 'save_interval', 'verbosity',
                 'max_thinking_steps', 'max_tree_depth')

    def __init__(self):
        self.max_pop_size = 12000
        self.fps = 1
        self.config_dir = '~/.config/EvoFighters'
        self.save_dir = 'saves'
        self.save_filename = 'default'
        self.winner_life_bonus = 5
        self.save_interval = 90
        self.verbosity = 0
        self.mutation_rate = 0.10 # higher = more mutations
        self.mating_cost = 40
        self.max_inv_size = 5
        # how many steps they are allowed to use to construct a thought
        self.max_thinking_steps = 200 
        # How deeply nested a thought tree is allowed to be
        self.max_tree_depth = 15

    def __str__(self):
        lines = ['{} = {!r}'.format(key, getattr(self, key))
                 for key in self.__slots__]
        return 'Settings:\n  ' + '\n  '.join(lines)

    @property
    def save_file(self):
        return os.path.join(
            os.path.expanduser(self.config_dir),
            self.save_dir,
            self.save_filename,
        )
    
    @property
    def config_file(self):
        return os.path.join(
            os.path.expanduser(self.config_dir), 'config')

    def set_from_strings(self, keyvals):
        '''Parse int/float/bool from string values'''
        for name, value in keyvals:
            if _int_rgx.match(value):
                setattr(self, name, int(value))
            elif _float_rgx.match(value):
                setattr(self, name, float(value))
            elif _bool_rgx.match(value):
                setattr(self, name, _make_bool(value))
            else:
                setattr(self, name, value)
        
    @staticmethod
    def from_config():
        config_dir = os.path.expanduser('~/.config/EvoFighters')
        if not os.path.exists(config_dir):
            os.makedirs(config_dir)
        filename = os.path.join(config_dir, 'config')
        s = Settings()
        if not os.path.exists(filename):
            return s
        config = ConfigParser.ConfigParser()
        with open(filename, 'r') as config_file:
            config.readfp(config_file)
            s.set_from_strings(config.items('global'))
        return s

    def write_config(self):
        config = ConfigParser.ConfigParser()
        config.add_section('global')
        for name in self.__slots__:
            config.set('global', name, getattr(self, name))
        with open(self.config_file, 'w') as config_file:
            config.write(config_file)


class SaveData(object):
    'Holds data about the run to save to disk'

    def __init__(self, creatures, feeder_count, num_encounters, count,
                 dead, settings=None):
        self.creatures = creatures
        self.feeder_count = feeder_count
        self.num_encounters = num_encounters
        self.count = count
        self.dead = dead
        self.settings = settings or Settings.from_config()

    def save(self):
        '''Saves a generation to a file, with the generation number
        for starting up again'''
        print('Saving progress to file.')
        self.count = Creature.count
        save_dir = os.path.dirname(self.settings.save_file)
        if not os.path.exists(save_dir):
            os.makedirs(save_dir)
        with open(self.settings.save_file, 'w+') as savefile:
            pickle.dump(self, file=savefile, protocol=2)
        print('Finished saving.')

    @staticmethod
    def loadfrom(savefile):
        '''Loads savedata from `savefile`'''
        sd = pickle.load(savefile)
        Creature.count = sd.count
        return sd

    def print1(self, *args, **kwargs):
        if self.settings.verbosity >= 1:
            print_helper(*args, prefix='***',**kwargs)

    def print2(self, *args, **kwargs):
        if self.settings.verbosity >= 2:
            print_helper(*args, prefix='**', **kwargs)

    def print3(self, *args, **kwargs):
        if self.settings.verbosity >= 3:
            print_helper(*args, prefix='*', **kwargs)

