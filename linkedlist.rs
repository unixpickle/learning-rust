// A doubly linked-list implementation, to help me understand
// weak references, refcells, etc.

use std::cell::{RefCell};
use std::mem::{take};
use std::rc::{Weak, Rc};

struct Node<T> {
    data: T,
    prev: Option<Weak<RefCell<Node<T>>>>,
    next: Option<Rc<RefCell<Node<T>>>>,
}

struct LinkedList<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Weak<RefCell<Node<T>>>>,
}

impl<T> LinkedList<T> {
    fn new() -> LinkedList<T> {
        LinkedList{head: None, tail: None}
    }

    fn unshift(&mut self, item: T) {
        let new_head = Rc::new(RefCell::new(Node{
            data: item,
            prev: None,
            next: None,
        }));
        match take(&mut self.head) {
            Some(old_head) => {
                old_head.borrow_mut().prev = Some(Rc::downgrade(&new_head));
                new_head.borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            },
            None => {
                self.tail = Some(Rc::downgrade(&new_head));
                self.head = Some(new_head);
            }
        }
    }
}

fn main() {
    let mut ll = LinkedList::<i32>::new();
    ll.unshift(32);
}
