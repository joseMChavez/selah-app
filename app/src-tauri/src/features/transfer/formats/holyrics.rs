// Listas de Holyrics (.hpls). No están cifradas: son registros [tipo: i32][largo: i32][objeto Java serializado],
// todo en big-endian. El tipo 0 es una canción (com.limagiran.holyrics.model.Music); de ella se toman
// title, artist, author y lyrics, con los párrafos separados por una línea en blanco (una diapositiva cada uno).
// Los demás registros (por ejemplo el texto final, tipo 3) se ignoran.
//
// Aquí hay un lector mínimo del formato de serialización de Java (java.io.ObjectInputStream, protocolo 2):
// lo justo para estos objetos, con límites de profundidad y de tamaño para que un archivo dañado dé un error
// y nunca cuelgue la app.
use std::collections::HashMap;
use std::rc::Rc;

use crate::features::transfer::domain::Incoming;

const MUSIC_CLASS: &str = "com.limagiran.holyrics.model.Music";
const STREAM_MAGIC: [u8; 4] = [0xAC, 0xED, 0x00, 0x05];
const MAX_DEPTH: usize = 64;

#[derive(Clone, Debug)]
enum Value {
    Null,
    Str(Rc<str>),
    Object(Rc<Object>),
    Class(Rc<ClassDesc>),
    /// Primitivos, arreglos, bloques de datos y lo que no hace falta para leer canciones.
    Other,
}

#[derive(Debug)]
struct ClassDesc {
    name: String,
    flags: u8,
    fields: Vec<(u8, String)>,
    parent: Option<Rc<ClassDesc>>,
}

#[derive(Debug)]
struct Object {
    class: String,
    fields: HashMap<String, Value>,
}

/// Indica si los bytes parecen una lista de Holyrics (sin leerla entera).
pub fn is_hpls(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes[8..12] == STREAM_MAGIC
}

/// Canciones de la lista, en su orden.
pub fn read(bytes: &[u8]) -> Result<Vec<Incoming>, String> {
    if !is_hpls(bytes) {
        return Err("no es una lista de Holyrics (.hpls)".into());
    }
    let mut songs = Vec::new();
    let mut offset = 0;
    while offset + 8 <= bytes.len() {
        let kind = i32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let len = i32::from_be_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
        let start = offset + 8;
        let end = usize::try_from(len).ok().and_then(|l| start.checked_add(l)).filter(|&e| e <= bytes.len());
        let Some(end) = end else {
            return Err("el archivo está incompleto o dañado".into());
        };
        if kind == 0 {
            let value = Reader::new(&bytes[start..end]).stream().map_err(|e| format!("canción {}: {e}", songs.len() + 1))?;
            if let Value::Object(music) = value
                && music.class == MUSIC_CLASS
            {
                songs.push(to_incoming(&music));
            }
        }
        offset = end;
    }
    if songs.is_empty() {
        return Err("la lista no tiene canciones".into());
    }
    Ok(songs)
}

fn to_incoming(music: &Object) -> Incoming {
    let text = |name: &str| match music.fields.get(name) {
        Some(Value::Str(s)) => s.trim().to_string(),
        _ => String::new(),
    };
    let lyrics = match music.fields.get("lyrics") {
        Some(Value::Str(s)) => s.to_string(),
        _ => String::new(),
    };
    Incoming { title: text("title"), singer: text("artist"), writer: text("author"), slides: paragraphs(&lyrics), background: None }
}

