import type { MistakeRow } from "../features/questions/questionTypes";
import { formatQuestionType, formatStatus } from "../utils/normalizeAnswer";
import { truncateText } from "../utils/textCleaners";

type MistakesTableProps = {
  mistakes: MistakeRow[];
};

export function MistakesTable({ mistakes }: MistakesTableProps) {
  if (mistakes.length === 0) {
    return <div className="empty-state">Ошибок по выбранным фильтрам нет.</div>;
  }

  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>Вопрос</th>
            <th>Дисциплина</th>
            <th>Тип</th>
            <th>Ошибки</th>
            <th>Последняя ошибка</th>
            <th>Серия</th>
            <th>Статус</th>
          </tr>
        </thead>
        <tbody>
          {mistakes.map((mistake) => (
            <tr key={mistake.questionId}>
              <td>{truncateText(mistake.question, 110)}</td>
              <td>{mistake.discipline}</td>
              <td>{formatQuestionType(mistake.questionType)}</td>
              <td>{mistake.mistakeCount}</td>
              <td>{mistake.lastMistakeAt ? new Date(mistake.lastMistakeAt).toLocaleString("ru-RU") : "—"}</td>
              <td>{mistake.correctStreak}</td>
              <td>
                <span className={`status-pill status-pill--${mistake.status}`}>
                  {formatStatus(mistake.status)}
                </span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
