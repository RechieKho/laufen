pub trait Morton {
    const BIT_COUNT_PER_COMPONENT: u16;
    const MAX_PER_COMPONENT: u16 = 1 << Self::BIT_COUNT_PER_COMPONENT;
    const MAX: u16 = Self::MAX_PER_COMPONENT.pow(3);

    fn pick(p_value: i32) -> (u16, i32) {
        if Self::BIT_COUNT_PER_COMPONENT == 0 {
            return (0, p_value);
        }

        let signum = p_value.signum();
        let p_value = p_value.unsigned_abs();
        let bit_mask = (1 << Self::BIT_COUNT_PER_COMPONENT) - 1;
        let picked = p_value & bit_mask;
        let remained = (p_value ^ picked >> Self::BIT_COUNT_PER_COMPONENT) as i32 * signum;

        (picked as u16, remained)
    }

    fn consume(p_value: glam::IVec3) -> (u16, glam::IVec3) {
        let (mut x, x_remained) = Self::pick(p_value.x);
        let (mut y, y_remained) = Self::pick(p_value.y);
        let (mut z, z_remained) = Self::pick(p_value.z);

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

        (result, glam::IVec3::new(x_remained, y_remained, z_remained))
    }
}

/// Morton 4 is morton code with 4 bits for each 3D axis.
pub struct Morton4;

impl Morton for Morton4 {
    const BIT_COUNT_PER_COMPONENT: u16 = 4;
}
