use dashu::{base::BitTest, base::DivRem, integer::UBig};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Debug)]
pub(crate) struct WorkItem<'a> {
    pub(crate) dividend: UBig,
    pub(crate) start_index: u32,
    pub(crate) remainders: &'a mut [u32],
}

type DivideType = u64;

// Final step that does the actual divisions on u64
pub(crate) fn divide(work: WorkItem) {
    let mut dividend = DivideType::try_from(work.dividend).unwrap();
    for (index, r) in work.remainders.iter_mut().enumerate() {
        let divisor = DivideType::from(work.start_index) + DivideType::try_from(index).unwrap();
        *r = u32::try_from(dividend % divisor).unwrap();

        dividend /= divisor;
    }
    // TODO use result here (How to handle result with parallelism?)
    assert_eq!(dividend, 0);
}

// Splits the work items into two smaller if it makes sense
// Second work item is None if the work item can be passed to the final division step
pub(crate) fn split(work: WorkItem) -> (WorkItem, Option<WorkItem>) {
    let length = work.dividend.bit_len();
    if length <= usize::try_from(DivideType::BITS).unwrap() {
        return (work, None);
    }
    // Since large divisions have MxN cost we split in a way to keep the divisor smaller
    let split_length = if length >= 20_000 {
        length / 16
    } else {
        length / 4
    };

    // Compute part factorial until we are larger than length
    let mut split_index = work.start_index;
    let mut factorial = UBig::ONE;
    loop {
        factorial *= split_index;
        split_index += 1;
        if factorial.bit_len() >= split_length {
            break;
        }
    }

    let (quotient, remain) = work.dividend.div_rem(factorial);

    let (left, right) = work
        .remainders
        .split_at_mut(usize::try_from(split_index - work.start_index).unwrap());

    (
        WorkItem {
            dividend: remain,
            start_index: work.start_index,
            remainders: left,
        },
        Some(WorkItem {
            dividend: quotient,
            start_index: split_index,
            remainders: right,
        }),
    )
}

pub(crate) fn recursive_divide(work: WorkItem) {
    let len = work.remainders.len();
    let (left, right) = split(work);

    if right.is_none() {
        divide(left);
        return;
    }

    // Speedup for parallel is abysmal :(
    if len > 1000 {
        rayon::join(
            || recursive_divide(left),
            || recursive_divide(right.unwrap()),
        );
    } else {
        recursive_divide(right.unwrap());
        recursive_divide(left);
    }
}

pub(crate) fn parallel_divide(work: WorkItem) {
    let mut work = work;
    let mut work_items = vec![];
    loop {
        let (left, right) = split(work);
        if right.is_some() {
            work = left;
            work_items.push(right.unwrap());
        } else {
            work_items.push(left);
            break;
        }
    }

    work_items.into_par_iter().for_each(recursive_divide);
}

/// Naive approach would be to create a list from 0 to N and then repeatedly remove elements from it
/// Very slightly faster than the naive approach
/// Basically a tree that stores prim counts and is adjusted while fetching a number
#[derive(Debug)]
pub(crate) struct DecodeAS {
    // Store number of primitives of the left subtree
    tree: Vec<u32>,
}
impl DecodeAS {
    pub fn new(element_count: u32) -> Self {
        let len = element_count.next_power_of_two();
        let nodes = (0..len)
            .map(|i| {
                if i == 0 {
                    return 1;
                }
                let height = i.trailing_zeros();
                1u32 << height
            })
            .collect();
        Self { tree: nodes }
    }

    pub fn remove(&mut self, number: u32) -> u32 {
        let length = u32::try_from(self.tree.len()).expect("Sequence must fit in u32");
        let mut left_count = 0;
        let mut node_id = length / 2;
        let mut jump = length / 4;

        loop {
            let node = &mut self.tree[node_id as usize];
            if number >= (*node + left_count) {
                // go right
                left_count += *node;
                node_id += jump;
                if jump == 0 {
                    break;
                }
            } else {
                // go left
                *node -= 1;
                node_id -= jump;
                if jump == 0 {
                    node_id -= 1;
                    break;
                }
            }

            jump /= 2;
        }
        node_id
    }
}
