use std::time::{SystemTime, UNIX_EPOCH};

pub struct HylRandom {
    seed: u64,
    s0: u64,
    s1: u64
}
impl HylRandom {
    pub fn new() -> Self {
        Self::from_seed(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64)
    }
    pub fn from_seed(mut seed: u64) -> Self {
        Self { seed, s0: Self::splitmix(&mut seed), s1: Self::splitmix(&mut seed) }
    }
    pub fn from_state(seed: u64, s: u128) -> Self {
        Self { seed, s0: s as u64, s1: (s >> 64) as u64 }
    }

    /// Fills `destination` with items chosen at random from `choices`.
    ///
    /// # Panics
    /// Panics if `choices` is empty.
    pub fn get_items<T: Copy>(&mut self, choices: &[T], destination: &mut [T]) {
        assert!(!choices.is_empty(), "choices cannot be empty");
        for slot in destination.iter_mut() {
            *slot = *self.choice(choices);
        }
    }

    /// Returns a new `Vec<T>` of `length` items chosen at random from `choices`.
    ///
    /// # Panics
    /// Panics if `choices` is empty or `length` is negative (impossible for
    /// `usize`, so the only panic here is the empty-choices guard).
    pub fn get_items_vec<T: Clone>(&mut self, choices: &[T], length: usize) -> Vec<T> {
        assert!(!choices.is_empty(), "choices cannot be empty");
        (0..length).map(|_| self.choice(choices).clone()).collect()
    }

