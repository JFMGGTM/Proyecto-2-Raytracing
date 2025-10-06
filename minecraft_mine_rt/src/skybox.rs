use crate::color::Color;
use crate::math::Vec3;
use crate::ppm::load_ppm;

/// Cubemap simple: 6 caras en RGB8.
/// Orden esperado de archivos: +X, -X, +Y, -Y, +Z, -Z.
pub struct CubeMap {
    pub w: usize,
    pub h: usize,
    pub faces: [Vec<u8>; 6], // cada face es w*h*3
}

impl CubeMap {
    pub fn from_folder(path: &str) -> Option<Self> {
        let names = ["posx", "negx", "posy", "negy", "posz", "negz"];
        let mut data: [Vec<u8>; 6] = Default::default();
        let mut w = 0usize; let mut h = 0usize;

        for (i, n) in names.iter().enumerate() {
            let p = format!("{}/{}.ppm", path, n);
            let img = load_ppm(&p).ok()?;
            if i == 0 { w = img.w; h = img.h; }
            if img.w != w || img.h != h { return None; }
            data[i] = img.data;
        }
        Some(Self { w, h, faces: data })
    }

    /// Mapea un vector de dirección a una cara y coord u,v en [0,1].
    fn dir_to_face_uv(dir: Vec3) -> (usize, f32, f32) {
        let x = dir.x; let y = dir.y; let z = dir.z;
        let ax = x.abs(); let ay = y.abs(); let az = z.abs();
        let (face, u, v);
        if ax >= ay && ax >= az {
            // ±X
            if x > 0.0 {
                // +X
                let uc = -z / ax;
                let vc = -y / ax;
                face = 0; u = (uc + 1.0) * 0.5; v = (vc + 1.0) * 0.5;
            } else {
                // -X
                let uc =  z / ax;
                let vc = -y / ax;
                face = 1; u = (uc + 1.0) * 0.5; v = (vc + 1.0) * 0.5;
            }
        } else if ay >= ax && ay >= az {
            // ±Y
            if y > 0.0 {
                // +Y
                let uc =  x / ay;
                let vc =  z / ay;
                face = 2; u = (uc + 1.0) * 0.5; v = (vc + 1.0) * 0.5;
            } else {
                // -Y
                let uc =  x / ay;
                let vc = -z / ay;
                face = 3; u = (uc + 1.0) * 0.5; v = (vc + 1.0) * 0.5;
            }
        } else {
            // ±Z
            if z > 0.0 {
                // +Z
                let uc =  x / az;
                let vc = -y / az;
                face = 4; u = (uc + 1.0) * 0.5; v = (vc + 1.0) * 0.5;
            } else {
                // -Z
                let uc = -x / az;
                let vc = -y / az;
                face = 5; u = (uc + 1.0) * 0.5; v = (vc + 1.0) * 0.5;
            }
        }
        (face, u.clamp(0.0,1.0), v.clamp(0.0,1.0))
    }

    pub fn sample(&self, dir: Vec3) -> Color {
        let (face, u, v) = Self::dir_to_face_uv(dir);
        let x = (u * self.w as f32) as usize % self.w;
        let y = (v * self.h as f32) as usize % self.h;
        let idx = (y * self.w + x) * 3;
        let d = &self.faces[face];
        Color::from_u8(d[idx], d[idx + 1], d[idx + 2])
    }
}

/// Gradiente día/noche como respaldo.
fn gradient_sky(dir: Vec3, k: f32) -> Color {
    let y = dir.y.clamp(-1.0, 1.0);
    let day = Color::new(0.45, 0.65, 0.95).mul(0.6 + 0.4 * (y * 0.5 + 0.5));
    let night = Color::new(0.05, 0.08, 0.15).mul(0.5 + 0.5 * (y * 0.5 + 0.5));
    Color::lerp(day, night, k)
}

/// Toma cubemap si existe, si no usa gradiente.
/// `k` es la mezcla día/noche (0=day, 1=night).
pub fn sample_sky(dir: Vec3, k: f32, cubemap: Option<&CubeMap>) -> Color {
    if let Some(cm) = cubemap {
        // Mezcla sutil hacia noche bajando intensidad
        let env = cm.sample(dir);
        let dark = env.mul(0.25);
        Color::lerp(env, dark, k)
    } else {
        gradient_sky(dir, k)
    }
}
