use super::{InsnMap, Instruction, Decoder};
use terminus_global::{InsnT, insn_len};
use crate::processor::trap::Exception;

struct TreeNode<T> {
    left: Option<*mut TreeNode<T>>,
    right: Option<*mut TreeNode<T>>,
    level: usize,
    mask: bool,
    value: Option<T>,
}

impl<T> TreeNode<T> {
    fn new(level: usize) -> TreeNode<T> {
        TreeNode {
            left: None,
            right: None,
            level: level,
            mask: false,
            value: None,
        }
    }

    fn insert(&mut self, key: InsnT, mask: InsnT, value: T) {
        if self.level == insn_len() {
            if self.value.is_some() {
                panic!(format!("duplicate definition! 0x{:x}", key))
            }
            self.value = Some(value)
        } else {
            self.mask = mask & ((1 as InsnT) << self.level as InsnT) != 0;
            let node = if key & ((1 as InsnT) << self.level as InsnT) == 0 {
                self.left.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            } else {
                self.right.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            };
            unsafe {
                (*node).as_mut().unwrap().insert(key, mask, value)
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

    fn compress(&mut self) {
        let path_compress = |node: &mut Option<*mut Self>| {
            if let Some(ref n) = node {
                let new_node = Self::get_node(*n);
                if *n != new_node {
                    *node = Some(new_node)
                }
            }
        };
        if self.level == insn_len() {
            return;
        } else {
            path_compress(&mut self.left);
            path_compress(&mut self.right);
            self.left.iter_mut().for_each(|n| { unsafe { n.as_mut().unwrap().compress() } });
            self.right.iter_mut().for_each(|n| { unsafe { n.as_mut().unwrap().compress() } });
        }
    }

    fn get(&self, key: InsnT) -> Option<&T> {
        if self.level == insn_len() {
            self.value.as_ref()
        } else {
            if let Some(node) = if (key & ((1 as InsnT) << self.level as InsnT) == 0) || !self.mask {
                self.left
            } else {
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

pub struct TreeInsnMap(TreeNode<Box<dyn Decoder>>);

impl TreeInsnMap {
    pub fn new() -> TreeInsnMap {
        TreeInsnMap(TreeNode::new(0))
    }
}

impl InsnMap for TreeInsnMap {
    fn registery<T: 'static + Decoder>(&mut self, decoder: T) {
        self.0.insert(decoder.code(), decoder.mask(), Box::new(decoder))
    }

    fn decode(&self, ir: InsnT) -> Result<Instruction, Exception> {
        if let Some(decoder) = self.0.get(ir) {
            if ir & decoder.mask() != decoder.code() {
                Err(Exception::IllegalInsn(ir))
            } else {
                Ok(decoder.decode(ir))
            }
        } else {
            Err(Exception::IllegalInsn(ir))
        }
    }
    fn lock(&mut self) {
        self.0.compress();
    }
}

//immutable after 'lock'
unsafe impl Sync for TreeInsnMap {}

//immutable after 'lock'
unsafe impl Send for TreeInsnMap {}

#[test]
fn test_insert() {
    let mut tree: TreeNode<u32> = TreeNode::new(0);
    tree.insert(7, 7, 7);
    tree.compress();
    assert_eq!(*tree.get(7).unwrap(), 7);
    assert_eq!(tree.get(8), None);
}