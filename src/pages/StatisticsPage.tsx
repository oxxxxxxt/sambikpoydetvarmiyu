import { StatsCard } from "../components/StatsCard";
import type { StatisticsSummary } from "../features/questions/questionTypes";
import { truncateText } from "../utils/textCleaners";

type StatisticsPageProps = {
  stats: StatisticsSummary | null;
};

export function StatisticsPage({ stats }: StatisticsPageProps) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <p className="eyebrow">Статистика</p>
          <h1>Прогресс подготовки</h1>
        </div>
      </div>

      <div className="stats-grid">
        <StatsCard label="Всего вопросов" value={stats?.totalQuestions ?? 0} />
        <StatsCard label="Всего попыток" value={stats?.totalAttempts ?? 0} />
        <StatsCard label="Правильных" value={stats?.correctAnswers ?? 0} tone="good" />
        <StatsCard label="Неправильных" value={stats?.wrongAnswers ?? 0} tone="bad" />
        <StatsCard label="Процент правильных" value={`${stats?.accuracy ?? 0}%`} />
        <StatsCard label="Выучено из ошибок" value={stats?.learnedQuestions ?? 0} tone="good" />
      </div>

      <section className="section">
        <h2>По дисциплинам</h2>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Дисциплина</th>
                <th>Вопросы</th>
                <th>Попытки</th>
                <th>Верно</th>
                <th>Неверно</th>
                <th>%</th>
              </tr>
            </thead>
            <tbody>
              {(stats?.byDiscipline ?? []).map((row) => (
                <tr key={row.discipline}>
                  <td>{row.discipline}</td>
                  <td>{row.totalQuestions}</td>
                  <td>{row.attempts}</td>
                  <td>{row.correct}</td>
                  <td>{row.wrong}</td>
                  <td>{row.accuracy}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className="section">
        <h2>Топ-10 проблемных вопросов</h2>
        <div className="problem-list">
          {(stats?.topProblemQuestions ?? []).map((row) => (
            <div className="problem-item" key={row.questionId}>
              <strong>{truncateText(row.question, 140)}</strong>
              <span>{row.discipline} · ошибок: {row.mistakeCount}</span>
            </div>
          ))}
          {stats?.topProblemQuestions.length === 0 && <div className="empty-state">Проблемных вопросов пока нет.</div>}
        </div>
      </section>
    </div>
  );
}
