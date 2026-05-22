use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerOption {
    pub key: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub id: String,
    pub discipline: String,
    pub block_type: String,
    pub question_type: String,
    pub question: String,
    pub options: Vec<AnswerOption>,
    pub correct_answers: Vec<String>,
    pub correct_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct QuestionFilter {
    pub discipline: Option<String>,
    pub question_type: Option<String>,
    pub only_mistakes: Option<bool>,
    pub mistake_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapState {
    pub db_path: String,
    pub total_questions: i64,
    pub total_attempts: i64,
    pub active_mistakes: i64,
    pub disciplines: Vec<String>,
    pub import: ImportSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSummary {
    pub source_path: String,
    pub source_hash: String,
    pub imported_questions: usize,
    pub disciplines: usize,
    pub single_choice: usize,
    pub multiple_choice: usize,
    pub open: usize,
    pub self_check: usize,
    pub reused: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingRequest {
    pub mode: String,
    pub discipline: Option<String>,
    pub question_type: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingSession {
    pub id: String,
    pub mode: String,
    pub question_ids: Vec<String>,
    pub current_index: usize,
    pub total: usize,
    pub answered: i64,
    pub correct: i64,
    pub wrong: i64,
    pub started_at: String,
    pub finished: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAnswerRequest {
    pub session_id: Option<String>,
    pub question_id: String,
    pub selected_answers: Vec<String>,
    pub user_text_answer: Option<String>,
    pub self_mark_correct: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerResult {
    pub is_correct: bool,
    pub question: Question,
    pub correct_streak: i64,
    pub mistake_count: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MistakeFilter {
    pub status: Option<String>,
    pub discipline: Option<String>,
    pub question_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MistakeRow {
    pub question_id: String,
    pub question: String,
    pub discipline: String,
    pub question_type: String,
    pub mistake_count: i64,
    pub last_mistake_at: Option<String>,
    pub correct_streak: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingSummary {
    pub session_id: String,
    pub total: usize,
    pub answered: i64,
    pub correct: i64,
    pub wrong: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsSummary {
    pub total_questions: i64,
    pub total_attempts: i64,
    pub correct_answers: i64,
    pub wrong_answers: i64,
    pub accuracy: f64,
    pub learned_questions: i64,
    pub active_mistakes: i64,
    pub by_discipline: Vec<DisciplineStats>,
    pub top_problem_questions: Vec<MistakeRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisciplineStats {
    pub discipline: String,
    pub total_questions: i64,
    pub attempts: i64,
    pub correct: i64,
    pub wrong: i64,
    pub accuracy: f64,
}
