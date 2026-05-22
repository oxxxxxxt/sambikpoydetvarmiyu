import { useEffect, useMemo, useState } from "react";
import { Eye, ThumbsDown, ThumbsUp } from "lucide-react";

import type {
  AnswerResult,
  Question,
  SubmitAnswerRequest,
} from "../features/questions/questionTypes";
import { formatQuestionType } from "../utils/normalizeAnswer";
import { AnswerOption } from "./AnswerOption";
import { TrainingControls } from "./TrainingControls";

type QuestionCardProps = {
  question: Question;
  index: number;
  total: number;
  result: AnswerResult | null;
  onSubmit: (request: Omit<SubmitAnswerRequest, "questionId">) => Promise<void>;
  onNext: () => void;
  onSkip: () => void;
  onFinish: () => void;
};

export function QuestionCard({
  question,
  index,
  total,
  result,
  onSubmit,
  onNext,
  onSkip,
  onFinish,
}: QuestionCardProps) {
  const [selected, setSelected] = useState<string[]>([]);
  const [textAnswer, setTextAnswer] = useState("");
  const [revealed, setRevealed] = useState(false);
  const checked = Boolean(result);

  useEffect(() => {
    setSelected([]);
    setTextAnswer("");
    setRevealed(false);
  }, [question.id]);

  const correctSet = useMemo(
    () => new Set(result?.question.correctAnswers ?? question.correctAnswers),
    [question.correctAnswers, result],
  );

  function toggleOption(key: string) {
    if (checked) {
      return;
    }
    if (question.questionType === "single_choice") {
      setSelected([key]);
      return;
    }
    setSelected((previous) =>
      previous.includes(key)
        ? previous.filter((value) => value !== key)
        : [...previous, key],
    );
  }

  function optionResultState(key: string) {
    if (!checked || !["single_choice", "multiple_choice"].includes(question.questionType)) {
      return undefined;
    }
    if (correctSet.has(key)) {
      return selected.includes(key) ? "correct" : "missed";
    }
    return selected.includes(key) ? "wrong" : undefined;
  }

  const isChoice = question.questionType === "single_choice" || question.questionType === "multiple_choice";
  const isSelfChecked = question.questionType === "open" || question.questionType === "self_check";
  const canSubmit = isChoice ? selected.length > 0 : revealed;

  return (
    <section className="question-card">
      <div className="question-card__meta">
        <span>{question.discipline}</span>
        <span>{formatQuestionType(question.questionType)}</span>
        <span>Вопрос {index + 1} из {total}</span>
      </div>

      <h2>{question.question}</h2>

      {isChoice && (
        <div className="answer-list">
          {question.options.map((option) => (
            <AnswerOption
              key={option.key}
              option={option}
              selected={selected.includes(option.key)}
              disabled={checked}
              mode={question.questionType === "single_choice" ? "single" : "multiple"}
              resultState={optionResultState(option.key)}
              onToggle={toggleOption}
            />
          ))}
        </div>
      )}

      {question.questionType === "open" && (
        <div className="open-answer">
          <label className="field">
            <span>Ваш ответ</span>
            <textarea
              value={textAnswer}
              disabled={checked}
              onChange={(event) => setTextAnswer(event.target.value)}
              rows={5}
              placeholder="Введите ответ для самопроверки"
            />
          </label>
        </div>
      )}

      {question.questionType === "self_check" && question.options.length > 0 && (
        <div className="self-check-items">
          {question.options.map((option) => (
            <div className="self-check-item" key={`${option.key}-${option.text}`}>
              {option.text}
            </div>
          ))}
        </div>
      )}

      {isSelfChecked && !revealed && !checked && (
        <button className="button button--primary" type="button" onClick={() => setRevealed(true)}>
          <Eye size={18} /> Показать правильный ответ
        </button>
      )}

      {(revealed || checked) && (question.correctText || question.correctAnswers.length > 0) && (
        <div className="correct-answer">
          <span>Правильный ответ</span>
          <strong>{question.correctText ?? question.correctAnswers.join(", ")}</strong>
        </div>
      )}

      {isSelfChecked && revealed && !checked && (
        <div className="self-grade">
          <button
            className="button button--success"
            type="button"
            onClick={() => onSubmit({ selectedAnswers: [], userTextAnswer: textAnswer, selfMarkCorrect: true })}
          >
            <ThumbsUp size={18} /> Я ответил верно
          </button>
          <button
            className="button button--danger"
            type="button"
            onClick={() => onSubmit({ selectedAnswers: [], userTextAnswer: textAnswer, selfMarkCorrect: false })}
          >
            <ThumbsDown size={18} /> Я ответил неверно
          </button>
        </div>
      )}

      {checked && (
        <div className={`answer-feedback ${result?.isCorrect ? "answer-feedback--correct" : "answer-feedback--wrong"}`}>
          <strong>{result?.isCorrect ? "Верно" : "Неверно"}</strong>
          <span>
            Ошибок: {result?.mistakeCount ?? 0}. Серия правильных: {result?.correctStreak ?? 0}.
          </span>
        </div>
      )}

      {isChoice && (
        <TrainingControls
          canSubmit={canSubmit}
          checked={checked}
          onSubmit={() => onSubmit({ selectedAnswers: selected, userTextAnswer: null, selfMarkCorrect: null })}
          onNext={onNext}
          onSkip={onSkip}
          onFinish={onFinish}
        />
      )}

      {isSelfChecked && checked && (
        <TrainingControls
          canSubmit={false}
          checked={checked}
          onSubmit={() => undefined}
          onNext={onNext}
          onSkip={onSkip}
          onFinish={onFinish}
        />
      )}
    </section>
  );
}
