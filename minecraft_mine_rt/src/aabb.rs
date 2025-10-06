use crate::math::{Vec3, Ray};

/// Índice de cara 
/// 0:-X, 1:+X, 2:-Y, 3:+Y, 4:-Z, 5:+Z
#[inline]
pub fn face_from_normal(n: Vec3) -> u8 {
    if n.x < -0.5 { 0 } else if n.x > 0.5 { 1 }
    else if n.y < -0.5 { 2 } else if n.y > 0.5 { 3 }
    else if n.z < -0.5 { 4 } else { 5 }
}

#[derive(Clone)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
    pub mat_id: usize,
    /// Orden: 0:-X, 1:+X, 2:-Y, 3:+Y, 4:-Z, 5:+Z
    pub face_tex: Option<[usize; 6]>,
}

pub struct Hit {
    pub t: f32,
    pub p: Vec3,
    pub n: Vec3,
    pub u: f32,
    pub v: f32,
    pub mat_id: usize,
    pub face_idx: u8,
    pub face_tex: Option<[usize; 6]>,
}

pub fn intersect_aabb(ray: &Ray, b: &Aabb) -> Option<Hit> {
    let inv = Vec3::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);

    let mut t1 = (b.min.x - ray.o.x) * inv.x;
    let mut t2 = (b.max.x - ray.o.x) * inv.x;
    let mut tmin = t1.min(t2);
    let mut tmax = t1.max(t2);

    t1 = (b.min.y - ray.o.y) * inv.y;
    t2 = (b.max.y - ray.o.y) * inv.y;
    tmin = tmin.max(t1.min(t2));
    tmax = tmax.min(t1.max(t2));

    t1 = (b.min.z - ray.o.z) * inv.z;
    t2 = (b.max.z - ray.o.z) * inv.z;
    tmin = tmin.max(t1.min(t2));
    tmax = tmax.min(t1.max(t2));

    if tmax >= tmin.max(0.0) {
        let t = tmin.max(0.0);
        let p = ray.o.add(ray.d.mul(t));
        let eps = 1e-3;

        let (u, v, n) = if (p.x - b.min.x).abs() < eps {
            let u = (p.z - b.min.z) / (b.max.z - b.min.z);
            let v = (p.y - b.min.y) / (b.max.y - b.min.y);
            (u, v, Vec3::new(-1.0, 0.0, 0.0))
        } else if (p.x - b.max.x).abs() < eps {
            let u = (b.max.z - p.z) / (b.max.z - b.min.z);
            let v = (p.y - b.min.y) / (b.max.y - b.min.y);
            (u, v, Vec3::new(1.0, 0.0, 0.0))
        } else if (p.y - b.min.y).abs() < eps {
            let u = (p.x - b.min.x) / (b.max.x - b.min.x);
            let v = (p.z - b.min.z) / (b.max.z - b.min.z);
            (u, v, Vec3::new(0.0, -1.0, 0.0))
        } else if (p.y - b.max.y).abs() < eps {
            let u = (p.x - b.min.x) / (b.max.x - b.min.x);
            let v = (b.max.z - p.z) / (b.max.z - b.min.z);
            (u, v, Vec3::new(0.0, 1.0, 0.0))
        } else if (p.z - b.min.z).abs() < eps {
            let u = (p.x - b.min.x) / (b.max.x - b.min.x);
            let v = (p.y - b.min.y) / (b.max.y - b.min.y);
            (u, v, Vec3::new(0.0, 0.0, -1.0))
        } else {
            let u = (b.max.x - p.x) / (b.max.x - b.min.x);
            let v = (p.y - b.min.y) / (b.max.y - b.min.y);
            (u, v, Vec3::new(0.0, 0.0, 1.0))
        };

        let face_idx = face_from_normal(n);
        Some(Hit {
            t,
            p,
            n,
            u,
            v,
            mat_id: b.mat_id,
            face_idx,
            face_tex: b.face_tex.clone(),
        })
    } else {
        None
    }
}
