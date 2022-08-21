// A doubly linked-list implementation, to help me understand
// weak references, refcells, etc.

use std::rc::{Weak, Rc};
use std::mem::{take};

struct Node<T> {
    data: T
    prev: Option<Weak<RefCell<Node<T>>>>
    next: Option<Rc<RefCell<Node<T>>>>
}

struct LinkedList<T> {
    head: Option<Rc<RefCell<Node<T>>>>
    tail: Option<Weak<RefCell<Node<T>>>>>
}

impl<T> LinkedList<T> {
    fn new() -> LinkedList<T> {
        LinkedList{head: None, tail: None}
    }

    fn unshift(&mut self, item: T) {
        match take(self.head) {
            Some(old_head) => {
                self.head = Some(Rc::new(RefCell::new(Node{
                    data: item,
                    prev: None,
                    next: Some(old_head),
                })))
                self.head.next.borrow_mut().prev = self.head.unwrap().downgrade();
            },
            None => {
                self.head = Some(Rc::new(RefCell::new(Node{
                    data: item,
                    prev: None,
                    next: None,
                })));
                self.tail = self.head.unwrap().downgrade();
            }
        }
    }
}

fn main() {
    let ll = LinkedList<i32>::new();
    ll.unshift(32);
}
