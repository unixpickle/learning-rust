// A doubly linked-list implementation, to help me understand
// weak references, refcells, etc.

use std::cell::{RefCell};
use std::collections::{VecDeque};
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

    fn push_front(&mut self, item: T) {
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

    fn pop_front(&mut self) -> Option<T> {
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

    fn push_back(&mut self, item: T) {
        let new_tail = Rc::new(RefCell::new(Node{
            data: Some(item),
            prev: None,
            next: None,
        }));
        match take(&mut self.tail) {
            Some(old_tail_weak) => {
                let old_tail = old_tail_weak.upgrade().unwrap();
                new_tail.borrow_mut().prev = Some(Rc::downgrade(&old_tail));
                self.tail = Some(Rc::downgrade(&new_tail));
                old_tail.borrow_mut().next = Some(new_tail);
            },
            None => {
                self.tail = Some(Rc::downgrade(&new_tail));
                self.head = Some(new_tail);
            }
        }
    }

    fn pop_back(&mut self) -> Option<T> {
        match take(&mut self.tail) {
            Some(old_tail_weak) => {
                let old_tail_strong = old_tail_weak.upgrade().unwrap();
                let mut old_tail = old_tail_strong.borrow_mut();
                match take(&mut old_tail.prev) {
                    Some(new_tail_weak) => {
                        let new_tail = new_tail_weak.upgrade().unwrap();
                        new_tail.borrow_mut().next = None;
                        self.tail = Some(Rc::downgrade(&new_tail));
                    },
                    None => {
                        self.head = None;
                    },
                }
                take(&mut old_tail.data)
            },
            None => None,
        }
    }
}

struct GarbageCounter {
    count: Rc<RefCell<usize>>,
}

impl GarbageCounter {
    fn new() -> Self {
        GarbageCounter{count: Rc::new(RefCell::new(1))}
    }

    fn get(&self) -> usize {
        *self.count.borrow()
    }
}

impl Drop for GarbageCounter {
    fn drop(&mut self) {
        *self.count.borrow_mut() -= 1;
    }
}

impl Clone for GarbageCounter {
    fn clone(&self) -> Self {
        *self.count.borrow_mut() += 1;
        GarbageCounter{count: self.count.clone()}
    }
}

fn check_equivalent() {
    for i in 0..10 {
        let mut ll = LinkedList::<i32>::new();
        let mut deque = VecDeque::<i32>::new();
        for (j, op) in random_sequence(1000, 6).into_iter().enumerate() {
            // Pseudo-randomly decide what to do, with more
            // weight put on insertion than deletion.
            if op < 4 {
                // Insertion.
                let n = ((j as i32) + i*1000) * 3613;
                if op < 2 {
                    ll.push_back(n);
                    deque.push_back(n);
                } else {
                    ll.push_front(n);
                    deque.push_front(n);
                }
            } else {
                // Deletion.
                if op == 4 {
                    let actual = ll.pop_back();
                    let expected = deque.pop_back();
                    assert_eq!(actual, expected);
                } else {
                    let actual = ll.pop_front();
                    let expected = deque.pop_front();
                    assert_eq!(actual, expected);
                }
            }
        }
    }
    println!("equivalence tests passed!")
}

fn check_cleaned_up() {
    let gc = GarbageCounter::new();
    for _ in 0..10 {
        assert_eq!(gc.get(), 1);
        let mut ll = LinkedList::<GarbageCounter>::new();
        for op in random_sequence(1000, 4) {
            if op < 2 {
                // Insertion.
                if op == 0 {
                    ll.push_back(gc.clone());
                } else {
                    ll.push_front(gc.clone());
                }
                assert!(gc.get() > 1);
            } else {
                // Deletion.
                let res = if op == 2 {
                    ll.pop_back()
                } else {
                    ll.pop_front()
                };
                if res.is_none() {
                    assert_eq!(gc.get(), 1);
                } else {
                    assert!(gc.get() > 1);
                }
            }
        }
    }
    println!("ownership tests passed!");
}

fn random_sequence(count: i32, max: i32) -> Vec<i32> {
    let mut n = 1;
    let mut res = Vec::new();
    for _ in 0..count {
        n = (n*5573 + 1921) % (max * 100);
        res.push(n / 100);
    }
    res
}

fn main() {
    check_equivalent();
    check_cleaned_up();
}
