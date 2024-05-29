extern crate big_lehmer;

use big_lehmer::Lehmer;

#[cfg(test)]
mod tests {
    use super::*;
    use dashu::integer::UBig;
    use dashu::rational::ops::EstimatedLog2;
    use rand::prelude::*;

    #[test]
    fn test_encode_size_samples() {
        // Values computed with wolfram alpha
        // the + values account for the padding we add to make sure it always enough
        assert_eq!(Lehmer::get_encode_size(0), 0);
        assert_eq!(Lehmer::get_encode_size(20), 8);
        assert_eq!(Lehmer::get_encode_size(21), 9);
        assert_eq!(Lehmer::get_encode_size(34), 16);
        assert_eq!(Lehmer::get_encode_size(35), 17);
        assert_eq!(Lehmer::get_encode_size(1024), 1097);
        assert_eq!(Lehmer::get_encode_size(4000), 5263 + 2);
        assert_eq!(Lehmer::get_encode_size(1_000_000), 2_311_111 + 32);
        // assert_eq!(Lehmer::get_encode_size(u32::MAX), 16405328180 + 32);
    }

    #[test]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    fn test_encode_size() {
        // Validate that the f64 approximation good enough
        // Dashu log2 is also just an estimate with relatively low precision.
        // Code snipped is also used to compute the necessary byte paddings to make sure we have convective bound
        let mut correct = UBig::ONE;
        let mut approx: f64 = 0.;

        for i in 2u32..4000 {
            approx += f64::log2(i.into());
            correct *= i;
            let tmp = correct.log2_bounds();
            assert!(
                (approx / 8.).ceil() as usize >= (tmp.1 / 8.).ceil() as usize,
                "left {}, right {}. count: {i}",
                approx.ceil(),
                tmp.1.ceil()
            );
        }
    }

    #[test]
    fn test_roundtrip_empty() {
        let sequence = [];

        let encoded = Lehmer::encode(&sequence).unwrap();
        let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
        Lehmer::decode(&encoded, &mut roundtrip).unwrap();
        assert_eq!(sequence, *roundtrip);
    }

    #[test]
    fn test_roundtrip_8() {
        let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
        let encoded = Lehmer::encode(&sequence).unwrap();
        let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
        Lehmer::decode(&encoded, &mut roundtrip).unwrap();
        assert_eq!(sequence, *roundtrip);
    }

    #[test]
    fn test_roundtrip_32() {
        let sequence = [
            3, 2, 15, 5, 23, 6, 16, 31, 19, 29, 21, 13, 17, 0, 27, 8, 24, 18, 12, 1, 9, 4, 14, 20,
            28, 30, 7, 11, 25, 22, 26, 10,
        ];
        let encoded = Lehmer::encode(&sequence).unwrap();
        let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
        Lehmer::decode(&encoded, &mut roundtrip).unwrap();
        assert_eq!(sequence, *roundtrip);
    }

    #[test]
    fn test_roundtrip_128() {
        let sequence = [
            61, 48, 3, 24, 60, 45, 35, 2, 33, 22, 55, 52, 18, 5, 36, 7, 1, 23, 28, 56, 50, 0, 4,
            63, 14, 11, 43, 53, 21, 34, 26, 32, 49, 20, 51, 62, 13, 19, 6, 46, 17, 39, 47, 58, 27,
            30, 44, 9, 12, 38, 10, 41, 42, 57, 40, 15, 29, 16, 25, 54, 8, 59, 37, 31,
        ];
        let encoded = Lehmer::encode(&sequence).unwrap();
        let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
        Lehmer::decode(&encoded, &mut roundtrip).unwrap();
        assert_eq!(sequence, *roundtrip);
    }

    #[test]
    fn test_roundtrip_random() {
        let mut sequence: Vec<u32> = (0..128).rev().collect();

        let mut rng = rand::thread_rng();

        for _ in 0..10000 {
            sequence.shuffle(&mut rng);

            let encoded = Lehmer::encode(&sequence).unwrap();
            assert!(!encoded.is_empty());
            let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
            Lehmer::decode(&encoded, &mut roundtrip).unwrap();
            assert_eq!(sequence, roundtrip);
        }
    }

    #[test]
    fn test_roundtrip_random_large() {
        let mut sequence: Vec<u32> = (0..10_000).rev().collect();

        let mut rng = rand::thread_rng();

        for _ in 0..5 {
            sequence.shuffle(&mut rng);

            let encoded = Lehmer::encode(&sequence).unwrap();
            assert!(!encoded.is_empty());
            let mut roundtrip: Vec<u32> = vec![0; sequence.len()];
            Lehmer::decode(&encoded, &mut roundtrip).unwrap();
            assert_eq!(sequence, roundtrip);
        }
    }
}
