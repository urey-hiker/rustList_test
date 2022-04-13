
## rust 笔记

#### 使用Box定义Link


#### 带有尾部指针的Link

使用Box定义Link的局限性在于无法使用多个Box指向同一个元素，因为Box没有trait Copy。
尝试使用Rc替换Box以获得指向同一个元素。

将Link定义为：

```rust
type Link<T> = Option<Rc<Node<T>>>;

```

在push操作中：

```rust
fn push(&mut self, v: T) {
        let mut n = Rc::new(Node { val: v, next: None });
        match &self.head {
            None => {
                self.head = Some(n.clone());
                self.tail = Some(n.clone());
            }
            Some(head) => {
                n.next = self.tail.take();
                self.tail = Some(n)
            }
        }
    }
```
报错：

```shell
error[E0594]: cannot assign to data in an `Rc`
  --> src/listB.rs:32:17
   |
32 |                 n.next = self.tail.take();
   |                 ^^^^^^ cannot assign
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Node<T>>`
```

这里的重点是help这一句，n由于是`Rc<Node<T>>`类型的，所以无法解构为变量，也就是没有DerefMut的操作。
这样就需要使用RefCell使其用用内部可变性（Interior mutability）。


修改Link定义为：
```rust
type Link<T> = Option<Rc<RefCell<Node<T>>>>;
```

修改Push方法为：

```rust
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
```

这里在 `tail.borrow_mut().next = Some(n.clone());` 使用了borrow_mut意为借出可变的内部变量，其返回类型为 `RefMut<'_, T>`，这是一个带有生命域标识的类型。

在这里知道了push中如何修改Rc类型中的next，那么pop_front的操作也就没那么难实现了。

在这里我们还是实现的单向链表。因此目前的操作只有push_back和pop_front，当然push_front也是可行的，但pop_back就不行，因为需要遍历至倒数第二个节点进行修改，并将tail修改为指向倒数第二个节点。

再实现一下迭代访问操作：

```rust
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
```

到目前为止的操作，我们在List之外都是获得的带有Rc包裹的`Link<T>`，这类似于获得了C++中的std::shared_ptr。
对于包裹其中的元素，获得了可访问和修改的权限。增加peek实现，对于peek我们不需要修改其中的值，只需要知道内容。

```rust
fn peek_front(&self) -> Option<T> {
    if let Some(head) = &self.head {
        let node = head.borrow();
        Some(node.val.clone())
    } else {
        None
    }
}
```

这里我们返回val的值，但是不期望转移所有权，因此尝试使用clone来获得val的拷贝。编译出现以下错误：

```shell
error[E0599]: no method named `clone` found for type parameter `T` in the current scope
  --> src/listB.rs:92:27
   |
92 |             Some(node.val.clone())
   |                           ^^^^^ method not found in `T`
   |
   = help: items from traits can only be used if the type parameter is bounded by the trait
help: the following trait defines an item `clone`, perhaps you need to restrict type parameter `T` with it:
   |
46 | impl<T: Clone> List<T> {
   |      ~~~~~~~~

```

对于我们这里的泛型T，是否实现了clone是未知的。尝试按照help中的指示进行修改。

修改后能够成功编译运行。这样子peek出来的是将val拷贝了一份。如果val是一个较大的结构就会有较大的损耗。尝试实现一个peek_ref来返回值的借用。

```rust
fn peek_front_ref(&self) -> Option<Ref<T>> {
    if let Some(head) = &self.head {
        Some(Ref::map(head.borrow(), |node| &node.val))
    } else {
        None
    }
}
```
这里参考了 [Learning Rust With Entirely Too Many Linked Lists](https://rust-unofficial.github.io/too-many-lists/fourth-peek.html) 使用了`Ref::map`方法:

```shell
pub fn map<U, F>(orig: Ref<'b, T>, f: F) -> Ref<'b, U> where
    F: FnOnce(&T) -> &U,
    U: ?Sized, 

    Make a new Ref for a component of the borrowed data.
    The RefCell is already immutably borrowed, so this cannot fail.
    This is an associated function that needs to be used as Ref::map(...). A method would interfere with methods of the same name on the contents of a RefCell used through Deref.
```

类似于option的map方法，将Ref中的元素，转换为另一个元素的引用。

需要注意，Ref的使用是带有生命周期限定的，并且需要满足不被两次借用的规则。假设我们在单测中有：

```rust
let v = list.peek_front_ref();

if let Some(v) = list.peek_front_ref() {
    assert_eq!(*v, 1);
}
if let Some(n) = list.pop_front().take() {
    assert_eq!(n.borrow().val, 1);
} else {
    assert!(false);
}
```
编译报错有：

```shell
error[E0502]: cannot borrow `list` as mutable because it is also borrowed as immutable
   --> src/listC.rs:136:26
    |
131 |         let v = list.peek_front_ref();
    |                 --------------------- immutable borrow occurs here
...
136 |         if let Some(n) = list.pop_front().take() {
    |                          ^^^^^^^^^^^^^^^^ mutable borrow occurs here
...
154 |     }
    |     - immutable borrow might be used here, when `v` is dropped and runs the destructor for type `Option<Ref<'_, i32>>`
```

可以看到对于Ref<T>的借用是可被感知的延伸到了List。if let语句对借用的作用域是进行了限定，因此在整个单测中是可以多次使用的。

这里的借用返回的值是不可变的，如果想要可变的借用，需要使用RefMut:

```rust
fn peek_front_refMut(&self) -> Option<RefMut<T>> {
    if let Some(head) = &self.head {
        Some(RefMut::map(head.borrow_mut(), |node| &mut node.val))
    } else {
        None
    }
}
```

在单测中：

```rust
if let Some(mut v) = list.peek_front_refMut() {
    assert_eq!(*v, 2);
    *v = 4;
    assert_eq!(*v, 4);
}
if let Some(n) = list.pop_front().take() {
    assert_eq!(n.borrow().val, 4);
} else {
    assert!(false);
}
```
成功，这确实有效。

我们尝试实现另一种Pop_front，前面的实现是将整个Rc结构返回，由于已经不需要这个Node了，那么我们是否返回Option<T>即可。
参考peek_front，我们的实现为：

```rust
fn pop_front_val(&mut self) -> Option<T> {
    if let Some(head) = self.head.take() {
        if let Some(next) = head.borrow_mut().next.take() {
            self.head = Some(next);
        } else {
            self.head = None;
            self.tail = None;
        }
        Some(head.borrow().val)
    } else {
        None
    }
}
```
编译遇到错误：

```shell
error[E0507]: cannot move out of dereference of `Ref<'_, Node<T>>`
  --> src/listC.rs:88:18
   |
88 |             Some(head.borrow().val)
   |                  ^^^^^^^^^^^^^^^^^ move occurs because value has type `T`, which does not implement the `Copy` trait
```

这与我们前面实现peek_front的实现没有clone的错误类似。我们在相同的地方加上T需要实现Copy这个trait之后就编译通过了。

```rust
impl<T: Clone + Copy> List<T>
```
这个地方也可以写为：

```rust
impl<T> List<T>
where T: Clone + Copy
```

都是表示对泛型的一种限定。

这个地方让人有两个疑问：
1. Clone和Copy有什么区别；
2. Node的释放是否会对val的生存产生影响？



