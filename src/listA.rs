use std::boxed::Box;

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    val: T,
    next: Link<T>,
}

struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    fn new() -> Self {
        List { head: None }
    }

    fn push(&mut self, v: T) {
        self.head = Some(Box::new(Node {
            val: v,
            next: self.head.take(),
        }));
    }
    fn pop(&mut self) -> Link<T> {
        if let Some(mut n) = self.head.take() {
            self.head = n.next.take();
            Some(n)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ListA() {
        let mut list: List<i32> = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        if let Some(n) = list.pop().take() {
            assert_eq!(n.val, 3);
        }
        if let Some(n) = list.pop().take() {
            assert_eq!(n.val, 2);
        }
        if let Some(n) = list.pop().take() {
            assert_eq!(n.val, 1);
        }
        if let Some(n) = list.pop().take() {
            assert!(false);
        } else {
            assert!(true);
        }
    }
}
