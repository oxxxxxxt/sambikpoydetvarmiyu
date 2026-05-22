import { Repeat } from "lucide-react";

import { DisciplineFilter } from "../components/DisciplineFilter";
import { MistakesTable } from "../components/MistakesTable";
import type { MistakeFilter, MistakeRow, QuestionType } from "../features/questions/questionTypes";

type MistakesPageProps = {
  mistakes: MistakeRow[];
  disciplines: string[];
  filter: MistakeFilter;
  onFilterChange: (filter: MistakeFilter) => void;
  onStartMistakes: () => void;
};

export function MistakesPage({
  mistakes,
  disciplines,
  filter,
  onFilterChange,
  onStartMistakes,
}: MistakesPageProps) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <p className="eyebrow">Ошибки</p>
          <h1>Проблемные вопросы</h1>
        </div>
        <button className="button button--primary" type="button" onClick={onStartMistakes}>
          <Repeat size={18} /> Пройти только ошибки
        </button>
      </div>

      <div className="filters">
        <label className="field">
          <span>Статус</span>
          <select
            value={filter.status ?? "all"}
            onChange={(event) => onFilterChange({ ...filter, status: event.target.value as MistakeFilter["status"] })}
          >
            <option value="all">Все ошибки</option>
            <option value="learning">Не выучено</option>
            <option value="learned">Выучено</option>
          </select>
        </label>
        <DisciplineFilter
          disciplines={disciplines}
          value={filter.discipline ?? ""}
          onChange={(discipline) => onFilterChange({ ...filter, discipline: discipline || null })}
        />
        <label className="field">
          <span>Тип вопроса</span>
          <select
            value={filter.questionType ?? ""}
            onChange={(event) =>
              onFilterChange({
                ...filter,
                questionType: event.target.value ? (event.target.value as QuestionType) : null,
              })
            }
          >
            <option value="">Все типы</option>
            <option value="single_choice">Один ответ</option>
            <option value="multiple_choice">Несколько ответов</option>
            <option value="open">Открытый</option>
            <option value="self_check">Самопроверка</option>
          </select>
        </label>
      </div>

      <MistakesTable mistakes={mistakes} />
    </div>
  );
}
