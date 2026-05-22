use std::{collections::HashMap, fs::File, io::Read, path::Path};

use regex::Regex;
use roxmltree::{Document, Node};
use zip::ZipArchive;

use crate::{
    domain::{AnswerOption, Question},
    error::AppResult,
};

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub questions: Vec<Question>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurrentBlock {
    Closed,
    SelfCheck,
    Open,
}

pub fn parse_docx(path: &Path) -> AppResult<ParsedDocument> {
    let mut archive = ZipArchive::new(File::open(path)?)?;
    let mut document_xml = String::new();
    archive
        .by_name("word/document.xml")?
        .read_to_string(&mut document_xml)?;

    let document = Document::parse(&document_xml)?;
    let Some(body) = document.descendants().find(|node| is_tag(*node, "body")) else {
        return Ok(ParsedDocument {
            questions: Vec::new(),
            warnings: vec!["word/document.xml does not contain w:body".to_string()],
        });
    };

    let discipline_re = Regex::new(r"^Дисциплина\s*(\d+)\s*:?\s*(.+)$").expect("valid regex");
    let mut current_discipline_number: Option<usize> = None;
    let mut current_discipline = String::new();
    let mut current_block: Option<CurrentBlock> = None;
    let mut counters: HashMap<(usize, String), usize> = HashMap::new();
    let mut questions = Vec::new();
    let mut warnings = Vec::new();
    let mut table_index = 0usize;

    for child in body.children().filter(|node| node.is_element()) {
        if is_tag(child, "p") {
            let text = clean_text(&paragraph_text(child));
            if text.is_empty() {
                continue;
            }

            if let Some(caps) = discipline_re.captures(&text) {
                current_discipline_number =
                    caps.get(1).and_then(|value| value.as_str().parse().ok());
                current_discipline = caps
                    .get(2)
                    .map(|value| clean_text(value.as_str()))
                    .unwrap_or_default();
                current_block = None;
                continue;
            }

            if text == "Задания закрытого типа" {
                current_block = Some(CurrentBlock::Closed);
            } else if text == "Задания на последовательность и установление соответствия"
            {
                current_block = Some(CurrentBlock::SelfCheck);
            } else if text == "Задания открытого типа" {
                current_block = Some(CurrentBlock::Open);
            }
        } else if is_tag(child, "tbl") {
            table_index += 1;
            let rows = table_rows(child);
            if rows.len() <= 1 {
                continue;
            }

            let Some(block) = current_block else {
                continue;
            };
            let Some(discipline_number) = current_discipline_number else {
                continue;
            };
            if !looks_like_question_table(&rows, block) {
                continue;
            }

            for (row_offset, row) in rows.iter().enumerate().skip(1) {
                let source_row = row_offset + 1;
                if row.len() < 3 {
                    warnings.push(format!(
                        "Table {table_index}, row {source_row}: skipped row with fewer than 3 cells"
                    ));
                    continue;
                }

                let question_cell = row.get(1).cloned().unwrap_or_default();
                let answer_cell = row.last().cloned().unwrap_or_default();
                let raw_answer = clean_text(&answer_cell.join(" "));
                if raw_answer.is_empty() {
                    warnings.push(format!(
                        "Table {table_index}, row {source_row}: empty correct answer"
                    ));
                }

                let (
                    block_type,
                    question_type,
                    options,
                    correct_answers,
                    correct_text,
                    question_text,
                ) = match block {
                    CurrentBlock::Closed => {
                        let options_cell = row.get(2).cloned().unwrap_or_default();
                        let options = parse_closed_options(&options_cell);
                        let correct_answers = parse_answer_keys(&raw_answer);
                        let question_type = match correct_answers.len() {
                            1 => "single_choice".to_string(),
                            n if n > 1 => "multiple_choice".to_string(),
                            _ => "self_check".to_string(),
                        };

                        let option_keys: Vec<&str> =
                            options.iter().map(|option| option.key.as_str()).collect();
                        for answer in &correct_answers {
                            if !option_keys.iter().any(|key| key == answer) {
                                warnings.push(format!(
                                        "Table {table_index}, row {source_row}: answer {answer} has no matching option"
                                    ));
                            }
                        }

                        (
                            "closed".to_string(),
                            question_type,
                            options,
                            correct_answers,
                            None,
                            clean_closed_question(&question_cell),
                        )
                    }
                    CurrentBlock::SelfCheck => (
                        "self_check".to_string(),
                        "self_check".to_string(),
                        parse_self_check_items(row.get(2).cloned().unwrap_or_default()),
                        Vec::new(),
                        Some(normalize_answer_text(&raw_answer)),
                        clean_lines(&question_cell),
                    ),
                    CurrentBlock::Open => (
                        "open".to_string(),
                        "open".to_string(),
                        Vec::new(),
                        Vec::new(),
                        Some(normalize_answer_text(&raw_answer)),
                        clean_lines(&question_cell),
                    ),
                };

                if question_text.is_empty() {
                    warnings.push(format!(
                        "Table {table_index}, row {source_row}: empty question text"
                    ));
                }

                let id_type = match question_type.as_str() {
                    "single_choice" => "single",
                    "multiple_choice" => "multiple",
                    "open" => "open",
                    _ => "self_check",
                };
                let key = (discipline_number, id_type.to_string());
                let counter = counters.entry(key).or_insert(0);
                *counter += 1;
                let id = format!("d{discipline_number}_{id_type}_{counter:03}");

                questions.push(Question {
                    id,
                    discipline: current_discipline.clone(),
                    block_type,
                    question_type,
                    question: question_text,
                    options,
                    correct_answers,
                    correct_text,
                });
            }
        }
    }

    Ok(ParsedDocument {
        questions,
        warnings,
    })
}

