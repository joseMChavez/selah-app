// Lectura del XML de Beblia: <bible translation status|info link> > testament > book[number] > chapter[number] > verse[number].
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::domain::{abbreviation_and_language, clean_name, normalize, Bible, BibleInfo, BookInfo, Verse};
use super::library::folder_language;

fn elements<'a, 'input>(
    node: roxmltree::Node<'a, 'input>,
    tag: &'static str,
) -> impl Iterator<Item = roxmltree::Node<'a, 'input>> {
    node.children()
        .filter(move |n| n.is_element() && n.tag_name().name() == tag)
}

fn number(node: roxmltree::Node<'_, '_>) -> Option<u32> {
    node.attribute("number").and_then(|n| n.trim().parse().ok())
}

pub fn parse(path: &Path, builtin: bool) -> Result<Bible, String> {
    let xml = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let doc = roxmltree::Document::parse(xml.trim_start_matches('\u{feff}'))
        .map_err(|e| e.to_string())?;
    let root = doc.root_element();
    if root.tag_name().name() != "bible" {
        return Err("falta el elemento <bible>".into());
    }

    let mut chapters = BTreeMap::new();
    for book in root
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "book")
    {
        let Some(book_number) = number(book) else {
            continue;
        };
        for chapter in elements(book, "chapter") {
            let Some(chapter_number) = number(chapter) else {
                continue;
            };
            let verses: Vec<Verse> = elements(chapter, "verse")
                .filter_map(|v| {
                    let text: String = v
                        .descendants()
                        .filter(|n| n.is_text())
                        .filter_map(|n| n.text())
                        .collect();
                    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    // Algunas traducciones unen versículos y dejan los demás vacíos.
                    (!text.is_empty()).then_some(Verse {
                        number: number(v)?,
                        text,
                    })
                })
                .collect();
            if !verses.is_empty() {
                chapters.insert((book_number, chapter_number), verses);
            }
        }
    }
    if chapters.is_empty() {
        return Err("no contiene versículos".into());
    }

    let mut books: BTreeMap<u32, u32> = BTreeMap::new();
    for &(book, chapter) in chapters.keys() {
        let max = books.entry(book).or_default();
        *max = (*max).max(chapter);
    }
    let search_index = chapters
        .iter()
        .flat_map(|(&(b, c), verses)| {
            verses
                .iter()
                .map(move |v| (b, c, v.number, normalize(&v.text)))
        })
        .collect();

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("biblia")
        .to_string();
    let (abbreviation, name_language) = abbreviation_and_language(&stem);
    let language = folder_language(path).or(name_language);
    let copyright = root
        .attribute("status")
        .or(root.attribute("info"))
        .unwrap_or("")
        .trim()
        .to_string();
    Ok(Bible {
        info: BibleInfo {
            id: stem.clone(),
            abbreviation,
            name: clean_name(root.attribute("translation").unwrap_or(&stem)),
            copyright,
            link: root.attribute("link").map(str::to_string),
            language,
            builtin,
            books: books
                .into_iter()
                .map(|(number, chapters)| BookInfo { number, chapters })
                .collect(),
        },
        chapters,
        search_index,
    })
}
