import { invoke, isTauri, type InvokeArgs } from "@tauri-apps/api/core";
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

const TAURI_RUNTIME_ERROR =
  "The app is running outside the Tauri shell. Use npm run dev or npm run tauri dev.";

function invokeCommand<T>(command: string, args?: InvokeArgs) {
  if (!isTauri()) {
    return Promise.reject(new Error(TAURI_RUNTIME_ERROR));
  }

  return invoke<T>(command, args);
}

export const questionService = {
  getBootstrapState: () => invokeCommand<BootstrapState>("get_bootstrap_state"),
  importSeedDocx: (force: boolean) => invokeCommand<ImportSummary>("import_seed_docx", { force }),
  listQuestions: (filter?: QuestionFilter) =>
    invokeCommand<Question[]>("list_questions", { filter: filter ?? null }),
  startTraining: (request: TrainingRequest) =>
    invokeCommand<TrainingSession>("start_training", { request }),
  submitAnswer: (request: SubmitAnswerRequest) =>
    invokeCommand<AnswerResult>("submit_answer", { request }),
  skipQuestion: (sessionId: string, questionId: string) =>
    invokeCommand<void>("skip_question", { sessionId, questionId }),
  finishTraining: (sessionId: string) =>
    invokeCommand<TrainingSummary>("finish_training", { sessionId }),
  listMistakes: (filter?: MistakeFilter) =>
    invokeCommand<MistakeRow[]>("list_mistakes", { filter: filter ?? null }),
  getStatistics: () => invokeCommand<StatisticsSummary>("get_statistics"),
};
