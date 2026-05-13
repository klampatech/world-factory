//! Noise generation utilities for procedural generation
//!
//! Provides simplex noise with multiple variants for terrain, biomes, and resources.

use serde::{Deserialize, Serialize};

/// Seeded pseudo-random number generator using xorshift64.
#[derive(Debug, Clone, Default)]
pub struct Rng(u64);

impl Rng {
    /// Create a new RNG with a seed.
    pub fn new(seed: u64) -> Self {
        Self(if seed == 0 { 1 } else { seed })
    }

    /// Generate next random value.
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Generate random float in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    /// Generate random float in [-1, 1).
    pub fn next_f64Signed(&mut self) -> f64 {
        self.next_f64() * 2.0 - 1.0
    }

    /// Generate random float in [0, 1).
    pub fn next_float(&mut self) -> f32 {
        (self.next() as u32 as f32) / (u32::MAX as f32)
    }

    /// Generate random value in range [min, max).
    pub fn range(&mut self, min: usize, max: usize) -> usize {
        let r = self.next() as usize;
        min + (r % (max - min))
    }

    /// Generate random usize.
    pub fn random_usize(&mut self) -> usize {
        self.next() as usize
    }

    /// Generate random f32 in [0, 1).
    pub fn random_f32(&mut self) -> f32 {
        (self.next() as u32 as f32) / (u32::MAX as f32)
    }

    /// Generate random f32 in [0, 1) - alias for random_f32.
    pub fn random_float(&mut self) -> f32 {
        self.random_f32()
    }
}

/// Simple 2D simplex noise wrapper for the Rng.
impl Rng {
    /// Get simplex noise value at (x, y) in range [-1, 1).
    /// Uses the same algorithm as SimplexNoise but simpler interface.
    pub fn simplex_2d(&mut self, x: f32, y: f32) -> f32 {
        // Use a pre-computed gradient table
        static GRADIENTS: [[f32; 2]; 8] = [
            [1.0, 1.0],
            [-1.0, 1.0],
            [1.0, -1.0],
            [-1.0, -1.0],
            [1.0, 0.0],
            [-1.0, 0.0],
            [0.0, 1.0],
            [0.0, -1.0],
        ];

        // Simple hash using the RNG
        let hash = (self.next() & 0x7) as usize;
        let g = GRADIENTS[hash];

        // Simple value noise (not full simplex, but good enough for density maps)
        let nx = x.floor();
        let ny = y.floor();
        let fx = x - nx;
        let fy = y - ny;

        // Smooth interpolation
        let ux = fx * fx * (3.0 - 2.0 * fx);
        let uy = fy * fy * (3.0 - 2.0 * fy);

        let n0 = g[0] * fx + g[1] * fy;
        let n1 = g[0] * (fx - 1.0) + g[1] * fy;
        let n2 = g[0] * fx + g[1] * (fy - 1.0);
        let n3 = g[0] * (fx - 1.0) + g[1] * (fy - 1.0);

        let v = n0 * (1.0 - ux) * (1.0 - uy)
            + n1 * ux * (1.0 - uy)
            + n2 * (1.0 - ux) * uy
            + n3 * ux * uy;

        v * std::f32::consts::FRAC_1_SQRT_2 // Normalize to [-1, 1]
    }
}

/// 2D Simplex noise generator with multiple octave support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplexNoise {
    seed: u64,
    perm: Vec<u8>,
    #[serde(skip)]
    rng: Rng,
}

impl SimplexNoise {
    /// Permutation table size.
    const PERM_SIZE: usize = 256;