    /// Returns a random element from `choices`.
    ///
    /// # Panics
    /// Panics if `choices` is empty.
    pub fn choice<'a, T>(&mut self, choices: &'a [T]) -> &'a T {
        &choices[self.next_below_usize(choices.len())]
    }

    /// Returns `true` with the given probability in `[0.0, 1.0)`.
    pub fn chance_f64(&mut self, chance: f64) -> bool {
        self.sample() < chance
    }
    /// Returns `true` with the given probability in `[0.0, 1.0)`.
    pub fn chance(&mut self, chance: f32) -> bool {
        (self.sample() as f32) < chance
    }

    /// Fills `buffer` with random bytes.
    pub fn next_bytes(&mut self, buffer: &mut [u8]) {
        let mut i = 0;
        while i + 8 <= buffer.len() {
            let rnd = self.internal_sample();
            buffer[i..i + 8].copy_from_slice(&rnd.to_le_bytes());
            i += 8;
        }
        if i < buffer.len() {
            let mut rnd = self.internal_sample();
            while i < buffer.len() {
                buffer[i] = rnd as u8;
                rnd >>= 8;
                i += 1;
            }
        }
    }

    /// Returns a random `u8` drawn from the high bits (Xoroshiro128+ has weaker
    /// low bits).
    pub fn next_u8(&mut self) -> u8 {
        (self.internal_sample() >> 56) as u8
    }

    /// Returns a random `u8` in `[0, max_value)`.
    /// Returns `0` if `max_value == 0`.
    pub fn next_u8_max(&mut self, max_value: u8) -> u8 {
        if max_value == 0 {
            return 0;
        }
        (self.internal_sample() % max_value as u64) as u8
    }

    /// Returns a random `u8` in `[min_value, max_value)`.
    /// Returns `min_value` if they are equal.
    pub fn next_u8_range(&mut self, min_value: u8, max_value: u8) -> u8 {
        if min_value == max_value {
            return min_value;
        }
        min_value + (self.internal_sample() % (max_value - min_value) as u64) as u8
    }

    /// Returns a random `u16` drawn from the high bits.
    pub fn next_u16(&mut self) -> u16 {
        (self.internal_sample() >> 48) as u16
    }

    /// Returns a random `u16` in `[0, max_value)`.
    /// Returns `0` if `max_value == 0`.
    pub fn next_u16_max(&mut self, max_value: u16) -> u16 {
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value as u64) as u16
    }

    /// Returns a random `u16` in `[min_value, max_value)`.
    /// Returns `min_value` if they are equal.
    pub fn next_u16_range(&mut self, min_value: u16, max_value: u16) -> u16 {
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below((max_value - min_value) as u64) as u16
    }

    /// Returns a random `u32` drawn from the high bits.
    pub fn next_u32(&mut self) -> u32 {
        (self.internal_sample() >> 32) as u32
    }

    /// Returns a random `u32` in `[0, max_value)`.
    /// Returns `0` if `max_value == 0`.
    pub fn next_u32_max(&mut self, max_value: u32) -> u32 {
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value as u64) as u32
    }

    /// Returns a random `u32` in `[min_value, max_value)`.
    /// Returns `min_value` if they are equal.
    pub fn next_u32_range(&mut self, min_value: u32, max_value: u32) -> u32 {
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below((max_value - min_value) as u64) as u32
    }

    /// Returns a random `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.internal_sample()
    }

    /// Returns a random `u64` in `[0, max_value)`.
    /// Returns `0` if `max_value == 0`.
    pub fn next_u64_max(&mut self, max_value: u64) -> u64 {
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value)
    }

    /// Returns a random `u64` in `[min_value, max_value)`.
    /// Returns `min_value` if they are equal.
    pub fn next_u64_range(&mut self, min_value: u64, max_value: u64) -> u64 {
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below(max_value - min_value)
    }

    /// Returns a random `i8` in `[0, i8::MAX)`.
    pub fn next_i8(&mut self) -> i8 {
        (self.internal_sample() >> 57) as i8
    }

    /// Returns a random `i8` in `[0, max_value)`.
    ///
    /// # Panics
    /// Panics if `max_value < 0`.
    pub fn next_i8_max(&mut self, max_value: i8) -> i8 {
        assert!(max_value >= 0, "max_value must be >= 0");
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value as u64) as i8
    }

    /// Returns a random `i8` in `[min_value, max_value)`.
    ///
    /// # Panics
    /// Panics if `min_value > max_value`.
    pub fn next_i8_range(&mut self, min_value: i8, max_value: i8) -> i8 {
        assert!(min_value <= max_value, "min_value cannot be greater than max_value");
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below((max_value - min_value) as u64) as i8
    }

    /// Returns a random `i16` in `[0, i16::MAX)`.
    pub fn next_i16(&mut self) -> i16 {
        (self.internal_sample() >> 49) as i16
    }

    /// Returns a random `i16` in `[0, max_value)`.
    ///
    /// # Panics
    /// Panics if `max_value < 0`.
    pub fn next_i16_max(&mut self, max_value: i16) -> i16 {
        assert!(max_value >= 0, "max_value must be >= 0");
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value as u64) as i16
    }

    /// Returns a random `i16` in `[min_value, max_value)`.
    ///
    /// # Panics
    /// Panics if `min_value > max_value`.
    pub fn next_i16_range(&mut self, min_value: i16, max_value: i16) -> i16 {
        assert!(min_value <= max_value, "min_value cannot be greater than max_value");
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below((max_value - min_value) as u64) as i16
    }

    /// Returns a random `i32` in `[0, i32::MAX)`.
    pub fn next_i32(&mut self) -> i32 {
        (self.internal_sample() >> 33) as i32
    }

    /// Returns a random `i32` in `[0, max_value)`.
    ///
    /// # Panics
    /// Panics if `max_value < 0`.
    pub fn next_i32_max(&mut self, max_value: i32) -> i32 {
        assert!(max_value >= 0, "max_value must be >= 0");
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value as u64) as i32
    }

    /// Returns a random `i32` in `[min_value, max_value)`.
    ///
    /// # Panics
    /// Panics if `min_value > max_value`.
    pub fn next_i32_range(&mut self, min_value: i32, max_value: i32) -> i32 {
        assert!(min_value <= max_value, "min_value cannot be greater than max_value");
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below((max_value - min_value) as u64) as i32
    }

    /// Returns a random `i64` in `[0, i64::MAX)`.
    pub fn next_i64(&mut self) -> i64 {
        (self.internal_sample() >> 1) as i64
    }

    /// Returns a random `i64` in `[0, max_value)`.
    ///
    /// # Panics
    /// Panics if `max_value < 0`.
    pub fn next_i64_max(&mut self, max_value: i64) -> i64 {
        assert!(max_value >= 0, "max_value must be >= 0");
        if max_value == 0 {
            return 0;
        }
        self.next_below(max_value as u64) as i64
    }

    /// Returns a random `i64` in `[min_value, max_value)`.
    ///
    /// # Panics
    /// Panics if `min_value > max_value`.
    pub fn next_i64_range(&mut self, min_value: i64, max_value: i64) -> i64 {
        assert!(min_value <= max_value, "min_value cannot be greater than max_value");
        if min_value == max_value {
            return min_value;
        }
        min_value + self.next_below((max_value - min_value) as u64) as i64
    }

    /// Returns a random `f32` in `[0.0, 1.0)`.
    pub fn next_f32(&mut self) -> f32 {
        self.sample() as f32
    }

    /// Returns a random `f32` in `[0.0, max_value)`.
    ///
    /// # Panics
    /// Panics if `max_value < 0.0`.
    pub fn next_f32_max(&mut self, max_value: f32) -> f32 {
        self.next_f64_max(max_value as f64) as f32
    }

    /// Returns a random `f32` in `[min_value, max_value)`.
    ///
    /// # Panics
    /// Panics if `min_value > max_value`.
    pub fn next_f32_range(&mut self, min_value: f32, max_value: f32) -> f32 {
        self.next_f64_range(min_value as f64, max_value as f64) as f32
    }

    /// Returns a random `f64` in `[0.0, 1.0)`.
    pub fn next_f64(&mut self) -> f64 {
        self.sample()
    }

    /// Returns a random `f64` in `[0.0, max_value)`.
    ///
    /// # Panics
    /// Panics if `max_value < 0.0`.
    pub fn next_f64_max(&mut self, max_value: f64) -> f64 {
        assert!(max_value >= 0.0, "max_value must be non-negative");
        self.sample() * max_value
    }

    /// Returns a random `f64` in `[min_value, max_value)`.
    ///
    /// # Panics
    /// Panics if `min_value > max_value`.
    pub fn next_f64_range(&mut self, min_value: f64, max_value: f64) -> f64 {
        assert!(min_value <= max_value, "min_value must be <= max_value");
        min_value + self.sample() * (max_value - min_value)
    }

    /// Performs an in-place Fisher-Yates shuffle of a slice.
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        let n = values.len();
        for i in (1..n).rev() {
            let j = self.next_below_usize(i + 1);
            values.swap(i, j);
        }
    }

    #[inline]
    pub fn sample(&mut self) -> f64 {
        (self.internal_sample() >> 11) as f64 * 1.1102230246251565E-16
    }

    #[inline]
    pub fn peek_sample(&self) -> u64 {
        self.s0.wrapping_add(self.s1)
    }

    #[inline]
    fn internal_sample(&mut self) -> u64 {
        let mut second = self.s1;
        let result = self.s0.wrapping_add(second);

        second ^= self.s0;

        // update internal seed
        self.s0 = self.s0.rotate_left(55) ^ second ^ (second << 14); // a, b
        self.s1 = second.rotate_left(36); // c

        result
    }

    /// PCG-style internal sample (alternative generator using the stored seed).
    #[inline]
    fn internal_sample_pcg(&mut self) -> u64 {
        let first = self.s0;
        let second = self.s1;
        let inc = (self.seed << 1) | 1;
        self.s0 = self.s0.wrapping_mul(0x2360ed051fc65da4).wrapping_add(inc);
        self.s1 = self.s1.wrapping_mul(0x4385df649fccf645).wrapping_add(inc ^ 0x5851f42d4c957f2d);

        let xorshifted = ((first ^ second) >> 18) ^ first;
        let rot = (second >> 59) as u16;
        (xorshifted >> rot) | (xorshifted << ((64 - rot) & 63))
    }

    /// Lemire-style unbiased bounded random in `[0, bound)`.
    /// Returns `0` when `bound == 0`.
    #[inline]
    fn next_below(&mut self, bound: u64) -> u64 {
        if bound == 0 {
            return 0;
        }
        let threshold = 0u64.wrapping_sub(bound) % bound;
        loop {
            let r = self.internal_sample();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// Convenience wrapper returning a `usize` bounded result.
    #[inline]
    fn next_below_usize(&mut self, bound: usize) -> usize {
        self.next_below(bound as u64) as usize
    }

    #[inline]
    fn splitmix(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = *seed;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
}
impl Default for HylRandom {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(debug_assertions)]
impl HylRandom {
    /// Runs the Xoroshiro128+ step function `i32::MAX` times without collecting
    /// output to test speed.
    pub fn lots_of_samples(&mut self) {
        for _ in 0..i32::MAX {
            let first = self.s0;
            let mut second = self.s1;
            // result intentionally unused — we're just exercising the step
            let _result = first.wrapping_add(second);

            second ^= first;

            // manual inline of rotl(first, 55) and rotl(second, 36)
            self.s0 = ((first << 55) | (first >> 9)) ^ second ^ (second << 14); // a, b
            self.s1 = (second << 36) | (second >> 28);                           // c
        }
    }

    /// Runs a few basic randomness tests on the generator and prints results to
    /// stdout via `println!`.
    ///
    /// Not a replacement for TestU01 or PractRand.
    pub fn test_generator(samples: usize) {
        let mut rand = HylRandom::new();

        let (mut ones, mut zeros) = (0u64, 0u64);
        let mut runs = 1i64;
        let mut prev_bit: i32 = -1;

        const BUCKETS: usize = 256;
        let mut counts = vec![0i64; BUCKETS];

        for _ in 0..samples {
            let value = rand.internal_sample();

            // Bit frequency test on the lowest 32 bits
            for b in 0..32 {
                let bit = ((value >> b) & 1) as i32;
                if bit == 1 { ones += 1; } else { zeros += 1; }

                if prev_bit != -1 && bit != prev_bit {
                    runs += 1;
                }
                prev_bit = bit;
            }

            // Chi-square bucket using the lowest 8 bits
            counts[(value & 0xFF) as usize] += 1;
        }

        let expected = samples as f64 / BUCKETS as f64;
        let chi_sq: f64 = counts
            .iter()
            .map(|&c| {
                let diff = c as f64 - expected;
                diff * diff / expected
            })
            .sum();

        let total_bits = ones + zeros;
        let ratio = ones as f64 / total_bits as f64;

        println!("=== Randomness test results ===");
        println!("Samples: {samples}");
        println!("Bit frequency: ones={ones}, zeros={zeros}, ratio={ratio:.4}");
        println!("Chi-square (df={}): {chi_sq:.2}", BUCKETS - 1);
        println!("Runs observed: {runs}");
        println!("================================");
    }

    /// Tests the `next_f64` distribution for uniformity, mean, and variance.
    pub fn test_doubles(samples: usize, buckets: usize) {
        let mut rand = HylRandom::new();

        let mut counts = vec![0i64; buckets];
        let (mut sum, mut sum_sq) = (0f64, 0f64);

        for _ in 0..samples {
            let x = rand.next_f64();
            sum += x;
            sum_sq += x * x;

            let mut bucket = (x * buckets as f64) as usize;
            if bucket == buckets {
                bucket -= 1; // handle the rare x == 1.0 edge case
            }
            counts[bucket] += 1;
        }

        let expected = samples as f64 / buckets as f64;
        let chi_sq: f64 = counts
            .iter()
            .map(|&c| {
                let diff = c as f64 - expected;
                diff * diff / expected
            })
            .sum();

        let mean = sum / samples as f64;
        let variance = sum_sq / samples as f64 - mean * mean;

        println!("=== Double distribution test results ===");
        println!("Samples: {samples}");
        println!("Chi-square: {chi_sq:.2}, df={}", buckets - 1);
        println!("Mean={mean:.6} (expected 0.5)");
        println!("Variance={variance:.6} (expected ~0.08333)");
        println!("========================================");
    }
}