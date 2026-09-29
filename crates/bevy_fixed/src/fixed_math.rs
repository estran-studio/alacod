use std::hash::{Hash, Hasher};

use bevy::{math::Affine3A, prelude::*};
use fixed::{
    types::extra::{U16, U32},
    FixedI32, FixedI64,
};
use fixed_trigonometry::atan::atan2;
use fixed_trigonometry::*;
use serde::{Deserialize, Serialize};

// Define our fixed-point types
// Using 16.16 fixed point for general use (good balance of range and precision)
pub type Fixed = FixedI32<U16>;
// Using 32.32 for intermediate calculations that need more precision
pub type FixedWide = FixedI64<U32>;

pub fn new(f: f32) -> Fixed {
    Fixed::from_num(f)
}

pub fn to_f32(f: Fixed) -> f32 {
    f.to_num::<f32>()
}

// Fixed-point 2D vector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FixedVec2 {
    pub x: Fixed,
    pub y: Fixed,
}

// Fixed-point 3D vector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FixedVec3 {
    pub x: Fixed,
    pub y: Fixed,
    pub z: Fixed,
}

// --- The Wrapper Type for Reflection ---
impl std::fmt::Display for FixedVec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "( {:.3} {:.3} )",
            self.x.to_num::<f32>(),
            self.y.to_num::<f32>()
        )
    }
}

impl std::fmt::Display for FixedVec3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "( {:.3} {:.3} {:.3} )",
            self.x.to_num::<f32>(),
            self.y.to_num::<f32>(),
            self.z.to_num::<f32>()
        )
    }
}

impl Hash for FixedVec3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl Hash for FixedVec2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

// Conversion constants
pub const FIXED_ZERO: Fixed = Fixed::from_bits(0);
pub const FIXED_ONE: Fixed = Fixed::from_bits(1 << 16);
pub const FIXED_HALF: Fixed = Fixed::from_bits(1 << 15);
pub const FIXED_PI: Fixed = Fixed::from_bits(205887);
pub const FIXED_TAU: Fixed = Fixed::from_bits(411775);
pub const FIXED_32_MAX: Fixed = FixedI32::<U16>::MAX;
pub const FIXED_32_MIN: Fixed = FixedI32::<U16>::MIN;

impl FixedVec2 {
    pub const ZERO: Self = Self {
        x: Fixed::from_bits(0),
        y: Fixed::from_bits(0),
    };

    pub fn new(x: Fixed, y: Fixed) -> Self {
        Self { x, y }
    }