    /// Create a new simplex noise generator.
    pub fn new(seed: u64) -> Self {
        // Properly seed the RNG using the provided seed
        let mut rng = Rng(seed);
        // Advance RNG state based on seed to ensure different seeds produce different permutation
        for _ in 0..seed.min(1) {
            rng.next();
        }

        let mut perm = vec![0u8; Self::PERM_SIZE * 2];

        // Initialize permutation table
        for i in 0..Self::PERM_SIZE {
            perm[i] = i as u8;
        }

        // Shuffle using Fisher-Yates
        for i in (1..Self::PERM_SIZE).rev() {
            let j = (rng.next() % (i + 1) as u64) as usize;
            perm.swap(i, j);
        }

        // Duplicate for overflow handling
        for i in 0..Self::PERM_SIZE {
            perm[Self::PERM_SIZE + i] = perm[i];
        }

        Self {
            seed,
            perm,
            rng: Rng::new(seed),
        }
    }

    /// Get noise value at (x, y) in range [-1, 1].
    pub fn get(&self, x: f64, y: f64) -> f64 {
        self.noise_2d(x, y)
    }

    /// Get noise value at (x, y) in range [-1, 1] using f32 inputs.
    pub fn get_f32(&self, x: f32, y: f32) -> f32 {
        self.noise_2d(x as f64, y as f64) as f32
    }

    /// Get FBM noise value using f32 inputs.
    pub fn get_fbm_f32(
        &self,
        x: f32,
        y: f32,
        octaves: usize,
        persistence: f32,
        lacunarity: f32,
    ) -> f32 {
        self.octave_noise_2d_f32(x, y, octaves, persistence, lacunarity)
    }

    /// Get FBM (Fractal Brownian Motion) noise value.
    pub fn get_fbm(
        &self,
        x: f64,
        y: f64,
        octaves: usize,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        self.octave_noise_2d(x, y, octaves, persistence, lacunarity)
    }

