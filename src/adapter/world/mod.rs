use voxelis::{
    spatial::{VoxOpsBatch, VoxOpsRead, VoxOpsWrite, VoxTree},
    MaxDepth, VoxInterner,
};

pub fn run_sample_block() -> anyhow::Result<()> {
    let mut interner = VoxInterner::<u8>::with_memory_budget(256 * 1024 * 1024);
    let mut tree = VoxTree::new(MaxDepth::new(5)); // 32³ voxels (chunk)

    let mut batch = tree.create_batch();
    batch.set(&mut interner, glam::IVec3::new(3, 0, 4), 1); // stone

    tree.apply_batch(&mut interner, &batch);
    assert_eq!(tree.get(&interner, glam::IVec3::new(3, 0, 4)), Some(1));

    Ok(())
}
