import { Check, ChevronRight, LogOut, SkipForward } from "lucide-react";

type TrainingControlsProps = {
  canSubmit: boolean;
  checked: boolean;
  onSubmit: () => void;
  onNext: () => void;
  onSkip: () => void;
  onFinish: () => void;
};

export function TrainingControls({
  canSubmit,
  checked,
  onSubmit,
  onNext,
  onSkip,
  onFinish,
}: TrainingControlsProps) {
  return (
    <div className="training-controls">
      {!checked ? (
        <button className="button button--primary" type="button" disabled={!canSubmit} onClick={onSubmit}>
          <Check size={18} /> Ответить
        </button>
      ) : (
        <button className="button button--primary" type="button" onClick={onNext}>
          <ChevronRight size={18} /> Следующий вопрос
        </button>
      )}
      <button className="button" type="button" disabled={checked} onClick={onSkip}>
        <SkipForward size={18} /> Пропустить
      </button>
      <button className="button button--ghost" type="button" onClick={onFinish}>
        <LogOut size={18} /> Завершить тренировку
      </button>
    </div>
  );
}
