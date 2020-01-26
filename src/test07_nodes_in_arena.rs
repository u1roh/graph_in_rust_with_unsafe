mod without_pointer {
    use std::cell::Cell;
    use typed_arena::Arena;

    struct Node<'a, T> {
        pub value: T,
        other: Cell<Option<&'a Self>>,
    }

    #[test]
    fn test1() {
        let nodes: Arena<Node<usize>> = Arena::new();
        let node1 = nodes.alloc(Node {
            value: 123,
            other: Cell::new(None),
        });
        let node2 = nodes.alloc(Node {
            value: 456,
            other: Cell::new(None),
        });
        node2.other.set(Some(node1));
        node1.other.set(Some(node2));
        let node3 = nodes.alloc(Node {
            value: 456,
            other: Cell::new(None),
        });
        node3.other.set(Some(node1));
    }

    struct Graph<'a, T> {
        nodes: Arena<Node<'a, T>>,
        head: &'a Node<'a, T>,
    }

    /* compilation error
    pub fn construct_graph<'a>() -> Graph<'a, usize> {
        let nodes: Arena<Node<usize>> = Arena::new();
        let node1 = nodes.alloc(Node {
            value: 123,
            other: Cell::new(None),
        });
        let node2 = nodes.alloc(Node {
            value: 456,
            other: Cell::new(None),
        });
        node2.other.set(Some(node1));
        node1.other.set(Some(node2));
        Graph { nodes, head: node1 }    // ERROR!
    }
    */
}

mod with_pointer {
    use typed_arena::Arena;

    pub struct Node<T> {
        pub value: T,
        other: *mut Self,
    }

    pub struct Graph<T> {
        nodes: Arena<Node<T>>,
        head: *mut Node<T>,
    }

    pub struct NodeRefMut<'a, T>(&'a mut Node<T>);
    impl<'a, T> NodeRefMut<'a, T> {
        pub fn value(&self) -> &T {
            &self.0.value
        }
        pub fn value_mut(&mut self) -> &mut T {
            &mut self.0.value
        }
        pub fn other(&mut self) -> Self {
            unsafe { Self(&mut *self.0.other) }
        }
    }

    pub fn construct_graph() -> Graph<usize> {
        let nodes: Arena<Node<usize>> = Arena::new();
        let node1 = nodes.alloc(Node {
            value: 123,
            other: std::ptr::null_mut(),
        });
        let node2 = nodes.alloc(Node {
            value: 456,
            other: std::ptr::null_mut(),
        });
        node2.other = node1 as *mut Node<usize>;
        node1.other = node2 as *mut Node<usize>;
        let head = node1 as *mut Node<usize>;
        Graph { nodes, head }
    }
    impl<T> Node<T> {
        pub fn other(&self) -> &Self {
            //unsafe { std::mem::transmute(self.other) }
            unsafe { &*self.other }
        }
    }
    impl<T> Graph<T> {
        pub fn head(&self) -> &Node<T> {
            //unsafe { std::mem::transmute(self.head) }
            unsafe { &*self.head }
        }
        pub fn head_mut(&mut self) -> NodeRefMut<T> {
            unsafe { NodeRefMut(&mut *self.head) }
        }
    }
}

#[test]
fn test_with_pointer() {
    use with_pointer::*;
    let graph = construct_graph();
    assert_eq!(graph.head().value, graph.head().other().other().value);

    let mut graph1 = construct_graph();
    let mut graph2 = construct_graph();
    //std::mem::swap(graph1.head_mut(), graph2.head_mut());
    *graph1.head_mut().value_mut() = 111;
    *graph2.head_mut().value_mut() = 222;
}

pub mod pool {
    use std::ptr::NonNull;
    use typed_arena::Arena;

    mod id {
        use lazy_static::lazy_static;
        use std::sync::atomic::{AtomicUsize, Ordering};

        lazy_static! {
            static ref COUNTER: AtomicUsize = AtomicUsize::new(1);
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct PoolId(usize);
        impl PoolId {
            pub(crate) fn gen() -> Self {
                Self(COUNTER.fetch_add(1, Ordering::Relaxed))
            }
            pub(crate) const ZERO: Self = Self(0);
        }
    }

    pub use id::PoolId;

