use crate::aabb::Aabb;
use crate::camera::Camera;
use crate::color::Color;
use crate::material::Material;
use crate::math::Vec3;
use crate::renderer::Scene;
use crate::texture::{TexKind, Texture};

// Animación de cámara (compartida)
pub struct Anim {
    pub angle: f32,   // órbita en radianes
    pub radius: f32,  // distancia al objetivo
    pub eye_h: f32,   // altura de la cámara
    pub sky_mix: f32, // 0 día, 1 noche
}

/* ===========================
   INICIO DE MI ESCENA
   =========================== */
pub fn build_scene_basic(t: f32) -> (Scene, Anim) {
    let elev = (std::f32::consts::PI * 2.0 * t).sin() * 0.6; // -0.6..0.6
    let az = std::f32::consts::PI * 2.0 * t;
    let sun_dir = Vec3::new(az.cos(), elev, az.sin()).norm();
    let dayness = (elev * 1.2).clamp(0.0, 1.0); // cuanto más alto, más día
    let sky_mix = 1.0 - dayness; // invertimos para mezclar sky nocturno

    let sun_col = Color::new(1.0, 0.95, 0.85).mul(0.8 + 0.2 * dayness);

    // Texturas (con fallback)
    let textures = vec![
        Texture::new(TexKind::Stone), // 0 piedra (antes checker)
        Texture::new(TexKind::Wood),  // 1 madera
        Texture::new(TexKind::Metal), // 2 metal
        Texture::new(TexKind::Water), // 3 agua
        Texture::new(TexKind::Lava),  // 4 lava
    ];

    // Materiales base
    let materials = vec![
        Material { tex_id: 0, albedo: 1.0, specular: 0.1, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 32.0, emissive: Color::black() }, // piedra
        Material { tex_id: 1, albedo: 1.0, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() }, // madera
        Material { tex_id: 2, albedo: 0.9, specular: 0.9, transparency: 0.0, reflectivity: 0.6, ior: 1.0, shininess: 64.0, emissive: Color::black() },   // metal
        Material { tex_id: 3, albedo: 0.98, specular: 0.5, transparency: 0.85, reflectivity: 0.05, ior: 1.33, shininess: 32.0, emissive: Color::black() }, // agua
        Material { tex_id: 4, albedo: 1.0, specular: 0.2, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 8.0, emissive: Color::new(1.8, 0.6, 0.1) }, // lava
    ];

    // Geometría de “mina” simplificada
    let mut cubes: Vec<Aabb> = Vec::new();
    for z in 0..12 {
        for x in -2..=2 {
            // piso
            cubes.push(Aabb {
                min: Vec3::new(x as f32, -1.0, z as f32),
                max: Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                mat_id: 0,
                face_tex: None,
            });
            // paredes
            if x == -2 || x == 2 {
                for y in 0..3 {
                    cubes.push(Aabb {
                        min: Vec3::new(x as f32, y as f32, z as f32),
                        max: Vec3::new(x as f32 + 1.0, y as f32 + 1.0, z as f32 + 1.0),
                        mat_id: 0,
                        face_tex: None,
                    });
                }
            }
        }
    }
    // vigas de madera
    for z in (0..12).step_by(3) {
        cubes.push(Aabb { min: Vec3::new(-2.0, 0.0, z as f32), max: Vec3::new(-1.0, 3.0, z as f32 + 1.0), mat_id: 1, face_tex: None });
        cubes.push(Aabb { min: Vec3::new( 2.0, 0.0, z as f32), max: Vec3::new( 3.0, 3.0, z as f32 + 1.0), mat_id: 1, face_tex: None });
        cubes.push(Aabb { min: Vec3::new(-2.0, 3.0, z as f32), max: Vec3::new( 3.0, 4.0, z as f32 + 1.0), mat_id: 1, face_tex: None });
    }
    // riel
    for z in 0..12 {
        cubes.push(Aabb { min: Vec3::new(-0.2, -0.2, z as f32 + 0.25), max: Vec3::new(0.2, 0.0, z as f32 + 0.35), mat_id: 2, face_tex: None });
        cubes.push(Aabb { min: Vec3::new(-0.2, -0.2, z as f32 + 0.65), max: Vec3::new(0.2, 0.0, z as f32 + 0.75), mat_id: 2, face_tex: None });
    }
    // agua y lava
    cubes.push(Aabb { min: Vec3::new(-1.0, -0.05, 6.0), max: Vec3::new(1.0, 0.0, 7.5), mat_id: 3, face_tex: None });
    cubes.push(Aabb { min: Vec3::new(-1.0, -0.2, 10.0), max: Vec3::new(1.5, -0.05, 11.5), mat_id: 4, face_tex: None });

    let scene = Scene { cubes, materials, textures, sun_dir, sun_col, sky_mix };

    let angle = std::f32::consts::PI * 2.0 * (t * 0.25);
    let radius = 6.5 + 1.0 * (0.5 - (t * 2.0 * std::f32::consts::PI).cos() * 0.5);
    let eye_h = 1.5;
    let anim = Anim { angle, radius, eye_h, sky_mix };

    (scene, anim)
}

