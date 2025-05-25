use std::cell::{Cell, RefCell};
use std::cmp::{max, min};
use std::ops::{Add, Sub, SubAssign};
use rand::Rng;
use std::mem;

/// A treap list is a data structure that combines the properties of a binary search tree and a heap.
/// It allows for efficient insertion, deletion, and range queries. (O(log n) on average)
#[derive(Debug, Clone)]
pub struct TreapList<V>
where
    V: Add<Output = V> + Clone + Default,
{

    root: Option<Box<Node<V>>>,
}

#[derive(Debug, Clone)]
struct Node<V>
where
    V: Add<Output = V> + Clone + Default,
{
    pub value: V,
    pub priority: u64,
    pub left: Option<Box<Self>>,
    pub right: Option<Box<Self>>,
    pub tree_sum: RefCell<V>,
    pub size: usize,
    pub lazy: Cell<bool>,
}

#[allow(dead_code)]
impl<V> Node<V>
where
    V: Add<Output = V> + Clone + Default,
{
   fn new(value: V, rng: &mut impl Rng) -> Self {
        let priority = rng.random();
        Node {
            value: value.clone(),
            priority,
            left: None,
            right: None,
            tree_sum: RefCell::new(value),
            size: 1,
            lazy: Cell::new(false),
        }
    }

    fn left_size(&self) -> usize {
        self.left.as_ref().map_or(0, |node| node.size)
    }

    fn right_size(&self) -> usize {
        self.right.as_ref().map_or(0, |node| node.size)
    }

    fn left_sum(&self) -> V {
        self.left.as_ref().map_or(V::default(), |node| node.get_tree_sum())
    }

    fn right_sum(&self) -> V {
        self.right.as_ref().map_or(V::default(), |node| node.get_tree_sum())
    }

    fn update_size(&mut self) {
        self.size = 1 + self.left_size() + self.right_size();
    }

    fn update_tree_sum(&self) {
        self.tree_sum.replace(
            self.value.clone() + self.left_sum() + self.right_sum()
        );
    }

    fn update_all(&mut self) {
        self.update_size();
        self.lazy.set(true);
    }

    fn get_tree_sum(&self) -> V {
        if self.lazy.get() {
            self.update_tree_sum();
            self.lazy.set(false);
        }
        self.tree_sum.borrow().clone()
    }

    pub fn insert_after_k_nodes(
        &mut self,
        mut new_node: Box<Self>,
        k: usize)
    {
        assert!(k <= self.size, "k must be less than or equal to the size of the node");
        let left_size = self.left_size();
        if k == left_size {
            new_node.left = self.left.take();
            mem::swap(self, &mut new_node);
            self.right = Some(new_node);
            self.right.as_mut().unwrap().update_all();
            self.update_all();
            return;
        }

        #[derive(PartialEq)]
        enum InsertPosition {
            Left,
            Right,
        }
        let insert_position;
        if k < left_size {
            insert_position = InsertPosition::Left;
            if let Some(left_node) = self.left.as_mut() {
                left_node.insert_after_k_nodes(new_node, k);
            } else {
                self.left = Some(new_node);
            }
        }
        else {
            insert_position = InsertPosition::Right;
            if let Some(right_node) = self.right.as_mut() {
                right_node.insert_after_k_nodes(new_node, k - left_size - 1);
            } else {
                self.right = Some(new_node);
            }
        }

        if insert_position == InsertPosition::Left && self.left.as_ref().unwrap().priority > self.priority {
            let mut l = self.left.take().unwrap();
            let lr = l.right.take();
            self.left = lr;
            mem::swap(self, &mut l);
            self.right = Some(l);
            self.right.as_mut().unwrap().update_all();
        }
        else if insert_position == InsertPosition::Right && self.right.as_ref().unwrap().priority > self.priority {
            let mut r = self.right.take().unwrap();
            let rl = r.left.take();
            self.right = rl;
            mem::swap(self, &mut r);
            self.left = Some(r);
            self.left.as_mut().unwrap().update_all();
        }


        self.update_all();
    }

    pub fn get_kth_node(&self, k: usize) -> Option<&Self> {
        assert!(k < self.size, "k must be less than the size of the node");
        let left_size = self.left_size();
        if k < left_size {
            self.left.as_ref()?.get_kth_node(k)
        } else if k == left_size {
            Some(self)
        } else {
            self.right.as_ref()?.get_kth_node(k - left_size - 1)
        }
    }

    pub fn replace_kth_node(&mut self, k: usize, new_value: V) {
        assert!(k < self.size, "k must be less than the size of the node");
        let left_size = self.left_size();
        if k < left_size {
            self.left.as_mut().unwrap().replace_kth_node(k, new_value);
        } else if k == left_size {
            self.value = new_value;
        } else {
            self.right.as_mut().unwrap().replace_kth_node(k - left_size - 1, new_value);
        }
        self.update_tree_sum();
    }

    fn merge(
        left: Option<Box<Self>>,
        right: Option<Box<Self>>,
    ) -> Option<Box<Self>> {
        if left.is_none() {
            return right;
        }
        if right.is_none() {
            return left;
        }
        let mut left_node = left.unwrap();
        let mut right_node = right.unwrap();
        if left_node.priority <= right_node.priority {
            left_node.right = Self::merge(left_node.right, Some(right_node));
            left_node.update_all();
            Some(left_node)
        }
        else
        {
            right_node.left = Self::merge(Some(left_node), right_node.left);
            right_node.update_all();
            Some(right_node)
        }

    }

    pub fn remove_range(
        &mut self,
        range: std::ops::Range<usize>,
    )
    {
        //! Removes the range [start, end) from the treap list.
        let start = range.start;
        let end = range.end;
        assert!(start <= end && end <= self.size, "Invalid range for remove");
        assert!(start > 0 || end < self.size, "Cannot remove the entire range");
        if start == end {
            return; // Nothing to remove
        }
        let left_size = self.left_size();
        let right_size = self.right_size();
        let left_range = min(start, left_size)..min(end, left_size);
        let mut right_range = max(start, left_size + 1)..max(end, left_size + 1);
        right_range.start -= left_size + 1;
        right_range.end -= left_size + 1;

        if left_range.start == 0 && left_range.end == left_size {
            self.left = None;
        }
        else if let Some(left_node) = self.left.as_mut()
        {
            left_node.remove_range(left_range);
        }

        if right_range.start == 0 && right_range.end == right_size {
            self.right = None;
        }
        else if let Some(right_node) = self.right.as_mut()
        {
            right_node.remove_range(right_range);
        }

        if start <= left_size && end > left_size {
            if self.left.is_none()
            {
                let mut r = self.right.take();
                mem::swap(self, r.as_mut().unwrap());
            }
            else if self.right.is_none()
            {
                let mut l = self.left.take();
                mem::swap(self, l.as_mut().unwrap());
            }
            else
            {
                let l = self.left.take();
                let r = self.right.take();
                let mut new_node = Self::merge(l, r);
                mem::swap(self, new_node.as_mut().unwrap());
            }
        }
        else
        {
            self.update_all();
        }
    }

    pub fn sum_range(&self, range: std::ops::Range<usize>) -> V {
        //! Returns the sum of values in the range [start, end)

        let start = range.start;
        let end = range.end;
        
        assert!(start <= end && end <= self.size, "Invalid range for sum");
        if start == end {
            return V::default();
        }
        if start == 0 && end == self.size {
            return self.get_tree_sum();
        }
        let left_size = self.left_size();
        if end <= left_size {
            self.left.as_ref().map_or(V::default(), |node| node.sum_range(start..end))
        } else if start >= left_size + 1 {
            self.right.as_ref().map_or(V::default(), |node| 
            node.sum_range((start - left_size - 1)..(end - left_size - 1)))
        } else {
            let left_sum = self.left.as_ref().map_or(V::default(), 
            |node| node.sum_range(start..left_size));
            let right_sum = self.right.as_ref().map_or(V::default(), 
            |node| node.sum_range(0..(end - left_size - 1)));
            left_sum + self.value.clone() + right_sum
        }
    }
}

