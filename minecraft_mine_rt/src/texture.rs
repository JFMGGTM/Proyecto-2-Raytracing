use crate::color::Color;
use crate::ppm::load_ppm;

// Tipos de textura (procedurales de respaldo)
#[derive(Clone)]
pub enum TexKind {
    Stone, Wood, Metal, Water, Lava,
    GrassTop, GrassSide, Dirt, Cobble, Sand, Leaves, Glass,
}

// Dos modos: Procedural o Imagen cargada
pub enum Texture {
    Procedural(TexKind),
    Image { w: usize, h: usize, data: Vec<u8> },
}

impl Texture {
    pub fn new(kind: TexKind) -> Self { Self::Procedural(kind) }

    pub fn from_ppm(path: &str) -> Option<Self> {
        if let Ok(img) = load_ppm(path) {
            Some(Texture::Image { w: img.w, h: img.h, data: img.data })
        } else {
            None
        }
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        // wrap [0,1)
        let uu = ((u % 1.0) + 1.0) % 1.0;
        let vv = ((v % 1.0) + 1.0) % 1.0;
        match self {
            Texture::Procedural(kind) => sample_procedural(kind, uu, vv),
            Texture::Image { w, h, data } => {
                let x = (uu * (*w as f32)) as usize % *w;
                let y = (vv * (*h as f32)) as usize % *h;
                let idx = (y * (*w) + x) * 3;
                Color::from_u8(data[idx], data[idx + 1], data[idx + 2])
            }
        }
    }
}

/* =======================
   Texturas procedurales
   ======================= */

fn fract(x: f32) -> f32 { x - x.floor() }

fn hash12(x: f32, y: f32) -> f32 {
    let v = (x * 12.9898 + y * 78.233).sin() * 43758.5453;
    fract(v)
}

fn noise2(u: f32, v: f32) -> f32 {
    let i = u.floor(); let j = v.floor();
    let fu = u - i; let fv = v - j;
    let a = hash12(i,     j     );
    let b = hash12(i + 1.,j     );
    let c = hash12(i,     j + 1.);
    let d = hash12(i + 1.,j + 1.);
    let ab = a + (b - a) * fu;
    let cd = c + (d - c) * fu;
    ab + (cd - ab) * fv
}

fn fbm(u: f32, v: f32, octaves: i32) -> f32 {
    let mut f = 0.0; let mut amp = 0.5; let mut freq = 1.0;
    for _ in 0..octaves {
        f += noise2(u * freq, v * freq) * amp;
        freq *= 2.0; amp *= 0.5;
    }
    f
}

fn checker(u: f32, v: f32, scale: f32, c0: Color, c1: Color) -> Color {
    let uu = (u * scale).floor() as i32;
    let vv = (v * scale).floor() as i32;
    if (uu + vv) & 1 == 0 { c0 } else { c1 }
}

fn sample_procedural(kind: &TexKind, u: f32, v: f32) -> Color {
    match kind {
        // Piedra: gris moteado con fBm
        TexKind::Stone => {
            let f = fbm(u * 8.0, v * 8.0, 4);
            let g = 0.5 + 0.5 * (f - 0.5);
            Color::new(0.55 * g, 0.58 * g, 0.62 * g)
        }
        // Madera: vetas simples
        TexKind::Wood => {
            let rings = ((u * 8.0).sin() * 0.5 + 0.5) * ((v * 8.0).cos() * 0.5 + 0.5);
            Color::new(0.45 + 0.3 * rings, 0.30 + 0.2 * rings, 0.12 + 0.1 * rings)
        }
        // Metal: bandas sutiles
        TexKind::Metal => {
            let band = ((u * 64.0).sin() * 0.5 + 0.5) * 0.25 + 0.65;
            Color::splat(band as f32)
        }
        // Agua: ondulaciones
        TexKind::Water => {
            let w = 0.7 + 0.3 * ((u * 10.0).sin() * (v * 12.0).cos());
            Color::new(0.15, 0.35, 0.8).mul(w as f32)
        }
        // Lava: incandescencia con vetas
        TexKind::Lava => {
            let t = ((u * 20.0).sin() + (v * 24.0).cos()) * 0.5 + 0.5;
            Color::new(0.95, 0.35 + 0.5 * t, 0.08)
        }
        // Césped superior
        TexKind::GrassTop => {
            let f = fbm(u * 8.0, v * 8.0, 3);
            Color::new(0.12 + 0.2 * f, 0.45 + 0.35 * f, 0.10 + 0.15 * f)
        }
        // Césped lateral: mezcla césped/tierra
        TexKind::GrassSide => {
            let f = fbm(u * 8.0, v * 8.0, 3);
            let top = Color::new(0.14 + 0.2 * f, 0.46 + 0.35 * f, 0.12 + 0.15 * f);
            let dirt = Color::new(0.40, 0.30, 0.18);
            let edge = ((v * 1.0) - 0.5) * 8.0;
            let t = (edge.clamp(-1.0, 1.0) * 0.5 + 0.5);
            top.mul(1.0 - t).add(dirt.mul(t))
        }
        TexKind::Dirt => {
            let f = fbm(u * 10.0, v * 10.0, 4);
            Color::new(0.42 + 0.1 * f, 0.32 + 0.07 * f, 0.20 + 0.05 * f)
        }
        TexKind::Cobble => {
            // adoquín: patrón de bloques grises con leve variación
            let base = checker(u, v, 8.0, Color::new(0.55, 0.56, 0.60), Color::new(0.45, 0.46, 0.50));
            let f = fbm(u * 16.0, v * 16.0, 2);
            base.mul(0.9 + 0.1 * f)
        }
        TexKind::Sand => {
            let f = fbm(u * 12.0, v * 12.0, 3);
            Color::new(0.90, 0.85 + 0.05 * f, 0.65 + 0.07 * f)
        }
        TexKind::Leaves => {
            let f = fbm(u * 9.0, v * 9.0, 3);
            Color::new(0.10 + 0.2 * f, 0.35 + 0.45 * f, 0.12 + 0.2 * f)
        }
        TexKind::Glass => {
            let f = 0.85 + 0.1 * ((u * 20.0).sin() * (v * 20.0).cos());
            Color::new(0.75, 0.9, 1.0).mul(f as f32)
        }
    }
}
