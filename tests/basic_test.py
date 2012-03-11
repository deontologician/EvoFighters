import EvoFighters.Arena as A
import cPickle as pickle

def test_fight():
    A.set_verbosity(0)
    a = A.Creature()
    b = A.Creature()
    A.fight(a, b)
    print repr(a)
    print repr(b)

def test_pickle():
    a = A.Creature()
    ap = pickle.dumps(a,2)
    b = pickle.loads(ap)
    print a
    print b

def test_repr():
    a = A.Creature()
    print str(a)
    print repr(a)

if __name__ == '__main__':
    test_fight()
    #test_pickle()
    #test_repr()
    
