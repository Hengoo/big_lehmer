use acceleration_structures::{DecodeAS, EncodeAS, EncodeCache};
use dashu::{base::DivRem, integer::UBig};
use error::Error;

mod acceleration_structures;
mod error;

/// Number sequences must not contain duplicates or missing numbers.
/// Basically means it is something like (0..N).shuffle
/// Technically supports sequences up to u32::MAX elements
///
/// Examples for valid input
///  `[4, 2, 0, 1, 3]`, `[0, 1, 2, 3, 4]`
///
/// Examples for invalid input:
///  `[0, 0]` (Duplicate number), `[1, 3, 2]` (Number 3 is out of range)
pub struct Lehmer;

impl Lehmer {
    /// Returns byte size of the Lehmer code.
    /// Bit size = log2(N!)
    /// Byte size = Ceil(log2(N!) / 8)
    pub fn get_encode_size(element_count: u32) -> usize {
        if element_count == 0 {
            return 0;
        }
        // Kinda naive implementation. But once this becomes a performance bottleneck we have
        // larger issues in encode / decode

        let element_count = u64::from(element_count);
        let mut result = 0.0;
        for i in 2..element_count + 1 {
            result += f64::log2(i as f64);
        }
        // Add some paddings to make sure float inaccuracy does not cause issues
        let byte_padding;
        if element_count < 4000 {
            byte_padding = 0;
        } else if element_count < 1000000 {
            byte_padding = 2;
        } else {
            byte_padding = 32;
        }
        (result / 8.0).ceil() as usize + byte_padding
    }

    pub fn encode(numbers: &[u32]) -> Result<Vec<u8>, Error> {
        if numbers.is_empty() {
            return Ok(Vec::new());
        }
        // supports up to u32::MAX elements
        u32::try_from(numbers.len()).map_err(|_| Error::SequenceToLong {
            element_count: numbers.len(),
        })?;

        let mut validation = vec![false; numbers.len()];

        let mut encode_as = EncodeAS::new(numbers.len());
        let mut result = UBig::ZERO;

        let mut cache = EncodeCache::default();

        for (index, &number) in numbers[..numbers.len() - 1].iter().enumerate() {
            // Validation is very cheap compared to the rest, so we alway do it
            let visited = validation
                .get_mut(number as usize)
                .ok_or(Error::ValidationOutOfRange)?;
            if *visited {
                return Err(Error::ValidationDuplicateNumber);
            }
            *visited = true;

            let add = encode_as.insert(number) as u64;
            let mul = (numbers.len() - (index + 1)) as u64;

            // Naive approach would now do result += add and result *= mul
            // with the cache we reduce the big number interactions
            if cache.add(add, mul).is_none() {
                result *= cache.mul;
                result += cache.add;
                cache = EncodeCache::new(add, mul);
            }
        }

        result *= cache.mul;
        result += cache.add;

        drop(validation);
        drop(encode_as);

        let bytes = result.to_le_bytes();
        // Turn big number into output
        let mut output = vec![0; Self::get_encode_size(numbers.len() as u32)];
        for (index, &byte) in bytes.iter().enumerate() {
            if let Some(out_byte) = output.get_mut(index) {
                *out_byte = byte;
            } else if byte != 0 {
                return Err(Error::ValidationDuplicateNumber);
            }
        }
        Ok(output)
    }

    /// element_count must be equal to what the data was encoded with
    pub fn decode(encoded: &[u8], element_count: u32) -> Result<Vec<u32>, Error> {
        if element_count == 0 {
            return Ok(Vec::new());
        }
        let element_count = usize::try_from(element_count).unwrap();

        // Intermediate stores the big number div results
        let mut intermediate = vec![0; element_count];

        // Input as one number
        let mut input: UBig = UBig::from_le_bytes(encoded);

        for (index, t) in intermediate[0..element_count - 1].iter_mut().enumerate() {
            let (div_result, remain) = input.div_rem(index + 2);
            input = div_result;
            *t = remain.try_into().unwrap();
        }

        let mut output = vec![0; element_count];
        let mut decode_as = DecodeAS::new(element_count);
        for (index, &t) in intermediate[0..element_count - 1].iter().rev().enumerate() {
            output[index] = decode_as.remove(t);
        }
        *output.last_mut().unwrap() = decode_as.remove(0);

        Ok(output)
    }
}
