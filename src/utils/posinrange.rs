use glam::IVec3;

pub fn pos_in_range(pos: &IVec3, min: &IVec3, max: &IVec3) -> bool {
    pos.x >= min.x
        && pos.x <= max.x
        && pos.y >= min.y
        && pos.y <= max.y
        && pos.z >= min.z
        && pos.z <= max.z
}
