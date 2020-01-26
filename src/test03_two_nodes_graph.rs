#![cfg(test)]
use std::ops::Deref;

mod not_compilable {

    struct Node<'a, T> {
        pub value: T,
        other: Option<&'a Self>,
    }

    #[test]
    fn test() {
        let mut node1 = Box::new(Node {
            value: 123,
            other: None, // 一旦 other は None で初期化
        });
        let mut node2 = Box::new(Node {
            value: 456,
            other: None, // 一旦 other は None で初期化
        });
        // ここで相互参照を作る
        node1.other = Some(&node2);
        //node2.other = Some(&node1); // compilation error!
        /*
          error[E0506]: cannot assign to `node2.other` because it is borrowed
          --> src/test03_two_nodes_graph.rs:22:9
           |
        21 |         node1.other = Some(&node2);
           |                            ------ borrow of `node2.other` occurs here
        22 |         node2.other = Some(&node1);
           |         ^^^^^^^^^^^^^^^^^^^^^^^^^^
           |         |
           |         assignment to borrowed `node2.other` occurs here
           |         borrow later used here
        */
    }
}

mod with_cell {
    use std::cell::*;

    struct Node<'a, T> {
        pub value: T,
        other: Cell<Option<&'a Self>>,
    }

    #[test]
    fn test1() {
        let node1 = Box::new(Node {
            value: 123,
            other: Cell::new(None), // 一旦 other は None で初期化
        });
        let node2 = Box::new(Node {
            value: 456,
            other: Cell::new(None), // 一旦 other は None で初期化
        });
        // ここで相互参照を作る
        node1.other.set(Some(&node2));
        node2.other.set(Some(&node1));
        assert_eq!(node1.other.get().unwrap().value, 456);
        assert_eq!(node2.other.get().unwrap().value, 123);
    }

    #[test]
    fn test2() {
        let mut nodes: Vec<Box<Node<usize>>> = Vec::new();
        nodes.push(Box::new(Node {
            value: 123,
            other: Cell::new(None),
        }));
        nodes.push(Box::new(Node {
            value: 456,
            other: Cell::new(None),
        }));
        nodes[1].other.set(Some(&nodes[0]));
        nodes[0].other.set(Some(&nodes[1]));
        /* compilation error
        nodes.push(Box::new(Node {
            value: 789,
            other: Cell::new(None),
        }));
        */
    }
}

mod graph {
    use super::*;

    pub struct Node<T> {
        pub value: T,
        other: *const Self,
    }
    impl<T> Node<T> {
        pub fn other(&self) -> &Self {
            unsafe { std::mem::transmute(self.other) }
        }
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
            unsafe { Self(std::mem::transmute(self.0.other)) }
        }
    }

    pub struct Graph<T> {
        node1: Box<Node<T>>,
        node2: Box<Node<T>>,
    }
    impl<T> Graph<T> {
        pub fn new(value1: T, value2: T) -> Self {
            let mut node1 = Box::new(Node {
                value: value1,
                other: std::ptr::null(), // 一旦 other は null で初期化
            });
            let mut node2 = Box::new(Node {
                value: value2,
                other: std::ptr::null(), // 一旦 other は null で初期化
            });
            // ここで相互参照を作る
            node1.other = node2.deref() as *const Node<T>;
            node2.other = node1.deref() as *const Node<T>;
            Self { node1, node2 }
        }
        pub fn node1(&self) -> &Node<T> {
            &self.node1
        }
        pub fn node2(&self) -> &Node<T> {
            &self.node2
        }
        pub fn node1_mut(&mut self) -> NodeRefMut<T> {
            NodeRefMut(&mut self.node1)
        }
        pub fn reset_node1(&mut self, value: T) {
            self.node1 = Box::new(Node {
                value,
                other: self.node2.deref() as *const Node<T>,
            });
            self.node2.other = self.node1.deref() as *const Node<T>;
        }
        pub fn reset_node2(&mut self, value: T) {
            self.node2 = Box::new(Node {
                value,
                other: self.node1.deref() as *const Node<T>,
            });
            self.node1.other = self.node2.deref() as *const Node<T>;
        }
    }
}

