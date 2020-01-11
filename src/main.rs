fn main() {
    println!("Hello, world!");
}

#[test]
fn box_and_ptr() {
    use std::ops::Deref;

    let obj1 = Box::new(123);
    let ptr1 = obj1.deref() as *const _;
    println!("ptr1 = {:?}", ptr1);

    let obj2 = obj1;
    let ptr2 = obj2.deref() as *const _;
    println!("ptr2 = {:?}", ptr2);

    assert_eq!(ptr1, ptr2);
}
