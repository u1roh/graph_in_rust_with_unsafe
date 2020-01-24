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
        pub fn free(&mut self, ptr: *const T) -> Option<Box<T>> {
            self.0.remove(&ptr)
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

    assert!(pool.free(p).is_some());
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
        pub fn is_sentinel(&self) -> bool {
            self.value.is_none()
        }
        pub fn next(&self) -> &Self {
            unsafe { std::mem::transmute(self.next) }
        }
        pub fn prev(&self) -> &Self {
            unsafe { std::mem::transmute(self.prev) }
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

    pub struct NodeMut<'a, T>(&'a mut Node<T>);
    impl<'a, T> Deref for NodeMut<'a, T> {
        type Target = T;
        fn deref(&self) -> &Self::Target {
            self.0.deref()
        }
    }
    impl<'a, T> DerefMut for NodeMut<'a, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.deref_mut()
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
        pub fn sentinel(&self) -> *const Node<T> {
            self.sentinel.deref() as *const _
        }
        pub fn get_ref(&self, ptr: *const Node<T>) -> Option<&Node<T>> {
            self.nodes.get(ptr)
        }

        // これは unsafe
        // ```
        //  let mut list1 = List::new();
        //  let mut list2 = List::new();
        //  ...
        //  let node1 = list1.get_mut_unchecked(ptr1).unwrap();
        //  let node2 = list2.get_mut_unchecked(ptr2).unwrap();
        //  std::mem::swap(node1, node2);    // 壊れる！
        // ```
        pub unsafe fn get_mut_unchecked(&mut self, ptr: *const Node<T>) -> Option<&mut Node<T>> {
            self.nodes.get_mut(ptr)
        }

        // こちらは安全に使える
        pub fn get_mut(&mut self, ptr: *const Node<T>) -> Option<NodeMut<T>> {
            self.nodes.get_mut(ptr).map(NodeMut)
        }

        pub fn head(&self) -> &Node<T> {
            self.sentinel.next()
        }
        pub fn tail(&self) -> &Node<T> {
            self.sentinel.prev()
        }
        pub fn is_empty(&self) -> bool {
            self.head().is_sentinel()
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
        pub fn remove(&mut self, node: *const Node<T>) -> Option<&Node<T>> {
            if let Some(node) = self.nodes.free(node) {
                let next = node.next as *mut Node<T>;
                let prev = node.prev as *mut Node<T>;
                unsafe {
                    (*next).prev = prev;
                    (*prev).next = next;
                    Some(&*next)
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
    assert_eq!(**list.head(), 1);
    assert_eq!(**list.tail(), 1);
    assert!(list.head().next().is_sentinel());
    assert!(list.head().prev().is_sentinel());

    list.push_back(2);
    assert_eq!(**list.head(), 1);
    assert_eq!(**list.tail(), 2);
    assert_eq!(**list.head().next(), 2);

    list.push_front(3);
    assert_eq!(**list.head(), 3);
    assert_eq!(**list.head().next(), 1);

    assert!(list.insert(list.head().next() as *const _, 4));
    assert_eq!(**list.head(), 3);
    assert_eq!(**list.head().next(), 4);
    assert_eq!(**list.head().next().next(), 1);
    assert_eq!(**list.head().next().next().next(), 2);

    assert!(list.remove(list.head() as *const _).is_some());
    assert_eq!(**list.head(), 4);

    let mut node = list.get_mut(list.head().next() as *const _).unwrap();
    *node = 5;
    assert_eq!(**list.head().next(), 5);

    unsafe {
        let mut list2: List<usize> = List::new();
        list2.push_back(6);
        let node1 = list.get_mut_unchecked(list.head() as *const _).unwrap();
        let node2 = list2.get_mut_unchecked(list2.head() as *const _).unwrap();
        std::mem::swap(node1, node2); // 壊れる！
    }

    /* not compilable
    let head = list.head();
    list.push_front(5);
    println!("{}", **head);
    */
}