    pub fn from_f32(x: f32, y: f32) -> Self {
        Self {
            x: Fixed::from_num(x),
            y: Fixed::from_num(y),
        }
    }

    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x.to_num::<f32>(), self.y.to_num::<f32>())
    }

    pub fn extend(&self) -> FixedVec3 {
        FixedVec3 {
            x: self.x,
            y: self.y,
            z: FIXED_ZERO,
        }
    }

    pub fn dot(&self, other: &Self) -> Fixed {
        self.x
            .saturating_mul(other.x)
            .saturating_add(self.y.saturating_mul(other.y))
    }

    /// Calculates the squared length of the vector using FixedWide for intermediate precision.
    pub fn length_squared_as_wide(&self) -> FixedWide {
        // Convert components from Fixed (I32F16) to FixedWide (I64F32) before squaring
        // Assuming FixedWide has a way to be created from Fixed.
        // If not, use .to_num::<f32>() and FixedWide::from_num(), but direct conversion is better.
        // Let's assume you add a helper or use from_num for now:
        let x_fw = FixedWide::from_num(self.x.to_num::<f32>());
        let y_fw = FixedWide::from_num(self.y.to_num::<f32>());

        // Perform multiplication in the wider FixedWide type
        let x_sq_fw = x_fw.saturating_mul(x_fw); // FixedWide * FixedWide -> FixedWide
        let y_sq_fw = y_fw.saturating_mul(y_fw); // FixedWide * FixedWide -> FixedWide

        x_sq_fw.saturating_add(y_sq_fw)
    }

    /// Calculates the length of the vector.
    pub fn length(&self) -> Fixed {
        // Returns your standard Fixed type (I32F16)
        let len_sq_fw = self.length_squared_as_wide(); // This is FixedWide (I64F32)

        // Ensure the squared length is not negative (shouldn't happen with squares)
        if len_sq_fw < FixedWide::ZERO {
            return FIXED_ZERO;
        }

        // Perform sqrt on the FixedWide type.
        // This assumes FixedSqrt trait is implemented for FixedWide, returning FixedWide.
        let len_fw = len_sq_fw.sqrt(); // FixedWide.sqrt() -> FixedWide

        // Convert the result (which is FixedWide) back to your standard Fixed type.
        // This might clamp if the actual length is too large for Fixed (I32F16),
        // but for your example distance of ~1831, it should fit.
        // (FixedI32<U16> max is ~32767, so 1831 fits)
        // Again, using .to_num::<f32>() for conversion:
        Fixed::from_num(len_fw.to_num::<f32>())
    }

    pub fn length_squared(&self) -> FixedWide {
        // Convert components from Fixed (I32F16) to FixedWide (I64F32) before squaring.
        // It's best if FixedWide can be directly constructed from Fixed.
        // Example: let x_fw = fixed_math::FixedWide::from(self.x);
        // If not, using .to_num::<f32>() is a common intermediate step:
        let x_fw = FixedWide::from_num(self.x.to_num::<f32>());
        let y_fw = FixedWide::from_num(self.y.to_num::<f32>());

        // Perform multiplication in the wider FixedWide type
        let x_sq_fw = x_fw.saturating_mul(x_fw); // FixedWide * FixedWide -> FixedWide
        let y_sq_fw = y_fw.saturating_mul(y_fw); // FixedWide * FixedWide -> FixedWide

        x_sq_fw.saturating_add(y_sq_fw)
    }

    // distance() method remains the same as it calls the updated length():
    pub fn distance(&self, other: &Self) -> Fixed {
        (*self - *other).length()
    }

    /// Calculates the squared distance to another vector.
    /// Returns a FixedWide for precision, consistent with length_squared().
    pub fn distance_squared(&self, other: &Self) -> FixedWide {
        // << Return FixedWide
        // This will now use the corrected length_squared() that returns FixedWide
        (*self - *other).length_squared()
    }

    // Optional: If you frequently need squared distance and want to avoid sqrt,
    // you might provide distance_squared_as_wide:
    pub fn distance_squared_as_wide(&self, other: &Self) -> FixedWide {
        (*self - *other).length_squared_as_wide()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > Fixed::from_bits(0) {
            Self {
                x: self.x.saturating_div(len),
                y: self.y.saturating_div(len),
            }
        } else {
            Self::ZERO
        }
    }

    pub fn normalize_or_zero(&self) -> Self {
        let len_sq = self.length_squared();
        if len_sq > Fixed::from_bits(256) {
            // Small epsilon to avoid division by very small numbers
            self.normalize()
        } else {
            Self::ZERO
        }
    }

    pub fn clamp_length_max(&self, max: Fixed) -> Self {
        let len_sq = self.length_squared();
        let max_sq = max.saturating_mul(max);
        if len_sq > max_sq {
            self.normalize() * max
        } else {
            *self
        }
    }
}

impl std::ops::Add for FixedVec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
        }
    }
}

impl std::ops::Sub for FixedVec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        }
    }
}

impl std::ops::Mul<Fixed> for FixedVec2 {
    type Output = Self;
    fn mul(self, scalar: Fixed) -> Self {
        Self {
            x: self.x.saturating_mul(scalar),
            y: self.y.saturating_mul(scalar),
        }
    }
}

impl std::ops::Div<Fixed> for FixedVec2 {
    type Output = Self;
    fn div(self, scalar: Fixed) -> Self {
        Self {
            x: self.x.saturating_div(scalar),
            y: self.y.saturating_div(scalar),
        }
    }
}

