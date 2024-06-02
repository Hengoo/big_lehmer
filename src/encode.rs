use dashu::integer::UBig;

/// Naive approach would create a list [0..N]
/// and on insert(number) find the index of the number, then remove it
/// This constructs a binary tree which node weights are adjusted with on insertion
#[derive(Debug)]
pub(crate) struct EncodeAS {
    tree: Vec<u32>,
}

impl EncodeAS {
    pub(crate) fn new(element_count: u32) -> Self {
        let len = element_count.next_power_of_two();
        EncodeAS {
            tree: vec![0; len.try_into().unwrap()],
        }
    }

    fn _left_child_id(node_id: u32) -> u32 {
        let zeroes = node_id.trailing_zeros();
        node_id - (1 << (zeroes - 1))
    }

    fn _right_child_id(node_id: u32) -> u32 {
        let zeroes = node_id.trailing_zeros();
        node_id + (1 << (zeroes - 1))
    }

    fn _parent_id(node_id: u32) -> u32 {
        let zeroes = node_id.trailing_zeros();
        let tmp = (node_id >> zeroes) & 3;
        match tmp {
            // Move least significant bit by one to left
            1 => node_id + (1 << zeroes),
            // Remove least significant bit
            3 => node_id & (node_id - 1),
            _ => unreachable!(),
        }
    }

    pub(crate) fn insert(&mut self, number: u32) -> u32 {
        let mut result = number;
        let element_count = u32::try_from(self.tree.len()).unwrap();
        let mut node = element_count / 2;
        let mut jump = element_count / 4;

        loop {
            if number >= node {
                result -= self.tree[node as usize];
                node += jump;
            } else {
                self.tree[node as usize] += 1;
                node -= jump;
            }
            if jump == 0 {
                break;
            }
            jump /= 2;
        }
        result
    }
}

/// Cache combines several steps of the encode loop to use more "small" numbers to minimize the cost of big number math
/// It stores a running add and running mul.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Cache {
    pub(crate) add: u64,
    pub(crate) mul: u64,
}

impl Cache {
    pub(crate) fn default() -> Self {
        Cache { add: 0, mul: 1 }
    }

    pub(crate) fn new(add: u64, mul: u64) -> Self {
        let new_add = add.checked_mul(mul).unwrap();
        Cache { add: new_add, mul }
    }

    pub(crate) fn add(&mut self, add: u64, mul: u64) -> Option<()> {
        let mut tmp = self.add.checked_add(add)?;
        tmp = tmp.checked_mul(mul)?;

        self.mul = self.mul.checked_mul(mul)?;
        self.add = tmp;
        Some(())
    }
}

/// The big number variant of the above `Cache` that is used during map reduce
#[derive(Debug)]
pub(crate) struct BigCache {
    pub(crate) add: UBig,
    pub(crate) mul: UBig,
}

impl BigCache {
    pub(crate) fn new(cache: &Cache) -> Self {
        Self {
            add: cache.add.into(),
            mul: cache.mul.into(),
        }
    }

    // -> reduce identity
    pub(crate) fn identity() -> Self {
        Self {
            add: UBig::ZERO,
            mul: UBig::ONE,
        }
    }

    // -> reduce combine
    pub(crate) fn combine(left: Self, right: Self) -> Self {
        Self {
            add: (left.add * &right.mul) + right.add,
            mul: left.mul * right.mul,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_as_helper(numbers: &[u32]) -> Box<[u32]> {
        let mut t = EncodeAS::new(u32::try_from(numbers.len()).unwrap());

        let mut result = vec![0u32; numbers.len()].into_boxed_slice();
        for (&number, r) in numbers.iter().zip(result.iter_mut()) {
            *r = t.insert(number);
        }
        result
    }

    #[test]
    fn test_encode_as_0() {
        let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
        let encoded = encode_as_helper(&sequence);
        assert_eq!(*encoded, [7, 2, 0, 4, 3, 0, 1, 0]);
    }

    #[test]
    fn test_encode_as_1() {
        let sequence = [0, 1, 2, 3, 4, 5, 6, 7];
        let encoded = encode_as_helper(&sequence);
        assert_eq!(*encoded, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_encode_as_2() {
        let sequence = [7, 6, 5, 4, 3, 2, 1, 0];
        let encoded = encode_as_helper(&sequence);
        assert_eq!(*encoded, [7, 6, 5, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn test_encode_as_3() {
        let sequence = [
            3, 2, 15, 5, 23, 6, 16, 31, 19, 29, 21, 13, 17, 0, 27, 8, 24, 18, 12, 1, 9, 4, 14, 20,
            28, 30, 7, 11, 25, 22, 26, 10,
        ];

        let encoded = encode_as_helper(&sequence);

        assert_eq!(
            *encoded,
            [
                3, 2, 13, 3, 19, 3, 11, 24, 13, 21, 14, 9, 10, 0, 15, 3, 11, 8, 6, 0, 2, 0, 3, 3,
                6, 6, 0, 1, 2, 1, 1, 0
            ]
        );
    }

    #[test]
    fn test_get_child() {
        assert_eq!(EncodeAS::_left_child_id(2), 1);
        assert_eq!(EncodeAS::_right_child_id(2), 3);
        assert_eq!(EncodeAS::_left_child_id(4), 2);
        assert_eq!(EncodeAS::_right_child_id(4), 6);
        assert_eq!(EncodeAS::_left_child_id(6), 5);
        assert_eq!(EncodeAS::_right_child_id(6), 7);
        assert_eq!(EncodeAS::_left_child_id(8), 4);
        assert_eq!(EncodeAS::_right_child_id(8), 12);
    }

    fn parent_child_roundtrip(node_id: u32) {
        let left = EncodeAS::_left_child_id(node_id);
        assert_eq!(EncodeAS::_parent_id(left), node_id);
        let right = EncodeAS::_right_child_id(node_id);
        assert_eq!(EncodeAS::_parent_id(right), node_id);
    }

    #[test]
    fn test_get_parent() {
        for i in (2..1024).step_by(2) {
            parent_child_roundtrip(i);
        }
    }
}
