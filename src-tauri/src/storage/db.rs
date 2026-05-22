use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::{
    domain::{
        AnswerResult, DisciplineStats, MistakeFilter, MistakeRow, Question, QuestionFilter,
        StatisticsSummary,
    },
    error::{msg, AppResult},
};

pub struct Database {
    path: PathBuf,
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: PathBuf) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;

        let database = Self {
            path,
            conn: Mutex::new(conn),
        };
        database.migrate()?;
        Ok(database)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn migrate(&self) -> AppResult<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS questions (
                id TEXT PRIMARY KEY,
                discipline TEXT NOT NULL,
                block_type TEXT NOT NULL,
                question_type TEXT NOT NULL,
                question TEXT NOT NULL,
                options_json TEXT NOT NULL,
                correct_answers_json TEXT NOT NULL,
                correct_text TEXT,
                source_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS attempts (
                id TEXT PRIMARY KEY,
                question_id TEXT NOT NULL,
                selected_answers_json TEXT NOT NULL,
                user_text_answer TEXT,
                is_correct INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mistakes (
                question_id TEXT PRIMARY KEY,
                mistake_count INTEGER NOT NULL DEFAULT 0,
                correct_streak INTEGER NOT NULL DEFAULT 0,
                last_mistake_at TEXT,
                status TEXT NOT NULL DEFAULT 'learning'
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_questions_discipline ON questions(discipline);
            CREATE INDEX IF NOT EXISTS idx_questions_type ON questions(question_type);
            CREATE INDEX IF NOT EXISTS idx_attempts_question ON attempts(question_id);
            CREATE INDEX IF NOT EXISTS idx_mistakes_status ON mistakes(status);
            "#,
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> AppResult<Option<String>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        Ok(conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> AppResult<()> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.execute(
            r#"
            INSERT INTO settings(key, value)
            VALUES (?1, ?2)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![key, value],
        )?;
        Ok(())
    }

    pub fn save_questions(&self, questions: &[Question], source_hash: &str) -> AppResult<()> {
        let mut conn = self.conn.lock().expect("database mutex poisoned");
        let tx = conn.transaction()?;
        let now = Utc::now().to_rfc3339();

        for question in questions {
            let options_json = serde_json::to_string(&question.options)?;
            let correct_answers_json = serde_json::to_string(&question.correct_answers)?;
            tx.execute(
                r#"
                INSERT INTO questions(
                    id, discipline, block_type, question_type, question, options_json,
                    correct_answers_json, correct_text, source_hash, created_at
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(id) DO UPDATE SET
                    discipline = excluded.discipline,
                    block_type = excluded.block_type,
                    question_type = excluded.question_type,
                    question = excluded.question,
                    options_json = excluded.options_json,
                    correct_answers_json = excluded.correct_answers_json,
                    correct_text = excluded.correct_text,
                    source_hash = excluded.source_hash
                "#,
                params![
                    question.id,
                    question.discipline,
                    question.block_type,
                    question.question_type,
                    question.question,
                    options_json,
                    correct_answers_json,
                    question.correct_text,
                    source_hash,
                    now,
                ],
            )?;
        }

        tx.execute(
            r#"
            INSERT INTO settings(key, value)
            VALUES ('imported_source_hash', ?1)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![source_hash],
        )?;
        tx.execute(
            r#"
            INSERT INTO settings(key, value)
            VALUES ('last_imported_at', ?1)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#,
            params![now],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn total_questions(&self) -> AppResult<i64> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        Ok(conn.query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))?)
    }

    pub fn total_attempts(&self) -> AppResult<i64> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        Ok(conn.query_row("SELECT COUNT(*) FROM attempts", [], |row| row.get(0))?)
    }

    pub fn active_mistakes(&self) -> AppResult<i64> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM mistakes WHERE mistake_count > 0 AND status = 'learning'",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn disciplines(&self) -> AppResult<Vec<String>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt =
            conn.prepare("SELECT DISTINCT discipline FROM questions ORDER BY discipline")?;
        let values = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(values)
    }

    pub fn list_questions(&self, filter: QuestionFilter) -> AppResult<Vec<Question>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt = conn.prepare(
            r#"
            SELECT id, discipline, block_type, question_type, question, options_json,
                   correct_answers_json, correct_text
            FROM questions
            ORDER BY discipline, id
            "#,
        )?;
        let mut questions = stmt
            .query_map([], row_to_question)?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(discipline) = filter.discipline {
            questions.retain(|question| question.discipline == discipline);
        }
        if let Some(question_type) = filter.question_type {
            questions.retain(|question| question.question_type == question_type);
        }
        if filter.only_mistakes.unwrap_or(false) {
            let active = self.active_mistake_ids_locked(&conn, filter.mistake_status.as_deref())?;
            questions.retain(|question| active.iter().any(|id| id == &question.id));
        }

        Ok(questions)
    }

    pub fn get_question(&self, question_id: &str) -> AppResult<Question> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        conn.query_row(
            r#"
            SELECT id, discipline, block_type, question_type, question, options_json,
                   correct_answers_json, correct_text
            FROM questions
            WHERE id = ?1
            "#,
            params![question_id],
            row_to_question,
        )
        .optional()?
        .ok_or_else(|| crate::error::AppError::Message(format!("Question {question_id} not found")))
    }

    pub fn record_attempt(
        &self,
        question_id: &str,
        selected_answers: &[String],
        user_text_answer: Option<String>,
        is_correct: bool,
    ) -> AppResult<AnswerResult> {
        let mut conn = self.conn.lock().expect("database mutex poisoned");
        let tx = conn.transaction()?;
        let question = tx
            .query_row(
                r#"
                SELECT id, discipline, block_type, question_type, question, options_json,
                       correct_answers_json, correct_text
                FROM questions
                WHERE id = ?1
                "#,
                params![question_id],
                row_to_question,
            )
            .optional()?
            .ok_or_else(|| {
                crate::error::AppError::Message(format!("Question {question_id} not found"))
            })?;

        let now = Utc::now().to_rfc3339();
        let attempt_id = Uuid::new_v4().to_string();
        let selected_json = serde_json::to_string(selected_answers)?;
        tx.execute(
            r#"
            INSERT INTO attempts(id, question_id, selected_answers_json, user_text_answer, is_correct, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                attempt_id,
                question_id,
                selected_json,
                user_text_answer,
                if is_correct { 1 } else { 0 },
                now,
            ],
        )?;

        let existing = tx
            .query_row(
                "SELECT mistake_count, correct_streak, status FROM mistakes WHERE question_id = ?1",
                params![question_id],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;

        let (mistake_count, correct_streak, status) = if is_correct {
            let (mistake_count, previous_streak, previous_status) =
                existing.unwrap_or((0, 0, "learning".to_string()));
            let correct_streak = previous_streak + 1;
            let status = if mistake_count > 0 && correct_streak >= 3 {
                "learned".to_string()
            } else {
                previous_status
            };
            tx.execute(
                r#"
                INSERT INTO mistakes(question_id, mistake_count, correct_streak, last_mistake_at, status)
                VALUES (?1, ?2, ?3, NULL, ?4)
                ON CONFLICT(question_id) DO UPDATE SET
                    correct_streak = excluded.correct_streak,
                    status = excluded.status
                "#,
                params![question_id, mistake_count, correct_streak, status],
            )?;
            (mistake_count, correct_streak, status)
        } else {
            let mistake_count = existing.map(|value| value.0).unwrap_or(0) + 1;
            let correct_streak = 0;
            let status = "learning".to_string();
            tx.execute(
                r#"
                INSERT INTO mistakes(question_id, mistake_count, correct_streak, last_mistake_at, status)
                VALUES (?1, ?2, 0, ?3, 'learning')
                ON CONFLICT(question_id) DO UPDATE SET
                    mistake_count = excluded.mistake_count,
                    correct_streak = 0,
                    last_mistake_at = excluded.last_mistake_at,
                    status = 'learning'
                "#,
                params![question_id, mistake_count, now],
            )?;
            (mistake_count, correct_streak, status)
        };

        tx.commit()?;

        Ok(AnswerResult {
            is_correct,
            question,
            correct_streak,
            mistake_count,
            status,
        })
    }

    pub fn list_mistakes(&self, filter: MistakeFilter) -> AppResult<Vec<MistakeRow>> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let mut stmt = conn.prepare(
            r#"
            SELECT q.id, q.question, q.discipline, q.question_type, m.mistake_count,
                   m.last_mistake_at, m.correct_streak, m.status
            FROM mistakes m
            JOIN questions q ON q.id = m.question_id
            WHERE m.mistake_count > 0
            ORDER BY m.status ASC, m.mistake_count DESC, m.last_mistake_at DESC
            "#,
        )?;
        let mut rows = stmt
            .query_map([], row_to_mistake)?
            .collect::<Result<Vec<_>, _>>()?;

        if let Some(status) = filter.status {
            let status = normalize_status_filter(&status);
            if status != "all" {
                rows.retain(|row| row.status == status);
            }
        }
        if let Some(discipline) = filter.discipline {
            rows.retain(|row| row.discipline == discipline);
        }
        if let Some(question_type) = filter.question_type {
            rows.retain(|row| row.question_type == question_type);
        }

        Ok(rows)
    }

    pub fn statistics(&self) -> AppResult<StatisticsSummary> {
        let conn = self.conn.lock().expect("database mutex poisoned");
        let total_questions =
            conn.query_row("SELECT COUNT(*) FROM questions", [], |row| row.get(0))?;
        let total_attempts =
            conn.query_row("SELECT COUNT(*) FROM attempts", [], |row| row.get(0))?;
        let correct_answers = conn.query_row(
            "SELECT COUNT(*) FROM attempts WHERE is_correct = 1",
            [],
            |row| row.get(0),
        )?;
        let wrong_answers = total_attempts - correct_answers;
        let learned_questions = conn.query_row(
            "SELECT COUNT(*) FROM mistakes WHERE mistake_count > 0 AND status = 'learned'",
            [],
            |row| row.get(0),
        )?;
        let active_mistakes = conn.query_row(
            "SELECT COUNT(*) FROM mistakes WHERE mistake_count > 0 AND status = 'learning'",
            [],
            |row| row.get(0),
        )?;

        let mut discipline_stmt =
            conn.prepare("SELECT DISTINCT discipline FROM questions ORDER BY discipline")?;
        let disciplines = discipline_stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        let mut by_discipline = Vec::new();

        for discipline in disciplines {
            let total_questions = conn.query_row(
                "SELECT COUNT(*) FROM questions WHERE discipline = ?1",
                params![&discipline],
                |row| row.get(0),
            )?;
            let attempts = conn.query_row(
                r#"
                SELECT COUNT(*)
                FROM attempts a
                JOIN questions q ON q.id = a.question_id
                WHERE q.discipline = ?1
                "#,
                params![&discipline],
                |row| row.get(0),
            )?;
            let correct = conn.query_row(
                r#"
                SELECT COUNT(*)
                FROM attempts a
                JOIN questions q ON q.id = a.question_id
                WHERE q.discipline = ?1 AND a.is_correct = 1
                "#,
                params![&discipline],
                |row| row.get(0),
            )?;
            let wrong = attempts - correct;
            by_discipline.push(DisciplineStats {
                discipline,
                total_questions,
                attempts,
                correct,
                wrong,
                accuracy: percentage(correct, attempts),
            });
        }

        let mut top_stmt = conn.prepare(
            r#"
            SELECT q.id, q.question, q.discipline, q.question_type, m.mistake_count,
                   m.last_mistake_at, m.correct_streak, m.status
            FROM mistakes m
            JOIN questions q ON q.id = m.question_id
            WHERE m.mistake_count > 0
            ORDER BY m.mistake_count DESC, m.last_mistake_at DESC
            LIMIT 10
            "#,
        )?;
        let top_problem_questions = top_stmt
            .query_map([], row_to_mistake)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(StatisticsSummary {
            total_questions,
            total_attempts,
            correct_answers,
            wrong_answers,
            accuracy: percentage(correct_answers, total_attempts),
            learned_questions,
            active_mistakes,
            by_discipline,
            top_problem_questions,
        })
    }

    fn active_mistake_ids_locked(
        &self,
        conn: &Connection,
        status_filter: Option<&str>,
    ) -> AppResult<Vec<String>> {
        let status_filter = normalize_status_filter(status_filter.unwrap_or("learning"));
        let mut stmt = if status_filter == "all" {
            conn.prepare("SELECT question_id FROM mistakes WHERE mistake_count > 0")?
        } else {
            conn.prepare(
                "SELECT question_id FROM mistakes WHERE mistake_count > 0 AND status = ?1",
            )?
        };

        let ids = if status_filter == "all" {
            stmt.query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map(params![status_filter], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?
        };
        Ok(ids)
    }
}