/// Párrafos separados por una o más líneas en blanco.
pub fn paragraphs(lyrics: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in lyrics.lines().map(str::trim_end) {
        if line.trim().is_empty() {
            if !current.is_empty() {
                out.push(current.join("\n").trim().to_string());
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        out.push(current.join("\n").trim().to_string());
    }
    out.retain(|p| !p.is_empty());
    out
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    handles: Vec<Value>,
    depth: usize,
}

type Parse<T> = std::result::Result<T, String>;

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0, handles: Vec::new(), depth: 0 }
    }

    fn stream(&mut self) -> Parse<Value> {
        if self.take(4)? != STREAM_MAGIC {
            return Err("no es un objeto Java".into());
        }
        self.content()
    }

    fn take(&mut self, n: usize) -> Parse<&'a [u8]> {
        let end = self.pos.checked_add(n).filter(|&e| e <= self.bytes.len()).ok_or("datos cortados")?;
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Parse<u8> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Parse<u16> {
        Ok(u16::from_be_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn i32(&mut self) -> Parse<i32> {
        Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn i64(&mut self) -> Parse<i64> {
        Ok(i64::from_be_bytes(self.take(8)?.try_into().unwrap()))
    }

    /// Texto en el «UTF-8 modificado» de Java (los caracteres fuera del plano básico van como dos sustitutos).
    fn utf(&mut self, len: usize) -> Parse<String> {
        let bytes = self.take(len)?;
        let mut units: Vec<u16> = Vec::with_capacity(len);
        let mut i = 0;
        while i < bytes.len() {
            let b = bytes[i];
            let (unit, used) = if b < 0x80 {
                (b as u16, 1)
            } else if b & 0xE0 == 0xC0 && i + 1 < bytes.len() {
                ((((b & 0x1F) as u16) << 6) | (bytes[i + 1] & 0x3F) as u16, 2)
            } else if b & 0xF0 == 0xE0 && i + 2 < bytes.len() {
                ((((b & 0x0F) as u16) << 12) | (((bytes[i + 1] & 0x3F) as u16) << 6) | (bytes[i + 2] & 0x3F) as u16, 3)
            } else {
                return Err("texto mal codificado".into());
            };
            units.push(unit);
            i += used;
        }
        Ok(char::decode_utf16(units).map(|c| c.unwrap_or('\u{FFFD}')).collect())
    }

    fn handle(&mut self) -> Parse<Value> {
        let index = self.i32()?.wrapping_sub(0x7E_0000);
        usize::try_from(index).ok().and_then(|i| self.handles.get(i).cloned()).ok_or_else(|| "referencia inválida".into())
    }

    fn content(&mut self) -> Parse<Value> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err("objetos demasiado anidados".into());
        }
        let value = self.content_inner();
        self.depth -= 1;
        value
    }

    fn content_inner(&mut self) -> Parse<Value> {
        match self.u8()? {
            0x70 => Ok(Value::Null),
            0x71 => self.handle(),
            0x74 => {
                let len = self.u16()? as usize;
                let s: Rc<str> = self.utf(len)?.into();
                self.handles.push(Value::Str(s.clone()));
                Ok(Value::Str(s))
            }
            0x7C => {
                let len = usize::try_from(self.i64()?).map_err(|_| "texto con largo inválido")?;
                let s: Rc<str> = self.utf(len)?.into();
                self.handles.push(Value::Str(s.clone()));
                Ok(Value::Str(s))
            }
            0x72 => {
                self.pos -= 1;
                Ok(self.class_desc()?.map(Value::Class).unwrap_or(Value::Null))
            }
            0x73 => self.object(),
            0x75 => self.array(),
            0x7E => {
                // Enum: su clase y el nombre de la constante.
                self.class_desc()?;
                let slot = self.handles.len();
                self.handles.push(Value::Other);
                let name = self.content()?;
                self.handles[slot] = name.clone();
                Ok(name)
            }
            0x77 => {
                let len = self.u8()? as usize;
                self.take(len)?;
                Ok(Value::Other)
            }
            0x7A => {
                let len = usize::try_from(self.i32()?).map_err(|_| "bloque con largo inválido")?;
                self.take(len)?;
                Ok(Value::Other)
            }
            0x78 => Err("fin de bloque inesperado".into()),
            tc => Err(format!("dato Java no admitido (0x{tc:02x})")),
        }
    }

    /// Descriptor de clase (o null, o una referencia a uno ya leído).
    fn class_desc(&mut self) -> Parse<Option<Rc<ClassDesc>>> {
        match self.u8()? {
            0x70 => Ok(None),
            0x71 => match self.handle()? {
                Value::Class(c) => Ok(Some(c)),
                _ => Err("referencia a clase inválida".into()),
            },
            0x72 => {
                let len = self.u16()? as usize;
                let name = self.utf(len)?;
                self.take(8)?; // serialVersionUID
                let slot = self.handles.len();
                self.handles.push(Value::Other);
                let flags = self.u8()?;
                let count = self.u16()?;
                let mut fields = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    let kind = self.u8()?;
                    let len = self.u16()? as usize;
                    let field = self.utf(len)?;
                    if kind == b'L' || kind == b'[' {
                        self.content()?; // nombre del tipo
                    }
                    fields.push((kind, field));
                }
                self.skip_annotations()?;
                let parent = self.class_desc()?;
                let class = Rc::new(ClassDesc { name, flags, fields, parent });
                self.handles[slot] = Value::Class(class.clone());
                Ok(Some(class))
            }
            0x7D => Err("clases proxy no admitidas".into()),
            tc => Err(format!("descriptor de clase inválido (0x{tc:02x})")),
        }
    }

    fn skip_annotations(&mut self) -> Parse<()> {
        loop {
            if self.bytes.get(self.pos) == Some(&0x78) {
                self.pos += 1;
                return Ok(());
            }
            self.content()?;
        }
    }

    fn object(&mut self) -> Parse<Value> {
        let class = self.class_desc()?.ok_or("objeto sin clase")?;
        let slot = self.handles.len();
        self.handles.push(Value::Other);
        let mut chain = Vec::new();
        let mut current = Some(class.clone());
        while let Some(c) = current {
            current = c.parent.clone();
            chain.push(c);
        }
        let mut fields = HashMap::new();
        for c in chain.iter().rev() {
            if c.flags & 0x04 != 0 {
                // Externalizable: datos propios hasta el fin de bloque.
                self.skip_annotations()?;
                continue;
            }
            for (kind, name) in &c.fields {
                let value = match kind {
                    b'B' | b'Z' => self.take(1).map(|_| Value::Other)?,
                    b'C' | b'S' => self.take(2).map(|_| Value::Other)?,
                    b'I' | b'F' => self.take(4).map(|_| Value::Other)?,
                    b'J' | b'D' => self.take(8).map(|_| Value::Other)?,
                    b'L' | b'[' => self.content()?,
                    _ => return Err("campo de tipo desconocido".into()),
                };
                fields.insert(name.clone(), value);
            }
            if c.flags & 0x01 != 0 {
                // writeObject propio (ArrayList, por ejemplo): datos extra hasta el fin de bloque.
                self.skip_annotations()?;
            }
        }
        let object = Value::Object(Rc::new(Object { class: class.name.clone(), fields }));
        self.handles[slot] = object.clone();
        Ok(object)
    }

    fn array(&mut self) -> Parse<Value> {
        let class = self.class_desc()?.ok_or("arreglo sin clase")?;
        let slot = self.handles.len();
        self.handles.push(Value::Other);
        let len = usize::try_from(self.i32()?).map_err(|_| "arreglo con largo inválido")?;
        let element = class.name.as_bytes().get(1).copied().unwrap_or(b'L');
        let size = match element {
            b'B' | b'Z' => 1,
            b'C' | b'S' => 2,
            b'I' | b'F' => 4,
            b'J' | b'D' => 8,
            _ => 0,
        };
        let value = if size > 0 {
            self.take(len.checked_mul(size).ok_or("arreglo demasiado grande")?)?;
            Value::Other
        } else {
            if len > self.bytes.len() {
                return Err("arreglo demasiado grande".into());
            }
            // Los elementos se leen para avanzar (y por sus referencias), pero las canciones no los necesitan.
            for _ in 0..len {
                self.content()?;
            }
            Value::Other
        };
        self.handles[slot] = value.clone();
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lista de Holyrics de prueba, con letras inventadas: tests/fixtures/ejemplo.hpls
    /// (la genera tests/fixtures/generar_hpls.py imitando la estructura de un .hpls real).
    const FIXTURE: &[u8] = include_bytes!("../../../../tests/fixtures/ejemplo.hpls");

    #[test]
    fn lee_la_lista_de_prueba() {
        assert!(is_hpls(FIXTURE));
        let songs = read(FIXTURE).unwrap();
        assert_eq!(songs.len(), 3);
        assert_eq!(songs[0].title, "Canción de prueba");
        assert_eq!(songs[0].singer, "Coro de ejemplo");
        assert_eq!(songs[0].slides, ["Primera estrofa\ncon dos líneas", "Coro: ¡aleluya! ñandú", "Final"]);
        assert_eq!(songs[1].writer, "Autora Inventada");
        assert_eq!(songs[2].title, "Himno sin párrafos");
        assert_eq!(songs[2].slides.len(), 1);
    }

    #[test]
    fn parrafos_por_linea_en_blanco() {
        assert_eq!(paragraphs("A\nB\n\n\nC  \n \nD\n"), ["A\nB", "C", "D"]);
        assert_eq!(paragraphs("\n\n"), Vec::<String>::new());
    }

    #[test]
    fn archivo_dañado_da_error_y_no_cuelga() {
        assert!(read(b"no es nada").is_err());
        assert!(read(&FIXTURE[..FIXTURE.len() / 2]).is_err());
        let mut altered = FIXTURE.to_vec();
        for b in altered.iter_mut().skip(40).step_by(7) {
            *b = 0xFF;
        }
        let _ = read(&altered);
    }

    /// Una lista real (fuera del repositorio): `cargo test -- --ignored lee_una_lista_real`.
    #[test]
    #[ignore]
    fn lee_una_lista_real() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples");
        let path = walk(&dir).into_iter().find(|p| p.extension().is_some_and(|e| e == "hpls")).expect("un .hpls en samples/");
        let songs = read(&std::fs::read(&path).unwrap()).unwrap();
        for s in &songs {
            println!("{} · {} · {} diapositivas", s.title, s.singer, s.slides.len());
        }
        assert!(!songs.is_empty());
    }

    #[cfg(test)]
    fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.extend(walk(&path));
            } else {
                out.push(path);
            }
        }
        out
    }
}
