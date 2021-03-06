#![cfg(test)]
use std::ops::Deref;

struct Data(usize);

#[test]
fn test01_0() {
    let obj1: Data = Data(123);
    let ptr1 = &obj1 as *const Data;

    let obj2: Data = obj1;
    let ptr2 = &obj2 as *const Data;

    assert_ne!(ptr1, ptr2);
}

#[test]
fn test01_1() {
    let obj1: Box<Data> = Box::new(Data(123));
    let ptr1 = obj1.deref() as *const Data;

    let obj2: Box<Data> = obj1;
    let ptr2 = obj2.deref() as *const Data;

    assert_eq!(ptr1, ptr2);
    assert_eq!(unsafe { (*ptr1).0 }, 123);
    assert_eq!(unsafe { (*ptr2).0 }, 123);

    let ref1: &Data = unsafe { &(*ptr1) };
    assert_eq!(ref1.0, 123);

    let ref2: &Data = unsafe { std::mem::transmute(ptr1) };
    assert_eq!(ref2.0, 123);
}

/*
fn not_compilable1<'a>() -> &'a X {
    let x = Box::new(X);
    x.deref()
}

    error[E0515]: cannot return value referencing local variable `x`
    --> src/test01_box_and_ptr.rs:21:5
       |
    21 |     x.deref()
       |     -^^^^^^^^
       |     |
       |     returns a value referencing data owned by the current function
       |     `x` is borrowed here
*/

/*
fn not_compilable2<'a>() -> (Box<X>, &'a X) {
    let x = Box::new(X);
    let y = x.deref();
    (x, y)
}

error[E0515]: cannot return value referencing local variable `x`
  --> src/test01_box_and_ptr.rs:38:5
   |
37 |     let y = x.deref();
   |             - `x` is borrowed here
38 |     (x, y)
   |     ^^^^^^ returns a value referencing data owned by the current function

error[E0505]: cannot move out of `x` because it is borrowed
  --> src/test01_box_and_ptr.rs:38:6
   |
35 | fn not_compilable1<'a>() -> (Box<X>, &'a X) {
   |                    -- lifetime `'a` defined here
36 |     let x = Box::new(X);
37 |     let y = x.deref();
   |             - borrow of `x` occurs here
38 |     (x, y)
   |     -^----
   |     ||
   |     |move out of `x` occurs here
   |     returning this value requires that `x` is borrowed for `'a`
*/

mod test01_2 {
    use super::*;

    fn get_obj_and_ptr() -> (Box<Data>, *const Data) {
        let obj: Box<Data> = Box::new(Data(123));
        let ptr = obj.deref() as *const Data;
        (obj, ptr)
    }

    #[test]
    fn test() {
        let (obj, ptr) = get_obj_and_ptr();
        assert_eq!(obj.deref() as *const Data, ptr);

        let refer: &Data = unsafe { std::mem::transmute(ptr) };
        assert_eq!(refer.0, 123);
    }
}

mod test01_3 {
    use super::*;

    struct ObjAndPtr {
        obj: Box<Data>,
        ptr: *const Data,
    }

    impl ObjAndPtr {
        fn new(value: usize) -> Self {
            let obj: Box<Data> = Box::new(Data(value));
            let ptr = obj.deref() as *const Data;
            Self { obj, ptr }
        }
        fn ref_safe(&self) -> &Data {
            self.obj.deref()
        }
        fn ref_unsafe(&self) -> &Data {
            unsafe { std::mem::transmute(self.ptr) }
        }
    }

    #[test]
    fn test() {
        let x = ObjAndPtr::new(123);
        assert_eq!(x.ref_safe().0, 123);
        assert_eq!(x.ref_unsafe().0, 123);

        let y = x;
        assert_eq!(y.ref_safe().0, 123);
        assert_eq!(y.ref_unsafe().0, 123);
    }
}