    #[derive(Debug, PartialEq, Eq, Hash)]
    pub struct Ptr<T> {
        ptr: NonNull<T>,
        pool_id: PoolId,
    }
    impl<T> std::ops::Deref for Ptr<T> {
        type Target = NonNull<T>;
        fn deref(&self) -> &Self::Target {
            &self.ptr
        }
    }
    impl<T> std::ops::DerefMut for Ptr<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.ptr
        }
    }
    impl<T> Clone for Ptr<T> {
        fn clone(&self) -> Self {
            Self {
                ptr: self.ptr,
                pool_id: self.pool_id,
            }
        }
    }
    impl<T> Copy for Ptr<T> {}
    impl<T> Ptr<T> {
        pub const DANGLING: Self = Self {
            ptr: NonNull::dangling(),
            pool_id: PoolId::ZERO,
        };
        pub unsafe fn as_ref<'a>(&self) -> Ref<'a, T> {
            Ref {
                value: &*self.ptr.as_ptr(),
                pool_id: self.pool_id,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Ref<'a, T> {
        value: &'a T,
        pool_id: PoolId,
    }
    impl<'a, T> Ref<'a, T> {
        pub fn get(&self) -> &'a T {
            self.value
        }
    }
    impl<'a, T> std::ops::Deref for Ref<'a, T> {
        type Target = T;
        fn deref(&self) -> &T {
            &self.value
        }
    }
    impl<'a, T> From<Ref<'a, T>> for Ptr<T> {
        fn from(src: Ref<'a, T>) -> Self {
            Self {
                ptr: src.value.into(),
                pool_id: src.pool_id,
            }
        }
    }

    pub struct Pool<T> {
        arena: Arena<T>,
        id: PoolId,
    }
    impl<T> Pool<T> {
        pub fn new() -> Self {
            Self {
                arena: Arena::new(),
                id: PoolId::gen(),
            }
        }
        pub fn alloc(&mut self, value: T) -> Ptr<T> {
            Ptr {
                ptr: self.arena.alloc(value).into(),
                pool_id: self.id,
            }
        }
        pub fn get(&self, p: Ptr<T>) -> Ref<T> {
            assert_eq!(p.pool_id, self.id);
            unsafe { p.as_ref() }
        }
        pub fn get_mut(&mut self, p: Ptr<T>) -> &mut T {
            assert_eq!(p.pool_id, self.id);
            unsafe { &mut *p.ptr.as_ptr() }
        }
    }
}

pub mod list {
    use super::pool::*;
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
            unsafe { self.next.as_ref() }
        }
        pub fn prev(&self) -> Ref<Self> {
            unsafe { self.prev.as_ref() }
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
            let mut sentinel = nodes.alloc(Node {
                value: None,
                next: Ptr::DANGLING,
                prev: Ptr::DANGLING,
            });
            unsafe {
                sentinel.as_mut().next = sentinel;
                sentinel.as_mut().prev = sentinel;
            }
            Self { nodes, sentinel }
        }
        pub fn sentinel(&self) -> Ptr<Node<T>> {
            self.sentinel
        }
        pub fn head(&self) -> Ref<Node<T>> {
            unsafe { self.sentinel.as_ref().get().next() }
        }
        pub fn tail(&self) -> Ref<Node<T>> {
            unsafe { self.sentinel.as_ref().get().prev() }
        }
        pub fn is_empty(&self) -> bool {
            self.head().is_sentinel()
        }
        pub fn get(&self, p: Ptr<Node<T>>) -> Option<Ref<Node<T>>> {
            let r = self.nodes.get(p);
            if r.value.is_some() {
                Some(r)
            } else {
                None
            }
        }
        unsafe fn insert_unsafe(&mut self, mut next: Ptr<Node<T>>, value: T) {
            let mut prev = self.nodes.get(next).prev;
            let node = self.nodes.alloc(Node {
                value: Some(value),
                next,
                prev,
            });
            unsafe {
                next.as_mut().prev = node;
                prev.as_mut().next = node;
            }
        }
        pub fn insert(&mut self, pos: Ptr<Node<T>>, value: T) -> bool {
            if self.get(pos).is_some() {
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
            unsafe { self.insert_unsafe(self.sentinel.as_ref().next, value) }
        }
        pub fn remove(&mut self, node: Ptr<Node<T>>) -> Option<Ref<Node<T>>> {
            if let Some(node_ref) = self.get(node) {
                let mut next = node_ref.next;
                let mut prev = node_ref.prev;
                unsafe {
                    next.as_mut().prev = prev;
                    prev.as_mut().next = next;
                    Some(next.as_ref())
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

    list.insert(list.sentinel(), 4);
    assert!(list.insert(list.head().next().into(), 4));
    assert_eq!(*list.head().value(), 3);
    assert_eq!(*list.head().next().value(), 4);
    assert_eq!(*list.head().next().next().value(), 1);
    assert_eq!(*list.head().next().next().next().value(), 2);

    assert!(list.remove(list.head().into()).is_some());
    assert_eq!(*list.head().value(), 4);

    /*
    let mut node = list.get_mut(list.head().next() as *const _).unwrap();
    *node = 5;
    assert_eq!(*list.head().next().value(), 5);

    unsafe {
        let mut list2: List<usize> = List::new();
        list2.push_back(6);
        let node1 = list.get_mut_unchecked(list.head() as *const _).unwrap();
        let node2 = list2.get_mut_unchecked(list2.head() as *const _).unwrap();
        std::mem::swap(node1, node2); // 壊れる！
    }

    /* not compilable
    let head = list.head();
    list.remove(list.head());
    println!("{}", head.value));
    */
    {
        let mut list = List::new();
        list.insert(list.head(), 1); // 先頭に 1 を挿入
        list.insert(list.sentinel(), 2); // 末尾に 2 を挿入
        assert_eq!(*list.head().value(), 1); // 先頭の値を取得
        assert_eq!(*list.head().next().value(), 2); // 2番目の値を取得
        assert!(list.remove(list.head().next()).is_some()); // 2番目の要素を削除
    }
    */
}
