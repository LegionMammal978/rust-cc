# rust-cc

A fast cycle collector for Rust programs with built-in support for creating cycles of references.

This crate provides a `Cc` (cycle collected) smart pointer, which is basically a `Rc` which automatically detects and 
deallocates cycles of references. If there are no cycles of references, then `Cc` behaves like `Rc` and deallocates 
immediately when the reference counter drops to zero.

Currently, the cycle collector is not concurrent. As such, `Cc` doesn't implement `Send` nor `Sync`.

This crate is currently in beta, so please report any bug you find.

## Example Usage

```rust
struct Data {
    a: Cc<u32>,
    b: Cc<RefCell<String>>,
    c: Option<Cc<Data>>,
}

// Objects allocated with Cc must implement the Trace and Finalize traits:

unsafe impl Trace for Data {
    fn trace(&self, ctx: &mut Context<'_>) {
        // Just call trace on every field.
        // In future versions there will be a (safe) macro to automatically derive Trace.
        // Also, make sure to read the Trace safety requirements in the documentation!
        self.a.trace(ctx);
        self.b.trace(ctx);
        self.c.trace(ctx);
    }
}

impl Finalize for Data {
    fn finalize(&self) {
        // Finalization code called when a Data object is about to be deallocated
        // to allow resource clean up (like closing file descriptors, etc)
    }
}

fn main() {
    // Cc::new lets you allocate an object, like Rc::new
    let cc: Cc<Data> = Cc::new(Data {
        a: Cc::new(5),
        b: Cc::new(RefCell::new("a_string".to_string())),
        c: None,
    });

    // RefCell can be used for interior mutability
    *cc.b.borrow_mut() = "another_string".to_string();

    // Cc(s) may be cloned freely, you don't need to worry about creating cycles!
    let cloned = cc.clone();

    drop(cc);
    drop(cloned);
    // Since there wasn't any cycle of references, the allocated Data instance gets immediately deallocated

    // Let's create a cycle. Cc::new_cyclic is like Rc::new_cyclic, but `this` is NOT a weak reference
    let cc = Cc::<Data>::new_cyclic(|this: &Cc<Data>| {
        // Dereferencing `this` inside the closure will lead to a panic,
        // since the object `this` points to hasn't been initialized yet
        Data {
            a: Cc::new(10),
            b: Cc::new(RefCell::new("a_string".to_string())),
            c: Some(this.clone()), // Cycle!
        }
    });

    assert!(Cc::ptr_eq(&cc, cc.c.as_ref().unwrap()));

    drop(cc);
    // Here, the allocated Data instance doesn't gets deallocated automatically, since there is a cycle.
    // We have to call the cycle collector
    collect_cycles();
    // collect_cycles() is automatically called from time to time when creating new Ccs,
    // calling it directly only ensures that a collection is run (like at the end of the program)
}
```

## The collection algorithm

The main idea is to discover the roots making use of the information contained in the reference counter,
instead of having them from another source.  

Usually, in garbage collected languages, the runtime has always a way to know which objects are roots (i.e. objects 
accessible from a pointer on the stack). However, since Rust has no runtime, this information isn't available. This way,
garbage collectors implemented in Rust for Rust programs have a very difficult time figuring out which objects can be 
deallocated and which cannot because they can still be accessed by the program.

`rust-cc`, using the reference counters, is able to find the roots at runtime while collecting, eliminating the need to
constantly keep track of the roots. This is also the reason why the standard `RefCell` can be safely used inside
cycle collected objects for interior mutability.

Moreover, the implemented cycle collector should be generally faster than mark-and-sweep garbage collectors on big 
(and fragmented) heaps, since there's no need to trace the whole heap every time it runs.

If you're interested in reading the source code, the algorithm is described more deeply in [CONTRIBUTING.md](./CONTRIBUTING.md#the-collection-algorithm).

## Benchmarks

Benchmarks can be found at <https://github.com/frengor/rust-cc-benchmarks>.

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, 
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
