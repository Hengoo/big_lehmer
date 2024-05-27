use acceleration_structures::{DecodeAS, EncodeAS, EncodeCache};
use dashu::{base::DivRem, integer::UBig};
use error::Error;

mod acceleration_structures;
mod error;
mod tree;

/// Number sequences must not contain duplicates or missing numbers
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
    pub fn get_encode_size(element_count: usize) -> usize {
        if element_count == 0 {
            return 0;
        }
        // Kinda naive implementation. But once this becomes a performance bottleneck we have
        // larger issues in encode / decode
        let mut result = 0.0;
        for i in 2..element_count + 1 {
            result += f64::log2(i as f64);
        }
        f64::ceil(result / 8.0) as usize
    }

    pub fn encode(numbers: &[u32]) -> Result<Vec<u8>, Error> {
        let mut vector = vec![0; Self::get_encode_size(numbers.len())];
        let error = Lehmer::encode_in(numbers, vector.as_mut_slice());
        error.map(|_| vector)
    }

    /// out needs to be of size get_encoded_size() or more
    pub fn encode_in(numbers: &[u32], out: &mut [u8]) -> Result<(), Error> {
        if numbers.is_empty() {
            return Ok(());
        }

        let mut validation = vec![false; numbers.len()];

        let mut encode_as = EncodeAS::new(numbers.len());
        let mut result = UBig::ZERO;

        let mut cache = EncodeCache::new(0, 1);

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
        // Write into result.
        for (index, &byte) in bytes.iter().enumerate() {
            if let Some(out_byte) = out.get_mut(index) {
                *out_byte = byte;
            } else {
                assert!(byte == 0);
            }
        }
        Ok(())
    }

    /// element_count must be equal to what the data was encoded with
    pub fn decode(encoded: &[u8], element_count: usize) -> Result<Vec<u32>, Error> {
        let mut vector = vec![0; element_count];
        let error = Lehmer::decode_in(encoded, vector.as_mut_slice());
        error.map(|_| vector)
    }

    /// out size must be equal to what the data was encoded with
    pub fn decode_in(encoded: &[u8], out: &mut [u32]) -> Result<(), Error> {
        if out.is_empty() {
            return Ok(());
        }

        let mut tmp = vec![0; out.len()];

        let mut input = UBig::from_le_bytes(encoded);

        for (index, t) in tmp[0..out.len() - 1].iter_mut().enumerate() {
            let (div_result, remain) = input.div_rem(index + 2);
            input = div_result;
            *t = remain as u32;
        }

        let mut decode_as = DecodeAS::new(out.len());
        for (index, &t) in tmp[0..out.len() - 1].iter().rev().enumerate() {
            out[index] = decode_as.remove(t);
        }
        *out.last_mut().unwrap() = decode_as.remove(0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::prelude::*;

    #[test]
    fn test_encode_size() {
        assert_eq!(Lehmer::get_encode_size(0), 0);
        assert_eq!(Lehmer::get_encode_size(20), 8);
        assert_eq!(Lehmer::get_encode_size(21), 9);
        assert_eq!(Lehmer::get_encode_size(34), 16);
        assert_eq!(Lehmer::get_encode_size(35), 17);
        assert_eq!(Lehmer::get_encode_size(1024), 1097);
        assert_eq!(Lehmer::get_encode_size(1000000), 2311111);
        // works up to 32 but that one is a bit slow
        // assert_eq!(Lehmer::get_encode_size(u32::MAX as usize), 16405328180);
    }

    #[test]
    fn test_roundtrip_empty() {
        let input: Vec<u32> = vec![];

        let encoded = Lehmer::encode(&input).unwrap();
        let roundtrip = Lehmer::decode(&encoded, input.len()).unwrap();
        assert_eq!(input, roundtrip);
    }

    #[test]
    fn test_roundtrip_large() {
        let mut input: Vec<u32> = (0..10000).rev().collect();

        let mut rng = rand::thread_rng();

        input.shuffle(&mut rng);

        let encoded = Lehmer::encode(&input).unwrap();
        assert!(!encoded.is_empty());
        let roundtrip = Lehmer::decode(&encoded, input.len()).unwrap();
        assert_eq!(input, roundtrip);
    }

    #[test]
    fn test_roundtrip_8() {
        let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
        let encoded = Lehmer::encode(&sequence).unwrap();
        let roundtrip = Lehmer::decode(&encoded, sequence.len()).unwrap();

        assert_eq!(sequence, *roundtrip);
    }

    #[test]
    fn test_roundtrip_32() {
        let sequence = [
            3, 2, 15, 5, 23, 6, 16, 31, 19, 29, 21, 13, 17, 0, 27, 8, 24, 18, 12, 1, 9, 4, 14, 20,
            28, 30, 7, 11, 25, 22, 26, 10,
        ];
        let encoded = Lehmer::encode(&sequence).unwrap();
        let roundtrip = Lehmer::decode(&encoded, sequence.len()).unwrap();

        assert_eq!(sequence, *roundtrip);
    }
}
