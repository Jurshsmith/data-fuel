use std::ptr::NonNull;

struct Node<V> {
    value: V,
    next: Option<NodePointer<V>>,
}

impl<V> Node<V> {
    fn new(value: V) -> Self {
        Self { value, next: None }
    }
}

pub struct NodePointer<V>(NonNull<Node<V>>);

unsafe impl<V> Send for NodePointer<V> {}
unsafe impl<V> Sync for NodePointer<V> {}

impl<V> From<NonNull<Node<V>>> for NodePointer<V> {
    fn from(non_null_ptr: NonNull<Node<V>>) -> Self {
        Self::from_non_null_ptr(non_null_ptr)
    }
}

impl<'a, V> From<&'a NodePointer<V>> for NodePointer<V> {
    fn from(value: &'a NodePointer<V>) -> Self {
        Self::from_non_null_ptr(value.as_non_null_ptr())
    }
}

impl<V> NodePointer<V> {
    pub fn new(value: V) -> Self {
        let node = Node::new(value);
        let non_null_ptr = unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(node))) };

        Self::from_non_null_ptr(non_null_ptr)
    }

    fn from_non_null_ptr(non_null_ptr: NonNull<Node<V>>) -> Self {
        Self(non_null_ptr)
    }
    fn as_non_null_ptr(&self) -> NonNull<Node<V>> {
        self.0
    }
    fn as_ptr(&self) -> *mut Node<V> {
        self.as_non_null_ptr().as_ptr()
    }

    pub fn set_next(&self, next: &Self) {
        unsafe { (*self.as_ptr()).next = Some(next.into()) }
    }
    fn get_next(&self) -> Option<&Self> {
        unsafe { (*self.as_ptr()).next.as_ref() }
    }

    pub fn get_value(&self) -> &V {
        unsafe { &(&*self.as_ptr()).value }
    }
    pub fn remove_value(&self) -> V {
        self.remove_node().value
    }
    fn remove_node(&self) -> Node<V> {
        unsafe { *Box::from_raw(self.as_ptr()) }
    }

    pub fn get_next_value_by_index(&self, index: usize) -> Option<&V> {
        self.iter().nth(index).map(|next| next.get_value())
    }
    pub fn iter(&self) -> NodePointerIter<'_, V> {
        NodePointerIter { next: Some(self) }
    }
}

pub struct NodePointerIter<'a, V> {
    next: Option<&'a NodePointer<V>>,
}

impl<'a, V> Iterator for NodePointerIter<'a, V> {
    type Item = &'a NodePointer<V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node_pointer| {
            self.next = node_pointer.get_next();
            node_pointer
        })
    }
}

pub struct List<V> {
    head: Option<NodePointer<V>>,
    tail: Option<NodePointer<V>>,
    len: usize,
}

impl<V> List<V> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn push(&mut self, value: V) {
        let new_tail = NodePointer::new(value);

        if let Some(old_tail) = &self.tail {
            old_tail.set_next(&new_tail);
        } else {
            self.head = Some((&new_tail).into());
        }

        self.tail = Some(new_tail);

        self.len += 1
    }

    pub fn pop(&mut self) -> Option<V> {
        if let Some(old_head) = &self.head {
            let new_head: Option<NodePointer<_>> = old_head.get_next().map(|h| h.into());
            let old_head_value = old_head.remove_value();

            self.head = new_head;
            self.len -= 1;

            Some(old_head_value)
        } else {
            None
        }
    }

    pub fn get(&mut self, index: usize) -> Option<&V> {
        self.head
            .as_ref()
            .and_then(|h| h.get_next_value_by_index(index))
    }

    pub fn iter(&self) -> ListIter<'_, V> {
        ListIter {
            next: self.head.as_ref(),
        }
    }

    pub fn last(&self) -> Option<&V> {
        self.tail.as_ref().map(|h| h.get_value())
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

impl<V> Drop for List<V> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

pub struct ListIter<'a, V> {
    next: Option<&'a NodePointer<V>>,
}

