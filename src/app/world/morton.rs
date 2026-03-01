use super::point_cluster;

pub trait Morton {
    const BIT_COUNT_PER_COMPONENT: u16;
    const MAX_PER_COMPONENT: u16 = 1 << Self::BIT_COUNT_PER_COMPONENT;
    const BIT_MASK_PER_COMPONENT: u16 = Self::MAX_PER_COMPONENT - 1;
    const MAX: u16 = Self::MAX_PER_COMPONENT.pow(3);

    fn compute_coordinate(p_value: glam::IVec3) -> (glam::IVec3, glam::UVec3) {
        let origin = p_value.div_euclid(glam::IVec3::splat(Self::MAX_PER_COMPONENT as _));
        let slot_local_position = (p_value - origin * Self::MAX_PER_COMPONENT as i32).as_uvec3();
        (origin, slot_local_position)
    }

    fn compute_cluster(p_value: glam::IVec3) -> point_cluster::PointCluster {
        point_cluster::PointCluster {
            min: glam::IVec3::ZERO + p_value * Self::MAX_PER_COMPONENT as i32,
            max: glam::IVec3::splat(Self::MAX_PER_COMPONENT as _)
                + p_value * Self::MAX_PER_COMPONENT as i32,
        }
    }

    fn consume(p_value: glam::IVec3) -> (u16, glam::IVec3) {
        let (origin, slot_local_position) = Self::compute_coordinate(p_value);

        let mut x = slot_local_position.x as u16 & Self::BIT_MASK_PER_COMPONENT;
        let mut y = slot_local_position.y as u16 & Self::BIT_MASK_PER_COMPONENT;
        let mut z = slot_local_position.z as u16 & Self::BIT_MASK_PER_COMPONENT;

        let mut result = 0u16;
        for _ in 0..Self::BIT_COUNT_PER_COMPONENT {
            let current_x_bit = x & 1;
            x ^= current_x_bit;
            x >>= 1;
            let current_y_bit = y & 1;
            y ^= current_y_bit;
            y >>= 1;
            let current_z_bit = z & 1;
            z ^= current_z_bit;
            z >>= 1;

            result = (result << 3) | (current_x_bit | current_y_bit << 1 | current_z_bit << 2);
        }

        (result, origin)
    }
}

/// Morton 4 is morton code with 4 bits for each 3D axis.
pub struct Morton4;

impl Morton for Morton4 {
    const BIT_COUNT_PER_COMPONENT: u16 = 4;
}