#[allow(dead_code)]
impl<V> TreapList<V>
where
V: Add<Output = V> + Clone + Default,
{
    pub fn new() -> Self {
        TreapList { root: None }
    }

    /// Inserts a new value at the end of the treap list.
    pub fn push(&mut self, value: V) {
        let mut rng = rand::rng();
        let new_node = Box::new(Node::new(value, &mut rng));
        if let Some(root) = self.root.as_mut() {
            root.insert_after_k_nodes(new_node, root.size);
        } else {
            self.root = Some(new_node);
        }
    }

    /// Inserts a new value after the k-th node in the treap list.
    pub fn insert_after_k_nodes(&mut self, value: V, k: usize) {
        let mut rng = rand::rng();
        let new_node = Box::new(Node::new(value, &mut rng));
        if let Some(root) = self.root.as_mut() {
            root.insert_after_k_nodes(new_node, k);
        } else {
            self.root = Some(new_node);
        }
    }

    /// Removes the k-th node from the treap list.
    pub fn get(&self, k: usize) -> Option<&V> {
        self.root.as_ref()?.get_kth_node(k).map(|node| &node.value)
    }

    /// Replaces the k-th node with a new value.
    pub fn replace(&mut self, k: usize, new_value: V) {
        assert!(k < self.root.as_ref().map_or(0, |node| node.size), "k must be less than the size of the treap");
        if let Some(root) = self.root.as_mut() {
            root.replace_kth_node(k, new_value);
        }
    }

    /// Removes the k-th node from the treap list.
    pub fn remove_range(&mut self, range: std::ops::Range<usize>) {
        assert!(range.start <= range.end, "Invalid range for remove");
        assert!(range.end <= self.root.as_ref().map_or(0, |node| node.size), "Range end exceeds size of the treap");
        if let Some(root) = self.root.as_mut() {
            if range.start == 0 && range.end == root.size {
                self.root = None; // Remove entire treap
            } else {
                root.remove_range(range);
            }
        }
    }

    /// Sum of values in the range [start, end)
    pub fn sum_range(&self, range: std::ops::Range<usize>) -> V {
        assert!(range.start <= range.end, "Invalid range for sum");
        assert!(range.end <= self.root.as_ref().map_or(0, |node| node.size), "Range end exceeds size of the treap");
        self.root.as_ref().map_or(V::default(), |node| node.sum_range(range))
    }

    /// Returns the number of nodes in the treap list.
    pub fn size(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.size)
    }

    /// Tests if the treap list is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<V> Default for TreapList<V>
