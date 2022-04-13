use std::cell::{Ref, RefCell};
use std::rc::Rc;

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

// #[derive(Clone, Debug)]
struct Node<T> {
    val: T,
    next: Link<T>,
}

struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

struct Inter<T> {
    iter: Link<T>,
}

impl<T> Iterator for Inter<T> {
    type Item = Rc<RefCell<Node<T>>>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cur) = self.iter.take() {
            self.iter = cur
                .borrow()
                .next
                .as_ref()
                .map_or(None, |next| Some(next.clone()));
            self.iter.as_ref().map_or(None, |cur| Some(cur.clone()))
        } else {
            None
        }
    }
}

// impl<T> Clone for Node<T> {
//     fn clone(&self) -> Node<T> {
//         Node {
//             val: self.val.clone(),
//             next: None,
//         }
//     }
// }

impl<T : Clone> List<T> {
    fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    fn IntoIter(&mut self) -> Inter<T> {
        Inter {
            iter: self.head.as_ref().map_or(None, |head| Some(head.clone())),
        }
    }

    fn push(&mut self, v: T) {
        let mut n = Rc::new(RefCell::new(Node { val: v, next: None }));
        match &self.head {
            None => {
                self.head = Some(n.clone());
                self.tail = Some(n.clone());
            }
            Some(head) => {
                if let Some(tail) = &self.tail {
                    tail.borrow_mut().next = Some(n.clone());
                    self.tail = Some(n.clone());
                }
            }
        }
    }
    fn pop_front(&mut self) -> Link<T> {
        if let Some(head) = self.head.take() {
            if let Some(next) = head.borrow_mut().next.take() {
                self.head = Some(next);
            } else {
                self.head = None;
                self.tail = None;
            }
            Some(head)
        } else {
            None
        }
    }

    fn peek_front(&self) -> Option<T> {
        if let Some(head) = &self.head {
            let node = head.borrow();
            Some(node.val.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ListB() {
        let mut list: List<i32> = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.IntoIter();
        if let Some(cur) = iter.iter.as_ref() {
            assert_eq!(cur.borrow().val, 1);
        }
        iter.next();
        if let Some(cur) = iter.iter.as_ref() {
            assert_eq!(cur.borrow().val, 2);
        }
        iter.next();
        if let Some(cur) = iter.iter.as_ref() {
            assert_eq!(cur.borrow().val, 3);
        }
        assert_eq!(iter.next().is_some(), false);

        if let Some(v) = list.peek_front() {
            assert_eq!(v, 1);
        }
        if let Some(n) = list.pop_front().take() {
            assert_eq!(n.borrow().val, 1);
        } else {
            assert!(false);
        }

        if let Some(v) = list.peek_front() {
            assert_eq!(v, 2);
        }
        if let Some(n) = list.pop_front().take() {
            assert_eq!(n.borrow().val, 2);
        } else {
            assert!(false);
        }

        if let Some(v) = list.peek_front() {
            assert_eq!(v, 3);
        }
        if let Some(n) = list.pop_front().take() {
            assert_eq!(n.borrow().val, 3);
        } else {
            assert!(false);
        }
        if let Some(v) = list.peek_front() {
            assert!(false);
        } else {
            assert!(true);
        }
    }
}