    /// Multi-octave noise for more natural patterns (f32 variant).
    pub fn octave_noise_2d_f32(
        &self,
        x: f32,
        y: f32,
        octaves: usize,
        persistence: f32,
        lacunarity: f32,
    ) -> f32 {
        let mut value = 0.0f32;
        let mut amplitude = 1.0f32;
        let mut frequency = 1.0f32;
        let mut max_value = 0.0f32;

        for _ in 0..octaves {
            value += self.noise_2d(x as f64 * frequency as f64, y as f64 * frequency as f64) as f32
                * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        value / max_value
    }

    /// Get random float in [0, 1) for erosion simulation.
    /// Uses the permutation table to generate pseudo-random values.
    pub fn get_float(&self, seed: f32) -> f32 {
        // Use permutation lookup for pseudo-random value
        let idx = ((seed * 1000.0) as usize) & 0xFF;
        (self.perm[idx] as f32) / 255.0
    }

    /// Get noise value in a bounded range [min, max] using f64.
    pub fn get_bounded(&self, seed: u64, min: f64, max: f64) -> f64 {
        let idx = (seed as usize) & 0xFF;
        let normalized = (self.perm[idx] as f64) / 255.0;
        min + normalized * (max - min)
    }

    /// Get noise value in a bounded range [min, max] using f32.
    pub fn get_bounded_f32(&self, seed: u64, min: f32, max: f32) -> f32 {
        let idx = (seed as usize) & 0xFF;
        let normalized = (self.perm[idx] as f32) / 255.0;
        min + normalized * (max - min)
    }

    /// Get seed-based value for deterministic selection.
    pub fn get_seed_u64(&self, seed: u64) -> u64 {
        let idx = (seed as usize) & 0xFF;
        let p = self.perm[idx] as u64;
        // Mix with seed for better distribution
        (seed.wrapping_mul(31).wrapping_add(p << 8)) | (self.perm[(idx + 1) & 0xFF] as u64)
    }

    /// Get billow noise (absolute value) - produces ridge patterns.
    pub fn get_billow(&self, x: f64, y: f64, octaves: usize) -> f64 {
        let value = self.octave_noise_2d(x, y, octaves, 0.5, 2.0);
        value.abs()
    }

    /// Get warped noise using domain distortion.
    pub fn get_warped(&self, x: f64, y: f64, warp_scale: f64) -> f64 {
        let wx = self.noise_2d(x + warp_scale, y + warp_scale);
        let wy = self.noise_2d(x - warp_scale, y - warp_scale);
        self.noise_2d(x + 4.0 * wx, y + 4.0 * wy)
    }

    /// Core 2D simplex noise implementation.
    fn noise_2d(&self, xin: f64, yin: f64) -> f64 {
        const F2: f64 = 0.5 * (std::f64::consts::SQRT_2 - 1.0);
        const G2: f64 = (std::f64::consts::SQRT_2 - 1.0) / 2.0;

        // Normalize gradients for proper [-1, 1] output range
        // Diagonal gradients have length sqrt(2), so we normalize by that
        const NORMALIZER: f64 = 0.7071067811865475; // 1/sqrt(2)

        // Skew input space to triangular grid
        let s = (xin + yin) * F2;
        let i = (xin + s).floor() as i64;
        let j = (yin + s).floor() as i64;

        // Unskew back to (x,y) space (triangular grid)
        let t = (i as f64 + j as f64) * G2;
        let x0 = xin - (i as f64 - t);
        let y0 = yin - (j as f64 - t);

        // Determine which simplex we're in
        let (i1, j1): (i64, i64) = if x0 > y0 { (1, 0) } else { (0, 1) };

        // Offsets for middle and last corners
        let x1 = x0 - i1 as f64 + G2;
        let y1 = y0 - j1 as f64 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        // Hash coordinates into simplex
        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;

        // Gradient function with proper normalization
        // For axis-aligned (h < 4): length = 1.0
        // For diagonal (h >= 4): length = sqrt(2), so multiply by NORMALIZER
        let grad = |hash: u8, x: f64, y: f64| -> f64 {
            let h = hash & 7;
            // Extract direction components
            let dx = if (h & 4) != 0 { 1.0 } else { 0.0 };
            let dy = if (h & 4) == 0 { 1.0 } else { 0.0 };
            // Flip signs based on bits
            let sx = if (h & 1) != 0 { -dx } else { dx };
            let sy = if (h & 2) != 0 { -dy } else { dy };
            // Apply normalization for diagonal gradients
            if h < 4 {
                sx * x + sy * y
            } else {
                (sx * x + sy * y) * NORMALIZER
            }
        };

        // Calculate contributions from three corners
        // Use squared distance falloff (quintic for smoother results)
        let t0 = 0.5 - x0 * x0 - y0 * y0;
        let n0 = if t0 < 0.0 {
            0.0
        } else {
            let t0_2 = t0 * t0;
            t0_2 * t0_2 * grad(self.perm[ii + self.perm[jj] as usize], x0, y0)
        };

        let t1 = 0.5 - x1 * x1 - y1 * y1;
        let n1 = if t1 < 0.0 {
            0.0
        } else {
            let t1_2 = t1 * t1;
            t1_2 * t1_2
                * grad(
                    self.perm[ii + i1 as usize + self.perm[jj + j1 as usize] as usize],
                    x1,
                    y1,
                )
        };

        let t2 = 0.5 - x2 * x2 - y2 * y2;
        let n2 = if t2 < 0.0 {
            0.0
        } else {
            let t2_2 = t2 * t2;
            t2_2 * t2_2
                * grad(
                    self.perm[ii + i1 as usize + 1 + self.perm[jj + j1 as usize + 1] as usize],
                    x2,
                    y2,
                )
        };

        // Scale to approximate [-1, 1] range
        70.0 * (n0 + n1 + n2).clamp(-1.0, 1.0)
    }

    /// Multi-octave noise for more natural patterns.
    pub fn octave_noise_2d(
        &self,
        x: f64,
        y: f64,
        octaves: usize,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += self.noise_2d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        value / max_value
    }
}

/// 3D Simplex noise generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplexNoise3D {
    seed: u64,
    perm: Vec<u8>,
}

impl SimplexNoise3D {
    const PERM_SIZE: usize = 256;

