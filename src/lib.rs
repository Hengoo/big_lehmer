#![doc = include_str!("../readme.md")]

use dashu::integer::UBig;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

mod decode;
mod encode;
mod error;

use decode::{recursive_split_divide, DecodeAS, WorkItem};
use encode::{BigCache, Cache, EncodeAS};
use error::Error;

/// Estimate bounded byte size of the Lehmer code.
/// Bit size = log2(N!)
/// Byte size = Ceil(log2(N!) / 8)
///
/// # Examples
///
/// ```
/// assert_eq!(big_lehmer::get_encode_size(20), 8);
/// ```
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
#[must_use]
pub fn get_encode_size(element_count: u32) -> usize {
    if element_count == 0 {
        return 0;
    }
    // Kinda naive implementation. But once this becomes a performance bottleneck we have
    // larger issues in encode / decode

    let element_count = u64::from(element_count);
    let mut result = 0.0;
    for i in 2..=element_count {
        result += f64::log2(i as f64);
    }
    // Add some paddings to make sure float inaccuracy does not cause issues
    let byte_padding;
    if element_count < 4000 {
        byte_padding = 0;
    } else if element_count < 1_000_000 {
        byte_padding = 2;
    } else {
        byte_padding = 32;
    }
    (result / 8.0).ceil() as usize + byte_padding
}

/// Encodes the number sequence into a Lehmer code.  
/// Approximate size of the code can be computed with `big_lehmer::get_encode_size`
/// The code can later be decoded with `big_lehmer::decode`
///
/// # Examples
/// ```
/// let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
/// let encoded = big_lehmer::encode(&sequence).unwrap();
/// let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
/// big_lehmer::decode(&encoded, &mut roundtrip).unwrap();
/// assert_eq!(sequence, *roundtrip);
/// ```
///
/// # Errors
///
/// Will also error when the input number sequence is not valid.  
/// Examples for invalid input:  
///  `[0, 0]` (Duplicate number)  
///  `[1, 3, 2]` (Number 3 is out of range)  
/// This automatically means it errors on sequences longer than `u32::Max`
///
/// # Panics
///
/// Generally it should not panic. There might be panics on 16 bit systems.
pub fn encode(numbers: &[u32]) -> Result<Box<[u8]>, Error> {
    if numbers.is_empty() {
        return Ok(Box::new([]));
    }
    // supports up to u32::MAX elements
    let element_count = u32::try_from(numbers.len()).map_err(|_| Error::SequenceToLong {
        element_count: numbers.len(),
    })?;

    let mut encode_as = EncodeAS::new(element_count);
    let mut validation = vec![false; numbers.len()];
    let mut cache = Cache::default();
    let mut caches = vec![];
    for (index, &number) in numbers[..numbers.len() - 1].iter().enumerate() {
        // Validation is basically free
        let visited = validation
            .get_mut(number as usize)
            .ok_or(Error::ValidationOutOfRange)?;
        if *visited {
            return Err(Error::ValidationDuplicateNumber);
        }
        *visited = true;

        let add = u64::from(encode_as.insert(number));
        let mul = u64::try_from(numbers.len() - (index + 1)).unwrap();

        // Naive approach would now do result += add and result *= mul
        // with the cache we reduce the big number interactions
        if cache.add(add, mul).is_none() {
            caches.push(cache);
            cache = Cache::new(add, mul);
        }
    }
    caches.push(cache);

    // Combine the smaller caches into final result
    // Besides parallelism, the reduce also keeps the UBig small for the majority of the steps
    let result = caches
        .par_iter()
        .map(BigCache::new)
        .reduce(BigCache::identity, BigCache::combine);

    Ok(result.add.to_le_bytes())
}

/// Decodes a Lehmer code generated by `big_lehmer::encode`  
/// the `result` slice must have the same length as the sequence that was used to create the code
///
/// # Errors
///
/// Can error when the code was created with more elements than you are trying to decode.  
/// Can error when `encoded` is not a valid lehmer code  
/// Will error when `results` has more than `u32::MAX` elements.  
///
/// # Examples
/// ```
/// use big_lehmer;
/// let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
/// let encoded = big_lehmer::encode(&sequence).unwrap();
/// let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
/// big_lehmer::decode(&encoded, &mut roundtrip).unwrap();
/// assert_eq!(sequence, *roundtrip);
/// ```
///
/// # Panics
///
/// Generally it should not panic. There might be panics on 16 bit systems.
pub fn decode(encoded: &[u8], results: &mut [u32]) -> Result<(), Error> {
    if results.is_empty() {
        return Ok(());
    }
    // supports up to u32::MAX elements
    let element_count = u32::try_from(results.len()).map_err(|_| Error::SequenceToLong {
        element_count: results.len(),
    })?;

    let mut remainders = vec![None; results.len()];

    let input: UBig = UBig::from_le_bytes(encoded);
    let work = WorkItem {
        dividend: input,
        start_index: 2,
        remainders: &mut remainders,
    };
    recursive_split_divide(work);

    let mut decode_as = DecodeAS::new(element_count);
    for (index, &t) in remainders[0..results.len() - 1].iter().rev().enumerate() {
        if let Some(t) = t {
            results[index] = decode_as.remove(t.get() - 1);
        } else {
            return Err(Error::Decode);
        }
    }
    *results.last_mut().unwrap() = decode_as.remove(0);

    Ok(())
}