impl std::ops::AddAssign for FixedVec2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl std::ops::Neg for FixedVec2 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl FixedVec3 {
    pub const ZERO: Self = Self {
        x: Fixed::from_bits(0),
        y: Fixed::from_bits(0),
        z: Fixed::from_bits(0),
    };

    pub const ONE: Self = Self {
        x: FIXED_ONE,
        y: FIXED_ONE,
        z: FIXED_ONE,
    };

    pub fn new(x: Fixed, y: Fixed, z: Fixed) -> Self {
        Self { x, y, z }
    }

    pub fn from_f32(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: Fixed::from_num(x),
            y: Fixed::from_num(y),
            z: Fixed::from_num(z),
        }
    }

    pub fn to_vec3(&self) -> Vec3 {
        Vec3::new(
            self.x.to_num::<f32>(),
            self.y.to_num::<f32>(),
            self.z.to_num::<f32>(),
        )
    }

    pub fn truncate(&self) -> FixedVec2 {
        FixedVec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn dot(&self, other: &Self) -> Fixed {
        self.x
            .saturating_mul(other.x)
            .saturating_add(self.y.saturating_mul(other.y))
            .saturating_add(self.z.saturating_mul(other.z))
    }

    /// Calculates the squared length of the 3D vector.
    /// Returns a FixedWide (e.g., FixedI64<U32>) to maintain precision for larger vectors,
    /// as Fixed (e.g., FixedI32<U16>) might not be able to represent the true squared length.
    pub fn length_squared(&self) -> FixedWide {
        // << Returns FixedWide
        // Convert components from Fixed (I32F16) to FixedWide (I64F32) before squaring.
        // Assuming FixedWide::from_num(fixed_value.to_num::<f32>()) is your conversion path.
        // If you have direct Fixed -> FixedWide conversion, use that (e.g., FixedWide::from(self.x)).
        let x_fw = FixedWide::from_num(self.x.to_num::<f32>());
        let y_fw = FixedWide::from_num(self.y.to_num::<f32>());
        let z_fw = FixedWide::from_num(self.z.to_num::<f32>());

        // Perform multiplication in the wider FixedWide type
        let x_sq_fw = x_fw.saturating_mul(x_fw); // FixedWide * FixedWide -> FixedWide
        let y_sq_fw = y_fw.saturating_mul(y_fw); // FixedWide * FixedWide -> FixedWide
        let z_sq_fw = z_fw.saturating_mul(z_fw); // FixedWide * FixedWide -> FixedWide

        // Sum the squared components (all FixedWide)
        x_sq_fw.saturating_add(y_sq_fw).saturating_add(z_sq_fw)
    }

    /// Calculates the length (magnitude) of the 3D vector.
    /// Returns the standard Fixed type (e.g., FixedI32<U16>).
    pub fn length(&self) -> Fixed {
        // << Final return type is Fixed (I32F16)
        // Step 1: self.length_squared() now returns FixedWide.
        let len_sq_fw: FixedWide = self.length_squared();

        // Step 2: Ensure the squared length is not negative.
        if len_sq_fw < FixedWide::ZERO {
            return FIXED_ZERO; // Or appropriate error/default
        }

        // Step 3: Perform sqrt on the FixedWide value.
        // This assumes your FixedWide type implements the FixedSqrt trait (or has a .sqrt() method)
        // and that this sqrt() method returns FixedWide.
        let len_fw: FixedWide = len_sq_fw.sqrt();

        // Step 4: Convert the FixedWide length back to the standard Fixed type.
        // This conversion might clamp if the true length exceeds Fixed::MAX (~32767),
        // but this is less likely for length than for length_squared.
        Fixed::from_num(len_fw.to_num::<f32>())
        // If you have a more direct/robust FixedWide -> Fixed conversion, use it.
    }

    // You should also provide distance and distance_squared for FixedVec3 if they don't exist
    // or update them if they do.

    /// Calculates the distance to another 3D vector.
    /// Returns the standard Fixed type.
    pub fn distance(&self, other: &Self) -> Fixed {
        // This will use the corrected length() method for FixedVec3
        (*self - *other).length() // Relies on Sub for FixedVec3 being implemented
    }

    /// Calculates the squared distance to another 3D vector.
    /// Returns a FixedWide for precision, consistent with length_squared().
    pub fn distance_squared(&self, other: &Self) -> FixedWide {
        // << Return FixedWide
        // This calls the length_squared() method that correctly returns FixedWide
        (*self - *other).length_squared() // Relies on Sub for FixedVec3
    }

    pub fn mul_element_wise(&self, other: Self) -> Self {
        Self {
            x: self.x.saturating_mul(other.x),
            y: self.y.saturating_mul(other.y),
            z: self.z.saturating_mul(other.z),
        }
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > Fixed::from_bits(0) {
            // Exactly zero check is fine for fixed point
            Self {
                x: self.x.saturating_div(len),
                y: self.y.saturating_div(len),
                z: self.z.saturating_div(len),
            }
        } else {
            // Behavior for zero-length vector normalization:
            // Option 1: Return ZERO (common)
            Self::ZERO
            // Option 2: Return a default direction e.g., (1,0,0) if that makes sense for your game
            // Self::new(super::FIXED_ONE, Fixed::ZERO, Fixed::ZERO)
            // Option 3: Panic, if normalizing a zero vector is considered an unrecoverable error
            // panic!("Attempted to normalize a zero-length FixedVec3");
        }
    }

    pub fn normalize_or_zero(&self) -> Self {
        let len_sq = self.length_squared();
        // Using the same epsilon as FixedVec2 (256 in bits for 16.16 is 256/65536 = 0.00390625)
        // This epsilon applies to length_squared.
        if len_sq > Fixed::from_bits(256) {
            self.normalize()
        } else {
            Self::ZERO
        }
    }

    pub fn clamp_length_max(&self, max: Fixed) -> Self {
        let len_sq = self.length_squared();
        let max_sq = max.saturating_mul(max);
        if len_sq > max_sq {
            // self.normalize() * max // Relies on Mul<Fixed> impl
            let normalized = self.normalize(); // Avoid normalizing if length is zero by using normalize logic
            if normalized == Self::ZERO && max > Fixed::ZERO {
                // If original was zero, but max is positive, result is still zero
                Self::ZERO
            } else {
                normalized.saturating_mul_scalar(max) // Using a helper for clarity or direct if Mul implemented
            }
        } else {
            *self
        }
    }

    // Helper for scalar multiplication if you prefer named methods too
    pub fn saturating_mul_scalar(&self, scalar: Fixed) -> Self {
        Self {
            x: self.x.saturating_mul(scalar),
            y: self.y.saturating_mul(scalar),
            z: self.z.saturating_mul(scalar),
        }
    }

    // Helper for scalar division if you prefer named methods too
    pub fn saturating_div_scalar(&self, scalar: Fixed) -> Self {
        // Consider division by zero: saturating_div will return Fixed::MAX/MIN.
        // Assert or handle as per your game's requirements if scalar can be zero.
        Self {
            x: self.x.saturating_div(scalar),
            y: self.y.saturating_div(scalar),
            z: self.z.saturating_div(scalar),
        }
    }

    // Cross product for FixedVec3
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self
                .y
                .saturating_mul(other.z)
                .saturating_sub(self.z.saturating_mul(other.y)),
            y: self
                .z
                .saturating_mul(other.x)
                .saturating_sub(self.x.saturating_mul(other.z)),
            z: self
                .x
                .saturating_mul(other.y)
                .saturating_sub(self.y.saturating_mul(other.x)),
        }
    }

    pub fn splat(value: Fixed) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }
}

