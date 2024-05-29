Big Lehmer is a lib for converting between arbitrarily sized permutation vectors into compact [Lehmer codes](https://en.wikipedia.org/wiki/Lehmer_code) and their uncompressed vector representation.

The number sequence must have similar properties as `[0.N].shuffle`. Basically sequential numbers in random order, starting at zero. The lib technically supports up to `u32::MAX` numbers, but performance will be the main issue beforehand.

### Usage
```rust
extern crate big_lehmer;

use big_lehmer::Lehmer;

fn main() {
    let sequence = [7, 2, 0, 6, 5, 1, 4, 3];
    let lehmer_code = Lehmer::encode(&sequence).unwrap();
    let mut roundtrip = [0; 8];
    Lehmer::decode(&lehmer_code, &mut roundtrip).unwrap();
    assert_eq!(sequence, roundtrip);
}
```

### Benchmarks:

TODO

Performance for large sequences is dominated by the decode big number math. A possible optimization is to replace Dashu with rug. Apparently rug (=GMP) is extremely well optimized, but it's not native rust and not trivial to get working.