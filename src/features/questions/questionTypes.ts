export type QuestionType = "single_choice" | "multiple_choice" | "open" | "self_check";
export type BlockType = "closed" | "open" | "self_check";

export type AnswerOption = {
  key: string;
  text: string;
};

export type Question = {
  id: string;
  discipline: string;
  blockType: BlockType;
  questionType: QuestionType;
  question: string;
  options: AnswerOption[];
  correctAnswers: string[];
  correctText: string | null;
};

export type QuestionFilter = {
  discipline?: string | null;
  questionType?: QuestionType | null;
  onlyMistakes?: boolean;
  mistakeStatus?: "all" | "learning" | "learned" | null;
};

export type ImportSummary = {
  sourcePath: string;
  sourceHash: string;
  importedQuestions: number;
  disciplines: number;
  singleChoice: number;
  multipleChoice: number;
  open: number;
  selfCheck: number;
  reused: boolean;
  warnings: string[];
};

export type BootstrapState = {
  dbPath: string;
  totalQuestions: number;
  totalAttempts: number;
  activeMistakes: number;
  disciplines: string[];
  import: ImportSummary;
};

export type TrainingRequest = {
  mode: "all" | "discipline" | "mistakes";
  discipline?: string | null;
  questionType?: QuestionType | null;
  limit?: number | null;
};

export type TrainingSession = {
  id: string;
  mode: string;
  questionIds: string[];
  currentIndex: number;
  total: number;
  answered: number;
  correct: number;
  wrong: number;
  startedAt: string;
  finished: boolean;
};

export type SubmitAnswerRequest = {
  sessionId?: string | null;
  questionId: string;
  selectedAnswers: string[];
  userTextAnswer?: string | null;
  selfMarkCorrect?: boolean | null;
};

export type AnswerResult = {
  isCorrect: boolean;
  question: Question;
  correctStreak: number;
  mistakeCount: number;
  status: "learning" | "learned";
};

export type MistakeFilter = {
  status?: "all" | "learning" | "learned" | null;
  discipline?: string | null;
  questionType?: QuestionType | null;
};

export type MistakeRow = {
  questionId: string;
  question: string;
  discipline: string;
  questionType: QuestionType;
  mistakeCount: number;
  lastMistakeAt: string | null;
  correctStreak: number;
  status: "learning" | "learned";
};

export type TrainingSummary = {
  sessionId: string;
  total: number;
  answered: number;
  correct: number;
  wrong: number;
};

export type DisciplineStats = {
  discipline: string;
  totalQuestions: number;
  attempts: number;
  correct: number;
  wrong: number;
  accuracy: number;
};

export type StatisticsSummary = {
  totalQuestions: number;
  totalAttempts: number;
  correctAnswers: number;
  wrongAnswers: number;
  accuracy: number;
  learnedQuestions: number;
  activeMistakes: number;
  byDiscipline: DisciplineStats[];
  topProblemQuestions: MistakeRow[];
};
