//! Right-handed homogeneous transforms for the public Coordinate Inspector.
//!
//! Conventions:
//! - millimetres and degrees,
//! - column vectors `[x, y, z, 1]ᵀ`,
//! - active, right-handed rotations,
//! - row-major storage,
//! - `a.compose(b)` means `a · b`, so `b` is applied first,
//! - `machine_from_cad` maps CAD coordinates into Machine coordinates.
use crate::{
    Result,
    core::{Vec3, dot, finite},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Mat4(pub [[f64; 4]; 4]);

impl Mat4 {
    pub const IDENTITY: Self = Self([
        [1., 0., 0., 0.],
        [0., 1., 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);

    pub fn translation(mm: Vec3) -> Result<Self> {
        if !finite(mm) {
            return Err("Translation must be finite".into());
        }
        let mut matrix = Self::IDENTITY;
        for (row, value) in mm.iter().enumerate() {
            matrix.0[row][3] = *value;
        }
        Ok(matrix)
    }

    pub fn rotation_x(degrees: f64) -> Result<Self> {
        Self::rotation([1., 0., 0.], degrees)
    }

    pub fn rotation_y(degrees: f64) -> Result<Self> {
        Self::rotation([0., 1., 0.], degrees)
    }

    pub fn rotation_z(degrees: f64) -> Result<Self> {
        Self::rotation([0., 0., 1.], degrees)
    }

    pub fn rotation(axis: Vec3, degrees: f64) -> Result<Self> {
        if !finite(axis) || !degrees.is_finite() || (dot(axis, axis) - 1.).abs() > 1e-10 {
            return Err("Rotation needs a finite unit axis and angle".into());
        }
        let [x, y, z] = axis;
        let (s, c) = degrees.to_radians().sin_cos();
        let t = 1. - c;
        Ok(Self([
            [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.],
            [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.],
            [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.],
            [0., 0., 0., 1.],
        ]))
    }

    pub fn compose(self, rhs: Self) -> Self {
        Self(std::array::from_fn(|row| {
            std::array::from_fn(|column| {
                (0..4)
                    .map(|index| self.0[row][index] * rhs.0[index][column])
                    .sum()
            })
        }))
    }

    pub fn point(self, point: Vec3) -> Vec3 {
        std::array::from_fn(|row| {
            self.0[row][3]
                + (0..3)
                    .map(|column| self.0[row][column] * point[column])
                    .sum::<f64>()
        })
    }

    pub fn vector(self, vector: Vec3) -> Vec3 {
        std::array::from_fn(|row| {
            (0..3)
                .map(|column| self.0[row][column] * vector[column])
                .sum()
        })
    }

    pub fn validate(self) -> Result<()> {
        if self.0.iter().flatten().any(|value| !value.is_finite()) || self.0[3] != [0., 0., 0., 1.]
        {
            return Err("Invalid homogeneous matrix".into());
        }
        let columns: [Vec3; 3] =
            std::array::from_fn(|column| std::array::from_fn(|row| self.0[row][column]));
        for row in 0..3 {
            for column in 0..3 {
                let expected = if row == column { 1. } else { 0. };
                if (dot(columns[row], columns[column]) - expected).abs() > 1e-9 {
                    return Err("Matrix is not a rigid transform".into());
                }
            }
        }
        let determinant = dot(cross(columns[0], columns[1]), columns[2]);
        if (determinant - 1.).abs() > 1e-9 {
            return Err("Reflection is not a right-handed rotation".into());
        }
        Ok(())
    }

    pub fn inverse(self) -> Result<Self> {
        self.validate()?;
        let mut inverse = Self::IDENTITY;
        for row in 0..3 {
            for column in 0..3 {
                inverse.0[row][column] = self.0[column][row];
            }
        }
        let origin = self.point([0.; 3]);
        let translated = inverse.vector(origin);
        for (row, value) in translated.iter().enumerate() {
            inverse.0[row][3] = -*value;
        }
        Ok(inverse)
    }
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MachinePose {
    pub axis_labels: [String; 2],
    pub positions_deg: [f64; 2],
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Setup {
    pub id: String,
    pub name: String,
    /// Mathematical active rotation of the CAD/workpiece frame: X, then Y, then Z.
    pub workpiece_rotation_xyz_deg: Vec3,
    pub machine_axis_pose: MachinePose,
    pub machine_from_cad: Mat4,
}

pub fn setup(
    id: &str,
    name: &str,
    angles_xyz_deg: Vec3,
    machine_axis_positions_deg: [f64; 2],
    stock_center_mm: Vec3,
) -> Result<Setup> {
    if !finite(angles_xyz_deg) || !machine_axis_positions_deg.iter().all(|v| v.is_finite()) {
        return Err("Setup values must be finite".into());
    }
    // Rotate about the stock centre while keeping it at the same Machine position.
    let rotation = Mat4::rotation_z(angles_xyz_deg[2])?
        .compose(Mat4::rotation_y(angles_xyz_deg[1])?)
        .compose(Mat4::rotation_x(angles_xyz_deg[0])?);
    let machine_from_cad = Mat4::translation(stock_center_mm)?
        .compose(rotation)
        .compose(Mat4::translation(stock_center_mm.map(|v| -v))?);
    machine_from_cad.validate()?;
    Ok(Setup {
        id: id.into(),
        name: name.into(),
        workpiece_rotation_xyz_deg: angles_xyz_deg,
        machine_axis_pose: MachinePose {
            axis_labels: ["A".into(), "B".into()],
            positions_deg: machine_axis_positions_deg,
            status: "Illustrative physical axis readout; independent from the mathematical workpiece rotation and not a solved machine kinematic pose.".into(),
        },
        machine_from_cad,
    })
}

pub fn demo_setups() -> Result<Vec<Setup>> {
    let center = [60., 38., 10.];
    Ok(vec![
        setup("S0", "Top / datum", [0., 0., 0.], [0., 0.], center)?,
        setup("S1", "Front face", [90., 0., 0.], [90., 0.], center)?,
        setup("S2", "Right face", [0., -90., 0.], [0., -90.], center)?,
        setup(
            "S3",
            "Compound inspection",
            [30., -35., 20.],
            [31., -47.],
            center,
        )?,
    ])
}
