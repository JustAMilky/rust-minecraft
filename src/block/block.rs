use crate::model::model::*;
use crate::block::blockstate::BlockState;

pub trait Block {
    fn get_model(&self, state: &BlockState) -> ModelId;
}