use crate::math::{Ray, Vec3};
use crate::color::Color;
use crate::aabb::{Aabb, Hit, intersect_aabb};
use crate::material::Material;
use crate::texture::Texture;
use crate::skybox::sample_sky;

pub struct Scene {
    pub cubes: Vec<Aabb>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub sun_dir: Vec3,
    pub sun_col: Color,
    pub sky_mix: f32, // 0=day,1=night
}

impl Scene {
    pub fn hit(&self, ray:&Ray)->Option<Hit>{
        let mut best: Option<Hit> = None;
        for c in &self.cubes {
            if let Some(h)=intersect_aabb(ray,c){
                if best.as_ref().map_or(true, |b| h.t < b.t){ best = Some(h); }
            }
        }
        best
    }
}

pub fn trace(scene:&Scene, ray:Ray)->Color{
    if let Some(h) = scene.hit(&ray) {
        let m = &scene.materials[h.mat_id];
        // sample UVs en [0,1] con wrap
        let u = h.u.fract().abs();
        let v = h.v.fract().abs();
        let base = scene.textures[m.tex_id].sample(u, v).mul(m.albedo);

        // iluminación simple: difuso Lambert + especular Blinn-Phong
        let n = h.n;
        let l = scene.sun_dir.mul(-1.0).norm(); // dir desde punto hacia la luz
        let vdir = ray.d.mul(-1.0).norm();
        let hvec = (l.add(vdir)).norm();

        let ndl = 0.0_f32.max(n.dot(l));
        let diff = base.mul(ndl);

        let spec = m.specular * 0.0_f32.max(n.dot(hvec)).powf(m.shininess);
        let spec_col = Color::splat(spec);

        // sombra dura opcional: rayo hacia la luz
        let shadow = {
            let eps = 1e-3;
            let shadow_ray = Ray{ o: h.p.add(n.mul(eps)), d: l };
            if let Some(sh) = scene.hit(&shadow_ray) {
                if sh.t > eps && sh.t < 100.0 { 0.2 } else { 1.0 }
            } else { 1.0 }
        };

        // emisivo del material (lava más tarde)
        let emis = m.emissive;

        diff.mul(shadow).add(spec_col).add(emis)
    } else {
        // fondo: sky día/noche
        sample_sky(ray.d, scene.sky_mix)
    }
}
