import cPickle as pickle
from Creatures import Creature

class SaveData(object):
    'Holds data about the run to save to disk'

    SAVE_FILENAME = 'evofighters.save'

    def __init__(self, creatures, feeder_count, num_encounters, count, dead,
                 filename = None):
        self.creatures = creatures
        self.feeder_count = feeder_count
        self.num_encounters = num_encounters
        self.count = count
        self.dead = dead
        self.filename = filename or SaveData.SAVE_FILENAME

    def save(self):
        '''Saves a generation to a file, with the generation number for starting
        up again'''
        print('Saving progress to file.')
        self.count = Creature.count
        with open(self.filename, 'w') as savefile:
            pickle.dump(self, file = savefile, protocol = 2)
        print('Finished saving.')

    @staticmethod
    def loadfrom(savefile):
        '''Loads savedata from `savefile`'''
        sd = pickle.load(savefile)
        Creature.count = sd.count
        return sd
