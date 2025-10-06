use crate::aabb::Aabb;
use crate::color::Color;
use crate::material::Material;
use crate::math::Vec3;
use crate::renderer::Scene;
use crate::skybox::CubeMap;
use crate::texture::{TexKind, Texture};

pub struct Anim {
    pub angle: f32,
    pub radius: f32,
    pub eye_h: f32,
    pub sky_mix: f32,
}

/// Altura “suave” para dos lomitas. Mantengo valores chicos para no romper siluetas.
fn height(x: i32, z: i32) -> i32 {
    let xf = x as f32;
    let zf = z as f32;

    // centros de las lomitas
    let c1 = (5.0, 5.0);
    let c2 = (11.0, 12.0);

    let d1 = ((xf - c1.0).hypot(zf - c1.1) / 4.0).min(3.0);
    let d2 = ((xf - c2.0).hypot(zf - c2.1) / 4.5).min(3.0);

    // dos cúpulas suaves (0..~2)
    let h1 = (1.8 - d1).max(0.0);
    let h2 = (2.2 - d2).max(0.0);

    let base = 1.0; // piso mínimo
    (base + h1 + h2).round() as i32  // 1..4 aprox
}

/// Helper: empuja un cubo (bloque unitario) al vector.
fn push_block(cubes: &mut Vec<Aabb>, x: f32, y: f32, z: f32, mat_id: usize, face_tex: [usize; 6]) {
    cubes.push(Aabb {
        min: Vec3::new(x, y, z),
        max: Vec3::new(x + 1.0, y + 1.0, z + 1.0),
        mat_id,
        face_tex: Some(face_tex),
    });
}