fn is_tag(node: Node<'_, '_>, name: &str) -> bool {
    node.is_element() && node.tag_name().name() == name
}

fn paragraph_text(paragraph: Node<'_, '_>) -> String {
    let mut out = String::new();
    for descendant in paragraph.descendants() {
        if is_tag(descendant, "t") {
            out.push_str(descendant.text().unwrap_or_default());
        } else if is_tag(descendant, "tab") {
            out.push(' ');
        } else if is_tag(descendant, "br") {
            out.push('\n');
        }
    }
    out
}

fn cell_paragraphs(cell: Node<'_, '_>) -> Vec<String> {
    cell.children()
        .filter(|node| is_tag(*node, "p"))
        .filter_map(|paragraph| {
            let text = clean_text(&paragraph_text(paragraph));
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        })
        .collect()
}

fn table_rows(table: Node<'_, '_>) -> Vec<Vec<Vec<String>>> {
    table
        .children()
        .filter(|node| is_tag(*node, "tr"))
        .map(|row| {
            row.children()
                .filter(|node| is_tag(*node, "tc"))
                .map(cell_paragraphs)
                .collect()
        })
        .collect()
}

fn looks_like_question_table(rows: &[Vec<Vec<String>>], block: CurrentBlock) -> bool {
    let Some(header) = rows.first() else {
        return false;
    };
    let flattened: Vec<String> = header
        .iter()
        .map(|cell| clean_text(&cell.join(" ")))
        .collect();
    let has_question = flattened
        .iter()
        .any(|cell| cell.contains("Содержание вопроса"));
    let has_answer = flattened.iter().any(|cell| cell.contains("Верный ответ"));
    let has_options = flattened
        .iter()
        .any(|cell| cell.contains("Варианты ответов"));

    match block {
        CurrentBlock::Open => has_question && has_answer && flattened.len() >= 3,
        CurrentBlock::Closed | CurrentBlock::SelfCheck => {
            has_question && has_answer && has_options && flattened.len() >= 4
        }
    }
}

fn clean_closed_question(paragraphs: &[String]) -> String {
    let joined = clean_lines(paragraphs);
    let instruction_re =
        Regex::new(r"(?iu)^выберите\s+правильн\p{L}*\s*вариант\p{L}*\s*ответа:\s*")
            .expect("valid regex");
    clean_text(&instruction_re.replace(&joined, ""))
}

