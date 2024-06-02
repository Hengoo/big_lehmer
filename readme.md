Big Lehmer is a lib for converting between arbitrarily sized permutation vectors into compact [Lehmer codes](https://en.wikipedia.org/wiki/Lehmer_code) and their uncompressed vector representation.

The number sequence must have similar properties as `[0.N].shuffle`. Basically sequential numbers in random order, starting at zero. The lib technically supports up to `u32::MAX` numbers, but performance will be the main issue beforehand.

### Usage
```rust
extern crate big_lehmer;

use big_lehmer;

fn main() {
    let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
    let lehmer_code = big_lehmer::encode(&sequence).unwrap();
    let mut roundtrip = [0; 8];
    big_lehmer::decode(&lehmer_code, &mut roundtrip).unwrap();
    assert_eq!(sequence, roundtrip);
}
```

### Benchmarks:

Measured on my "old system" (i7- 6700k). Not very accurate, just to showcase performance expectations.

| Sequence length | Lehmer code size         | encode time | decode time |
| --------------- | ------------------------ | ----------- | ----------- |
| 512             | 4485 bytes               | 470.70µs    | 107.40µs    |
| 10_000          | 14808 bytes   = 15 KB    | 2.40ms      | 4.11ms      |
| 100_000         | 189588 bytes  = ~190 KB  | 61.21ms     | 205.44ms    |
| 1_000_000       | 2311111 bytes = ~2.2 MiB | 2.15s       | 11.39s      |

The crate is mainly optimized for large sequences.

Performance for large sequences is dominated by the big integer math. A possible optimization is to replace Dashu with rug. Apparently rug (=GMP) is extremely well optimized, but it's not native rust and not trivial to get working.