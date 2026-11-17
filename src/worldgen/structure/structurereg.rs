use glam::IVec3;

use crate::{blockdata::blockstate::BlockState, worldgen::structure::structure::Structure};

pub fn tree(trunk_height: i32, pos: IVec3) -> Option<Structure> {
    if trunk_height < 1 || trunk_height > 3 {
        None
    } else {
        let mut tree = Structure::new(
            IVec3::new(-2, 0, -2),
            IVec3::new(2, trunk_height + 4, 2),
            IVec3::new(0, 0, 0),
        );

        for x in -2..=2 {
            for y in (trunk_height + 1)..(trunk_height + 3) {
                for z in -2..=2 {
                    tree.set_block(IVec3::new(x, y, z), BlockState::Leaves);
                }
            }
        }

        for x in -1..=1 {
            for z in -1..=1 {
                tree.set_block(IVec3::new(x, trunk_height + 3, z), BlockState::Leaves);
            }
        }

        for x in -1..=1 {
            tree.set_block(IVec3::new(x, trunk_height + 4, 0), BlockState::Leaves);
        }
        for z in -1..=1 {
            tree.set_block(IVec3::new(0, trunk_height + 4, z), BlockState::Leaves);
        }

        for y in 0..(trunk_height + 2) {
            tree.set_block(IVec3::new(0, y, 0), BlockState::Log);
        }

        tree.pos = pos;

        Some(tree)
    }
}