impl<'a, V> Iterator for ListIter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node_pointer| {
            self.next = node_pointer.get_next();
            node_pointer.get_value()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_appends_new_value_as_last_value() {
        let mut list = List::new();
        assert_eq!(list.last(), None);
        list.push(3);
        assert_eq!(list.last(), Some(&3));
    }

    #[test]
    fn push_appends_new_value_to_existing_values() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        list.push(4);
        assert_eq!(list.last(), Some(&4));
    }

    #[test]
    fn pop_returns_first_pushed_value() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        assert_eq!(list.pop(), Some(1));
    }

    #[test]
    fn exhausts_list_by_returning_pushed_values() {
        let mut list = List::new();
        list.push(40);
        list.push(600);
        list.push(800);
        list.push(2800);
        list.push(2900);
        list.push(333);
        assert_eq!(list.pop(), Some(40));
        assert_eq!(list.pop(), Some(600));
        assert_eq!(list.pop(), Some(800));
        assert_eq!(list.pop(), Some(2800));
        assert_eq!(list.pop(), Some(2900));
        assert_eq!(list.pop(), Some(333));
        assert_eq!(list.pop(), None);
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn get_returns_value_by_index() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        assert_eq!(list.get(0), Some(&1));
        assert_eq!(list.get(1), Some(&2));
        assert_eq!(list.get(2), Some(&3));
        list.push(4);
        list.push(20);
        assert_eq!(list.get(2), Some(&3));
        assert_eq!(list.get(3), Some(&4));
        assert_eq!(list.get(4), Some(&20));
    }

    #[test]
    fn get_returns_none_when_list_is_empty() {
        let mut list: List<u32> = List::new();
        assert_eq!(list.get(0), None);
        assert_eq!(list.get(1), None);
    }

    #[test]
    fn get_returns_none_when_index_does_not_have_value() {
        let mut list = List::new();
        list.push(1);
        list.push(3);
        assert_eq!(list.get(2), None);
        assert_eq!(list.get(3), None);
    }

    #[test]
    fn len_returns_zero_when_no_value_in_list() {
        let list: List<u32> = List::new();
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn len_returns_number_of_values_in_list() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        assert_eq!(list.len(), 3);
        list.push(4);
        list.push(5);
        assert_eq!(list.len(), 5);
        list.push(6);
        list.push(7);
        assert_eq!(list.len(), 7);
    }

    #[test]
    fn len_returns_number_of_repeating_values() {
        let mut list = List::new();
        list.push(1);
        list.push(1);
        assert_eq!(list.len(), 2);
        list.push(1);
        assert_eq!(list.len(), 3);
        list.push(2);
        list.push(2);
        assert_eq!(list.len(), 5);
        list.push(2);
        list.push(2);
        assert_eq!(list.len(), 7);
    }

    #[test]
    fn len_returns_number_of_unpopped_values() {
        let mut list = List::new();
        list.push(1);
        list.push(1);
        list.push(1);
        list.push(2);
        list.push(2);
        list.pop();
        list.pop();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn is_empty_returns_true_when_no_value_is_in_list() {
        let list: List<u32> = List::new();
        assert!(list.is_empty());
    }

    #[test]
    fn is_empty_returns_false_when_at_least_one_value_is_in_list() {
        let mut list = List::new();
        list.push(1);
        assert!(!list.is_empty());
    }

    #[test]
    fn iter_returns_exhaustive_iterator() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[tokio::test]
    async fn thread_safe_with_thread_safe_values() {
        let mut list = List::new();

        tokio::spawn(async move {
            list.push(1);
            list.push(2);
            list.push(3);

            let mut iter = list.iter();
            assert_eq!(iter.next(), Some(&1));
            assert_eq!(iter.next(), Some(&2));
            assert_eq!(iter.next(), Some(&3));
            assert_eq!(iter.next(), None);
        });
    }
}