fn row_to_question(row: &rusqlite::Row<'_>) -> rusqlite::Result<Question> {
    let options_json: String = row.get(5)?;
    let correct_answers_json: String = row.get(6)?;
    let options = serde_json::from_str(&options_json).unwrap_or_default();
    let correct_answers = serde_json::from_str(&correct_answers_json).unwrap_or_default();
    Ok(Question {
        id: row.get(0)?,
        discipline: row.get(1)?,
        block_type: row.get(2)?,
        question_type: row.get(3)?,
        question: row.get(4)?,
        options,
        correct_answers,
        correct_text: row.get(7)?,
    })
}

fn row_to_mistake(row: &rusqlite::Row<'_>) -> rusqlite::Result<MistakeRow> {
    Ok(MistakeRow {
        question_id: row.get(0)?,
        question: row.get(1)?,
        discipline: row.get(2)?,
        question_type: row.get(3)?,
        mistake_count: row.get(4)?,
        last_mistake_at: row.get(5)?,
        correct_streak: row.get(6)?,
        status: row.get(7)?,
    })
}

fn normalize_status_filter(status: &str) -> String {
    match status {
        "not_learned" | "active" | "learning" => "learning".to_string(),
        "learned" | "resolved" => "learned".to_string(),
        "all" | "" => "all".to_string(),
        other => other.to_string(),
    }
}

