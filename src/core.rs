use crate::Result;
use serde::{Deserialize, Serialize};

pub type Vec3 = [f64; 3];
pub fn dot(a: Vec3, b: Vec3) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub fn sub(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn add(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn norm(a: Vec3) -> f64 {
    dot(a, a).sqrt()
}
pub fn finite(a: Vec3) -> bool {
    a.iter().all(|v| v.is_finite())
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Face {
    XMin,
    XMax,
    YMin,
    YMax,
    ZMin,
    ZMax,
}
impl Face {
    pub const ALL: [Self; 6] = [
        Self::XMin,
        Self::XMax,
        Self::YMin,
        Self::YMax,
        Self::ZMin,
        Self::ZMax,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::XMin => "X_MIN",
            Self::XMax => "X_MAX",
            Self::YMin => "Y_MIN",
            Self::YMax => "Y_MAX",
            Self::ZMin => "Z_MIN",
            Self::ZMax => "Z_MAX",
        }
    }
    pub fn normal(self) -> Vec3 {
        match self {
            Self::XMin => [1., 0., 0.],
            Self::XMax => [-1., 0., 0.],
            Self::YMin => [0., 1., 0.],
            Self::YMax => [0., -1., 0.],
            Self::ZMin => [0., 0., 1.],
            Self::ZMax => [0., 0., -1.],
        }
    }
    pub fn axis(self) -> usize {
        match self {
            Self::XMin | Self::XMax => 0,
            Self::YMin | Self::YMax => 1,
            _ => 2,
        }
    }
    /// Global, unmirrored coordinate projections; labels explicitly name the axes.
    pub fn uv_axes(self) -> (usize, usize) {
        match self {
            Self::XMin | Self::XMax => (1, 2),
            Self::YMin | Self::YMax => (0, 2),
            _ => (0, 1),
        }
    }
    pub fn uv(self, p: Vec3, stock: &Stock) -> [f64; 2] {
        let (u, v) = self.uv_axes();
        [p[u] - stock.min_mm[u], p[v] - stock.min_mm[v]]
    }
    pub fn point(self, uv: [f64; 2], stock: &Stock) -> Vec3 {
        let (u, v) = self.uv_axes();
        let mut p = stock.min_mm;
        if self.normal()[self.axis()] < 0. {
            p[self.axis()] = stock.max_mm[self.axis()];
        }
        p[u] += uv[0];
        p[v] += uv[1];
        p
    }
    pub fn size(self, stock: &Stock) -> [f64; 2] {
        let (u, v) = self.uv_axes();
        [
            stock.max_mm[u] - stock.min_mm[u],
            stock.max_mm[v] - stock.min_mm[v],
        ]
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stock {
    pub min_mm: Vec3,
    pub max_mm: Vec3,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transform {
    pub rotation: [Vec3; 3],
    pub translation_mm: Vec3,
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            rotation: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
            translation_mm: [0.; 3],
        }
    }
}
impl Transform {
    pub fn validate(&self) -> Result<()> {
        if !finite(self.translation_mm) || self.rotation.iter().any(|r| !finite(*r)) {
            return Err("Nieskończona transformacja".into());
        }
        for i in 0..3 {
            for j in 0..3 {
                if (dot(self.rotation[i], self.rotation[j]) - if i == j { 1. } else { 0. }).abs()
                    > 1e-8
                {
                    return Err("Transformacja musi być sztywna".into());
                }
            }
        }
        let r = &self.rotation;
        let det = r[0][0] * (r[1][1] * r[2][2] - r[1][2] * r[2][1])
            - r[0][1] * (r[1][0] * r[2][2] - r[1][2] * r[2][0])
            + r[0][2] * (r[1][0] * r[2][1] - r[1][1] * r[2][0]);
        if (det - 1.).abs() > 1e-8 {
            return Err("Odbicie nie jest obrotem".into());
        }
        Ok(())
    }
    pub fn vector(&self, p: Vec3) -> Vec3 {
        self.rotation.map(|r| dot(r, p))
    }
    pub fn point(&self, p: Vec3) -> Vec3 {
        add(self.vector(p), self.translation_mm)
    }
    pub fn inverse_point(&self, p: Vec3) -> Vec3 {
        let p = sub(p, self.translation_mm);
        std::array::from_fn(|i| (0..3).map(|j| self.rotation[j][i] * p[j]).sum())
    }
}
