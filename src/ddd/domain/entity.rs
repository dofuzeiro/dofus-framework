use std::hash::Hash;

pub trait Identifiable: Hash + Eq {}

impl<T> Identifiable for T where T: Hash + Eq {}

pub trait Entity<K: Identifiable> {
    fn id(&self) -> K;
}
