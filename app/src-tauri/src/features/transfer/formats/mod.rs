// Formatos de archivo que entiende la importación. Uno nuevo (OpenLyrics, EasyWorship…) va en su propio
// archivo con una función `read(&[u8]) -> Result<…, String>`, y se reconoce en `commands::read_file`.
pub mod holyrics;
pub mod emetpro;
