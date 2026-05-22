import type { ImportSummary } from "./questionTypes";

export function describeImport(summary: ImportSummary) {
  return `${summary.importedQuestions} вопросов, ${summary.disciplines} дисциплин`;
}
