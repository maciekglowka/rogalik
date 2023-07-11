use std::{
    any::Any,
    cell::RefCell
};

use super::Storage;

pub struct ResourceCell<T: 'static> {
    pub inner: RefCell<T>
}
impl<T: 'static> Storage for ResourceCell<T> {
    fn as_any(&self) -> &dyn Any { self }
}