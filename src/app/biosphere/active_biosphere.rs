use crate::adapter::shared;

pub type SharedActiveBiosphere = shared::Shared<ActiveBiosphere>;

#[derive(Default, getset::Getters, getset::MutGetters)]
pub struct ActiveBiosphere {
    #[getset(get = "pub", get_mut = "pub")]
    entities: shipyard::World,
}
