#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointCluster {
    pub min: glam::IVec3,
    pub max: glam::IVec3,
}

impl IntoIterator for PointCluster {
    type Item = glam::IVec3;
    type IntoIter = Box<dyn Iterator<Item = Self::Item>>;

    fn into_iter(self) -> Self::IntoIter {
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

impl From<parry3d::bounding_volume::Aabb> for PointCluster {
    fn from(p_value: parry3d::bounding_volume::Aabb) -> Self {
        Self {
            min: glam::IVec3::new(
                p_value.mins.x.ceil() as _,
                p_value.mins.y.ceil() as _,
                p_value.mins.z.ceil() as _,
            ),
            max: glam::IVec3::new(
                p_value.maxs.x.ceil() as _,
                p_value.maxs.y.ceil() as _,
                p_value.maxs.z.ceil() as _,
            ),
        }
    }
}
