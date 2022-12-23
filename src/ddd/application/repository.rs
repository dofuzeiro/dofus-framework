use crate::ddd::domain::entity::{Entity, Identifiable};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("Entity with key: {0} already exists")]
    DuplicatedEntity(String),
}

pub trait Repository<K: Identifiable, E: Entity<K>> {
    fn save(&mut self, entity: E);
    fn get_by_id(&self, key: &K) -> Option<&E>;
    fn get_all(&self) -> Vec<&E>;
    fn delete(&mut self, key: &K) -> Option<E>;
}
