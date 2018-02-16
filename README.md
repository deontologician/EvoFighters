```
   __             ___
  /              /    /      /    /
 (___       ___ (___    ___ (___ (___  ___  ___  ___
 |     \  )|   )|    | |   )|   )|    |___)|   )|___
 |__    \/ |__/ |    | |__/ |  / |__  |__  |     __/
                       __/  1.0
Josh Kuhn <deontologician@gmail.com>
Evolving fighting bots

USAGE:
    evofighters [OPTIONS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --file <SAVEFILE>    Name of save file [default: evofighters.evo]

SUBCOMMANDS:
    cycle-check    Does a cycle detection on the given bases
    help           Prints this message or the help of the given subcommand(s)
    simulate       Main command. Runs an evofighters simulation
```

## What is this?

This is a simulation I've written on and off over several years. It's
fundamentally about little bots with a very simple fighting
system. They attack, defend, pick up items, and occasionally try to
mate with each other.

When they mate, they combine their dna similar to how sexually reproducing
organisms in real life do, randomly selecting genes from each of the
two parents. There's a chance of different kinds of mutations like
gene swaps, transcription errors and the like.

Once you have replication, a competitive environment, and mutations,
you get evolution. It's not super sensitive to starting conditions, it
just works and is kind of fun.

## How it works

There are some rough phases the simulation occurs in for each creature:

1. Creating a new creature, which randomly chooses between each pair
   of genes from the parents. It also has a chance of random mutation.
2. Compiling/Parsing, which iterates through the bases of the
   creature's dna, and builds an ast with the creature's program for
   how to behave in an encounter. The parsing process is very
   forgiving: if the next base isn't a valid value for the next term,
   we just skip it and go on to the next base. If we reach the end of
   the dna, we wrap around to the beginning. If parsing takes too many
   steps we abort.
3. Evaluation, when the creature is in an actual encounter with
   another creature. The parsed ast is evaluated in the context of the
   fight (some variables depend on the opponent, some are random, so
   can't be done ahead of time). The outcome of this is a decision for
   which action the creature should take.
4. The creature takes an action. The 3 most interesting actions are
   attacking, defending and mating. Mating requires some reciprocity
   and some spare energy (otherwise you could create something from
   nothing and mating constantly would be the best strategy to pass on
   genes). Fighting allows you to gain energy if you win, but if the
   opponent is smart and defends correctly against you, you will
   expend resources fighting and not get anything in return.

There is an initial population of creatures, with a bootstrapped DNA
that parses out to "Turn 1: Try to mate. Turn 2: Flee". This is a
reasonable starting point since it ensures mating gets off the ground
(can't evolve if you don't reproduce), but it also ensures short
fights since they flee right afterward (and don't waste all of their
energy trying to mate over and over again).

For each encounter, two random creatures are selected from the
population, and pitted against each other. The fight is given a
randomized maximum number of rounds to last. Then the creatures go at
it, fighting or mating as their dna instructs them.

## History

I initially wrote a version of this in python. Being a simulation
though, it needed to run fast, so I made it work in pypy. I left the
project alone for a while.

When I came back about 3 years later, I rewrote the main portion of it
in pre-1.0 rust. It used several experimental extensions which were
never standardized. But I made some advances over the initial python
version. Mostly in speed, but also I wrote a clever compiler that
allowed precompiling and simplifying a creatures genes. Previously,
every time a fight was run, some evaluation/parsing occurred. Then I
left the project alone for a while.

When I came back about 3 years later, I fixed almost all of the
pre-1.0 problems and got it almost working in standard rust. The only
issue was that standard rust still didn't have box pattern syntax,
which was really useful for digging through deep recursive data
structures. So, we're on nightly rust, and only using two language
extensions (`box_syntax` and `nll` which is actually expected to be
standardized).

## Contributing

This is mostly a fun project for me. I don't actually think it's
useful for anyone else, nor is it very interesting to watch. The fun
of it is developing it and seeing what kinds of strategies the
creatures come up with after millions of generations.

So, since the goal isn't actually to create useful software, but to
enjoy the process, I don't really forsee a need to accept
contributions. That being said, if you want to fork this, be my guest,
and if you feel like contributing, feel free to open up an issue and
we can talk!
