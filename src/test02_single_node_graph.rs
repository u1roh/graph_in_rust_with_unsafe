#![cfg(test)]
use std::ops::Deref;

/*
mod not_compilable {
    use super::*;

    pub struct Node<T> {
        pub value: T,
    }

    pub struct Graph<'a, T> {
        node: Box<Node<T>>,
        head: &'a Node<T>,
    }

    impl<'a, T> Graph<'a, T> {
        pub fn new(value: T) -> Self {
            let node = Box::new(Node { value });
            let head = node.deref();
            Self { node, head }
        }
    }
}
error[E0515]: cannot return value referencing local variable `node`
  --> src/test02_single_node_graph.rs:20:13
   |
19 |             let head = node.deref();
   |                        ---- `node` is borrowed here
20 |             Self { node, head }
   |             ^^^^^^^^^^^^^^^^^^^ returns a value referencing data owned by the current function

error[E0505]: cannot move out of `node` because it is borrowed
  --> src/test02_single_node_graph.rs:20:20
   |
16 |     impl<'a, T> Graph<'a, T> {
   |          -- lifetime `'a` defined here
...
19 |             let head = node.deref();
   |                        ---- borrow of `node` occurs here
20 |             Self { node, head }
   |             -------^^^^--------
   |             |      |
   |             |      move out of `node` occurs here
   |             returning this value requires that `node` is borrowed for `'a`

error: aborting due to 2 previous errors
*/

mod graph {
    use super::*;

    pub struct Node<T> {
        pub value: T,
    }

    pub struct Graph<T> {
        node: Box<Node<T>>,
        head: *const Node<T>,
    }

    impl<T> Graph<T> {
        pub fn new(value: T) -> Self {
            let node = Box::new(Node { value });
            let head = node.deref() as *const Node<T>;
            Self { node, head }
        }
        pub fn head(&self) -> &Node<T> {
            unsafe { std::mem::transmute(self.head) }
        }
        pub fn head_mut(&mut self) -> &mut Node<T> {
            unsafe { std::mem::transmute(self.head) }
        }
        pub fn reset_node(&mut self, value: T) {
            self.node = Box::new(Node { value });
            self.head = self.node.deref() as *const Node<T>; // CAUTION! これを書かないと head が dangling pointer になる！
        }
    }
}

#[test]
fn test02_1() {
    use graph::*;
    let mut graph: Graph<usize> = Graph::new(123);
    assert_eq!(graph.head().value, 123);

    graph.head_mut().value = 456;
    assert_eq!(graph.head().value, 456);

    graph.reset_node(789);
    assert_eq!(graph.head().value, 789);

    /*  not compilable
    let head = graph.head();
    graph.reset_node(321);
    println!("{}", head.value);
    */
    /*
      error[E0502]: cannot borrow `graph` as mutable because it is also borrowed as immutable
      --> src/test02_single_node_graph.rs:94:5
       |
    93 |     let head = graph.head();
       |                ----- immutable borrow occurs here
    94 |     graph.reset_node(321);
       |     ^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
    95 |     println!("{}", head.value);
       |                    ---------- immutable borrow later used here
    */

    let ptr = graph.head() as *const Node<usize>;
    let moved_graph: Graph<usize> = graph;
    assert_eq!(moved_graph.head().value, 789);
    assert_eq!(moved_graph.head() as *const Node<usize>, ptr);
}
