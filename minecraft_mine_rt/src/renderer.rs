use crate::aabb::{intersect_aabb, Aabb, Hit};
use crate::color::Color;
use crate::math::{reflect, refract, schlick, Ray, Vec3};
use crate::material::Material;
use crate::skybox::{sample_sky, CubeMap};
use crate::texture::Texture;

const MAX_DEPTH: u32 = 5;
const BIAS: f32 = 1e-3;

pub struct Scene {
    pub cubes: Vec<Aabb>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub sun_dir: Vec3,
    pub sun_col: Color,
    pub sky_mix: f32,            // 0 = día, 1 = noche
    pub skybox: Option<CubeMap>, // cubemap opcional
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

fn env(scene: &Scene, dir: Vec3) -> Color {
    sample_sky(dir, scene.sky_mix, scene.skybox.as_ref())
}

fn trace_rec(scene: &Scene, ray: Ray, depth: u32) -> Color {
    if depth == 0 {
        return env(scene, ray.d);
    }

    if let Some(h) = scene.hit(&ray) {
        let m = &scene.materials[h.mat_id];

        // textura por cara si existe
        let tex_id = if let Some(faces) = &h.face_tex {
            faces[h.face_idx as usize]
        } else {
            m.tex_id
        };

        // UVs envueltos
        let mut u = h.u.fract().abs();
        let mut v = h.v.fract().abs();
        // La lateral de grass iba invertida en Y (índice 1 = grass_side)
        if tex_id == 1 { v = 1.0 - v; }

        let base = scene.textures[tex_id].sample(u, v).mul(m.albedo);

        // Luz direccional (Lambert) teñida con el color del sol
        let n = h.n;
        let l = scene.sun_dir.mul(-1.0).norm();
        let vdir = ray.d.mul(-1.0).norm();
        let hvec = (l.add(vdir)).norm();

        let ndl = 0.0_f32.max(n.dot(l));
        let diff = base.hadamard(scene.sun_col).mul(ndl);

        // Blinn-Phong sencillo
        let spec = m.specular * 0.0_f32.max(n.dot(hvec)).powf(m.shininess);
        let spec_col = scene.sun_col.mul(spec);

        // Sombras duras pero no tan negras (deja pasar algo de luz indirecta)
        let shadow_fac = {
            let shadow_ray = Ray { o: h.p.add(n.mul(BIAS)), d: l };
            if let Some(sh) = scene.hit(&shadow_ray) {
                if sh.t > BIAS && sh.t < 100.0 { 0.55 } else { 1.0 }
            } else { 1.0 }
        };

        // “Skylight” simple: color del cielo por arriba, hace de luz ambiente
        let sky_col = env(scene, Vec3::new(0.0, 1.0, 0.0));
        // Más día => más ambiente; de noche baja pero nunca a cero
        let amb_k = (0.25 * (1.0 - scene.sky_mix)) + (0.12 * scene.sky_mix);
        let ambient = base.hadamard(sky_col).mul(amb_k);

        // Emisivo (lava) sube un poco de noche
        let emis = m.emissive.mul(1.0 + scene.sky_mix * 1.5);

        // Fresnel para mezclar reflexión / refracción
        let i = ray.d;
        let front_face = n.dot(i) < 0.0;
        let (n1, n2, n_use) = if front_face { (1.0, m.ior, n) } else { (m.ior, 1.0, n.mul(-1.0)) };
        let cosi = (-i.dot(n_use)).clamp(-1.0, 1.0);
        let mut kr = schlick(cosi, n1, n2) * m.reflectivity;
        let kt = m.transparency * (1.0 - kr);

        // Local = difuso + especular + ambiente + emisivo
        let local = diff.mul(shadow_fac).add(spec_col).add(ambient).add(emis);

        // Reflexión: si se agota profundidad, toma el entorno
        let refl_col = if kr > 0.0 {
            let rdir = reflect(i, n).norm();
            let rayo = Ray { o: h.p.add(n.mul(BIAS)), d: rdir };
            let c = if depth > 1 { trace_rec(scene, rayo, depth - 1) } else { env(scene, rdir) };
            c.mul(kr)
        } else { Color::black() };

        // Refracción: igual, con leve atenuación azulada (agua/vidrio)
        let refr_col = if kt > 0.0 {
            let eta = n1 / n2;
            if let Some(tdir) = refract(i, n_use, eta) {
                let tdir = tdir.norm();
                let rayo = Ray { o: h.p.sub(n.mul(BIAS)), d: tdir };
                let through = if depth > 1 { trace_rec(scene, rayo, depth - 1) } else { env(scene, tdir) };
                through.hadamard(Color::new(0.96, 0.98, 0.99)).mul(kt)
            } else {
                Color::black()
            }
        } else { Color::black() };

        // Factor local para que el balance cierre
        let local_fac = (1.0 - m.reflectivity - m.transparency).clamp(0.0, 1.0);
        local.mul(local_fac).add(refl_col).add(refr_col)
    } else {
        env(scene, ray.d)
    }
}
