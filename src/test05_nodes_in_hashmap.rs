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

    pub struct Node<T> {
        pub value: T,
        next: *mut Self,
    }
    impl<T> Node<T> {
        pub fn next(&self) -> Option<&Self> {
            unsafe { std::mem::transmute(self.next) }
        }
    }

    pub struct List<T> {
        nodes: Pool<Node<T>>,
        head: *mut Node<T>,
        tail: *mut Node<T>,
    }
    impl<T> List<T> {
        pub fn new() -> Self {
            Self {
                nodes: Pool::new(),
                head: std::ptr::null_mut(),
                tail: std::ptr::null_mut(),
            }
        }
        pub fn head(&self) -> Option<&Node<T>> {
            unsafe { std::mem::transmute(self.head) }
        }
        pub fn tail(&self) -> Option<&Node<T>> {
            unsafe { std::mem::transmute(self.tail) }
        }
        pub fn is_empty(&self) -> bool {
            self.head.is_null()
        }
        pub fn push_back(&mut self, value: T) {
            let ptr = self.nodes.alloc(Node {
                value,
                next: std::ptr::null_mut(),
            }) as *mut Node<T>;
            if self.is_empty() {
                self.head = ptr;
                self.tail = ptr;
            } else {
                unsafe { (*self.tail).next = ptr };
                self.tail = ptr;
            }
        }
        pub fn insert(&mut self, pos: *const Node<T>, value: T) -> bool {
            if let Some(node) = self.nodes.get_mut(pos).map(|node| node as *mut Node<T>) {
                unsafe {
                    (*node).next = self.nodes.alloc(Node {
                        value,
                        next: (*node).next,
                    }) as *mut Node<T>;
                }
                true
            } else {
                false
            }
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
fn test() {
    let mut list: List<usize> = List::new();
    assert!(list.head().is_none());
    assert!(list.tail().is_none());
    assert!(list.is_empty());

    list.push_back(1);
    assert!(list.head().is_some());
    assert!(list.tail().is_some());
    assert_eq!(list.head_f().value, 1);
    assert_eq!(list.tail_f().value, 1);
    assert!(list.head_f().next().is_none());

    list.push_back(2);
    assert_eq!(list.head_f().value, 1);
    assert_eq!(list.tail_f().value, 2);
    assert_eq!(list.head_f().next_f().value, 2);

    assert!(list.insert(list.head_f() as *const Node<usize>, 3));
    assert_eq!(list.head_f().value, 1);
    assert_eq!(list.head_f().next_f().value, 3);
    assert_eq!(list.head_f().next_f().next_f().value, 2);
}