use creatures::Creature;

#[derive(Show, PartialEq, Eq, Copy)]
pub enum FightStatus {
    End, Continue,
}

fn encounter(p1: &mut Creature, p2: &mut Creature) -> Vec<Creature> {
    panic!("ok")
}
