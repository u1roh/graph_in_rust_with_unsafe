mod list {
    use mepoo::{Pool, Ptr, Ref};
    use std::ops::{Deref, DerefMut};

    pub struct Node<T> {
        value: Option<T>,
        next: Ptr<Self>,
        prev: Ptr<Self>,
    }
    impl<T> Node<T> {
        fn next_node(&self) -> Ref<Self> {
            unsafe { self.next.as_ref() }.unwrap()
        }
        fn prev_node(&self) -> Ref<Self> {
            unsafe { self.prev.as_ref() }.unwrap()
        }
        fn to_option<'a>(node: Ref<'a, Self>) -> Option<Ref<'a, Self>> {
            if node.value.is_some() {
                Some(node)
            } else {
                None
            }
        }
        pub fn next(&self) -> Option<Ref<Self>> {
            Self::to_option(self.next_node())
        }
        pub fn prev(&self) -> Option<Ref<Self>> {
            Self::to_option(self.prev_node())
        }
    }
    impl<T> Deref for Node<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.value.as_ref().unwrap()
        }
    }
    impl<T> DerefMut for Node<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.value.as_mut().unwrap()
        }
    }

    pub struct List<T> {
        nodes: Pool<Node<T>>,
        sentinel: Ptr<Node<T>>,
    }
    impl<T> List<T> {
        pub fn new() -> Self {
            let mut nodes = Pool::new();
            let sentinel = nodes.insert(Node::<T> {
                value: None,
                next: Ptr::dangling(),
                prev: Ptr::dangling(),
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
        pub fn head(&self) -> Option<Ref<Node<T>>> {
            self.sentinel().get().next()
        }
        pub fn tail(&self) -> Option<Ref<Node<T>>> {
            self.sentinel().get().prev()
        }
        pub fn is_empty(&self) -> bool {
            self.head().is_none()
        }
        unsafe fn insert_unsafe(&mut self, next: Ptr<Node<T>>, value: T) {
            let prev = next.as_ref().unwrap().prev;
            let node = self.nodes.insert(Node {
                value: Some(value),
                next,
                prev,
            });
            next.as_mut().unwrap().prev = node;
            prev.as_mut().unwrap().next = node;
        }
        pub fn insert(&mut self, pos: Ptr<Node<T>>, value: T) -> bool {
            if self.nodes.get(&pos).is_some() {
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
        pub fn remove(&mut self, node: Ptr<Node<T>>) -> bool {
            if let Some(node_ref) = self.nodes.get(&node) {
                let next = node_ref.next;
                let prev = node_ref.prev;
                unsafe {
                    next.as_mut().unwrap().prev = prev;
                    prev.as_mut().unwrap().next = next;
                }
                self.nodes.remove(node)
            } else {
                false
            }
        }
    }
}

pub use list::*;
use mepoo::Ref;

// テストコードから unwrap() を減らして読みやすくするためのユーティリティ
impl<T> Node<T> {
    fn next_f(&self) -> Ref<Node<T>> {
        self.next().unwrap()
    }
}
impl<T> List<T> {
    fn head_f(&self) -> Ref<Node<T>> {
        self.head().unwrap()
    }
    fn tail_f(&self) -> Ref<Node<T>> {
        self.tail().unwrap()
    }
}

#[test]
fn test_list() {
    let mut list: List<usize> = List::new();
    assert!(list.head().is_none());
    assert!(list.tail().is_none());
    assert!(list.is_empty());

    list.push_back(1);
    assert!(list.head().is_some());
    assert!(list.tail().is_some());
    assert_eq!(**list.head_f(), 1);
    assert_eq!(**list.tail_f(), 1);
    assert!(list.head_f().next().is_none());
    assert!(list.head_f().prev().is_none());

    list.push_back(2);
    assert_eq!(**list.head_f(), 1);
    assert_eq!(**list.tail_f(), 2);
    assert_eq!(**list.head_f().next_f(), 2);

    list.push_front(3);
    assert_eq!(**list.head_f(), 3);
    assert_eq!(**list.head_f().next_f(), 1);

    assert!(list.insert(list.head_f().next_f().into(), 4));
    assert_eq!(**list.head_f(), 3);
    assert_eq!(**list.head_f().next_f(), 4);
    assert_eq!(**list.head_f().next_f().next_f(), 1);
    assert_eq!(**list.head_f().next_f().next_f().next_f(), 2);

    assert!(list.remove(list.head_f().into()));
    assert_eq!(**list.head_f(), 4);

    // not compilable
    /*
    let head = list.head_f();
    list.push_front(5);
    println!("{}", **head);
    */
}
