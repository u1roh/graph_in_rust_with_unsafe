#![cfg(test)]
use std::ops::DerefMut;

mod list {
    use super::*;

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
        nodes: Vec<Box<Node<T>>>,
        head: *mut Node<T>,
        tail: *mut Node<T>,
    }
    impl<T> List<T> {
        pub fn new() -> Self {
            Self {
                nodes: Vec::new(),
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
            let mut node = Box::new(Node {
                value,
                next: std::ptr::null_mut(),
            });
            let ptr = node.deref_mut() as *mut Node<T>;
            if self.is_empty() {
                self.head = ptr;
                self.tail = ptr;
            } else {
                unsafe { (*self.tail).next = ptr };
                self.tail = ptr;
            }
            self.nodes.push(node);
        }
    }
}

use list::*;

#[test]
fn test() {
    let mut list: List<usize> = List::new();
    assert!(list.head().is_none());
    assert!(list.tail().is_none());
    assert!(list.is_empty());

    list.push_back(123);
    assert!(list.head().is_some());
    assert!(list.tail().is_some());
    assert_eq!(list.head().unwrap().value, 123);
    assert_eq!(list.tail().unwrap().value, 123);
    assert!(list.head().unwrap().next().is_none());

    list.push_back(456);
    assert_eq!(list.head().unwrap().value, 123);
    assert_eq!(list.tail().unwrap().value, 456);
    assert_eq!(list.head().unwrap().next().unwrap().value, 456);
}
