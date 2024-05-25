use super::Error;

// Naive approach would create a list [0..N]
// and on insert(number) find the index of the number, then remove it
#[derive(Debug)]
pub(crate) struct EncodeAS {
    tree: Vec<u32>,
}

impl EncodeAS {
    pub fn new(element_count: usize) -> Self {
        let len = element_count.next_power_of_two();
        EncodeAS { tree: vec![0; len] }
    }

    fn _left_child_id(node: u32) -> u32 {
        let zeroes = node.trailing_zeros();
        node - (1 << (zeroes - 1))
    }

    fn _right_child_id(node: u32) -> u32 {
        let zeroes = node.trailing_zeros();
        node + (1 << (zeroes - 1))
    }

    fn _parent(node: u32) -> u32 {
        let zeroes = node.trailing_zeros();
        let tmp = (node >> zeroes) & 3;
        match tmp {
            // Move least significant bit by one to left
            1 => node + (1 << zeroes),
            // Remove least significant bit
            3 => node & (node - 1),
            _ => unreachable!(),
        }
    }

    pub fn insert(&mut self, number: u32) -> u32 {
        let mut result = number;
        let mut node = self.tree.len() as u32 / 2;
        let mut jump = self.tree.len() as u32 / 4;

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

// This is currently the naive approach.
// create a list [0..N]
// remove(index) return the number on that index and afterwards remove it from the list
#[derive(Debug)]
pub(crate) struct DecodeAs {
    numbers: Vec<u32>,
}
impl DecodeAs {
    pub fn new(element_count: usize) -> DecodeAs {
        DecodeAs {
            numbers: (0..element_count as u32).collect(),
        }
    }

    pub fn remove(&mut self, index: u32) -> Result<u32, Error> {
        let tmp = *self.numbers.get(index as usize).ok_or(Error::DecodeError)?;
        self.numbers.remove(index as usize);
        Ok(tmp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_as_helper(numbers: &[u32]) -> Box<[u32]> {
        let mut t = EncodeAS::new(numbers.len());

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
        assert_eq!(EncodeAS::_parent(left), node_id);
        let right = EncodeAS::_right_child_id(node_id);
        assert_eq!(EncodeAS::_parent(right), node_id);
    }

    #[test]
    fn test_get_parent() {
        for i in (2..1024).step_by(2) {
            parent_child_roundtrip(i);
        }
    }
}
