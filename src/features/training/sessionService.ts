import type { TrainingSession } from "../questions/questionTypes";

export function currentQuestionId(session: TrainingSession | null) {
  if (!session || session.currentIndex >= session.questionIds.length) {
    return null;
  }
  return session.questionIds[session.currentIndex];
}
