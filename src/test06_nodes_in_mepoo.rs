mod list {
    use mepoo::{Pool, Ptr, Ref};
    use std::ops::{Deref, DerefMut};

    pub struct Node<T> {
        value: Option<T>,
        next: Ptr<Self>,
        prev: Ptr<Self>,
    }
    impl<T> Node<T> {
        pub fn is_sentinel(&self) -> bool {
            self.value.is_none()
        }
        pub fn next(&self) -> Ref<Self> {
            unsafe { self.next.as_ref() }.unwrap()
        }
        pub fn prev(&self) -> Ref<Self> {
            unsafe { self.prev.as_ref() }.unwrap()
        }
        pub fn value(&self) -> &T {
            assert!(!self.is_sentinel());
            self.value.as_ref().unwrap()
        }
    }

    pub struct NodeMut<'a, T>(&'a mut Node<T>);
    impl<'a, T> Deref for NodeMut<'a, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.0.value()
        }
    }
    impl<'a, T> DerefMut for NodeMut<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.value.as_mut().unwrap()
        }
    }

    pub struct List<T> {
        nodes: Pool<Node<T>>,
        sentinel: Ptr<Node<T>>,
    }
    impl<T> List<T> {
        pub fn new() -> Self {
            let mut nodes = Pool::new();
            let sentinel = nodes.alloc(Node::<T> {
                value: None,
                next: Ptr::DANGLING,
                prev: Ptr::DANGLING,
            });
            unsafe {
                let sentinel_mut = sentinel.as_mut().unwrap();
                sentinel_mut.next = sentinel;
                sentinel_mut.prev = sentinel;
            }
            Self { nodes, sentinel }
        }
        fn sentinel(&self) -> Ref<Node<T>> {
            unsafe { self.sentinel.as_ref() }.unwrap()
        }
        pub fn get_ref(&self, ptr: Ptr<Node<T>>) -> Option<Ref<Node<T>>> {
            self.nodes.get(ptr)
        }
        pub unsafe fn get_mut_unchecked(&mut self, ptr: Ptr<Node<T>>) -> Option<&mut Node<T>> {
            self.nodes.get_mut(ptr)
        }
        pub fn get_mut(&mut self, ptr: Ptr<Node<T>>) -> Option<NodeMut<T>> {
            self.nodes.get_mut(ptr).map(NodeMut)
        }
        pub fn head(&self) -> Ref<Node<T>> {
            self.sentinel().get().next()
        }
        pub fn tail(&self) -> Ref<Node<T>> {
            self.sentinel().get().prev()
        }
        pub fn is_empty(&self) -> bool {
            self.head().is_sentinel()
        }
        unsafe fn insert_unsafe(&mut self, next: Ptr<Node<T>>, value: T) {
            let prev = next.as_ref().unwrap().prev;
            let node = self.nodes.alloc(Node {
                value: Some(value),
                next,
                prev,
            });
            next.as_mut().unwrap().prev = node;
            prev.as_mut().unwrap().next = node;
        }
        pub fn insert(&mut self, pos: Ptr<Node<T>>, value: T) -> bool {
            if self.nodes.get(pos).is_some() {
                unsafe { self.insert_unsafe(pos, value) }
                true
            } else {
                false
            }
        }
        pub fn push_back(&mut self, value: T) {
            unsafe { self.insert_unsafe(self.sentinel, value) }
        }
        pub fn push_front(&mut self, value: T) {
            unsafe {
                let next = self.sentinel.as_ref().unwrap().next;
                self.insert_unsafe(next, value)
            }
        }
        pub fn remove(&mut self, node: Ptr<Node<T>>) -> Option<Ref<Node<T>>> {
            if let Some(node_ref) = self.nodes.get(node) {
                let next = node_ref.next;
                let prev = node_ref.prev;
                assert!(self.nodes.free(node));
                unsafe {
                    next.as_mut().unwrap().prev = prev;
                    prev.as_mut().unwrap().next = next;
                    next.as_ref()
                }
            } else {
                None
            }
        }
    }
}

pub use list::*;

#[test]
fn test_list() {
    let mut list: List<usize> = List::new();
    assert!(list.head().is_sentinel());
    assert!(list.tail().is_sentinel());
    assert!(list.is_empty());

    list.push_back(1);
    assert_eq!(*list.head().value(), 1);
    assert_eq!(*list.tail().value(), 1);
    assert!(list.head().next().is_sentinel());
    assert!(list.head().prev().is_sentinel());

    list.push_back(2);
    assert_eq!(*list.head().value(), 1);
    assert_eq!(*list.tail().value(), 2);
    assert_eq!(*list.head().next().value(), 2);

    list.push_front(3);
    assert_eq!(*list.head().value(), 3);
    assert_eq!(*list.head().next().value(), 1);

    assert!(list.insert(list.head().next().into(), 4));
    assert_eq!(*list.head().value(), 3);
    assert_eq!(*list.head().next().value(), 4);
    assert_eq!(*list.head().next().next().value(), 1);
    assert_eq!(*list.head().next().next().next().value(), 2);

    assert!(list.remove(list.head().into()).is_some());
    assert_eq!(*list.head().value(), 4);

    let mut node = list.get_mut(list.head().next().into()).unwrap();
    *node = 5;
    assert_eq!(*list.head().next().value(), 5);

    unsafe {
        let mut list2: List<usize> = List::new();
        list2.push_back(6);
        let node1 = list.get_mut_unchecked(list.head().into()).unwrap();
        let node2 = list2.get_mut_unchecked(list2.head().into()).unwrap();
        std::mem::swap(node1, node2); // 壊れる！
    }

    // not compilable
    /*
    let head = list.head_f();
    list.push_front(5);
    println!("{}", **head);
    */
}