use graph::*;

// 不変条件
fn test_invariant<T>(graph: &Graph<T>) {
    assert_eq!(graph.node1().other() as *const _, graph.node2() as *const _);
    assert_eq!(graph.node2().other() as *const _, graph.node1() as *const _);
}

#[test]
fn test03_1() {
    let mut graph: Graph<usize> = Graph::new(123, 456);
    assert_eq!(graph.node1().value, 123);
    assert_eq!(graph.node2().value, 456);
    test_invariant(&graph);

    assert_eq!(graph.node1().other().value, 456);
    assert_eq!(graph.node2().other().value, 123);

    {
        let node1: &Node<usize> = graph.node1(); // graph と同じ lifetime
        let node2: &Node<usize> = node1.other(); // node1 と同じ lifetime すなわち graph と同じ lifetime
        assert_eq!(node2.value, graph.node2().value);
    }

    graph.reset_node1(321);
    test_invariant(&graph);

    graph.reset_node2(654);
    test_invariant(&graph);

    let mut graph1 = Graph::new(1, 2);
    let mut graph2 = Graph::new(3, 4);
    *graph1.node1_mut().value_mut() = 5;
    *graph1.node1_mut().other().value_mut() = 6;
    //std::mem::swap(graph1.node1_mut(), graph2.node1_mut());

    // unable to compile
    /*
    {
        let node1: &Node<usize> = graph.node1();
        graph.reset_node1(789);
        println!("{}", node1.value);
        /*
          error[E0502]: cannot borrow `graph` as mutable because it is also borrowed as immutable
          --> src/test03_two_nodes_graph.rs:91:9
           |
        90 |         let node1: &Node<usize> = graph.node1();
           |                                   ----- immutable borrow occurs here
        91 |         graph.reset_node1(789);
           |         ^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
        92 |         println!("{}", node1.value);
           |                        ----------- immutable borrow later used here
        */
    }
    */
}

/*
mod graph2 {
    use super::*;
    use std::cell::Cell;

    pub struct Node<'a, T> {
        pub value: T,
        pub other: Cell<Option<&'a Self>>,
    }

    pub struct Graph<'a, T> {
        node1: Box<Node<'a, T>>,
        node2: Box<Node<'a, T>>,
    }
    impl<'a, T> Graph<'a, T> {
        pub fn new(value1: T, value2: T) -> Self {
            let mut node1 = Box::new(Node {
                value: value1,
                other: Cell::new(None),
            });
            let mut node2 = Box::new(Node {
                value: value2,
                other: Cell::new(None),
            });
            // ここで相互参照を作る
            node1.other.set(Some(node2.deref()));
            node2.other.set(Some(node1.deref()));
            Self { node1, node2 }
        }
    }
}
*/

mod graph3 {
    use super::*;
    use std::cell::Cell;
    use typed_arena::Arena;

    pub struct Node<'a, T> {
        pub value: T,
        pub other: Cell<Option<&'a Self>>,
    }

    #[test]
    fn test() {
        let arena = Arena::new();
        let mut node1 = arena.alloc(Node {
            value: 1,
            other: Cell::new(None),
        });
        let mut node2 = arena.alloc(Node {
            value: 2,
            other: Cell::new(None),
        });
        // ここで相互参照を作る
        node1.other.set(Some(node2.deref()));
        node2.other.set(Some(node1.deref()));
    }

    /*
    pub struct Graph<'a, T> {
        node1: Box<Node<'a, T>>,
        node2: Box<Node<'a, T>>,
    }
    impl<'a, T> Graph<'a, T> {
        pub fn new(value1: T, value2: T) -> Self {
            let arena = Arena::new();
            let mut node1 = arena.alloc(Node {
                value: value1,
                other: Cell::new(None),
            });
            let mut node2 = arena.alloc(Node {
                value: value2,
                other: Cell::new(None),
            });
            // ここで相互参照を作る
            node1.other.set(Some(node2.deref()));
            node2.other.set(Some(node1.deref()));
            Self { node1, node2 }
        }
    }
    */
}