    pub fn new(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut perm = vec![0u8; Self::PERM_SIZE * 2];

        for i in 0..Self::PERM_SIZE {
            perm[i] = i as u8;
        }

        for i in (1..Self::PERM_SIZE).rev() {
            let j = (rng.next() % (i + 1) as u64) as usize;
            perm.swap(i, j);
        }

        for i in 0..Self::PERM_SIZE {
            perm[Self::PERM_SIZE + i] = perm[i];
        }

        Self { seed, perm }
    }

    /// 3D simplex noise.
    pub fn get(&self, x: f64, y: f64, z: f64) -> f64 {
        self.noise_3d(x, y, z)
    }

    fn noise_3d(&self, xin: f64, yin: f64, zin: f64) -> f64 {
        const F3: f64 = 1.0 / 3.0;
        const G3: f64 = 1.0 / 6.0;

        let s = (xin + yin + zin) * F3;
        let i = (xin + s).floor() as i64;
        let j = (yin + s).floor() as i64;
        let k = (zin + s).floor() as i64;

        let t = (i as f64 + j as f64 + k as f64) * G3;
        let x0 = xin - (i as f64 - t);
        let y0 = yin - (j as f64 - t);
        let z0 = zin - (k as f64 - t);

        let (i1, j1, k1, x1, y1, z1, x2, y2, z2);

        if x0 >= y0 {
            if y0 >= z0 {
                i1 = 1;
                j1 = 0;
                k1 = 0;
                x1 = 1.0;
                y1 = 0.0;
                z1 = 0.0;
                x2 = 1.0;
                y2 = 1.0;
                z2 = 0.0;
            } else if x0 >= z0 {
                i1 = 1;
                j1 = 0;
                k1 = 0;
                x1 = 1.0;
                y1 = 0.0;
                z1 = 0.0;
                x2 = 1.0;
                y2 = 0.0;
                z2 = 1.0;
            } else {
                i1 = 0;
                j1 = 0;
                k1 = 1;
                x1 = 0.0;
                y1 = 0.0;
                z1 = 1.0;
                x2 = 1.0;
                y2 = 0.0;
                z2 = 1.0;
            }
        } else if y0 < z0 {
            i1 = 0;
            j1 = 0;
            k1 = 1;
            x1 = 0.0;
            y1 = 0.0;
            z1 = 1.0;
            x2 = 0.0;
            y2 = 1.0;
            z2 = 1.0;
        } else if x0 < z0 {
            i1 = 0;
            j1 = 1;
            k1 = 0;
            x1 = 0.0;
            y1 = 1.0;
            z1 = 0.0;
            x2 = 0.0;
            y2 = 1.0;
            z2 = 1.0;
        } else {
            i1 = 0;
            j1 = 1;
            k1 = 0;
            x1 = 0.0;
            y1 = 1.0;
            z1 = 0.0;
            x2 = 1.0;
            y2 = 1.0;
            z2 = 0.0;
        }

        let x3 = x0 - x1 + G3;
        let y3 = y0 - y1 + G3;
        let z3 = z0 - z1 + G3;
        let x4 = x0 - x2 + 2.0 * G3;
        let y4 = y0 - y2 + 2.0 * G3;
        let z4 = z0 - z2 + 2.0 * G3;
        let x5 = x0 - 1.0 + 3.0 * G3;
        let y5 = y0 - 1.0 + 3.0 * G3;
        let z5 = z0 - 1.0 + 3.0 * G3;

        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        let kk = (k & 255) as usize;

        let grad = |hash: u8, x: f64, y: f64, z: f64| -> f64 {
            let h = hash & 15;
            let u = if h < 8 { x } else { y };
            let vt = if h == 12 || h == 15 { x } else { z };
            let v = if h < 4 { y } else { vt };
            (if (h & 1) != 0 { -u } else { u }) + if (h & 2) != 0 { -v } else { v }
        };

        let mut n0 = 0.0;
        let mut n1 = 0.0;
        let mut n2 = 0.0;
        let mut n3 = 0.0;

        let t0 = 0.6 - x0 * x0 - y0 * y0 - z0 * z0;
        if t0 >= 0.0 {
            let t0_2 = t0 * t0;
            n0 = t0_2
                * t0_2
                * grad(
                    self.perm[ii + self.perm[jj + self.perm[kk] as usize] as usize],
                    x0,
                    y0,
                    z0,
                );
        }

        let t1 = 0.6 - x1 * x1 - y1 * y1 - z1 * z1;
        if t1 >= 0.0 {
            let t1_2 = t1 * t1;
            n1 = t1_2
                * t1_2
                * grad(
                    self.perm[ii + i1 + self.perm[jj + j1 + self.perm[kk + k1] as usize] as usize],
                    x1,
                    y1,
                    z1,
                );
        }

        let t2 = 0.6 - x2 * x2 - y2 * y2 - z2 * z2;
        if t2 >= 0.0 {
            let t2_2 = t2 * t2;
            n2 = t2_2
                * t2_2
                * grad(
                    self.perm[ii
                        + i1
                        + 1
                        + self.perm[jj + j1 + 1 + self.perm[kk + k1 + 1] as usize] as usize],
                    x2,
                    y2,
                    z2,
                );
        }

        let t3 = 0.6 - x3 * x3 - y3 * y3 - z3 * z3;
        if t3 >= 0.0 {
            let t3_2 = t3 * t3;
            n3 = t3_2
                * t3_2
                * grad(
                    self.perm[ii + 1 + self.perm[jj + 1 + self.perm[kk + 1] as usize] as usize],
                    x3,
                    y3,
                    z3,
                );
        }

        let t4 = 0.6 - x4 * x4 - y4 * y4 - z4 * z4;
        if t4 >= 0.0 {
            let t4_2 = t4 * t4;
            n3 += t4_2
                * t4_2
                * grad(
                    self.perm[ii + i1 + self.perm[jj + j1 + self.perm[kk + k1] as usize] as usize],
                    x4,
                    y4,
                    z4,
                );
        }

        let t5 = 0.6 - x5 * x5 - y5 * y5 - z5 * z5;
        if t5 >= 0.0 {
            let t5_2 = t5 * t5;
            n3 += t5_2
                * t5_2
                * grad(
                    self.perm[ii + 1 + self.perm[jj + 1 + self.perm[kk + 1] as usize] as usize],
                    x5,
                    y5,
                    z5,
                );
        }

        32.0 * (n0 + n1 + n2 + n3)
    }

