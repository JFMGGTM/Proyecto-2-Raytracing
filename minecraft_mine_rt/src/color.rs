#[derive(Clone, Copy, Debug, Default)]
pub struct Color { pub r:f32, pub g:f32, pub b:f32 }

impl Color {
    pub fn new(r:f32,g:f32,b:f32)->Self{Self{r,g,b}}
    pub fn black()->Self{Self::new(0.0,0.0,0.0)}
    pub fn white()->Self{Self::new(1.0,1.0,1.0)}
    pub fn from_u8(r:u8,g:u8,b:u8)->Self{
        Self::new(r as f32/255.0, g as f32/255.0, b as f32/255.0)
    }
    pub fn to_u8_gamma(self)->[u8;3]{
        // gamma 2.2
        let gr = self.r.clamp(0.0,1.0).powf(1.0/2.2);
        let gg = self.g.clamp(0.0,1.0).powf(1.0/2.2);
        let gb = self.b.clamp(0.0,1.0).powf(1.0/2.2);
        [(gr*255.0) as u8, (gg*255.0) as u8, (gb*255.0) as u8]
    }
    pub fn add(self,o:Self)->Self{Self::new(self.r+o.r,self.g+o.g,self.b+o.b)}
    pub fn mul(self,s:f32)->Self{Self::new(self.r*s,self.g*s,self.b*s)}
    pub fn hadamard(self,o:Self)->Self{Self::new(self.r*o.r,self.g*o.g,self.b*o.b)}
    pub fn lerp(a:Self,b:Self,t:f32)->Self{ a.mul(1.0-t).add(b.mul(t)) }
    pub fn splat(k:f32)->Self{ Self::new(k,k,k) }
}
