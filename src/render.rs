use crate::color::{hyperlink, paint, BOLD, CYAN, DIM, GREEN, YELLOW};
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
            let mut out = String::new();
            if meta.sl == "auto" {
                if let Some(code) = &translation.detected_source {
                    out.push_str(&paint(
                        color,
                        DIM,
                        &format!("detected: {}", lang::display_name(code)),
                    ));
                    out.push('\n');
                }
            }
            if let Some(correction) = &translation.correction {
                out.push_str(&paint(color, DIM, &format!("did you mean: {correction}")));
                out.push('\n');
            }
            out.push_str(&paint(color, BOLD, &translation.primary));
            if let Some(translit) = &translation.target_translit {
                out.push('\n');
                out.push_str(&paint(color, DIM, translit));
            }
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
    let auto = meta.sl == "auto";
    let source = if auto {
        translation.detected_source.as_deref().unwrap_or("auto")
    } else {
        meta.sl
    };
    let source_name = lang::display_name(source);
    let target_name = lang::display_name(meta.tl);
    let source_label = if auto {
        format!("{source_name} (detected)")
    } else {
        source_name.clone()
    };

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
        paint(color, CYAN, &source_label),
        paint(color, GREEN, "->"),
        paint(color, CYAN, &target_name),
    );
    let mut original = meta.text.to_string();
    if let Some(translit) = &translation.source_translit {
        original.push('\n');
        original.push_str(&paint(color, DIM, translit));
    }
    if let Some(correction) = &translation.correction {
        original.push('\n');
        original.push_str(&paint(color, DIM, &format!("did you mean: {correction}")));
    }
    out.push_str(&format!(
        "{} {}\n{}\n\n",
        label("ORIGINAL"),
        paint(color, DIM, &format!("({source_label})")),
        indent(&original)
    ));

    let mut translated = paint(color, BOLD, &translation.primary);
    if let Some(translit) = &translation.target_translit {
        translated.push('\n');
        translated.push_str(&paint(color, DIM, translit));
    }
    out.push_str(&format!(
        "{} {}\n{}\n",
        label("TRANSLATION"),
        paint(color, DIM, &format!("({target_name})")),
        indent(&translated)
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

pub fn default_hint(sl: &str, tl: &str) -> String {
    format!(
        "sl={sl} and tl={tl} come from the default profile; \
change them with `tt default sl=<lang> tl=<lang>`"
    )
}

pub fn footer(color: bool, hint: Option<&str>, url: &str) -> String {
    let mut out = String::from("\n");
    if let Some(hint) = hint {
        out.push_str(&paint(color, DIM, hint));
        out.push('\n');
    }
    out.push_str(&hyperlink(color, url, url));
    out
}
