use crate::color::{paint, BOLD, CYAN, DIM, GREEN, YELLOW};
use crate::engine::Translation;
use crate::lang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    Full,
    Quiet,
    Synonyms,
    Verbose,
}

pub struct Meta<'a> {
    pub sl: &'a str,
    pub tl: &'a str,
    pub text: &'a str,
}

const NO_SYNONYMS: &str = "no synonyms for this translation";

pub fn render(translation: &Translation, filter: Filter, color: bool, meta: &Meta) -> String {
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
        Filter::Verbose => verbose(translation, color, meta),
    }
}

fn verbose(translation: &Translation, color: bool, meta: &Meta) -> String {
    let source = if meta.sl == "auto" {
        translation.detected_source.as_deref().unwrap_or("auto")
    } else {
        meta.sl
    };
    let source_name = lang::name(source).unwrap_or(source);
    let target_name = lang::name(meta.tl).unwrap_or(meta.tl);

    let label = |t: &str| paint(color, YELLOW, t);
    let indent = |t: &str| {
        t.lines()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut out = format!(
        "{} {} {} {}\n\n",
        paint(color, DIM, "translating"),
        paint(color, CYAN, source_name),
        paint(color, GREEN, "->"),
        paint(color, CYAN, target_name),
    );
    out.push_str(&format!(
        "{} {}\n{}\n\n",
        label("ORIGINAL"),
        paint(color, DIM, &format!("({source_name})")),
        indent(meta.text)
    ));
    out.push_str(&format!(
        "{} {}\n{}\n",
        label("TRANSLATION"),
        paint(color, DIM, &format!("({target_name})")),
        indent(&paint(color, BOLD, &translation.primary))
    ));

    let block = synonyms_block(translation, color);
    let synonyms = if block.is_empty() {
        paint(color, DIM, NO_SYNONYMS)
    } else {
        block
    };
    out.push_str(&format!("\n{}\n{}", label("SYNONYMS"), indent(&synonyms)));
    out
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
