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
}
