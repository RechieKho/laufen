use oktree::prelude::*;

pub fn run_sample_block() -> Result<(), TreeError> {
    let aabb = Aabb::new(TUVec3::splat(16), 16u8);
    let mut tree = Octree::from_aabb_with_capacity(aabb?, 10);

    let c1 = SampleBlock::new(TUVec3::splat(1u8));
    let c2 = SampleBlock::new(TUVec3::splat(8u8));

    let c1_id = tree.insert(c1)?;
    let c2_id = tree.insert(c2)?;

    // Searching by position
    assert_eq!(tree.find(&TUVec3::new(1, 1, 1)), Some(c1_id));
    assert_eq!(tree.find(&TUVec3::new(8, 8, 8)), Some(c2_id));
    assert_eq!(tree.find(&TUVec3::new(1, 2, 8)), None);
    assert_eq!(tree.find(&TUVec3::splat(100)), None);

    assert_eq!(
        tree.intersect_with(|p_element| p_element.contains(&TUVec3::splat(1u8))),
        vec![c1_id]
    );

    Ok(())
}

struct SampleBlock {
    position: TUVec3<u8>,
}

impl Position for SampleBlock {
    type U = u8;
    fn position(&self) -> TUVec3<u8> {
        self.position
    }
}

impl SampleBlock {
    fn new(position: TUVec3<u8>) -> Self {
        SampleBlock { position }
    }
}
