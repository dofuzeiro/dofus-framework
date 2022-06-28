pub struct Person {
    age: u8,
    name: String
}

impl Person {

    pub fn default() -> Self {
        Person {
            age: 18,
            name: String::from("John Doe")
        }
    }
}