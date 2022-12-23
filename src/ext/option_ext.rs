pub trait OptionExt<T> {
    fn if_present<E: Fn(&T)>(&self, action: E);
}

impl<T> OptionExt<T> for Option<T> {
    fn if_present<E: Fn(&T)>(&self, action: E) {
        if let Some(value) = self {
            action(value);
        }
    }
}
