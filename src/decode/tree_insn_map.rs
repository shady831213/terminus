use super::{InsnMap, Instruction, Decoder};
use std::cell::RefCell;
use super::execption::*;
use terminus_global::{InsnT, insn_len};

struct TreeNode<T> {
    left: Option<*mut TreeNode<T>>,
    right: Option<*mut TreeNode<T>>,
    level: usize,
    value: Option<T>,
}

impl<T> TreeNode<T> {
    fn new(level: usize) -> TreeNode<T> {
        TreeNode {
            left: None,
            right: None,
            level: level,
            value: None,
        }
    }

    fn insert(&mut self, key: InsnT, value: T) {
        if self.level == insn_len() {
            if self.value.is_some() {
                panic!(format!("duplicate definition! 0x{:x}", key))
            }
            self.value = Some(value)
        } else {
            let node = if key & ((1 as InsnT) << self.level as InsnT) == 0 {
                self.left.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            } else {
                self.right.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            };
            unsafe {
                (*node).as_mut().unwrap().insert(key, value)
            }
        }
    }

    fn get_node(node: *mut Self) -> *mut Self {
        let n = unsafe {
            node.as_mut().unwrap()
        };
        if n.left.is_some() && n.right.is_none() {
            Self::get_node(n.left.unwrap())
        } else if n.right.is_some() && n.left.is_none() {
            Self::get_node(n.right.unwrap())
        } else {
            node
        }
    }

    fn get(&mut self, key: InsnT) -> Option<&T> {
        //naturally impl mask
        let path_compress = |node: &mut Option<*mut Self>| {
            if let Some(ref n) = node {
                let new_node = Self::get_node(*n);
                if *n != new_node {
                    *node = Some(new_node)
                }
            }
        };
        if self.level == insn_len() {
            self.value.as_ref()
        } else {
            if let Some(node) = if key & ((1 as InsnT) << self.level as InsnT) == 0 {
                path_compress(&mut self.left);
                self.left
            } else {
                path_compress(&mut self.right);
                self.right
            } {
                unsafe {
                    node.as_mut().unwrap().get(key)
                }
            } else {
                None
            }
        }
    }
}

pub struct TreeInsnMap(RefCell<TreeNode<Box<dyn Decoder>>>);

impl TreeInsnMap {
    pub fn new() -> TreeInsnMap {
        TreeInsnMap(RefCell::new(TreeNode::new(0)))
    }
}

impl InsnMap for TreeInsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T) {
        self.0.get_mut().insert(decoder.code(), Box::new(decoder))
    }

    fn decode(&self, ir: InsnT) -> Result<Instruction, Exception> {
        if let Some(decoder) = self.0.borrow_mut().get(ir) {
            if ir & decoder.mask() != decoder.code() {
                Err(Exception::IllegalInsn(ir))
            } else {
                Ok(decoder.decode(ir))
            }
        } else {
            Err(Exception::IllegalInsn(ir))
        }
    }
}

#[test]
fn test_insert() {
    let mut tree: TreeNode<u32> = TreeNode::new(0);
    tree.insert(7, 7);
    assert_eq!(*tree.get(7).unwrap(), 7);
    assert_eq!(tree.get(8), None);
}