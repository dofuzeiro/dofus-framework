use crate::ddd::application::repository::Repository;
use crate::ddd::domain::entity::{Entity, Identifiable};
use std::collections::HashMap;

pub struct InMemoryRepository<K: Identifiable, E: Entity<K>> {
    entities: HashMap<K, E>,
}

impl<K: Identifiable, E: Entity<K>> Repository<K, E> for InMemoryRepository<K, E> {
    fn save(&mut self, entity: E) {
        self.entities.insert(entity.id(), entity);
    }

    fn get_by_id(&self, key: &K) -> Option<&E> {
        self.entities.get(key)
    }

    fn get_all(&self) -> Vec<&E> {
        self.entities.values().collect()
    }

    fn delete(&mut self, key: &K) -> Option<E> {
        self.entities.remove(key)
    }
}
