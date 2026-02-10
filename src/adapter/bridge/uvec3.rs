pub struct UVec3(pub glam::u32::UVec3);

impl std::ops::Deref for UVec3 {
    type Target = glam::u32::UVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for UVec3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<glam::u32::UVec3> for UVec3 {
    fn from(p_value: glam::u32::UVec3) -> Self {
        Self(p_value)
    }
}

impl From<UVec3> for glam::u32::UVec3 {
    fn from(p_value: UVec3) -> Self {
        p_value.0
    }
}

impl From<oktree::bounding::TUVec3<u32>> for UVec3 {
    fn from(p_value: oktree::bounding::TUVec3<u32>) -> Self {
        Self(glam::u32::UVec3::new(p_value.x, p_value.y, p_value.z))
    }
}

impl From<UVec3> for oktree::bounding::TUVec3<u32> {
    fn from(p_value: UVec3) -> Self {
        Self {
            x: p_value.x,
            y: p_value.y,
            z: p_value.z,
        }
    }
}