// --- Operator Overloads ---

impl std::ops::Add for FixedVec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
            z: self.z.saturating_add(other.z),
        }
    }
}

impl std::ops::AddAssign for FixedVec3 {
    fn add_assign(&mut self, other: Self) {
        self.x = self.x.saturating_add(other.x);
        self.y = self.y.saturating_add(other.y);
        self.z = self.z.saturating_add(other.z);
    }
}

impl std::ops::Sub for FixedVec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
            z: self.z.saturating_sub(other.z),
        }
    }
}

impl std::ops::SubAssign for FixedVec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x = self.x.saturating_sub(other.x);
        self.y = self.y.saturating_sub(other.y);
        self.z = self.z.saturating_sub(other.z);
    }
}

impl std::ops::Mul<Fixed> for FixedVec3 {
    // Scalar multiplication
    type Output = Self;
    fn mul(self, scalar: Fixed) -> Self {
        self.saturating_mul_scalar(scalar)
    }
}

// Optional: if you want `Fixed * FixedVec3`
// impl std::ops::Mul<FixedVec3> for Fixed {
//     type Output = FixedVec3;
//     fn mul(self, vec: FixedVec3) -> FixedVec3 {
//         vec.saturating_mul_scalar(self)
//     }
// }

impl std::ops::Div<Fixed> for FixedVec3 {
    // Scalar division
    type Output = Self;
    fn div(self, scalar: Fixed) -> Self {
        // Add handling for division by zero if `saturating_div`'s behavior (returning MAX/MIN)
        // is not desired for your game logic.
        // e.g., if scalar == Fixed::from_bits(0) { panic!("Division by zero"); }
        self.saturating_div_scalar(scalar)
    }
}

