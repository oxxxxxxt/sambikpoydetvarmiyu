import type { AnswerOption as AnswerOptionModel } from "../features/questions/questionTypes";

type AnswerOptionProps = {
  option: AnswerOptionModel;
  selected: boolean;
  disabled: boolean;
  resultState?: "correct" | "wrong" | "missed";
  mode: "single" | "multiple";
  onToggle: (key: string) => void;
};

export function AnswerOption({
  option,
  selected,
  disabled,
  resultState,
  mode,
  onToggle,
}: AnswerOptionProps) {
  const className = [
    "answer-option",
    selected ? "answer-option--selected" : "",
    resultState ? `answer-option--${resultState}` : "",
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <button
      type="button"
      className={className}
      disabled={disabled}
      aria-pressed={selected}
      onClick={() => onToggle(option.key)}
    >
      <span className="answer-option__key">{mode === "multiple" && selected ? "✓ " : ""}{option.key}</span>
      <span className="answer-option__text">{option.text}</span>
    </button>
  );
}