fn percentage(value: i64, total: i64) -> f64 {
    if total == 0 {
        0.0
    } else {
        ((value as f64 / total as f64) * 1000.0).round() / 10.0
    }
}

pub fn ensure_non_empty_session(question_ids: &[String]) -> AppResult<()> {
    if question_ids.is_empty() {
        msg("No questions match the selected training filters")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::AnswerOption;

    #[test]
    fn records_mistake_and_marks_learned_after_three_correct_answers() {
        let db_path = std::env::temp_dir().join(format!(
            "exam-trainer-storage-test-{}.sqlite3",
            Uuid::new_v4()
        ));
        let db = Database::open(db_path.clone()).expect("database opens");
        let question = Question {
            id: "d1_single_001".to_string(),
            discipline: "Test discipline".to_string(),
            block_type: "closed".to_string(),
            question_type: "single_choice".to_string(),
            question: "Question?".to_string(),
            options: vec![
                AnswerOption {
                    key: "А".to_string(),
                    text: "Wrong".to_string(),
                },
                AnswerOption {
                    key: "Б".to_string(),
                    text: "Right".to_string(),
                },
            ],
            correct_answers: vec!["Б".to_string()],
            correct_text: None,
        };
        db.save_questions(&[question], "test-hash")
            .expect("question saved");

        let wrong = db
            .record_attempt("d1_single_001", &["А".to_string()], None, false)
            .expect("wrong attempt saved");
        assert_eq!(wrong.mistake_count, 1);
        assert_eq!(wrong.correct_streak, 0);
        assert_eq!(wrong.status, "learning");

        for expected_streak in 1..=3 {
            let result = db
                .record_attempt("d1_single_001", &["Б".to_string()], None, true)
                .expect("correct attempt saved");
            assert_eq!(result.correct_streak, expected_streak);
        }

        let mistakes = db
            .list_mistakes(MistakeFilter::default())
            .expect("mistakes listed");
        assert_eq!(mistakes.len(), 1);
        assert_eq!(mistakes[0].status, "learned");
        assert_eq!(mistakes[0].correct_streak, 3);

        let _ = std::fs::remove_file(db_path);
    }
}