impl std::ops::Neg for FixedVec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x, // Assumes Fixed implements Neg
            y: -self.y,
            z: -self.z,
        }
    }
}

// Fixed-point angle functions
pub fn atan2_fixed(y: Fixed, x: Fixed) -> Fixed {
    // Convert to the format expected by fixed_trigonometry
    // This returns a fixed-point angle in radians
    atan2(y, x)
}

pub fn sin_fixed(angle: Fixed) -> Fixed {
    sin(angle)
}

pub fn cos_fixed(angle: Fixed) -> Fixed {
    cos(angle)
}

// Fixed-point matrix for rotations
#[derive(Debug, Clone, Copy)]
pub struct FixedMat2 {
    pub m00: Fixed,
    pub m01: Fixed,
    pub m10: Fixed,
    pub m11: Fixed,
}

impl FixedMat2 {
    pub fn from_angle(angle: Fixed) -> Self {
        let c = cos_fixed(angle);
        let s = sin_fixed(angle);
        Self {
            m00: c,
            m01: -s,
            m10: s,
            m11: c,
        }
    }

    pub fn mul_vec2(&self, v: FixedVec2) -> FixedVec2 {
        FixedVec2 {
            x: self
                .m00
                .saturating_mul(v.x)
                .saturating_add(self.m01.saturating_mul(v.y)),
            y: self
                .m10
                .saturating_mul(v.x)
                .saturating_add(self.m11.saturating_mul(v.y)),
        }
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct FixedMat3 {
    // Columns of the matrix
    pub x_axis: FixedVec3, // First column
    pub y_axis: FixedVec3, // Second column
    pub z_axis: FixedVec3, // Third column
}

impl FixedMat3 {
    pub const IDENTITY: Self = Self {
        x_axis: FixedVec3 {
            x: FIXED_ONE,
            y: Fixed::ZERO,
            z: Fixed::ZERO,
        },
        y_axis: FixedVec3 {
            x: Fixed::ZERO,
            y: FIXED_ONE,
            z: Fixed::ZERO,
        },
        z_axis: FixedVec3 {
            x: Fixed::ZERO,
            y: Fixed::ZERO,
            z: FIXED_ONE,
        },
    };

    // Helper to create from a Bevy Quat (via Bevy Mat3)
    // This is useful for initializing your fixed-point rotation from Bevy's system
    pub fn from_rotation_bevy_quat(q: Quat) -> Self {
        let mat3 = Mat3::from_quat(q);
        Self {
            x_axis: vec3_to_fixed(mat3.x_axis),
            y_axis: vec3_to_fixed(mat3.y_axis),
            z_axis: vec3_to_fixed(mat3.z_axis),
        }
    }

    // Multiply by a FixedVec3 (M * v)
    pub fn mul_vec3(&self, v: FixedVec3) -> FixedVec3 {
        FixedVec3 {
            x: (self.x_axis.x.saturating_mul(v.x))
                .saturating_add(self.y_axis.x.saturating_mul(v.y))
                .saturating_add(self.z_axis.x.saturating_mul(v.z)),
            y: (self.x_axis.y.saturating_mul(v.x))
                .saturating_add(self.y_axis.y.saturating_mul(v.y))
                .saturating_add(self.z_axis.y.saturating_mul(v.z)),
            z: (self.x_axis.z.saturating_mul(v.x))
                .saturating_add(self.y_axis.z.saturating_mul(v.y))
                .saturating_add(self.z_axis.z.saturating_mul(v.z)),
        }
    }

    // If you need to create rotation matrices from Euler angles (e.g., YXZ order):
    pub fn from_euler_angles_yxz(angles_rad_fixed: FixedVec3) -> Self {
        let sx = sin_fixed(angles_rad_fixed.x);
        let cx = cos_fixed(angles_rad_fixed.x);
        let sy = sin_fixed(angles_rad_fixed.y);
        let cy = cos_fixed(angles_rad_fixed.y);
        let sz = sin_fixed(angles_rad_fixed.z);
        let cz = cos_fixed(angles_rad_fixed.z);

        let x_axis = FixedVec3 {
            x: cy
                .saturating_mul(cz)
                .saturating_add(sx.saturating_mul(sy).saturating_mul(sz)),
            y: cx.saturating_mul(sz),
            z: (cy.saturating_mul(sx).saturating_mul(sz)).saturating_sub(sy.saturating_mul(cz)),
        };
        let y_axis = FixedVec3 {
            x: (cy.saturating_mul(sz)).saturating_sub(sx.saturating_mul(sy).saturating_mul(cz)),
            y: cx.saturating_mul(cz),
            z: sy
                .saturating_mul(sz)
                .saturating_add(cy.saturating_mul(sx).saturating_mul(cz)),
        };
        let z_axis = FixedVec3 {
            x: cx.saturating_mul(sy),
            y: sx.saturating_mul(-FIXED_ONE), // -sx
            z: cy.saturating_mul(cx),
        };
        Self {
            x_axis,
            y_axis,
            z_axis,
        }
    }

    pub fn from_rotation_z(angle: Fixed) -> Self {
        let c = cos_fixed(angle); // Assumes cos_fixed(Fixed) -> Fixed exists
        let s = sin_fixed(angle); // Assumes sin_fixed(Fixed) -> Fixed exists

        // Standard Z-axis rotation matrix:
        // [ c, -s,  0 ]
        // [ s,  c,  0 ]
        // [ 0,  0,  1 ]
        // Stored column-major in FixedMat3 (x_axis, y_axis, z_axis)

        Self {
            x_axis: FixedVec3::new(c, s, Fixed::ZERO),
            y_axis: FixedVec3::new(-s, c, Fixed::ZERO), // -s uses std::ops::Neg for Fixed
            z_axis: FixedVec3::new(Fixed::ZERO, Fixed::ZERO, FIXED_ONE), // Assumes FIXED_ONE is your 1.0 Fixed value
        }
    }

    // Multiply this matrix by another FixedMat3 (self * other)
    pub fn mul_mat3(&self, other: &FixedMat3) -> Self {
        // Each column of the new matrix is self (this matrix) multiplied
        // by the corresponding column-vector of the 'other' matrix.
        let new_x_axis = self.mul_vec3(other.x_axis); // Or pass by reference if mul_vec3 accepts it
        let new_y_axis = self.mul_vec3(other.y_axis);
        let new_z_axis = self.mul_vec3(other.z_axis);

        Self {
            x_axis: new_x_axis,
            y_axis: new_y_axis,
            z_axis: new_z_axis,
        }
    }
}

// Ensure FixedVec3 has arithmetic operations like Add, Sub.
// Example for Add, if not already present:
/*
impl std::ops::Add for FixedVec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_add(other.x),
            y: self.y.saturating_add(other.y),
            z: self.z.saturating_add(other.z),
        }
    }
}
impl std::ops::Sub for FixedVec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
            z: self.z.saturating_sub(other.z),
        }
    }
}
// ... and other ops like AddAssign, SubAssign, Neg, Mul<Fixed>, Div<Fixed>
*/

// Conversion helpers
pub fn vec2_to_fixed(v: Vec2) -> FixedVec2 {
    FixedVec2::from_f32(v.x, v.y)
}

pub fn vec3_to_fixed(v: Vec3) -> FixedVec3 {
    FixedVec3 {
        x: Fixed::from_num(v.x),
        y: Fixed::from_num(v.y),
        z: Fixed::from_num(v.z),
    }
}

pub fn fixed_to_vec3(v: FixedVec3) -> Vec3 {
    Vec3::new(
        v.x.to_num::<f32>(),
        v.y.to_num::<f32>(),
        v.z.to_num::<f32>(),
    )
}

pub fn fixed_to_vec2(v: FixedVec2) -> Vec2 {
    Vec2::new(v.x.to_num::<f32>(), v.y.to_num::<f32>())
}

#[derive(Component, Clone, Copy, Default, Serialize, Deserialize)]
pub struct FixedPosition {
    pub value: FixedVec3,
}

impl From<&FixedVec3> for FixedPosition {
    fn from(val: &FixedVec3) -> Self {
        FixedPosition { value: *val }
    }
}

#[derive(Debug, Component, Clone, Hash, Serialize, Deserialize)]
pub struct FixedTransform3D {
    pub translation: FixedVec3,
    pub rotation: FixedMat3,
    pub scale: FixedVec3,
}

impl FixedTransform3D {
    pub const IDENTITY: Self = Self {
        translation: FixedVec3::ZERO,
        rotation: FixedMat3::IDENTITY,
        scale: FixedVec3::ONE, // Default scale is 1,1,1
    };

    pub fn new(translation: FixedVec3, rotation: FixedMat3, scale: FixedVec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    // Equivalent to Bevy's Transform::transform_point, now including scale
    // Order: Scale -> Rotate -> Translate
    pub fn transform_point(&self, local_point: FixedVec3) -> FixedVec3 {
        // 1. Apply scale
        let scaled_point = local_point.mul_element_wise(self.scale);

        // 2. Apply rotation
        let rotated_point = self.rotation.mul_vec3(scaled_point);

        // 3. Apply translation
        // Assumes FixedVec3 implements std::ops::Add
        rotated_point + self.translation
    }

    // Helper to create from Bevy's Transform for initialization
    pub fn from_bevy_transform(transform: &Transform) -> Self {
        // Compute the Affine3A matrix from the Transform
        let affine: Affine3A = transform.compute_affine();

        // Now decompose the Affine3A matrix
        let (scale_f32, rot_quat_f32, translation_f32) = affine.to_scale_rotation_translation();

        Self {
            translation: vec3_to_fixed(translation_f32), // Assuming vec3_to_fixed exists
            rotation: FixedMat3::from_rotation_bevy_quat(rot_quat_f32), // Assuming this exists
            scale: vec3_to_fixed(scale_f32),             // Convert f32 scale to FixedVec3
        }
    }

    // If you need to convert back to a Bevy Transform (e.g., for rendering sync)
    // This can be lossy, especially the rotation part (Mat3 -> Quat).
    pub fn to_bevy_transform(&self) -> Transform {
        let translation = fixed_to_vec3(self.translation); // Assuming fixed_to_vec3 exists
        let scale = fixed_to_vec3(self.scale);

        // Rotation conversion is the trickiest part to do accurately and robustly
        // from a FixedMat3 back to a Bevy Quat.
        // Placeholder: ideally, you'd have a robust FixedMat3 -> Bevy Quat conversion.
        // One way is FixedMat3 -> bevy::Mat3 -> bevy::Quat
        let bevy_rot_mat3 = bevy::math::Mat3::from_cols(
            fixed_to_vec3(self.rotation.x_axis),
            fixed_to_vec3(self.rotation.y_axis),
            fixed_to_vec3(self.rotation.z_axis),
        );
        let rotation_quat = Quat::from_mat3(&bevy_rot_mat3); // Note: Mat3 to Quat can have issues for some matrices (e.g. non-orthogonal)

        Transform {
            translation,
            rotation: rotation_quat,
            scale,
        }
    }
}

pub fn sync_bevy_transforms_from_fixed(mut query: Query<(&FixedTransform3D, &mut Transform)>) {
    for (fixed_transform, mut bevy_transform) in query.iter_mut() {
        // Sync translation
        bevy_transform.translation = fixed_to_vec3(fixed_transform.translation); // Or fixed_transform.translation.to_vec3()

        // Sync scale - THIS IS THE NEWLY ADDED/UPDATED PART
        bevy_transform.scale = fixed_to_vec3(fixed_transform.scale); // Or fixed_transform.scale.to_vec3()

        // Sync rotation (remains the same as before, handling Mat3 to Quat conversion)
        // This part requires a robust way to convert your FixedMat3 to Bevy's Quat.
        // For example, converting FixedMat3 -> bevy::math::Mat3 -> bevy::prelude::Quat
        let bevy_rot_mat3 = bevy::math::Mat3::from_cols(
            fixed_to_vec3(fixed_transform.rotation.x_axis), // Or .x_axis.to_vec3()
            fixed_to_vec3(fixed_transform.rotation.y_axis), // Or .y_axis.to_vec3()
            fixed_to_vec3(fixed_transform.rotation.z_axis), // Or .z_axis.to_vec3()
        );
        bevy_transform.rotation = Quat::from_mat3(&bevy_rot_mat3);
        // Note: Mat3 to Quat conversion can be problematic for non-orthogonal matrices
        // or matrices with shear. Ensure your FixedMat3 correctly represents a pure rotation.
        // If your primary fixed-point rotation representation is Euler angles or a fixed-point quaternion,
        // converting from that to Bevy's Quat might be more direct or stable.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_vec2_display_precision() {
        // Test with values that have many decimal places
        let vec = FixedVec2::from_f32(-30.34894123, -53.05115789);
        let display = format!("{}", vec);

        // Should only show 3 decimal places
        assert!(
            display.contains("-30.349"),
            "FixedVec2 display should show 3 decimal places for x: {}",
            display
        );
        assert!(
            display.contains("-53.051"),
            "FixedVec2 display should show 3 decimal places for y: {}",
            display
        );

        // Should not contain more than 3 decimal places
        assert!(
            !display.contains("-30.3489"),
            "FixedVec2 display should not show more than 3 decimal places: {}",
            display
        );

        println!("FixedVec2 display: {}", display);
    }

    #[test]
    fn test_fixed_vec3_display_precision() {
        // Test with values that have many decimal places
        let vec = FixedVec3::from_f32(-22.27951234, 15.12345678, -100.99999999);
        let display = format!("{}", vec);

        // Should only show 3 decimal places
        assert!(
            display.contains("-22.280"),
            "FixedVec3 display should show 3 decimal places for x: {}",
            display
        );
        assert!(
            display.contains("15.123"),
            "FixedVec3 display should show 3 decimal places for y: {}",
            display
        );
        assert!(
            display.contains("-101.000"),
            "FixedVec3 display should show 3 decimal places for z: {}",
            display
        );

        println!("FixedVec3 display: {}", display);
    }
}
