// Miniaturas del PDF que está en pantalla, para que el teléfono vea lo que se proyecta y elija página.
// Las dibuja la consola (ya tiene el PDF abierto con pdf.js) y las guarda aquí, en memoria; el teléfono
// las pide a /api/thumb. Solo hay un documento a la vez: al pasar otro PDF a pantalla se vacía.
use std::collections::HashMap;
use std::sync::Arc;

/// Más páginas que esto no se guardan (un PDF enorme no debe llenar la memoria).
pub const MAX_PAGES: u32 = 1000;
/// Una miniatura de unos 480 px en JPEG pesa decenas de KB; más que esto no es una miniatura.
pub const MAX_BYTES: usize = 1024 * 1024;
/// El id del documento lo inventa la consola (un UUID): más largo no es uno.
const MAX_DOC: usize = 64;

#[derive(Default)]
pub struct Thumbs {
    /// El documento en pantalla; vacío si no hay ninguno.
    doc: String,
    /// Páginas del documento: más allá no hay nada que esperar.
    total: u32,
    pages: HashMap<u32, Arc<Vec<u8>>>,
}

impl Thumbs {
    /// Empieza un documento nuevo de `total` páginas. Las miniaturas del anterior se descartan.
    pub fn start(&mut self, doc: &str, total: u32) -> bool {
        if doc.is_empty() || doc.len() > MAX_DOC {
            return false;
        }
        if self.doc != doc {
            self.doc = doc.to_string();
            self.pages.clear();
        }
        self.total = total.min(MAX_PAGES);
        true
    }

    /// Guarda una página del documento vigente. false si no es de él o no parece una miniatura JPEG.
    pub fn put(&mut self, doc: &str, page: u32, jpeg: Vec<u8>) -> bool {
        let valid = !doc.is_empty()
            && doc == self.doc
            && (1..=self.total).contains(&page)
            && jpeg.len() <= MAX_BYTES
            && jpeg.starts_with(&[0xFF, 0xD8]);
        if valid {
            self.pages.insert(page, Arc::new(jpeg));
        }
        valid
    }

    pub fn get(&self, doc: &str, page: u32) -> Lookup {
        if doc.is_empty() || doc != self.doc {
            return Lookup::Gone;
        }
        match self.pages.get(&page) {
            Some(jpeg) => Lookup::Ready(jpeg.clone()),
            None if (1..=self.total).contains(&page) => Lookup::Pending,
            None => Lookup::Gone,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Lookup {
    Ready(Arc<Vec<u8>>),
    /// Del documento en pantalla, pero la consola aún no la dibujó.
    Pending,
    /// De otro documento, o una página imposible: no va a llegar.
    Gone,
}

#[cfg(test)]
mod tests {
    use super::*;

    const JPEG: [u8; 4] = [0xFF, 0xD8, 0xFF, 0xD9];

    #[test]
    fn guarda_las_del_documento_vigente() {
        let mut thumbs = Thumbs::default();
        assert!(thumbs.start("a", 3));
        assert_eq!(thumbs.get("a", 2), Lookup::Pending);
        assert_eq!(thumbs.get("a", 4), Lookup::Gone, "no existe: no se hace esperar al teléfono");
        assert!(thumbs.put("a", 2, JPEG.to_vec()));
        assert_eq!(thumbs.get("a", 2), Lookup::Ready(Arc::new(JPEG.to_vec())));
        assert_eq!(thumbs.get("b", 2), Lookup::Gone);
    }

    #[test]
    fn otro_documento_vacia_lo_anterior() {
        let mut thumbs = Thumbs::default();
        thumbs.start("a", 3);
        thumbs.put("a", 1, JPEG.to_vec());
        thumbs.start("a", 3);
        assert!(matches!(thumbs.get("a", 1), Lookup::Ready(_)), "el mismo documento no se vacía");
        thumbs.start("b", 3);
        assert_eq!(thumbs.get("a", 1), Lookup::Gone);
        assert_eq!(thumbs.get("b", 1), Lookup::Pending);
        assert!(!thumbs.put("a", 1, JPEG.to_vec()), "llega tarde, de un documento que ya no está");
        assert!(!thumbs.start("", 3));
        assert_eq!(thumbs.get("", 1), Lookup::Gone);
    }

    #[test]
    fn rechaza_lo_que_no_es_una_miniatura() {
        let mut thumbs = Thumbs::default();
        thumbs.start("a", MAX_PAGES + 50);
        assert!(!thumbs.put("a", 1, b"<svg>".to_vec()));
        assert!(!thumbs.put("a", 0, JPEG.to_vec()));
        assert!(!thumbs.put("a", MAX_PAGES + 1, JPEG.to_vec()));
        let mut big = JPEG.to_vec();
        big.resize(MAX_BYTES + 1, 0);
        assert!(!thumbs.put("a", 1, big));
        assert!(!thumbs.start(&"x".repeat(65), 3));
        assert_eq!(thumbs.get("a", 0), Lookup::Gone);
    }
}
