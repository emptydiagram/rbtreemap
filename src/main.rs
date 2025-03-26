use std::ptr::NonNull;
use std::fmt::{self, Debug};


#[derive(PartialEq, Eq, Debug)]
enum NodeColor {
    Red,
    Black
}

struct Node<K, V>
where
    K: Ord
{
    key: K,
    value: V,
    parent:Option<NonNull<Node<K, V>>>,
    left: Option<Box<Node<K, V>>>,
    right: Option<Box<Node<K, V>>>,
    color: NodeColor,
}

impl<K, V> Debug for Node<K, V>
where
    K: Ord + Debug,
    V: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ [{:?}] {:?} -> {:?}, l: {:?}, r: {:?}, p: {:?} }}", self.color, self.key, self.value, self.left, self.right, self.parent)
    }
}

impl <K, V> Node<K, V>
where K: Ord
{
    #[inline]
    fn is_black(&self) -> bool {
        self.color == NodeColor::Black
    }

    #[inline]
    fn is_red(&self) -> bool {
        self.color == NodeColor::Red
    }

    fn unwrap_parent(&self) -> &Node<K, V> {
        match self.parent {
            None => panic!("No parent"),
            Some(p_ptr) => unsafe { p_ptr.as_ref() },
        }
    }

    fn unwrap_parent_mut(&mut self) -> &mut Node<K, V> {
        match self.parent {
            None => panic!("No parent"),
            Some(mut p_ptr) => unsafe { p_ptr.as_mut() },
        }
    }
}

struct RBTreeMap<K, V>
where
    K: Ord
{
    root: Option<Box<Node<K, V>>>,
    size: usize,
}


impl<K, V> RBTreeMap<K, V>
where
    K: Ord + Debug,
    V: Debug
{
    pub fn new() -> Self {
        RBTreeMap { root: None, size: 0 }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.size = 0;
    }

    pub fn insert(&mut self, key: K, value: V) {
        let fixup_ptr: NonNull<Node<K, V>> = {
            let mut current = &mut self.root;
            let mut parent: Option<*mut Node<K, V>> = None;
            while let Some(node) = current {
                parent = Some(&mut **node);
                if key < node.key {
                    current = &mut node.left;
                } else {
                    current = &mut node.right;
                }
            }

            let parent_nn: Option<NonNull<Node<K, V>>> = parent.map(|n| unsafe { NonNull::new_unchecked(n) });
            let z = Box::new(Node {
                key, value, parent: parent_nn, left: None, right: None, color: NodeColor::Red
            });

            *current = Some(z);
            self.size += 1;
            current
                .as_mut()
                .map(|b| unsafe { NonNull::new_unchecked(b.as_mut()) })
                .unwrap()
        };
        self.insert_fixup(fixup_ptr);
    }


    fn insert_fixup(&mut self, mut z: NonNull<Node<K, V>>) {
        // invariants:
        //  - z is red,
        //  - if z.parent is root then it's black,
        //  - if there's a violation, then (root is red) XOR (there is a red-red parent-child combo)
        loop {
            // if no parent, z is root. there are no other R-B tree violations, so exit the loop
            let z_node = unsafe { z.as_mut() };
            if z_node.parent.is_none() {
                break;
            }
            let z_p: *mut Node<K, V> = z_node.unwrap_parent_mut();

            unsafe {
                if (*z_p).is_black() || (*z_p).parent.is_none() { break; }

                // if z.parent.parent is None, then z.parent is root, so z.parent is black
                // so z.parent.parent is Some
                let z_p_p: *mut Node<K, V> = (*z_p).unwrap_parent_mut();

                if (*z_p_p).left.is_some() && std::ptr::eq(z_p, (*z_p_p).left.as_ref().unwrap().as_ref()) {
                    // z.parent is left child
                    let maybe_z_p_p_r = (*z_p_p).right.as_mut();
                    if maybe_z_p_p_r.is_some() && maybe_z_p_p_r.as_ref().unwrap().is_red() {
                        let y = maybe_z_p_p_r.unwrap();
                        (*z_p).color = NodeColor::Black;
                        y.color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        z = (*z_p).parent.unwrap();
                    } else {
                        // z.parent doesn't have a red sibling
                        if (*z_p).right.is_some() && std::ptr::eq(z.as_ref(), &**(*z_p).right.as_ref().unwrap()) {
                            z = z.as_ref().parent.unwrap();
                            self.rotate_left(z);
                        }
                        (*z_p).color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        self.rotate_right(NonNull::new_unchecked(z_p_p));
                    }
                } else {
                    // z.parent is right child
                    let maybe_z_p_p_l = (*z_p_p).left.as_mut();
                    if maybe_z_p_p_l.is_some() && maybe_z_p_p_l.as_ref().unwrap().is_red() {
                        let y = maybe_z_p_p_l.unwrap();
                        (*z_p).color = NodeColor::Black;
                        y.color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        z = (*z_p).parent.unwrap();
                    } else {
                        if (*z_p).left.is_some() && std::ptr::eq(z.as_ref(), &**(*z_p).left.as_ref().unwrap()) {
                            z = z.as_ref().parent.unwrap();
                            self.rotate_right(z);
                        }
                        (*z_p).color = NodeColor::Black;
                        (*z_p_p).color = NodeColor::Red;
                        self.rotate_left(NonNull::new_unchecked(z_p_p));
                    }
                }
            }
        }
        self.root.as_mut().unwrap().color = NodeColor::Black;
    }

    fn rotate_left(&mut self, x_ptr: NonNull<Node<K, V>>) {
        unsafe {
            let x = x_ptr.as_ptr();
            let mut y = (*x).right.take().expect("Only call rotate_left on a node with a right child");
            let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut()) };
            (*x).right = y.left.take();
            if let Some(ref mut x_right) = (*x).right {
                x_right.parent = Some(x_ptr);
            }

            y.parent = (*x).parent;

            match (*x).parent {
                None => {
                    let x = self.root.take().unwrap();
                    self.root = Some(y);
                    self.root.as_mut().unwrap().left = Some(x);
                },
                Some(x_parent_ptr) => {
                    let x_parent = &mut(*x_parent_ptr.as_ptr());
                    if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x) }) {
                        let x = x_parent.left.take().unwrap();
                        x_parent.left = Some(y);
                        x_parent.left.as_mut().unwrap().left = Some(x);
                    } else {
                        let x = x_parent.right.take().unwrap();
                        x_parent.right = Some(y);
                        x_parent.right.as_mut().unwrap().left = Some(x);
                    }
                }
            }
            (*x).parent = Some(y_ptr);

        }
    }

    //   x      y
    //  /   =>   \
    // y          x

    fn rotate_right(&mut self, x_ptr: NonNull<Node<K, V>>) {
        unsafe {
            let x = x_ptr.as_ptr();
            let mut y = (*x).left.take().expect("Only call rotate_right on a node with a left child");
            let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut()) };
            (*x).left = y.right.take();
            if let Some(ref mut x_left) = (*x).left {
                x_left.parent = Some(x_ptr);
            }

            y.parent = (*x).parent;

            match (*x).parent {
                None => {
                    let x = self.root.take().unwrap();
                    self.root = Some(y);
                    self.root.as_mut().unwrap().right = Some(x);
                },
                Some(x_parent_ptr) => {
                    // if x_parent.left is x, set left to y, else set right to y
                    let x_parent = &mut(*x_parent_ptr.as_ptr());
                    if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x) }) {
                        let x = x_parent.left.take().unwrap();
                        x_parent.left = Some(y);
                        x_parent.left.as_mut().unwrap().right = Some(x);
                    } else {
                        let x = x_parent.right.take().unwrap();
                        x_parent.right = Some(y);
                        x_parent.right.as_mut().unwrap().right = Some(x);
                    }
                }
            }
            (*x).parent = Some(y_ptr);
        }
    }

}



