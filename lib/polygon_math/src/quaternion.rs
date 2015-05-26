use std::ops::Mul;
use std::f32::consts::PI;

use vector::Vector3;
use matrix::Matrix4;
use IsZero;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Quaternion {
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Quaternion {
    /// Creates an identity quaternion.
    ///
    /// The identity quaternion is the quaternion that can be multiplied into any other quaternion
    /// without changing it.
    pub fn identity() -> Quaternion {
        Quaternion {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Creates a quaternion from an axis and a rotation around that axis.
    ///
    /// # Params
    ///
    /// - axis - The axis being used to represent the rotation. This should
    ///   be normalized before being passed into `axis_angle()`.
    pub fn axis_angle(axis: Vector3, angle: f32) -> Quaternion {
        let s = (angle * 0.5).sin();
        Quaternion {
            w: (angle * 0.5).cos(),
            x: s * axis.x,
            y: s * axis.y,
            z: s * axis.z,
        }
    }

    /// Creates a quaternion that rotates an object to look in the specified direction.
    pub fn look_rotation(forward: Vector3, up: Vector3) -> Quaternion {
        let source = Vector3::forward();
        let forward = forward.normalized();
        let up = up.normalized();

        let dot = source.dot(forward);

        if (dot + 1.0).is_zero() {
            // vector a and b point exactly in the opposite direction,
            // so it is a 180 degrees turn around the up-axis
            return Quaternion::axis_angle(up, PI)
        }

        if (dot - 1.0).is_zero() {
            // Vector a and b point exactly in the same direction
            // so we return the identity quaternion.
            return Quaternion::identity()
        }

        let rotAngle = dot.acos();
        let rotAxis = Vector3::cross(source, forward).normalized();// source.cross(forward).normalized();
        return Quaternion::axis_angle(rotAxis, rotAngle)
    }

    /// Creates a quaternion from a set of euler angles.
    pub fn from_eulers(x: f32, y: f32, z: f32) -> Quaternion {
        Quaternion::axis_angle(Vector3::new(1.0, 0.0, 0.0), x)
      * Quaternion::axis_angle(Vector3::new(0.0, 1.0, 0.0), y)
      * Quaternion::axis_angle(Vector3::new(0.0, 0.0, 1.0), z)
    }

    /// Converts the quaternion to the corresponding rotation matrix.
    pub fn as_matrix(&self) -> Matrix4 {
        Matrix4::from_quaternion(self)
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: (self.w * rhs.w) - (self.x * rhs.x) - (self.y * rhs.y) - (self.z * rhs.z),
            x: (self.w * rhs.x) + (self.x * rhs.w) + (self.y * rhs.z) - (self.z * rhs.y),
            y: (self.w * rhs.y) - (self.x * rhs.z) + (self.y * rhs.w) + (self.z * rhs.x),
            z: (self.w * rhs.z) + (self.x * rhs.y) - (self.y * rhs.x) + (self.z * rhs.w),
        }
    }
}