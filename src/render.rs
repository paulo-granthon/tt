use crate::color::{paint, BOLD, CYAN, DIM};
use crate::engine::Translation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    Full,
    Quiet,
    Synonyms,
}

const NO_SYNONYMS: &str = "no synonyms for this translation";

pub fn render(translation: &Translation, filter: Filter, color: bool) -> String {
    match filter {
        Filter::Quiet => translation.primary.clone(),
        Filter::Synonyms => {
            let block = synonyms_block(translation, color);
            if block.is_empty() {
                paint(color, DIM, NO_SYNONYMS)
            } else {
                block
            }
        }
        Filter::Full => {
            let mut out = paint(color, BOLD, &translation.primary);
            let block = synonyms_block(translation, color);
            if !block.is_empty() {
                out.push_str("\n\n");
                out.push_str(&block);
            }
            out
        }
    }
}

fn synonyms_block(translation: &Translation, color: bool) -> String {
    translation
        .synonyms
        .iter()
        .map(|synonym| {
            if synonym.back.is_empty() {
                paint(color, CYAN, &synonym.word)
            } else {
                format!(
                    "{}  {}",
                    paint(color, CYAN, &synonym.word),
                    paint(color, DIM, &synonym.back.join(", "))
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