where
    V: Add<Output = V> + Clone + Default,
{
    fn default() -> Self {
        TreapList::new()
    }
}

impl<V> FromIterator<V>  for TreapList<V>
where
    V: Add<Output = V> + Clone + Default,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let mut treap_list = TreapList::new();
        for value in iter {
            treap_list.push(value);
        }
        treap_list
    }
}

impl<V> Iterator for TreapList<V>
where
    V: Add<Output = V> + Sub<Output = V> + SubAssign + Clone + Default,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }
        let value = self.get(0)
            .expect("TreapList is not empty, but get(0) returned None").clone();
        self.remove_range(0..1);
        Some(value)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    
    #[test]
    fn test_treap_list1() {
        let mut tl = vec![1, 2, 3, 4, 5].into_iter().collect::<TreapList<i32>>();
        // 1, 2, 3, 4, 5
        assert_eq!(tl.size(), 5);
        assert_eq!(tl.sum_range(0..5), 15);
        assert_eq!(tl.get(2).unwrap().clone(), 3);
        assert_eq!(tl.sum_range(1..4), 9);
        tl.push(6);
        assert_eq!(tl.size(), 6);
        tl.push(7);
        assert_eq!(tl.size(), 7);
        // 1, 2, 3, 4, 5, 6, 7
        assert_eq!(tl.sum_range(4..6), 11); // 5 + 6 
        tl.remove_range(2..5);
        // 1, 2, 6, 7
        assert_eq!(tl.size(), 4);
        assert_eq!(tl.sum_range(0..3), 9); // 1 + 2 + 6
        tl.remove_range(0..1);
        // 2, 6, 7
        assert_eq!(tl.sum_range(1..3), 13); // 6 + 7
        tl.remove_range(0..3);
        // Empty treap
        assert!(tl.is_empty());
    }

    #[test]
    fn test_treap_list2() {
        // Test with an empty treap
        let mut tl = TreapList::<i32>::new();
        assert!(tl.is_empty());
        assert_eq!(tl.size(), 0);
        assert_eq!(tl.sum_range(0..0), 0);
        
        // Insert elements at specific positions
        tl.push(10);
        tl.insert_after_k_nodes(20, 1); // Insert after first element
        tl.insert_after_k_nodes(15, 1); // Insert between 10 and 20
        // Now: [10, 15, 20]
        assert_eq!(tl.size(), 3);
        assert_eq!(tl.sum_range(0..3), 45);
        assert_eq!(tl.get(1).unwrap().clone(), 15);
        
        // Replace elements
        tl.replace(0, 5);  // Replace 10 with 5
        tl.replace(2, 25); // Replace 20 with 25
        // Now: [5, 15, 25]
        assert_eq!(tl.sum_range(0..3), 45);
        assert_eq!(tl.get(0).unwrap().clone(), 5);
        assert_eq!(tl.get(2).unwrap().clone(), 25);
        
        // Test iterator
        let mut iter_tl = tl.clone();
        assert_eq!(iter_tl.next(), Some(5));
        assert_eq!(iter_tl.next(), Some(15));
        assert_eq!(iter_tl.next(), Some(25));
        assert_eq!(iter_tl.next(), None);
        assert!(iter_tl.is_empty());
        
        // More complex operations
        tl.push(30);
        tl.push(35);
        tl.push(40);
        // Now: [5, 15, 25, 30, 35, 40]
        
        // Remove from middle
        tl.remove_range(2..4);
        // Now: [5, 15, 35, 40]
        assert_eq!(tl.size(), 4);
        assert_eq!(tl.sum_range(0..4), 95);
        
        // Insert after removing
        tl.insert_after_k_nodes(20, 1);
        // Now: [5, 15, 20, 35, 40]
        assert_eq!(tl.size(), 5);
        assert_eq!(tl.sum_range(0..5), 115);
        
        // Test partial sums
        assert_eq!(tl.sum_range(1..4), 70);  // 15 + 20 + 35
        assert_eq!(tl.sum_range(3..5), 75);  // 35 + 40
        
        // Test large number of elements
        let mut large_tl = TreapList::new();
        for i in 0..100 {
            large_tl.push(i);
        }
        assert_eq!(large_tl.size(), 100);
        assert_eq!(large_tl.sum_range(0..100), (0..100).sum::<i32>());
        assert_eq!(large_tl.sum_range(25..75), (25..75).sum::<i32>());
        
        // Complex test with different operations
        let mut complex_tl = TreapList::new();
        // Build treap with insertion at different positions
        complex_tl.push(50);  // [50]
        complex_tl.insert_after_k_nodes(30, 0);  // [30, 50]
        complex_tl.insert_after_k_nodes(70, 0);  // [70, 30, 50]
        complex_tl.insert_after_k_nodes(10, 2);  // [70, 30, 10, 50]
        complex_tl.insert_after_k_nodes(90, 1);  // [70, 90, 30, 10, 50]
        
        assert_eq!(complex_tl.size(), 5);
        assert_eq!(complex_tl.sum_range(0..5), 250);
        
        // Test range sums in a complex structure
        assert_eq!(complex_tl.sum_range(1..4), 130);  // 90 + 30 + 10
        
        // Remove and insert in complex patterns
        complex_tl.remove_range(1..3);  // [70, 10, 50]
        complex_tl.insert_after_k_nodes(20, 2);  // [70, 10, 20, 50]
        complex_tl.push(40);  // [70, 10, 20, 50, 40]
        
        assert_eq!(complex_tl.size(), 5);
        assert_eq!(complex_tl.sum_range(0..5), 190);
    }
}