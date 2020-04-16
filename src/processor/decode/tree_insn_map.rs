use super::{InsnMap, Instruction, Decoder};
use terminus_global::{InsnT, insn_len};
use crate::processor::trap::Exception;

struct TreeNode<T> {
    left: Option<*mut TreeNode<T>>,
    right: Option<*mut TreeNode<T>>,
    level: usize,
    value: Option<(T, Box<dyn Fn(InsnT, &T) -> bool + 'static>)>,
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

    fn insert<F: Fn(InsnT, &T) -> bool + 'static>(&mut self, key: InsnT, value: T, f:F) -> Option<&T> {
        if self.level == insn_len() {
            if let Some((ref v,_)) = self.value {
                Some(v)
            } else {
                self.value = Some((value, Box::new(f)));
                None
            }
        } else {
            let node = if key & ((1 as InsnT) << self.level as InsnT) == 0 {
                self.left.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            } else {
                self.right.get_or_insert(Box::into_raw(Box::new(Self::new(self.level + 1))))
            };
            unsafe {
                (*node).as_mut().unwrap().insert(key, value, f)
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
            return self.value.iter().find(|(v, m)| { (*m)(key, v) }).map(|(v, _)|{v});
        } else {
            if key & ((1 as InsnT) << self.level as InsnT) == 0 {
                if let Some(n) = self.left {
                    if let Some(v) = unsafe { n.as_ref().unwrap().get(key) } {
                        return Some(v);
                    }
                }
                if let Some(n) = self.right {
                    if let Some(v) = unsafe { n.as_ref().unwrap().get(key) } {
                        return Some(v);
                    }
                }
            } else {
                if let Some(n) = self.right {
                    if let Some(v) = unsafe { n.as_ref().unwrap().get(key) } {
                        return Some(v);
                    }
                }
                if let Some(n) = self.left {
                    if let Some(v) = unsafe { n.as_ref().unwrap().get(key) } {
                        return Some(v);
                    }
                }
            }
        }
        None
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
        let name = decoder.name();
        let code = decoder.code();
        let mask = decoder.mask();
        if let Some(v) = self.0.insert(decoder.code(), Box::new(decoder), |key, dec| { key & dec.mask() == dec.code() }) {
            panic!(format!("inst {}(code = {:#x}; mask = {:#x}) is duplicated with inst {}(code = {:#x}; mask = {:#x})!", name, code, mask,v.name(), v.code(), v.mask()))
        }
    }

    fn decode(&self, ir: InsnT) -> Result<Instruction, Exception> {
        if let Some(decoder) = self.0.get(ir) {
            Ok(decoder.decode(ir))
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
    tree.insert(7, 7, |k, v| { k == *v });
    tree.compress();
    assert_eq!(*tree.get(7).unwrap(), 7);
    assert_eq!(tree.get(8), None);
}