fn main() {
    println!("Hello, world!");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singleton() {
        let mut map = RBTreeMap::new();
        assert_eq!(map.len(), 0);
        map.insert(12, "abc");
        assert_eq!(map.len(), 1);
        assert!(map.get(&12).is_some());
        assert_eq!(map.get(&12), Some(&"abc"));
    }

    #[test]
    fn test_two_elements() {
        let mut map = RBTreeMap::new();
        map.insert(12, "abc");
        map.insert(34, "def");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_three_elements_1() {
        let mut map = RBTreeMap::new();
        map.insert(1, "a");
        map.insert(2, "b");
        map.insert(3, "c");
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_three_elements_2() {
        let mut map = RBTreeMap::new();
        map.insert(1, "a");
        map.insert(3, "c");
        map.insert(2, "b");
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_three_elements_3() {
        let mut map = RBTreeMap::new();
        map.insert(2, "a");
        map.insert(1, "b");
        map.insert(3, "c");
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_three_elements_4() {
        let mut map = RBTreeMap::new();
        map.insert(2, "a");
        map.insert(3, "b");
        map.insert(1, "c");
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_three_elements_5() {
        let mut map = RBTreeMap::new();
        map.insert(3, "a");
        map.insert(1, "b");
        map.insert(2, "c");
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_three_elements_6() {
        let mut map = RBTreeMap::new();
        map.insert(3, "a");
        map.insert(2, "b");
        map.insert(1, "c");
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_five_elements_1() {
        let mut map = RBTreeMap::new();
        map.insert(1, "a");
        map.insert(2, "b");
        map.insert(3, "c");
        map.insert(4, "d");
        map.insert(5, "e");
        assert_eq!(map.len(), 5);
    }

    #[test]
    fn test_five_elements_2() {
        let mut map = RBTreeMap::new();
        map.insert(3, "a");
        map.insert(1, "b");
        map.insert(2, "c");
        map.insert(5, "d");
        map.insert(4, "e");
        assert_eq!(map.len(), 5);
    }

}