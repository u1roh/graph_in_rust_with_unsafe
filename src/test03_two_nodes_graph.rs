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
