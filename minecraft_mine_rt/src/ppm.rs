use std::fs::File;
use std::io::{self, Read, BufRead, BufReader};

pub struct PpmImage { pub w:usize, pub h:usize, pub data:Vec<u8> } // RGB

fn next_token<R:BufRead>(r:&mut R)->io::Result<String>{
    let mut buf = String::new();
    loop {
        buf.clear();
        let n = r.read_line(&mut buf)?;
        if n==0 { return Ok(String::new()); }
        let line = buf.trim();
        if line.is_empty() || line.starts_with('#') { continue; }
        return Ok(line.to_string());
    }
}

pub fn load_ppm(path:&str)->io::Result<PpmImage>{
    let mut f = File::open(path)?;
    // leemos todo para detectar magic
    let mut bytes = Vec::new(); f.read_to_end(&mut bytes)?;
    if bytes.len() < 2 { return Err(io::Error::new(io::ErrorKind::InvalidData,"PPM too short")); }

    if &bytes[0..2] == b"P6" {
        // parse header tokens con comentarios
        let mut rdr = BufReader::new(&bytes[..]);
        let mut header = String::new(); rdr.read_line(&mut header)?; // "P6"
        // tokens w,h,max
        // Nota:  ignorar comentarios y espacios
        let mut tokens: Vec<String> = Vec::new();
        while tokens.len() < 3 {
            let mut l = String::new();
            rdr.read_line(&mut l)?;
            if l.is_empty() { break; }
            let lt = l.trim();
            if lt.starts_with('#') || lt.is_empty() { continue; }
            tokens.extend(lt.split_whitespace().map(|s| s.to_string()));
        }
        let w:usize = tokens[0].parse().unwrap();
        let h:usize = tokens[1].parse().unwrap();
        let maxv:usize = tokens[2].parse().unwrap();
        if maxv != 255 { return Err(io::Error::new(io::ErrorKind::InvalidData, "PPM maxval must be 255")); }

        // posición actual en el buffer
        let mut _consumed = header.len();
        for t in tokens {
            _consumed += t.len()+1; // aprox
        }
        let marker = b"255";
        let start = bytes.windows(marker.len()).position(|w| w==marker)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,"bad header"))?;
        let mut pos = start + marker.len();
        // salta whitespace
        while pos < bytes.len() && (bytes[pos]==b' ' || bytes[pos]==b'\n' || bytes[pos]==b'\r' || bytes[pos]==b'\t') { pos+=1; }

        let expected = w*h*3;
        if pos+expected > bytes.len() { return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"short data")); }
        let data = bytes[pos..pos+expected].to_vec();
        Ok(PpmImage{ w,h,data })
    } else if &bytes[0..2] == b"P3" {
        // ASCII: parse tokens
        let mut rdr = BufReader::new(&bytes[..]);
        let mut line = String::new(); rdr.read_line(&mut line)?; // "P3"
        // consumir comentarios/vacío y leer w,h,max
        let mut dims = String::new();
        loop {
            dims.clear();
            let n = rdr.read_line(&mut dims)?;
            if n==0 { return Err(io::Error::new(io::ErrorKind::InvalidData,"bad header")); }
            let t = dims.trim();
            if !t.is_empty() && !t.starts_with('#') { break; }
        }
        let mut it = dims.split_whitespace();
        let w:usize = it.next().unwrap().parse().unwrap();
        let h:usize = it.next().unwrap().parse().unwrap();
        let mut maxv_line = String::new(); rdr.read_line(&mut maxv_line)?;
        let maxv:usize = maxv_line.trim().parse().unwrap();
        if maxv != 255 { return Err(io::Error::new(io::ErrorKind::InvalidData,"maxval must be 255")); }

        // lee todos los números RGB
        let mut nums = String::new(); rdr.read_to_string(&mut nums)?;
        let mut data = Vec::with_capacity(w*h*3);
        for tok in nums.split_whitespace() {
            let v:u8 = tok.parse::<u16>().map(|z| z.min(255) as u8).unwrap_or(0);
            data.push(v);
        }
        if data.len() < w*h*3 { return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"not enough rgb")); }
        data.truncate(w*h*3);
        Ok(PpmImage{ w,h,data })
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData,"unsupported PPM magic (use P3 or P6)"))
    }
}

pub fn write_ppm(path:&str, w:usize, h:usize, rgb:&[u8])->std::io::Result<()>{
    use std::io::Write;
    let mut f = File::create(path)?;
    write!(f, "P6\n{} {}\n255\n", w, h)?;
    f.write_all(rgb)?;
    Ok(())
}