    pub fn octave_noise_3d(
        &self,
        x: f64,
        y: f64,
        z: f64,
        octaves: usize,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        let mut value = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            value += self.noise_3d(x * frequency, y * frequency, z * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        value / max_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_deterministic() {
        let noise = SimplexNoise::new(12345);

        let v1 = noise.get(0.5, 0.5);
        let v2 = noise.get(0.5, 0.5);

        assert_eq!(v1, v2);
    }

    #[test]
    fn test_noise_range() {
        let noise = SimplexNoise::new(42);

        for i in 0..100 {
            let x = i as f64 * 0.1;
            let y = i as f64 * 0.1;
            let value = noise.get(x, y);
            assert!(
                value >= -1.0 && value <= 1.0,
                "Noise out of range: {}",
                value
            );
        }
    }

    #[test]
    fn test_octave_noise() {
        let noise = SimplexNoise::new(42);

        let value = noise.octave_noise_2d(0.5, 0.5, 4, 0.5, 2.0);
        assert!(value >= -1.0 && value <= 1.0);
    }

    #[test]
    fn test_different_seeds() {
        let noise1 = SimplexNoise::new(100);
        let noise2 = SimplexNoise::new(200);

        let v1 = noise1.get(0.5, 0.5);
        let v2 = noise2.get(0.5, 0.5);

        assert_ne!(v1, v2);
    }

    #[test]
    fn test_3d_noise() {
        let noise = SimplexNoise3D::new(42);

        let v = noise.get(0.5, 0.5, 0.5);
        assert!(v >= -1.0 && v <= 1.0);
    }
}
