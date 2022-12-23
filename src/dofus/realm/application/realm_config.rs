pub struct RealmConfig {
    pub name: String,
    pub address: String,
    pub port: u16,
}

impl RealmConfig {
    pub fn new(name: String, address: String, port: u16) -> Self {
        RealmConfig {
            name,
            address,
            port,
        }
    }
}
