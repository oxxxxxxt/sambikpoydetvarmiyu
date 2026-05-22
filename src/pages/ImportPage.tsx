import { RefreshCw } from "lucide-react";

import { StatsCard } from "../components/StatsCard";
import type { ImportSummary } from "../features/questions/questionTypes";

type ImportPageProps = {
  summary: ImportSummary | null;
  importing: boolean;
  onReimport: () => void;
};

export function ImportPage({ summary, importing, onReimport }: ImportPageProps) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <p className="eyebrow">Импорт</p>
          <h1>Автоматический импорт Word-файла</h1>
        </div>
        <button className="button button--primary" type="button" disabled={importing} onClick={onReimport}>
          <RefreshCw size={18} /> {importing ? "Импорт..." : "Импортировать заново"}
        </button>
      </div>

      <div className="stats-grid">
        <StatsCard label="Импортировано" value={summary?.importedQuestions ?? 0} />
        <StatsCard label="Дисциплины" value={summary?.disciplines ?? 0} />
        <StatsCard label="Один ответ" value={summary?.singleChoice ?? 0} />
        <StatsCard label="Несколько ответов" value={summary?.multipleChoice ?? 0} />
        <StatsCard label="Самопроверка" value={summary?.selfCheck ?? 0} />
        <StatsCard label="Открытые" value={summary?.open ?? 0} />
      </div>

      <section className="section">
        <h2>Источник</h2>
        <div className="source-box">
          <span>{summary?.sourcePath || "Файл еще не импортирован"}</span>
          {summary?.sourceHash && <code>{summary.sourceHash}</code>}
        </div>
      </section>

      {summary?.warnings.length ? (
        <section className="section">
          <h2>Предупреждения</h2>
          <div className="warning-list">
            {summary.warnings.map((warning) => (
              <span key={warning}>{warning}</span>
            ))}
          </div>
        </section>
      ) : null}
    </div>
  );
}