/* ===========================
   ESCENA
   =========================== */
pub fn build_scene_minecraft(t: f32) -> (Scene, Anim) {
    // Ciclo día/noche
    let elev = (std::f32::consts::PI * 2.0 * t).sin() * 0.6;
    let az = std::f32::consts::PI * 2.0 * t;
    let sun_dir = Vec3::new(az.cos(), elev, az.sin()).norm();
    let dayness = (elev * 1.2).clamp(0.0, 1.0);
    let sky_mix = 1.0 - dayness;
    let sun_col = Color::new(1.0, 0.95, 0.85).mul(0.8 + 0.2 * dayness);

    // Cargar texturas
    let tx = |name: &str, kind: TexKind| {
        Texture::from_ppm(&format!("assets/textures/{}.ppm", name)).unwrap_or(Texture::new(kind))
    };
    let tx_grass_top = tx("grass_top", TexKind::GrassTop);
    let tx_grass_side = tx("grass_side", TexKind::GrassSide);
    let tx_dirt = tx("dirt", TexKind::Dirt);
    let tx_cobble = tx("cobble", TexKind::Cobble);
    let tx_planks = tx("planks", TexKind::Wood);
    let tx_leaves = tx("leaves", TexKind::Leaves);
    let tx_glass = tx("glass", TexKind::Glass);
    let tx_stone = tx("stone", TexKind::Stone);
    let tx_water = tx("water", TexKind::Water);
    let tx_lava = tx("lava", TexKind::Lava);

    let textures = vec![
        tx_grass_top,  // 0
        tx_grass_side, // 1
        tx_dirt,       // 2
        tx_cobble,     // 3
        tx_planks,     // 4
        tx_leaves,     // 5
        tx_glass,      // 6
        tx_stone,      // 7
        tx_water,      // 8
        tx_lava,       // 9
    ];

    // Materiales
    let mat_grass = Material { tex_id: 1, albedo: 1.0, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_dirt  = Material { tex_id: 2, albedo: 1.0, specular: 0.02, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 8.0,  emissive: Color::black() };
    let mat_cobb  = Material { tex_id: 3, albedo: 1.0, specular: 0.1,  transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_wood  = Material { tex_id: 4, albedo: 1.0, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_leaf  = Material { tex_id: 5, albedo: 0.95, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 8.0,  emissive: Color::black() };
    let mat_glass = Material { tex_id: 6, albedo: 0.98, specular: 0.2,  transparency: 0.85, reflectivity: 0.05, ior: 1.5, shininess: 32.0, emissive: Color::black() };
    let mat_stone = Material { tex_id: 7, albedo: 1.0, specular: 0.1,  transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_water = Material { tex_id: 8, albedo: 0.98, specular: 0.5,  transparency: 0.9,  reflectivity: 0.05, ior: 1.33, shininess: 32.0, emissive: Color::black() };
    let mat_lava  = Material { tex_id: 9, albedo: 1.0, specular: 0.2,  transparency: 0.0,  reflectivity: 0.0, ior: 1.0,  shininess: 8.0,  emissive: Color::new(1.8, 0.6, 0.1) };

    let materials = vec![mat_grass, mat_dirt, mat_cobb, mat_wood, mat_leaf, mat_glass, mat_stone, mat_water, mat_lava];

    // Utilidades
    let grass_faces = [1, 1, 2, 0, 1, 1]; // -X:side, +X:side, -Y:dirt, +Y:top, -Z:side, +Z:side
    let one_tex = |tid: usize| [tid, tid, tid, tid, tid, tid];

    // Geometría
    let mut cubes: Vec<Aabb> = Vec::new();

    // Planicie de grass blocks 16x16
    let y0 = 0.0;
    for z in 0..16 {
        for x in 0..16 {
            cubes.push(Aabb {
                min: Vec3::new(x as f32, y0, z as f32),
                max: Vec3::new(x as f32 + 1.0, y0 + 1.0, z as f32 + 1.0),
                mat_id: 0, // mat_grass
                face_tex: Some(grass_faces),
            });
        }
    }

    // Camino de adoquín
    for z in 0..16 {
        cubes.push(Aabb {
            min: Vec3::new(7.0, y0 + 1.0, z as f32),
            max: Vec3::new(9.0, y0 + 1.1, z as f32 + 1.0),
            mat_id: 2, // cobble
            face_tex: Some(one_tex(3)),
        });
    }

    // Charco de agua y de lava
    cubes.push(Aabb {
        min: Vec3::new(2.0, y0 + 1.0, 3.0),
        max: Vec3::new(6.0, y0 + 1.05, 7.0),
        mat_id: 7, // water
        face_tex: Some(one_tex(8)),
    });
    cubes.push(Aabb {
        min: Vec3::new(11.0, y0 + 1.0, 10.0),
        max: Vec3::new(14.0, y0 + 1.05, 13.0),
        mat_id: 8, // lava
        face_tex: Some(one_tex(9)),
    });

    // NOTA: Casa de madera 5x4x5 con techo de piedra y ventanas de vidrio
    let base = Vec3::new(5.0, y0 + 1.0, 5.0);
    for z in 0..5 {
        for y in 0..4 {
            // pared X=5
            cubes.push(Aabb {
                min: base.add(Vec3::new(0.0, y as f32, z as f32)),
                max: base.add(Vec3::new(0.5, y as f32 + 1.0, z as f32 + 1.0)),
                mat_id: 3,
                face_tex: Some(one_tex(4)),
            });
            // pared X=9
            cubes.push(Aabb {
                min: base.add(Vec3::new(4.5, y as f32, z as f32)),
                max: base.add(Vec3::new(5.0, y as f32 + 1.0, z as f32 + 1.0)),
                mat_id: 3,
                face_tex: Some(one_tex(4)),
            });
        }
    }
    for x in 0..5 {
        for y in 0..4 {
            // pared Z=5
            cubes.push(Aabb {
                min: base.add(Vec3::new(x as f32, y as f32, 0.0)),
                max: base.add(Vec3::new(x as f32 + 1.0, y as f32 + 1.0, 0.5)),
                mat_id: 3,
                face_tex: Some(one_tex(4)),
            });
            // pared Z=9
            cubes.push(Aabb {
                min: base.add(Vec3::new(x as f32, y as f32, 4.5)),
                max: base.add(Vec3::new(x as f32 + 1.0, y as f32 + 1.0, 5.0)),
                mat_id: 3,
                face_tex: Some(one_tex(4)),
            });
        }
    }
    // Ventanas de vidrio
    cubes.push(Aabb {
        min: base.add(Vec3::new(2.0, 1.0, 0.0)),
        max: base.add(Vec3::new(3.0, 2.0, 0.5)),
        mat_id: 5,
        face_tex: Some(one_tex(6)),
    });
    cubes.push(Aabb {
        min: base.add(Vec3::new(2.0, 1.0, 4.5)),
        max: base.add(Vec3::new(3.0, 2.0, 5.0)),
        mat_id: 5,
        face_tex: Some(one_tex(6)),
    });
    // Techo de piedra
    for z in 0..5 {
        for x in 0..5 {
            cubes.push(Aabb {
                min: base.add(Vec3::new(x as f32, 4.0, z as f32)),
                max: base.add(Vec3::new(x as f32 + 1.0, 4.5, z as f32 + 1.0)),
                mat_id: 6,
                face_tex: Some(one_tex(7)),
            });
        }
    }

    // Árbol: tronco
    for y in 0..4 {
        cubes.push(Aabb {
            min: Vec3::new(3.0, y0 + 1.0 + y as f32, 12.0),
            max: Vec3::new(3.5, y0 + 2.0 + y as f32, 12.5),
            mat_id: 3,
            face_tex: Some(one_tex(4)),
        });
    }
    for z in 11..=13 {
        for x in 2..=4 {
            cubes.push(Aabb {
                min: Vec3::new(x as f32, y0 + 4.0, z as f32),
                max: Vec3::new(x as f32 + 1.0, y0 + 5.0, z as f32 + 1.0),
                mat_id: 4,
                face_tex: Some(one_tex(5)),
            });
        }
    }

    let scene = Scene { cubes, materials, textures, sun_dir, sun_col, sky_mix };

    let angle = std::f32::consts::PI * 2.0 * (t * 0.2);
    let radius = 18.0 + 2.0 * (0.5 - (t * 2.0 * std::f32::consts::PI).cos() * 0.5);
    let eye_h = 6.0;
    let anim = Anim { angle, radius, eye_h, sky_mix };

    (scene, anim)
}
