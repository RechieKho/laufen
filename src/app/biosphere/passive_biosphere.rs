use crate::adapter::shared;

pub type SharedPassiveBiosphere = shared::Shared<PassiveBiosphere>;

#[derive(Default, getset::Getters, getset::MutGetters)]
pub struct PassiveBiosphere {
    #[getset(get = "pub", get_mut = "pub")]
    entities: shipyard::World,
}
