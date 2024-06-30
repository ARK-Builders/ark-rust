use crate::base_storage::BaseStorage;
use std::cell::RefCell;
use std::collections::btree_map::Iter;
use std::rc::Rc;

pub struct BTreeMapIterator<'a, K, V> {
    iter: Rc<RefCell<Iter<'a, K, V>>>,
}

impl<'a, K, V> BTreeMapIterator<'a, K, V>
where
    K: Ord + Clone,
    V: Clone,
{
    pub fn new<S: BaseStorage<K, V>>(storage: &'a S) -> Self {
        BTreeMapIterator {
            iter: Rc::new(RefCell::new(storage.as_ref().iter())),
        }
    }

    pub fn has_next(&mut self) -> bool {
        let borrow = self.iter.borrow();
        borrow.clone().peekable().peek().is_some()
    }

    pub fn native_next(&mut self) -> Option<(K, V)> {
        let mut borrow = self.iter.borrow_mut();
        borrow.next().map(|(k, v)| (k.clone(), v.clone()))
    }
}
