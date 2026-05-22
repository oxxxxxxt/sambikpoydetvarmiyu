import { ProgressBar } from "../components/ProgressBar";
import { QuestionCard } from "../components/QuestionCard";
import type {
  AnswerResult,
  Question,
  SubmitAnswerRequest,
  TrainingSession,
} from "../features/questions/questionTypes";

type TrainingPageProps = {
  session: TrainingSession | null;
  question: Question | null;
  result: AnswerResult | null;
  onSubmit: (request: Omit<SubmitAnswerRequest, "questionId">) => Promise<void>;
  onNext: () => void;
  onSkip: () => void;
  onFinish: () => void;
};

export function TrainingPage({
  session,
  question,
  result,
  onSubmit,
  onNext,
  onSkip,
  onFinish,
}: TrainingPageProps) {
  if (!session || !question) {
    return (
      <div className="page">
        <div className="empty-state">Тренировка не запущена.</div>
      </div>
    );
  }

  return (
    <div className="page page--training">
      <ProgressBar value={Math.min(session.currentIndex + 1, session.total)} max={session.total} />
      <QuestionCard
        question={question}
        index={session.currentIndex}
        total={session.total}
        result={result}
        onSubmit={onSubmit}
        onNext={onNext}
        onSkip={onSkip}
        onFinish={onFinish}
      />
    </div>
  );
}
