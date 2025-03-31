use std::ptr::NonNull;
use std::fmt::{self, Debug};

#[cfg(test)]
mod tests;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NodeColor {
    Red,
    Black
}

struct Node<K, V>
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
    K: Debug,
    V: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node {{ [{:?}] {:?} -> {:?}, l: {:?}, r: {:?}", self.color, self.key, self.value, self.left, self.right)
    }
}

impl <K, V> Node<K, V>
{
    #[inline]
    fn is_black(&self) -> bool {
        self.color == NodeColor::Black
    }

    #[inline]
    fn is_red(&self) -> bool {
        self.color == NodeColor::Red
    }

    fn unwrap_parent(&mut self) -> NonNull<Node<K, V>> {
        match self.parent {
            None => panic!("No parent"),
            Some(p_ptr) => p_ptr,
        }
    }

    fn leftmost_descendant(&self) -> &Node<K, V> {
        let mut curr = self;
        while let Some(ref next) = curr.left {
            curr = next;
        }
        curr
    }


    fn rightmost_descendant(&self) -> &Node<K, V> {
        let mut curr = self;
        while let Some(ref next) = curr.right {
            curr = next;
        }
        curr
    }

    fn first_right_strict_ancestor(&self) -> Option<&Node<K, V>> {
        let mut curr = self;
        while let Some(curr_parent) = curr.parent.map(|node_ptr| unsafe { node_ptr.as_ref() }) {
            if curr_parent.left.is_some() && std::ptr::eq(&**curr_parent.left.as_ref().unwrap(), curr) {
                return Some(curr_parent);
            }
            curr = curr_parent;
        }
        None
    }
}

pub struct RBTreeMap<K, V>
where
    K: Ord
{
    root: Option<Box<Node<K, V>>>,
    size: usize,
}

