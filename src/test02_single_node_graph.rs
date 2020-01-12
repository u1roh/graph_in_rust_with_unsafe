#![cfg(test)]
use std::ops::Deref;

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
    let mut graph = Graph::new(123);
    assert_eq!(graph.head().value, 123);

    graph.head_mut().value = 456;
    assert_eq!(graph.head().value, 456);

    graph.reset_node(789);
    assert_eq!(graph.head().value, 789);
}
