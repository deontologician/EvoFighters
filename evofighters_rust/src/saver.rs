use serde_json;

use creatures::Creature;

#[derive(Debug,Deserialize,Serialize)]
struct SaveFile {
    max_thinking_steps: usize,
    max_tree_depth: usize,
    max_inv_size: usize,
    default_energy: usize,
    mating_cost: usize,
    mutation_rate: f64,
    max_gene_value: i8,
    winner_life_bonus: usize,
    max_population_size: usize,
    gene_min_size: usize,
    num_encounters: usize,
    feeder_count: usize,
    creatures: Vec<Creature>,
}

impl SaveFile {
    fn new(creatures: &[Creature],
           feeder_count: usize,
           num_encounters: usize) -> SaveFile {
        SaveFile {
            max_thinking_steps: settings::MAX_THINKING_STEPS,
            max_tree_depth: settings::MAX_TREE_DEPTH,
            max_inv_size: settings::MAX_INV_SIZE,
            default_energy: settings::DEFAULT_ENERGY,
            mating_cost: settings::MATING_COST,
            mutation_rate: settings::MUTATION_RATE,
            max_gene_value: settings::MAX_GENE_VALUE,
            winner_life_bonus: settings::WINNER_LIFE_BONUS,
            max_population_size: settings::MAX_POPULATION_SIZE,
            gene_min_size: settings::GENE_MIN_SIZE,
            num_encounters: num_encounters,
            feeder_count: feeder_count,
            creatures: creatures.to_owned(),
        }
    }

    fn save(&self, filename: &str) {
        let encoded = serde_json::to_string(&savefile);
        let mut save_file = File::create("evofighters.save").unwrap();
        save_file.write_all(encoded.as_ref()).unwrap();
    }
}
