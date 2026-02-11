use shipyard::IntoIter;

#[derive(shipyard::Component)]
struct SampleLabel(String);
#[derive(shipyard::Component)]
struct SampleHealth(u32);
#[derive(shipyard::Component)]
struct SamplePosition {
    x: f32,
    y: f32,
}

fn is_in_acid(p_position: &SamplePosition) -> bool {
    // it's wet season
    p_position.x == 0.0 && p_position.y == 0.0
}

fn in_acid(
    p_positions: shipyard::View<SamplePosition>,
    mut p_healths: shipyard::ViewMut<SampleHealth>,
) {
    for (_, health) in (&p_positions, &mut p_healths)
        .iter()
        .filter(|(pos, _)| is_in_acid(pos))
    {
        health.0 -= 1;
    }
}

fn print_health(p_labels: shipyard::View<SampleLabel>, p_healths: shipyard::View<SampleHealth>) {
    for (label, health) in (&p_labels, &p_healths).iter() {
        println!("{}'s Health is {}", label.0, health.0);
    }
}

pub fn run_sample_model() {
    let mut world = shipyard::World::new();

    world.add_entity((
        SamplePosition { x: 0.0, y: 0.0 },
        SampleHealth(1000),
        SampleLabel(String::from("Bob")),
    ));

    world.run(print_health);
    world.run(in_acid);
    world.run(print_health);
}
