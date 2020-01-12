#![cfg(test)]

mod pool {
    use std::collections::HashMap;
    use std::ops::{Deref, DerefMut};

    pub struct Pool<T>(HashMap<*const T, Box<T>>);
    impl<T> Pool<T> {
        pub fn new() -> Self {
            Self(HashMap::new())
        }
        pub fn alloc(&mut self, x: T) -> &mut T {
            let obj = Box::new(x);
            self.0.entry(obj.deref() as *const T).or_insert(obj)
        }
        pub fn free(&mut self, ptr: *const T) -> bool {
            self.0.remove(&ptr).is_some()
        }
        pub fn get(&self, ptr: *const T) -> Option<&T> {
            self.0.get(&ptr).map(Deref::deref)
        }
        pub fn get_mut(&mut self, ptr: *const T) -> Option<&mut T> {
            self.0.get_mut(&ptr).map(DerefMut::deref_mut)
        }
    }
}

#[test]
fn test_pool() {
    use pool::Pool;
    let mut pool = Pool::new();
    let r: &usize = pool.alloc(123);
    assert_eq!(*r, 123);

    let p = r as *const usize;
    assert!(pool.get(p).is_some());
    assert_eq!(pool.get(p).unwrap(), &123);

    *pool.get_mut(p).unwrap() = 456;
    assert_eq!(pool.get(p).unwrap(), &456);

    assert!(pool.free(p));
    assert!(pool.get(p).is_none());
}

mod list {
    use super::pool::Pool;
    use std::ops::{Deref, DerefMut};

    pub struct Node<T> {
        value: Option<T>,
        next: *mut Self,
        prev: *mut Self,
    }
    impl<T> Node<T> {
        fn to_option(&self) -> Option<&Self> {
            if self.value.is_some() {
                Some(self)
            } else {
                None
            }
        }
        pub fn next(&self) -> Option<&Self> {
            Self::to_option(unsafe { std::mem::transmute(self.next) })
        }
        pub fn prev(&self) -> Option<&Self> {
            Self::to_option(unsafe { std::mem::transmute(self.prev) })
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
        sentinel: Box<Node<T>>,
    }
    impl<T> List<T> {
        pub fn new() -> Self {
            let mut sentinel = Box::new(Node::<T> {
                value: None,
                next: std::ptr::null_mut(),
                prev: std::ptr::null_mut(),
            });
            sentinel.next = sentinel.deref_mut() as *mut Node<T>;
            sentinel.prev = sentinel.deref_mut() as *mut Node<T>;
            Self {
                nodes: Pool::new(),
                sentinel,
            }
        }
        pub fn head(&self) -> Option<&Node<T>> {
            Node::to_option(unsafe { std::mem::transmute(self.sentinel.next) })
        }
        pub fn tail(&self) -> Option<&Node<T>> {
            Node::to_option(unsafe { std::mem::transmute(self.sentinel.prev) })
        }
        pub fn is_empty(&self) -> bool {
            self.head().is_none()
        }
        unsafe fn insert_unsafe(&mut self, next: *mut Node<T>, value: T) {
            let prev: *mut Node<T> = (*next).prev;
            let node: *mut Node<T> = self.nodes.alloc(Node {
                value: Some(value),
                next,
                prev,
            }) as *mut Node<T>;
            (*next).prev = node;
            (*prev).next = node;
        }
        pub fn insert(&mut self, pos: *const Node<T>, value: T) -> bool {
            if let Some(next) = self.nodes.get_mut(pos) {
                let next = next as *mut Node<T>;
                unsafe { self.insert_unsafe(next, value) }
                true
            } else {
                false
            }
        }
        pub fn push_back(&mut self, value: T) {
            let next = self.sentinel.deref_mut() as *mut Node<T>;
            unsafe { self.insert_unsafe(next, value) }
        }
        pub fn push_front(&mut self, value: T) {
            let next = self.sentinel.next;
            unsafe { self.insert_unsafe(next, value) }
        }
    }
}

use list::*;

// テストコードから unwrap() を減らして読みやすくするためのユーティリティ
impl<T> Node<T> {
    fn next_f(&self) -> &Node<T> {
        self.next().unwrap()
    }
}
impl<T> List<T> {
    fn head_f(&self) -> &Node<T> {
        self.head().unwrap()
    }
    fn tail_f(&self) -> &Node<T> {
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

    assert!(list.insert(list.head_f().next_f() as *const _, 4));
    assert_eq!(**list.head_f(), 3);
    assert_eq!(**list.head_f().next_f(), 4);
    assert_eq!(**list.head_f().next_f().next_f(), 1);
    assert_eq!(**list.head_f().next_f().next_f().next_f(), 2);

    let head = list.head_f();
    //list.push_front(5);
    println!("{}", **head);
}
