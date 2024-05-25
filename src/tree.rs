
/*
// Must be pow of 2
const NODE_WIDTH_BITS: u8 = 4;
const NODE_WIDTH: u8 = 2u8.pow(NODE_WIDTH_BITS as u32);
const NODE_WIDTH_MASK: u32 = (1u32 << NODE_WIDTH_BITS) - 1;

pub(crate) struct Tree {
    // Could be done more memory efficiently, since the weights at the leaves are < NODE_WIDTH
    nodes: Vec<u32>,
    paths: Vec<Path>,
    depth: u8,
}

// Bit encoding of Path
struct Path {
    // Bit encoding of path
    path: u32,
}

impl Path {
    fn new(path: &[u8]) -> Self {
        let mut tmp = 0;

        for (index, &p) in path.iter().enumerate() {
            tmp |= p << (NODE_WIDTH_BITS * index.try_into().unwrap());
        }
        Path { path: tmp }
    }

    fn get(&self, depth: u8) -> u8 {
        let tmp = self.path >> (depth * NODE_WIDTH_BITS);
        let masked = tmp & NODE_WIDTH_MASK;
        masked.try_into().unwrap()
    }
}

impl Tree {
    pub(crate) fn new(numbers: &[u32]) -> Self {
        todo!()
    }

    /// Removes the number from the tree and returns its index.
    pub(crate) fn remove(&mut self, number: u32) -> u32 {
        todo!()
    }

    /// Computes (depth, node_count) of a tree with the given numbers count
    fn get_depth_node_count(number_count: u32) -> (u8, u32) {
        let mut depth = 0u8;
        let mut node_count = 1u32;

        loop {
            depth += 1;
            let layer_node_count = u32::from(NODE_WIDTH).pow(depth);
            if layer_node_count >= number_count {
                node_count += number_count;
                return (depth, node_count);
            }
            node_count += layer_node_count;
        }
    }
}
 */