pub fn build_scene_minecraft(t: f32) -> (Scene, Anim) {
    // ciclo de sol
    let elev = (std::f32::consts::PI * 2.0 * t).sin() * 0.6;
    let az = std::f32::consts::PI * 2.0 * t;
    let sun_dir = Vec3::new(az.cos(), elev, az.sin()).norm();
    let dayness = (elev * 1.2).clamp(0.0, 1.0);
    let sky_mix = 1.0 - dayness;
    let sun_col = Color::new(1.0, 0.95, 0.85).mul(0.9 + 0.3 * dayness); // un poquito más brillante

    // carga de texturas (con fallback procedural)
    let tx = |name: &str, kind: TexKind| {
        Texture::from_ppm(&format!("assets/textures/{}.ppm", name)).unwrap_or(Texture::new(kind))
    };
    let textures = vec![
        tx("grass_top",  TexKind::GrassTop),  // 0
        tx("grass_side", TexKind::GrassSide), // 1
        tx("dirt",       TexKind::Dirt),      // 2
        tx("cobble",     TexKind::Cobble),    // 3
        tx("planks",     TexKind::Wood),      // 4
        tx("leaves",     TexKind::Leaves),    // 5
        tx("glass",      TexKind::Glass),     // 6
        tx("stone",      TexKind::Stone),     // 7
        tx("water",      TexKind::Water),     // 8
        tx("lava",       TexKind::Lava),      // 9
        Texture::new(TexKind::Metal),         // 10
    ];

    // materiales
    let mat_grass = Material { tex_id: 1, albedo: 1.0, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_dirt  = Material { tex_id: 2, albedo: 1.0, specular: 0.02, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 8.0,  emissive: Color::black() };
    let mat_cobb  = Material { tex_id: 3, albedo: 1.0, specular: 0.1,  transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_wood  = Material { tex_id: 4, albedo: 1.0, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_leaf  = Material { tex_id: 5, albedo: 0.95, specular: 0.05, transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 8.0,  emissive: Color::black() };
    let mat_glass = Material { tex_id: 6, albedo: 0.98, specular: 0.2,  transparency: 0.85, reflectivity: 0.05, ior: 1.5, shininess: 32.0, emissive: Color::black() };
    let mat_stone = Material { tex_id: 7, albedo: 1.0, specular: 0.1,  transparency: 0.0, reflectivity: 0.0, ior: 1.0, shininess: 16.0, emissive: Color::black() };
    let mat_water = Material { tex_id: 8, albedo: 0.98, specular: 0.5,  transparency: 0.9,  reflectivity: 0.05, ior: 1.33, shininess: 32.0, emissive: Color::black() };
    let mat_lava  = Material { tex_id: 9, albedo: 1.0, specular: 0.2,  transparency: 0.0,  reflectivity: 0.0, ior: 1.0,  shininess: 8.0,  emissive: Color::new(1.8, 0.6, 0.1) };
    let mat_metal = Material { tex_id:10, albedo: 1.0, specular: 0.9,  transparency: 0.0,  reflectivity: 0.7, ior: 1.0,  shininess: 64.0, emissive: Color::black() };

    let materials = vec![mat_grass, mat_dirt, mat_cobb, mat_wood, mat_leaf, mat_glass, mat_stone, mat_water, mat_lava, mat_metal];

    let grass_faces = [1, 1, 2, 0, 1, 1];               // césped con top/side/dirt
    let one_tex = |tid: usize| [tid, tid, tid, tid, tid, tid];

    // --- Terreno 16x16 con lomitas ---
    let mut cubes: Vec<Aabb> = Vec::new();
    let size = 16;

    // Reservo un área para el lago (no se colocan bloques altos ahí)
    let lake_min = (10, 2);
    let lake_max = (15, 6);

    // Lava aislada (esquina superior derecha)
    let lava_min = (13, 13);
    let lava_max = (15, 15);

    for z in 0..size {
        for x in 0..size {
            let in_lake = x >= lake_min.0 && x < lake_max.0 && z >= lake_min.1 && z < lake_max.1;
            let in_lava = x >= lava_min.0 && x <= lava_max.0 && z >= lava_min.1 && z <= lava_max.1;

            // altura del terreno
            let mut h = height(x as i32, z as i32);
            if in_lake { h = 1; } // el lago queda despejado al nivel 1
            if in_lava { h = 1; } // lava también a nivel base

            // capas de tierra (relleno)
            for y in 0..(h - 1).max(0) {
                push_block(&mut cubes, x as f32, y as f32, z as f32, 1, one_tex(2));
            }
            // bloque superior (grass con texturas por cara)
            push_block(&mut cubes, x as f32, (h - 1).max(0) as f32, z as f32, 0, grass_faces);
        }
    }

    // --- Lago despejado (no toca la casa) ---
    cubes.push(Aabb {
        min: Vec3::new(lake_min.0 as f32, 1.0, lake_min.1 as f32),
        max: Vec3::new(lake_max.0 as f32, 1.08, lake_max.1 as f32),
        mat_id: 7, // water
        face_tex: Some(one_tex(8)),
    });

    // --- Lava en su propia esquina ---
    cubes.push(Aabb {
        min: Vec3::new(lava_min.0 as f32, 1.0, lava_min.1 as f32),
        max: Vec3::new((lava_max.0 + 1) as f32, 1.06, (lava_max.1 + 1) as f32),
        mat_id: 8, // lava
        face_tex: Some(one_tex(9)),
    });

    // --- Camino de cobble que cruza el valle ---
    for x in 0..size {
        cubes.push(Aabb {
            min: Vec3::new(x as f32, 1.0, 8.0),
            max: Vec3::new(x as f32 + 1.0, 1.1, 9.0),
            mat_id: 2,
            face_tex: Some(one_tex(3)),
        });
    }

    // --- Casa reubicada (no pisa el agua) ---
    // La coloco en la lomita izquierda, adaptando su base a la altura local.
    let bx = 2; let bz = 9;        // esquina inferior-izquierda de la casa
    let base_h = height(bx as i32, bz as i32) as f32; // altura de referencia
    let base = Vec3::new(bx as f32, base_h, bz as f32);

    // *** CIMENTOS *** (evita esquinas flotantes)
    // Relleno con dirt desde la altura real del terreno de cada celda hasta base_h.
    for z in 0..5 {
        for x in 0..5 {
            let gh = height((bx + x) as i32, (bz + z) as i32) as i32;
            let top = base_h as i32;
            for y in gh..top {
                push_block(&mut cubes, (bx + x) as f32, y as f32, (bz + z) as f32, 1, one_tex(2));
            }
        }
    }

    // paredes X
    for z in 0..5 {
        for y in 0..4 {
            cubes.push(Aabb { min: base.add(Vec3::new(0.0, y as f32, z as f32)),  max: base.add(Vec3::new(0.5, y as f32 + 1.0, z as f32 + 1.0)), mat_id: 3, face_tex: Some(one_tex(4)) });
            cubes.push(Aabb { min: base.add(Vec3::new(4.5, y as f32, z as f32)), max: base.add(Vec3::new(5.0, y as f32 + 1.0, z as f32 + 1.0)), mat_id: 3, face_tex: Some(one_tex(4)) });
        }
    }
    // paredes Z
    for x in 0..5 {
        for y in 0..4 {
            cubes.push(Aabb { min: base.add(Vec3::new(x as f32, y as f32, 0.0)),  max: base.add(Vec3::new(x as f32 + 1.0, y as f32 + 1.0, 0.5)),   mat_id: 3, face_tex: Some(one_tex(4)) });
            cubes.push(Aabb { min: base.add(Vec3::new(x as f32, y as f32, 4.5)),  max: base.add(Vec3::new(x as f32 + 1.0, y as f32 + 1.0, 5.0)),   mat_id: 3, face_tex: Some(one_tex(4)) });
        }
    }
    // ventanas
    cubes.push(Aabb { min: base.add(Vec3::new(2.0, 1.0, 0.0)),  max: base.add(Vec3::new(3.0, 2.0, 0.5)),  mat_id: 5, face_tex: Some(one_tex(6)) });
    cubes.push(Aabb { min: base.add(Vec3::new(2.0, 1.0, 4.5)),  max: base.add(Vec3::new(3.0, 2.0, 5.0)),  mat_id: 5, face_tex: Some(one_tex(6)) });
    // techo de piedra
    for z in 0..5 {
        for x in 0..5 {
            cubes.push(Aabb { min: base.add(Vec3::new(x as f32, 4.0, z as f32)), max: base.add(Vec3::new(x as f32 + 1.0, 4.5, z as f32 + 1.0)), mat_id: 6, face_tex: Some(one_tex(7)) });
        }
    }
    // bloque metálico reflectivo delante de la casa (para que se noten los reflejos)
    cubes.push(Aabb {
        min: base.add(Vec3::new(5.5, 0.0, 1.5)),
        max: base.add(Vec3::new(6.5, 1.0, 2.5)),
        mat_id: 9,                 // metal
        face_tex: Some(one_tex(10)),
    });

    // --- Árboles de distintos tamaños con COPA EN NIVELES ---
    // (x, z, tronco_altura, niveles_de_copa, tamaño_base_copa)
    let trees = vec![
        (4, 4, 3, 3, 2),  // 3 niveles: 2 -> 1 -> 0 (tope 1x1)
        (12, 11, 4, 4, 3),// 4 niveles: 3 -> 2 -> 1 -> 0
        (8, 3, 2, 2, 2),  // 2 niveles: 2 -> 1 -> 0
        (6, 14, 5, 4, 3),
    ];

    for (tx, tz, trunk_h, levels, base_size) in trees {
        let th = height(tx as i32, tz as i32) as f32;

        // tronco (0.5x0.5 centrado)
        for y in 0..trunk_h {
            cubes.push(Aabb {
                min: Vec3::new(tx as f32 + 0.25, th + y as f32, tz as f32 + 0.25),
                max: Vec3::new(tx as f32 + 0.75, th + y as f32 + 1.0, tz as f32 + 0.75),
                mat_id: 3, face_tex: Some(one_tex(4)),
            });
        }

        // copa por niveles, cada nivel más pequeño y más alto
        let crown_base_y = th + trunk_h as f32; // arranque de la copa
        for i in 0..levels {
            let level_size = (base_size as i32 - i as i32).max(0); // 2->1->0...
            let y = crown_base_y + i as f32;
            // nivel cuadrado de (2*level_size + 1)^2 bloques; si size==0 => 1 bloque
            for z in (tz - level_size)..=(tz + level_size) {
                for x in (tx - level_size)..=(tx + level_size) {
                    push_block(&mut cubes, x as f32, y, z as f32, 4, one_tex(5)); // hojas
                }
            }
        }
    }

    // skybox opcional
    let skybox = CubeMap::from_folder("assets/skybox");

    let scene = Scene { cubes, materials, textures, sun_dir, sun_col, sky_mix, skybox };

    // Cámara: una vuelta completa, altura un poco mayor para leer mejor el relieve
    let angle = std::f32::consts::PI * 2.0 * t;
    let radius = 18.0 + 1.5 * (0.5 - (t * 2.0 * std::f32::consts::PI).cos() * 0.5);
    let eye_h = 6.5;
    let anim = Anim { angle, radius, eye_h, sky_mix };

    (scene, anim)
}

/* La escena básica no se usa en este flujo; si quisieras, puedes redirigirla aquí. */
pub fn build_scene_basic(t: f32) -> (Scene, Anim) {
    build_scene_minecraft(t)
}
