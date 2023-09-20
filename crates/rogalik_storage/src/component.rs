pub trait Component {
    fn as_str(&self) -> String { String::new() }
}

impl<T: Component> Component for &'_ T {
    fn as_str(&self) -> String {
        (**self).as_str()
    }
}

impl<T: Component> Component for &'_ mut T {
    fn as_str(&self) -> String {
        (**self).as_str()
    }
}