use std::{collections::HashSet, fs::File, io::Read, path::PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
use uuid::Uuid;

use crate::{
    domain::{
        BootstrapState, ImportSummary, MistakeFilter, MistakeRow, Question, QuestionFilter,
        StatisticsSummary, SubmitAnswerRequest, TrainingRequest, TrainingSession, TrainingSummary,
    },
    error::{msg, AppError, AppResult},
    import::docx::parser::parse_docx,
    storage::db::ensure_non_empty_session,
    AppState,
};

const SEED_DOCX: &str = "fos_gia_vo_bak_09_03_02_rsob.docx";

#[tauri::command]
pub fn get_bootstrap_state(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<BootstrapState> {
    let import = import_seed_docx_impl(&app, &state, false)?;
    Ok(BootstrapState {
        db_path: state.db.path().display().to_string(),
        total_questions: state.db.total_questions()?,
        total_attempts: state.db.total_attempts()?,
        active_mistakes: state.db.active_mistakes()?,
        disciplines: state.db.disciplines()?,
        import,
    })
}

#[tauri::command]
pub fn import_seed_docx(
    force: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<ImportSummary> {
    import_seed_docx_impl(&app, &state, force)
}

#[tauri::command]
pub fn list_questions(
    filter: Option<QuestionFilter>,
    state: State<'_, AppState>,
) -> AppResult<Vec<Question>> {
    state.db.list_questions(filter.unwrap_or_default())
}

#[tauri::command]
pub fn start_training(
    request: TrainingRequest,
    state: State<'_, AppState>,
) -> AppResult<TrainingSession> {
    let mut filter = QuestionFilter {
        discipline: request.discipline.clone(),
        question_type: request.question_type.clone(),
        only_mistakes: Some(matches!(
            request.mode.as_str(),
            "mistakes" | "errors" | "repeat_mistakes"
        )),
        mistake_status: Some("learning".to_string()),
    };

    if request.mode == "all" {
        filter.only_mistakes = Some(false);
    }

    let mut questions = state.db.list_questions(filter)?;
    if let Some(limit) = request.limit {
        questions.truncate(limit);
    }
    let question_ids = questions
        .into_iter()
        .map(|question| question.id)
        .collect::<Vec<_>>();
    ensure_non_empty_session(&question_ids)?;

    let session = TrainingSession {
        id: Uuid::new_v4().to_string(),
        mode: request.mode,
        total: question_ids.len(),
        question_ids,
        current_index: 0,
        answered: 0,
        correct: 0,
        wrong: 0,
        started_at: Utc::now().to_rfc3339(),
        finished: false,
    };
    state
        .sessions
        .lock()
        .expect("sessions mutex poisoned")
        .insert(session.id.clone(), session.clone());
    Ok(session)
}

#[tauri::command]
pub fn submit_answer(
    request: SubmitAnswerRequest,
    state: State<'_, AppState>,
) -> AppResult<crate::domain::AnswerResult> {
    let question = state.db.get_question(&request.question_id)?;
    let is_correct = check_answer(
        &question,
        &request.selected_answers,
        request.self_mark_correct,
    );
    let result = state.db.record_attempt(
        &request.question_id,
        &request.selected_answers,
        request.user_text_answer,
        is_correct,
    )?;

    if let Some(session_id) = request.session_id {
        if let Some(session) = state
            .sessions
            .lock()
            .expect("sessions mutex poisoned")
            .get_mut(&session_id)
        {
            session.answered += 1;
            if is_correct {
                session.correct += 1;
            } else {
                session.wrong += 1;
            }
            session.current_index = (session.current_index + 1).min(session.total);
            session.finished = session.current_index >= session.total;
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn skip_question(
    session_id: String,
    _question_id: String,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut sessions = state.sessions.lock().expect("sessions mutex poisoned");
    let Some(session) = sessions.get_mut(&session_id) else {
        return msg(format!("Training session {session_id} not found"));
    };
    session.current_index = (session.current_index + 1).min(session.total);
    session.finished = session.current_index >= session.total;
    Ok(())
}

#[tauri::command]
pub fn finish_training(
    session_id: String,
    state: State<'_, AppState>,
) -> AppResult<TrainingSummary> {
    let mut sessions = state.sessions.lock().expect("sessions mutex poisoned");
    let Some(session) = sessions.get_mut(&session_id) else {
        return msg(format!("Training session {session_id} not found"));
    };
    session.finished = true;
    Ok(TrainingSummary {
        session_id,
        total: session.total,
        answered: session.answered,
        correct: session.correct,
        wrong: session.wrong,
    })
}

#[tauri::command]
pub fn list_mistakes(
    filter: Option<MistakeFilter>,
    state: State<'_, AppState>,
) -> AppResult<Vec<MistakeRow>> {
    state.db.list_mistakes(filter.unwrap_or_default())
}

#[tauri::command]
pub fn get_statistics(state: State<'_, AppState>) -> AppResult<StatisticsSummary> {
    state.db.statistics()
}

fn import_seed_docx_impl(
    app: &AppHandle,
    state: &State<'_, AppState>,
    force: bool,
) -> AppResult<ImportSummary> {
    let source_path = locate_seed_docx(app)?;
    let source_hash = sha256_file(&source_path)?;
    let existing_hash = state.db.get_setting("imported_source_hash")?;
    if !force
        && existing_hash.as_deref() == Some(source_hash.as_str())
        && state.db.total_questions()? > 0
    {
        let questions = state.db.list_questions(QuestionFilter::default())?;
        return Ok(summary_from_questions(
            source_path,
            source_hash,
            &questions,
            Vec::new(),
            true,
        ));
    }

    let parsed = parse_docx(&source_path)?;
    if parsed.questions.is_empty() {
        return Err(AppError::Message(format!(
            "No questions were parsed from {}",
            source_path.display()
        )));
    }
    state.db.save_questions(&parsed.questions, &source_hash)?;
    Ok(summary_from_questions(
        source_path,
        source_hash,
        &parsed.questions,
        parsed.warnings,
        false,
    ))
}

fn summary_from_questions(
    source_path: PathBuf,
    source_hash: String,
    questions: &[Question],
    mut warnings: Vec<String>,
    reused: bool,
) -> ImportSummary {
    if questions.len() != 450 {
        warnings.push(format!(
            "Expected 450 questions from the seed document, got {}",
            questions.len()
        ));
    }

    let disciplines = questions
        .iter()
        .map(|question| question.discipline.clone())
        .collect::<HashSet<_>>()
        .len();
    ImportSummary {
        source_path: source_path.display().to_string(),
        source_hash,
        imported_questions: questions.len(),
        disciplines,
        single_choice: questions
            .iter()
            .filter(|question| question.question_type == "single_choice")
            .count(),
        multiple_choice: questions
            .iter()
            .filter(|question| question.question_type == "multiple_choice")
            .count(),
        open: questions
            .iter()
            .filter(|question| question.question_type == "open")
            .count(),
        self_check: questions
            .iter()
            .filter(|question| question.question_type == "self_check")
            .count(),
        reused,
        warnings,
    }
}

fn locate_seed_docx(app: &AppHandle) -> AppResult<PathBuf> {
    let candidates = [
        std::env::current_dir()
            .ok()
            .map(|path| path.join(SEED_DOCX)),
        std::env::current_dir()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.join(SEED_DOCX))),
        Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .join(SEED_DOCX),
        ),
        app.path().resolve(SEED_DOCX, BaseDirectory::Resource).ok(),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|path| path.exists())
        .ok_or_else(|| {
            AppError::Message(format!(
                "Could not find {SEED_DOCX} in the project root or bundled resources"
            ))
        })
}

fn sha256_file(path: &PathBuf) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn check_answer(
    question: &Question,
    selected_answers: &[String],
    self_mark_correct: Option<bool>,
) -> bool {
    match question.question_type.as_str() {
        "single_choice" => {
            selected_answers.len() == 1
                && question.correct_answers.len() == 1
                && normalize_key(&selected_answers[0])
                    == normalize_key(&question.correct_answers[0])
        }
        "multiple_choice" => {
            let mut selected = selected_answers
                .iter()
                .map(|answer| normalize_key(answer))
                .collect::<Vec<_>>();
            let mut correct = question
                .correct_answers
                .iter()
                .map(|answer| normalize_key(answer))
                .collect::<Vec<_>>();
            selected.sort();
            selected.dedup();
            correct.sort();
            correct.dedup();
            !selected.is_empty() && selected == correct
        }
        "open" | "self_check" => self_mark_correct.unwrap_or(false),
        _ => false,
    }
}

fn normalize_key(value: &str) -> String {
    match value.trim().chars().next().unwrap_or('?') {
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
