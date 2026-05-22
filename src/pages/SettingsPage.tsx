import type { BootstrapState } from "../features/questions/questionTypes";

type SettingsPageProps = {
  bootstrap: BootstrapState | null;
};

export function SettingsPage({ bootstrap }: SettingsPageProps) {
  return (
    <div className="page">
      <div className="page-header">
        <div>
          <p className="eyebrow">Настройки</p>
          <h1>Локальные данные</h1>
        </div>
      </div>

      <section className="section">
        <h2>Хранилище</h2>
        <div className="source-box">
          <span>SQLite база</span>
          <code>{bootstrap?.dbPath ?? "Путь пока не получен"}</code>
        </div>
      </section>

      <section className="section">
        <h2>Правила обучения</h2>
        <div className="settings-list">
          <span>Закрытые вопросы проверяются автоматически по выбранным карточкам.</span>
          <span>Открытые вопросы и соответствие работают через самопроверку.</span>
          <span>После трех правильных ответов подряд вопрос считается выученным.</span>
        </div>
      </section>
    </div>
  );
}
