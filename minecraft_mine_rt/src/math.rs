#[derive(Clone, Copy, Debug, Default)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

impl Vec3 {
    pub fn new(x:f32,y:f32,z:f32)->Self{Self{x,y,z}}
    pub fn add(self,o:Self)->Self{Self::new(self.x+o.x,self.y+o.y,self.z+o.z)}
    pub fn sub(self,o:Self)->Self{Self::new(self.x-o.x,self.y-o.y,self.z-o.z)}
    pub fn mul(self,s:f32)->Self{Self::new(self.x*s,self.y*s,self.z*s)}
    pub fn hadamard(self,o:Self)->Self{Self::new(self.x*o.x,self.y*o.y,self.z*o.z)}
    pub fn dot(self,o:Self)->f32{self.x*o.x + self.y*o.y + self.z*o.z}
    pub fn cross(self,o:Self)->Self{
        Self::new(self.y*o.z-self.z*o.y, self.z*o.x-self.x*o.z, self.x*o.y-self.y*o.x)
    }
    pub fn len(self)->f32{ self.dot(self).sqrt() }
    pub fn norm(self)->Self{ let l=self.len(); if l>1e-8 { self.mul(1.0/l) } else { self } }
}

#[derive(Clone, Copy, Debug)]
pub struct Ray { pub o: Vec3, pub d: Vec3 } // d debe venir normalizado

pub fn reflect(i:Vec3, n:Vec3)->Vec3 { i.sub(n.mul(2.0*i.dot(n))) }

pub fn refract(i:Vec3, n:Vec3, eta:f32)->Option<Vec3>{
    let cosi = (-i.dot(n)).clamp(-1.0,1.0);
    let k = 1.0 - eta*eta*(1.0 - cosi*cosi);
    if k < 0.0 { None } else {
        Some(i.mul(eta).add(n.mul(eta*cosi - k.sqrt())))
    }
}

pub fn schlick(cos:f32, n1:f32, n2:f32)->f32{
    let r0 = ((n1-n2)/(n1+n2)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}
