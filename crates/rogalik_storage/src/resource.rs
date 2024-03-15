use std::{
    any::Any,
    cell::RefCell
};
#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use super::Storage;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ResourceCell<T: 'static> {
    pub inner: RefCell<T>
}
impl<T: 'static> Storage for ResourceCell<T> {
    fn as_any(&self) -> &dyn Any { self }
}