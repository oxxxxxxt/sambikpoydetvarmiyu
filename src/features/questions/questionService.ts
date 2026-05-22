import { invoke } from "@tauri-apps/api/core";
import type {
  AnswerResult,
  BootstrapState,
  ImportSummary,
  MistakeFilter,
  MistakeRow,
  Question,
  QuestionFilter,
  StatisticsSummary,
  SubmitAnswerRequest,
  TrainingRequest,
  TrainingSession,
  TrainingSummary,
} from "./questionTypes";

export const questionService = {
  getBootstrapState: () => invoke<BootstrapState>("get_bootstrap_state"),
  importSeedDocx: (force: boolean) => invoke<ImportSummary>("import_seed_docx", { force }),
  listQuestions: (filter?: QuestionFilter) =>
    invoke<Question[]>("list_questions", { filter: filter ?? null }),
  startTraining: (request: TrainingRequest) =>
    invoke<TrainingSession>("start_training", { request }),
  submitAnswer: (request: SubmitAnswerRequest) =>
    invoke<AnswerResult>("submit_answer", { request }),
  skipQuestion: (sessionId: string, questionId: string) =>
    invoke<void>("skip_question", { sessionId, questionId }),
  finishTraining: (sessionId: string) =>
    invoke<TrainingSummary>("finish_training", { sessionId }),
  listMistakes: (filter?: MistakeFilter) =>
    invoke<MistakeRow[]>("list_mistakes", { filter: filter ?? null }),
  getStatistics: () => invoke<StatisticsSummary>("get_statistics"),
};
