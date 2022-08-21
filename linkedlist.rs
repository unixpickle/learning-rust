// A doubly linked-list implementation, to help me understand
// weak references, refcells, etc.

use std::cell::{RefCell};
use std::mem::{take};
use std::rc::{Weak, Rc};

struct Node<T> {
    // I can't figure out a way to make this not an Option.
    // When we pop an item, we would need to consume the data
    // to return it, which would need to consume the Node<T>,
    // which doesn't seem possible if the Node<T> is in an Rc.
    data: Option<T>,

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
            data: Some(item),
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

    fn shift(&mut self) -> Option<T> {
        match take(&mut self.head) {
            Some(old_head) => {
                let mut obj = old_head.borrow_mut();
                match take(&mut obj.next) {
                    None => {
                        self.tail = None;
                    },
                    Some(x) => {
                        x.borrow_mut().prev = None;
                        self.head = Some(x);
                    }
                };
                take(&mut obj.data)
            },
            None => {
                None
            }
        }
    }
}

fn main() {
    let mut ll = LinkedList::<i32>::new();
    for i in 0..10 {
        ll.unshift(i);
    }
    loop {
        match ll.shift() {
            Some(x) => {
                println!("hello: {}", x);
            },
            None => {
                break;
            },
        };
    }
}
