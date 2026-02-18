#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IAabb {
    pub min: glam::IVec3,
    pub max: glam::IVec3,
}

impl IAabb {
    pub fn new(p_min: glam::IVec3, p_max: glam::IVec3) -> Self {
        Self {
            min: p_min,
            max: p_max,
        }
    }

    pub fn merge_point(&mut self, p_point: glam::IVec3) {
        self.min = self.min.min(p_point);
        self.max = self.max.max(p_point);
    }

    pub fn intersects(&self, p_other: &IAabb) -> bool {
        self.min.cmple(p_other.max).all() && self.max.cmpge(p_other.min).all()
    }

    pub fn intersects_point(&self, p_other: glam::IVec3) -> bool {
        self.min.cmple(p_other).all() && self.max.cmpge(p_other).all()
    }

    pub fn iter_points(&self) -> Box<dyn Iterator<Item = glam::IVec3>> {
        Box::new(
            itertools::iproduct!(
                self.min.x..self.max.x,
                self.min.y..self.max.y,
                self.min.z..self.max.z
            )
            .map(|(x, y, z)| glam::IVec3::new(x, y, z)),
        )
    }
}
