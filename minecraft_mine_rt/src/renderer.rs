use crate::aabb::{intersect_aabb, Aabb, Hit};
use crate::color::Color;
use crate::math::{reflect, refract, schlick, Ray, Vec3};
use crate::material::Material;
use crate::skybox::sample_sky;
use crate::texture::Texture;

const MAX_DEPTH: u32 = 5;
const BIAS: f32 = 1e-3;

pub struct Scene {
    pub cubes: Vec<Aabb>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub sun_dir: Vec3,
    pub sun_col: Color,
    pub sky_mix: f32, // 0=day,1=night
}

impl Scene {
    pub fn hit(&self, ray: &Ray) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        for c in &self.cubes {
            if let Some(h) = intersect_aabb(ray, c) {
                if best.as_ref().map_or(true, |b| h.t < b.t) {
                    best = Some(h);
                }
            }
        }
        best
    }
}

pub fn trace(scene: &Scene, ray: Ray) -> Color {
    trace_rec(scene, ray, MAX_DEPTH)
}

fn trace_rec(scene: &Scene, ray: Ray, depth: u32) -> Color {
    if depth == 0 {
        return sample_sky(ray.d, scene.sky_mix);
    }

    if let Some(h) = scene.hit(&ray) {
        let m = &scene.materials[h.mat_id];

        // === Selección de textura: por cara si está disponible, si no la del material ===
        let tex_id = if let Some(faces) = &h.face_tex {
            faces[h.face_idx as usize]
        } else {
            m.tex_id
        };

        let u = h.u.fract().abs();
        let v = h.v.fract().abs();
        let base = scene.textures[tex_id].sample(u, v).mul(m.albedo);

        // === Iluminación local (difuso + especular) con luz direccional ===
        let n = h.n;
        let l = scene.sun_dir.mul(-1.0).norm(); // hacia la luz
        let vdir = ray.d.mul(-1.0).norm();
        let hvec = (l.add(vdir)).norm();

        let ndl = 0.0_f32.max(n.dot(l));
        let diff = base.mul(ndl);

        let spec = m.specular * 0.0_f32.max(n.dot(hvec)).powf(m.shininess);
        let spec_col = Color::splat(spec);

        // Sombra dura
        let shadow_fac = {
            let shadow_ray = Ray {
                o: h.p.add(n.mul(BIAS)),
                d: l,
            };
            if let Some(sh) = scene.hit(&shadow_ray) {
                if sh.t > BIAS && sh.t < 100.0 {
                    0.2
                } else {
                    1.0
                }
            } else {
                1.0
            }
        };

        // Lava
        let emis_boost = 1.0 + scene.sky_mix * 1.5;
        let emis = m.emissive.mul(emis_boost);

        // === Reflexión / Refracción con Fresnel (Schlick) ===
        let i = ray.d;
        let front_face = n.dot(i) < 0.0;
        let (n1, n2, n_use) = if front_face {
            (1.0, m.ior, n)
        } else {
            (m.ior, 1.0, n.mul(-1.0))
        };

        let cosi = (-i.dot(n_use)).clamp(-1.0, 1.0);
        let mut kr = schlick(cosi, n1, n2);
        kr *= m.reflectivity;
        let kt = m.transparency * (1.0 - kr);

        let local = diff.mul(shadow_fac).add(spec_col).add(emis);

        let refl_col = if kr > 0.0 {
            let rdir = reflect(i, n).norm();
            let rayo = Ray {
                o: h.p.add(n.mul(BIAS)),
                d: rdir,
            };
            trace_rec(scene, rayo, depth - 1).mul(kr)
        } else {
            Color::black()
        };

        let refr_col = if kt > 0.0 {
            let eta = n1 / n2;
            if let Some(tdir) = refract(i, n_use, eta) {
                let tdir = tdir.norm();
                let rayo = Ray {
                    o: h.p.sub(n.mul(BIAS)),
                    d: tdir,
                };
                // Atenuación tenue
                let att = Color::new(0.96, 0.98, 0.99);
                trace_rec(scene, rayo, depth - 1).hadamard(att).mul(kt)
            } else {
                // TIR: nada de transmisión
                Color::black()
            }
        } else {
            Color::black()
        };

        let local_fac = (1.0 - m.reflectivity - m.transparency).clamp(0.0, 1.0);
        local.mul(local_fac).add(refl_col).add(refr_col)
    } else {
        sample_sky(ray.d, scene.sky_mix)
    }
}