pub struct Iter<'a, K: Ord, V>
{
    curr: Option<&'a Node<K, V>>,
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

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut maybe_node = self.root.as_ref().map(|box_node| &*box_node);
        while let Some(curr_node) = maybe_node {
            if curr_node.key == *key {
                return Some(&curr_node.value);
            } else if *key < curr_node.key {
                maybe_node = curr_node.left.as_ref();
            } else {
                maybe_node = curr_node.right.as_ref();
            }
        }
        None
    }

    pub fn first_key_value(&self) -> Option<(&K, &V)> {
        self.root.as_ref().map(|r| {
            let first = r.leftmost_descendant();
            (&first.key, &first.value)
        })
    }

    pub fn last_key_value(&self) -> Option<(&K, &V)> {
        self.root.as_ref().map(|r| {
            let first = r.rightmost_descendant();
            (&first.key, &first.value)
        })
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
            let z_ref = unsafe { z.as_ref() };
            if z_node.parent.is_none() {
                break;
            }
            let z_p: &mut Node<K, V> = unsafe { z_node.unwrap_parent().as_mut() };

            if z_p.is_black() || z_p.parent.is_none() { break; }

            let z_p_p: &mut Node<K, V> = unsafe { z_p.unwrap_parent().as_mut() };

            if z_p_p.left.is_some() && std::ptr::eq(z_p, z_p_p.left.as_ref().unwrap().as_ref()) {
                // z.parent is left child
                let maybe_z_p_p_r = z_p_p.right.as_mut();
                if maybe_z_p_p_r.is_some() && maybe_z_p_p_r.as_ref().unwrap().is_red() {
                    let y = maybe_z_p_p_r.unwrap();
                    z_p.color = NodeColor::Black;
                    y.color = NodeColor::Black;
                    z_p_p.color = NodeColor::Red;
                    z = z_p.parent.unwrap();
                } else {
                    // z.parent doesn't have a red sibling
                    if z_p.right.is_some() && std::ptr::eq(z_ref, &**z_p.right.as_ref().unwrap()) {
                        z = z_ref.parent.unwrap();
                        self.rotate_left(z);
                    }
                    z_p.color = NodeColor::Black;
                    z_p_p.color = NodeColor::Red;
                    self.rotate_right(unsafe { NonNull::new_unchecked(z_p_p) });
                }
            } else {
                // z.parent is right child
                let maybe_z_p_p_l = z_p_p.left.as_mut();
                if maybe_z_p_p_l.is_some() && maybe_z_p_p_l.as_ref().unwrap().is_red() {
                    let y = maybe_z_p_p_l.unwrap();
                    z_p.color = NodeColor::Black;
                    y.color = NodeColor::Black;
                    z_p_p.color = NodeColor::Red;
                    z = z_p.parent.unwrap();
                } else {
                    if z_p.left.is_some() && std::ptr::eq(z_ref, &**z_p.left.as_ref().unwrap()) {
                        z = z_ref.parent.unwrap();
                        self.rotate_right(z);
                    }
                    z_p.color = NodeColor::Black;
                    z_p_p.color = NodeColor::Red;
                    self.rotate_left(unsafe { NonNull::new_unchecked(z_p_p) });
                }
            }
        }
        self.root.as_mut().unwrap().color = NodeColor::Black;
    }

    fn rotate_left(&mut self, mut x_ptr: NonNull<Node<K, V>>) {
        let x_ref = unsafe { x_ptr.as_mut() };
        let mut y = x_ref.right.take().expect("Only call rotate_left on a node with a right child");
        let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut()) };
        x_ref.right = y.left.take();
        if let Some(ref mut x_right) = x_ref.right {
            x_right.parent = Some(x_ptr);
        }

        y.parent = x_ref.parent;

        match x_ref.parent {
            None => {
                let x = self.root.take().unwrap();
                self.root = Some(y);
                self.root.as_mut().unwrap().left = Some(x);
            },
            Some(mut x_parent_ptr) => {
                let x_parent = unsafe { x_parent_ptr.as_mut() };
                if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x_ref) }) {
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
        x_ref.parent = Some(y_ptr);
    }

    //   x      y
    //  /   =>   \
    // y          x

    fn rotate_right(&mut self, mut x_ptr: NonNull<Node<K, V>>) {
        let x_ref = unsafe { x_ptr.as_mut() };
        let mut y = x_ref.left.take().expect("Only call rotate_right on a node with a left child");
        let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut()) };
        x_ref.left = y.right.take();
        if let Some(ref mut x_left) = x_ref.left {
            x_left.parent = Some(x_ptr);
        }

        y.parent = x_ref.parent;

        match x_ref.parent {
            None => {
                let x = self.root.take().unwrap();
                self.root = Some(y);
                self.root.as_mut().unwrap().right = Some(x);
            },
            Some(mut x_parent_ptr) => {
                // if x_parent.left is x, set left to y, else set right to y
                let x_parent = unsafe { x_parent_ptr.as_mut() };
                if x_parent.left.as_ref().map_or(false, |nbox| { std::ptr::eq(&**nbox, x_ref) }) {
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
        x_ref.parent = Some(y_ptr);
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        let mut prev: Option<&Box<Node<K, V>>> = None;
        let mut curr = self.root.as_ref();
        while let Some(node) = curr {
            prev = curr;
            curr = node.left.as_ref();
        }
        Iter {
            curr: prev.map(|n| &**n),
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut curr = &mut self.root;
        let target: Option<*mut Box<Node<K, V>>> =  loop {
            match curr {
                None => break None,
                Some(node) if key == &node.key => break Some(node),
                Some(node) => {
                    curr = if key < &node.key {
                        &mut node.left
                    } else {
                        &mut node.right
                    }
                }
            }
        };

        if target.is_none() {
            return None;
        }

        unsafe {
            let z: *mut Node<K, V> = &mut **target.unwrap();
            let mut y_orig_color = (*z).color;

            let removed: Option<V> = Some(std::ptr::read(&(*z).value));
            // Transplant(u, v) is taking ownership of v, getting the parent of u, taking u out of u.parent and replacing it with v, and then returning v
            let source_ptr: Option<*mut Node<K, V>>;
            if (*z).left.is_none() {
                let mut source = (*z).right.take();
                source_ptr = self.transplant(&*z, source);
            } else if (*z).right.is_none() {
                let mut source = (*z).left.take();
                source_ptr = self.transplant(&*z, source);
            } else {
                let mut z_right = (*z).right.take();
                let z_right_ref = z_right.as_mut().map(|n| &mut **n).unwrap();

                // TODO: need to take ownership of the leftmost descendant of z.right
                // let mut y = z_right_ref.take_leftmost_descendant();

                let mut curr: *mut Node<K, V> = z_right_ref;
                let mut curr_parent: *mut Node<K, V> = &mut *z;
                while let Some(ref mut next) = (*curr).left {
                    let next_node = &mut **next;
                    curr_parent = curr;
                    curr = next_node;
                }

                let mut y = (*curr_parent).left.take().unwrap();

                y_orig_color = y.color;
                let mut source = y.right.take();
                if !std::ptr::eq(&*y, z_right_ref) {
                    self.transplant(&*y, source);
                    z_right_ref.parent = Some(NonNull::new_unchecked(&mut *y));
                    y.right = z_right;
                } else {
                    if let Some(source_node) = source.as_mut() {
                        source_node.parent = Some(NonNull::new_unchecked(&mut *y));
                    }
                }
                let removed_node = self.transplant(&*z, Some(y));
                panic!("Not implemented");
            }


            // z == curr
            if y_orig_color == NodeColor::Black {
                self.delete_fixup(source_ptr);
            }
            self.size -= 1;
            removed
        }
    }

    fn transplant<'a, 'b>(&'a mut self, u: &'b Node<K, V>, v: Option<Box<Node<K, V>>>) -> Option<*mut Node<K, V>> {
        let target: &mut Option<Box<Node<K, V>>>;
        match u.parent {
            None => {
                self.root = v;
                if let Some(ref mut n) = self.root {
                    n.parent = None;
                }
                target = &mut self.root;
            },
            Some (mut u_p) => {
                let u_p = unsafe { u_p.as_mut() };
                if u_p.left.is_some() && std::ptr::eq(u, &**u_p.left.as_ref().unwrap()) {
                    u_p.left = v;
                    if let Some(ref mut n) = u_p.left {
                        n.parent = u.parent;
                    }
                    target = &mut u_p.left;
                } else {
                    u_p.right = v;
                    if let Some(ref mut n) = u_p.right {
                        n.parent = u.parent;
                    }
                    target = &mut u_p.right;
                }
            }
        }
        target.as_mut().map(|n| &mut **n as *mut Node<K, V>)
    }


    fn delete_fixup(&mut self, mut x: Option<*mut Node<K, V>>) {
        if self.root.is_none() {
            return;
        }
        let root_node = self.root.as_ref().map(|n| &**n as *const Node<K, V>).unwrap();
        unsafe {
            while let Some(x_node) = x {
                if !std::ptr::eq(root_node, x_node) || (*x_node).color == NodeColor::Black {
                    break;
                }

                let x_parent_node = (*x_node).parent.unwrap().as_mut();
                if x_parent_node.left.is_some() && std::ptr::eq(x_node, x_parent_node.left.as_ref().map(|n| &**n).unwrap()) {
                    // x is left child
                    let mut w = x_parent_node.right.as_mut().map(|n| &mut **n as *mut Node<K, V>);
                    if let Some(w_node) = w {
                        if (*w_node).color == NodeColor::Red {
                            (*w_node).color = NodeColor::Black;
                            x_parent_node.color = NodeColor::Red;
                            self.rotate_left(NonNull::new_unchecked(x_parent_node));
                            w = x_parent_node.right.as_mut().map(|n| &mut **n as *mut Node<K, V>);
                        }

                        if ((*w_node).left.is_none() || (*w_node).left.as_ref().unwrap().color == NodeColor::Black)
                            && ((*w_node).right.is_none() || (*w_node).right.as_ref().unwrap().color == NodeColor::Black) {
                            (*w_node).color = NodeColor::Red;
                            x = Some(x_parent_node);
                        } else {
                            if (*w_node).right.is_none() || (*w_node).right.as_ref().unwrap().color == NodeColor::Black {
                                if let Some(w_left) = (*w_node).left.as_mut() {
                                    w_left.color = NodeColor::Black;
                                }
                                (*w_node).color = NodeColor::Red;
                                self.rotate_right(NonNull::new_unchecked(w_node));
                                w = x_parent_node.right.as_mut().map(|n| n.as_mut() as *mut Node<K, V>);
                            }

                            (*w_node).color = x_parent_node.color;
                            (*x_parent_node).color = NodeColor::Black;
                            if let Some(w_right) = (*w_node).right.as_mut() {
                                w_right.color = NodeColor::Black;
                            }
                            self.rotate_left(NonNull::new_unchecked(x_parent_node));
                            x = self.root.as_mut().map(|n| &mut **n as *mut Node<K, V>);
                        }
                    } else {
                        panic!("w is None");
                    }
                } else {
                    // x is right child
                    panic!("(fixup) right child not implemented");
                }
            }
        }
        if let Some(x_node) = x {
            unsafe {
                (*x_node).color = NodeColor::Black;
            }
        }
    }


}



impl<'a, K: Ord + Debug, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let head = self.curr.take();
        if let Some(node) = head {
            // find the next node, if there is one, and add it to nodes
            // two ways for next node:
            //  - it's the leftmost descendant of the right child
            //  - it's the first right ancestor
            let maybe_rc_lm_desc = node.right.as_ref().map(|rc| rc.leftmost_descendant());
            if let Some(rc_lm_desc) = maybe_rc_lm_desc {
                self.curr = Some(rc_lm_desc);
            } else {
                if let Some(frs_anc) = node.first_right_strict_ancestor() {
                    self.curr = Some(frs_anc);
                }
            }

        }
        head.map(|node| (&node.key, &node.value))
    }
}
