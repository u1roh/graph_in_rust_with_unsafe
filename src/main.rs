mod test01_box_and_ptr;
mod test02_single_node_graph;
mod test03_two_nodes_graph;
mod test04_nodes_in_vec;
mod test05_nodes_in_hashmap;
mod test06_nodes_in_mepoo;
mod test07_nodes_in_arena;

use rand::Rng;
const N: usize = 1000000;

fn benchmark_05() {
    use test05_nodes_in_hashmap::*;
    let mut random = rand::thread_rng();
    let mut list: List<usize> = List::new();
    for i in 0..N {
        list.push_back(i);
    }
    let mut ptr = list.head() as *const Node<_>;
    for k in 0..N {
        if random.gen::<i32>() % 2 == 0 {
            if let Some(node) = list.get_ref(ptr) {
                ptr = match random.gen::<i32>() % 2 {
                    0 => node.next() as *const _,
                    _ => node.prev() as *const _,
                };
            }
        } else {
            if random.gen::<i32>() % 2 == 0 {
                list.insert(ptr, k);
            } else if let Some(node) = list.remove(ptr) {
                ptr = node as *const _;
            } else {
                ptr = list.head() as *const _;
            }
        }
    }
}

fn benchmark_06() {
    use mepoo::*;
    use test06_nodes_in_mepoo::*;
    let mut random = rand::thread_rng();
    let mut list: List<usize> = List::new();
    for i in 0..N {
        list.push_back(i);
    }
    let mut ptr: Ptr<_> = list.head().into();
    for k in 0..N {
        if random.gen::<i32>() % 2 == 0 {
            if let Some(node) = list.get_ref(ptr) {
                ptr = match random.gen::<i32>() % 2 {
                    0 => node.next().into(),
                    _ => node.prev().into(),
                };
            }
        } else {
            if random.gen::<i32>() % 2 == 0 {
                list.insert(ptr, k);
            } else if let Some(node) = list.remove(ptr) {
                ptr = node.into();
            } else {
                ptr = list.head().into();
            }
        }
    }
}

fn benchmark_07() {
    use test07_nodes_in_arena::list::*;
    use test07_nodes_in_arena::pool::*;
    let mut random = rand::thread_rng();
    let mut list: List<usize> = List::new();
    for i in 0..N {
        list.push_back(i);
    }
    let mut ptr: Ptr<_> = list.head().into();
    for k in 0..N {
        if random.gen::<i32>() % 2 == 0 {
            if let Some(node) = list.get(ptr) {
                ptr = match random.gen::<i32>() % 2 {
                    0 => node.next().into(),
                    _ => node.prev().into(),
                };
            }
        } else {
            if random.gen::<i32>() % 2 == 0 {
                list.insert(ptr, k);
            } else if let Some(node) = list.remove(ptr) {
                ptr = node.into();
            } else {
                ptr = list.head().into();
            }
        }
    }
}

fn run(action: impl Fn(), caption: &str) {
    let instant = std::time::Instant::now();
    action();
    println!("{:?} @ {}", instant.elapsed(), caption);
}

fn main() {
    run(benchmark_05, "benchmark 05");
    run(benchmark_06, "benchmark 06");
    run(benchmark_07, "benchmark 07");
}
