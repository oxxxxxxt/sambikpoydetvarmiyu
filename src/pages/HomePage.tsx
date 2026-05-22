import { BarChart3, BookOpen, GraduationCap, Repeat, RotateCcw, Upload } from "lucide-react";

import { DisciplineFilter } from "../components/DisciplineFilter";
import { StatsCard } from "../components/StatsCard";
import type { BootstrapState, StatisticsSummary } from "../features/questions/questionTypes";

type HomePageProps = {
  bootstrap: BootstrapState | null;
  stats: StatisticsSummary | null;
  disciplines: string[];
  selectedDiscipline: string;
  onDisciplineChange: (value: string) => void;
  onStartAll: () => void;
  onStartMistakes: () => void;
  onStartDiscipline: () => void;
  onNavigate: (page: "mistakes" | "statistics" | "import") => void;
};

export function HomePage({
  bootstrap,
  stats,
  disciplines,
  selectedDiscipline,
  onDisciplineChange,
  onStartAll,
  onStartMistakes,
  onStartDiscipline,
  onNavigate,
}: HomePageProps) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <p className="eyebrow">Тренажер экзаменационных билетов</p>
          <h1>Быстрая тренировка и работа над ошибками</h1>
        </div>
      </div>

      <div className="stats-grid">
        <StatsCard label="Всего вопросов" value={bootstrap?.totalQuestions ?? 0} />
        <StatsCard label="Пройдено попыток" value={stats?.totalAttempts ?? 0} />
        <StatsCard label="Активных ошибок" value={stats?.activeMistakes ?? 0} tone="bad" />
        <StatsCard label="Процент правильных" value={`${stats?.accuracy ?? 0}%`} tone="good" />
      </div>

      <section className="action-panel">
        <div className="action-panel__main">
          <button className="big-action" type="button" onClick={onStartAll}>
            <BookOpen size={22} /> Начать все вопросы
          </button>
          <button className="big-action big-action--warn" type="button" onClick={onStartMistakes}>
            <Repeat size={22} /> Пройти только ошибки
          </button>
          <button className="big-action" type="button" onClick={onStartMistakes}>
            <RotateCcw size={22} /> Повторить сложные
          </button>
        </div>
        <div className="action-panel__side">
          <DisciplineFilter
            disciplines={disciplines}
            value={selectedDiscipline}
            onChange={onDisciplineChange}
          />
          <button className="button button--primary" type="button" disabled={!selectedDiscipline} onClick={onStartDiscipline}>
            <GraduationCap size={18} /> Выбрать дисциплину
          </button>
          <button className="button" type="button" onClick={() => onNavigate("statistics")}>
            <BarChart3 size={18} /> Статистика
          </button>
          <button className="button" type="button" onClick={() => onNavigate("mistakes")}>
            <Repeat size={18} /> Ошибки
          </button>
          <button className="button" type="button" onClick={() => onNavigate("import")}>
            <Upload size={18} /> Импортировать вопросы заново
          </button>
        </div>
      </section>

      {bootstrap?.import.warnings.length ? (
        <div className="warning-box">
          <strong>Предупреждения импорта</strong>
          {bootstrap.import.warnings.slice(0, 4).map((warning) => (
            <span key={warning}>{warning}</span>
          ))}
        </div>
      ) : null}
    </div>
  );
}
