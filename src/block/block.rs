use crate::model::{ model::*, modelregistry::* };
use crate::block::blockstate::BlockState;

pub trait Block {
    fn name(&self) -> &str;
    fn is_solid(&self) -> bool;
    fn get_model(&self, state: &BlockState) -> BlockModel;
}

pub struct Air;

impl Block for Air {
    fn name(&self) -> &str { "Air" }
    fn is_solid(&self) -> bool { false }
    fn get_model(&self, _: &BlockState) -> BlockModel { air_model() }
}

pub struct Stone;

impl Block for Stone {
    fn name(&self) -> &str { "Stone" }
    fn is_solid(&self) -> bool { true }
    fn get_model(&self, _: &BlockState) -> BlockModel { stone_model() }
}

pub struct Dirt;

impl Block for Dirt {
    fn name(&self) -> &str { "Dirt" }
    fn is_solid(&self) -> bool { true }
    fn get_model(&self, _: &BlockState) -> BlockModel { stone_model() }
}

pub struct Grass;

impl Block for Grass {
    fn name(&self) -> &str { "Grass Block" }
    fn is_solid(&self) -> bool { true }
    fn get_model(&self, _: &BlockState) -> BlockModel { grass_model() }
}