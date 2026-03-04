pub mod player;

#[derive(Default, getset::Getters, getset::MutGetters)]
pub struct Biosphere {
    #[getset(get = "pub", get_mut = "pub")]
    entities: shipyard::World,
}
