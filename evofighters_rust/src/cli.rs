use clap;
use dna;
use sim;
use simplify;
use saver::SettingsBuilder;

pub fn parse_args() -> clap::ArgMatches<'static> {
    clap::App::new(
        r"   __             ___
  /              /    /      /    /
 (___       ___ (___    ___ (___ (___  ___  ___  ___
 |     \  )|   )|    | |   )|   )|    |___)|   )|___
 |__    \/ |__/ |    | |__/ |  / |__  |__  |     __/
                       __/ ",
    ).version("1.0")
        .author("Josh Kuhn <deontologician@gmail.com>")
        .about("Evolving fighting bots")
        .arg(
            clap::Arg::with_name("savefile")
                .short("f")
                .long("file")
                .default_value("evofighters.evo")
                .value_name("SAVEFILE")
                .help("Name of save file")
                .takes_value(true)
                .global(true),
        )
        .arg(
            clap::Arg::with_name("mutation_rate")
                .short("m")
                .long("mutation-rate")
                .value_name("MUTATION_RATE")
                .help("Chance of a new creature having a mutation")
                .takes_value(true)
                .global(true),
        )
        .arg(
            clap::Arg::with_name("max_population_size")
                .short("p")
                .long("max-population-size")
                .value_name("MAX_POP_SIZE")
                .help("Maximum population to allow")
                .takes_value(true)
                .global(true),
        )
        .arg(
            clap::Arg::with_name("metric_fps")
                .short("f")
                .long("fps")
                .value_name("FPS")
                .help("Framerate at which to emit metrics")
                .takes_value(true)
                .global(true),
        )
        .subcommand(
            clap::SubCommand::with_name("simulate")
                .about("Main command. Runs an evofighters simulation"),
        )
        .subcommand(
            clap::SubCommand::with_name("cycle-check")
                .about("Does a cycle detection on the given bases")
                .arg(
                    clap::Arg::with_name("bases")
                        .required(true)
                        .multiple(true)
                        .value_name("BASE"),
                ),
        )
        .get_matches()
}

pub fn execute_command(app: &clap::ArgMatches) {
    match app.subcommand() {
        ("cycle-check", Some(check)) => {
            cycle_check(check.values_of("bases").unwrap())
        }
        _ => run_simulation(app),
    }
}

pub fn run_simulation(app: &clap::ArgMatches) {
    let filename = app.value_of("savefile").unwrap();
    let mut sb = SettingsBuilder::default();
    if let Some(mr) = app.value_of("mutation_rate") {
        let mut_rate = mr.parse().unwrap();
        info!("Mutation rate set by user to {}", mut_rate);
        sb.mutation_rate(mut_rate);
    }
    if let Some(pop_size) = app.value_of("max_population_size") {
        let pop = pop_size.parse().unwrap();
        info!("Population size set by user to {}", pop);
        sb.max_population_size(pop);
    }
    if let Some(metric_fps) = app.value_of("metric_fps") {
        let fps = metric_fps.parse().unwrap();
        info!("FPS set by user to {}", fps);
        sb.metric_fps(fps);
    }
    sim::SingleThreadedSimulation::new(filename, sb).simulate();
}

pub fn cycle_check(bases: clap::Values) {
    let dna_args: dna::DNA = dna::DNA::from(
        bases
            .map(|x| x.parse().expect("Well that wasn't an integer"))
            .collect::<Vec<i8>>(),
    );
    match simplify::cycle_detect(&dna_args) {
        Ok(_thought_cycle) => println!("Got a cycle!"),
        Err(failure) => println!("Failed to get a cycle: {:?}", failure),
    }
}