fn clean_lines(paragraphs: &[String]) -> String {
    paragraphs
        .iter()
        .map(|text| clean_text(text))
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_closed_options(paragraphs: &[String]) -> Vec<AnswerOption> {
    let marker_re = Regex::new(r"(?u)([АБВГДЕЖЗA-Fa-fа-бв-гд-е])\s*[\)\.]").expect("valid regex");
    let mut parsed: Vec<(Option<String>, String)> = Vec::new();
    let mut found_markers = false;

    for paragraph in paragraphs {
        let matches: Vec<_> = marker_re.find_iter(paragraph).collect();
        if matches.is_empty() {
            let text = clean_text(paragraph);
            if !text.is_empty() {
                parsed.push((None, text));
            }
            continue;
        }

        found_markers = true;
        for (index, marker) in matches.iter().enumerate() {
            let raw_key = marker
                .as_str()
                .chars()
                .next()
                .map(|ch| ch.to_string())
                .unwrap_or_default();
            let key = normalize_label(&raw_key);
            let start = marker.end();
            let end = matches
                .get(index + 1)
                .map(|next| next.start())
                .unwrap_or_else(|| paragraph.len());
            let text = clean_text(&paragraph[start..end]);
            if !text.is_empty() {
                parsed.push((Some(key), text));
            }
        }
    }

    let letters = ["А", "Б", "В", "Г", "Д", "Е", "Ж", "З"];
    let mut used = Vec::<String>::new();
    let mut next_letter = 0usize;
    let mut options = Vec::new();

    if !found_markers && parsed.len() == 1 {
        return parsed
            .into_iter()
            .enumerate()
            .map(|(index, (_, text))| AnswerOption {
                key: letters.get(index).unwrap_or(&"?").to_string(),
                text,
            })
            .collect();
    }

    for (key, text) in parsed {
        let key = match key {
            Some(value) => value,
            None => {
                while next_letter < letters.len()
                    && used.iter().any(|used_key| used_key == letters[next_letter])
                {
                    next_letter += 1;
                }
                let value = letters.get(next_letter).unwrap_or(&"?").to_string();
                next_letter += 1;
                value
            }
        };
        used.push(key.clone());
        options.push(AnswerOption { key, text });
    }

    options
}

fn parse_self_check_items(paragraphs: Vec<String>) -> Vec<AnswerOption> {
    if paragraphs.is_empty() {
        return Vec::new();
    }

    paragraphs
        .into_iter()
        .enumerate()
        .filter_map(|(index, text)| {
            let text = clean_text(&text);
            if text.is_empty() {
                None
            } else {
                Some(AnswerOption {
                    key: (index + 1).to_string(),
                    text,
                })
            }
        })
        .collect()
}

fn parse_answer_keys(answer: &str) -> Vec<String> {
    let normalized = normalize_answer_text(answer);
    let answer_re = Regex::new(r"(?u)[АБВГДЕЖЗA-Fa-fа-бв-гд-е]").expect("valid regex");
    let mut keys = Vec::new();
    for item in answer_re.find_iter(&normalized) {
        let key = normalize_label(item.as_str());
        if !keys.iter().any(|existing| existing == &key) {
            keys.push(key);
        }
    }
    keys
}

fn normalize_answer_text(answer: &str) -> String {
    clean_text(answer)
        .replace('–', "-")
        .replace('—', "-")
        .replace('−', "-")
        .replace('_', "-")
        .trim_matches(|ch| ch == ',' || ch == '.')
        .to_string()
}

fn normalize_label(raw: &str) -> String {
    match raw.trim().chars().next().unwrap_or('?') {
        'A' | 'a' | 'А' | 'а' => "А",
        'B' | 'b' | 'Б' | 'б' => "Б",
        'C' | 'c' | 'В' | 'в' => "В",
        'D' | 'd' | 'Г' | 'г' => "Г",
        'E' | 'e' | 'Д' | 'д' => "Д",
        'F' | 'f' | 'Е' | 'е' => "Е",
        'Ж' | 'ж' => "Ж",
        'З' | 'з' => "З",
        other => return other.to_string().to_uppercase(),
    }
    .to_string()
}

fn clean_text(input: &str) -> String {
    input
        .replace('\u{00a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_expected_seed_document_shape() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("repo root")
            .join("fos_gia_vo_bak_09_03_02_rsob.docx");
        if !path.exists() {
            return;
        }

        let parsed = parse_docx(&path).expect("seed docx parses");
        assert_eq!(parsed.questions.len(), 450);
        assert_eq!(
            parsed
                .questions
                .iter()
                .filter(|q| q.question_type == "single_choice")
                .count(),
            150
        );
        assert_eq!(
            parsed
                .questions
                .iter()
                .filter(|q| q.question_type == "multiple_choice")
                .count(),
            100
        );
        assert_eq!(
            parsed
                .questions
                .iter()
                .filter(|q| q.question_type == "self_check")
                .count(),
            150
        );
        assert_eq!(
            parsed
                .questions
                .iter()
                .filter(|q| q.question_type == "open")
                .count(),
            50
        );
    }
}
