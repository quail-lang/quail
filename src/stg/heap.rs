use std::collections::HashMap;

use super::machine::{Closure, Addr};

#[derive(Debug, Clone)]
pub struct Heap {
    map: HashMap<Addr, Closure>,
    next_addr: Addr,
 }

impl Heap {
    pub fn new() -> Self {
        Heap {
            map: HashMap::new(),
            next_addr: 0 as Addr,
        }
    }

    // TODO all lookup functions should return Option types.
    pub fn lookup_mut(&mut self, addr: Addr) -> &mut Closure {
        self.map.get_mut(&addr).unwrap()
    }

    pub fn lookup(&self, addr: Addr) -> &Closure {
        &self.map[&addr]
    }

    pub fn lookup_many(&self, addrs: &[Addr]) -> Vec<&Closure> {
        let mut closures = Vec::new();

        for addr in addrs {
            closures.push(&self.map[&addr]);
        }

        closures
    }

    pub fn alloc(&mut self, closure: Closure) -> Addr {
        if self.should_gc() {
            self.gc();
        }

        let addr = self.next_addr;
        self.next_addr += 1;

        self.map.insert(addr, closure);
        addr
    }

    fn should_gc(&self) -> bool {
        false
    }

    pub fn gc(&mut self) {
        todo!()
    }
}

pub fn heap_to_string(heap: &Heap) -> String {
    let mut s = String::new();
    for (k, v) in heap.map.iter() {
        s.push_str(&format!("{} ~> {:?}\n", k, v));
    }
    s
}
