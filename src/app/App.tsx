import { useEffect, useMemo, useState } from "react";
import { BarChart3, Database, Home, Repeat, Settings, Upload } from "lucide-react";

import { questionService } from "../features/questions/questionService";
import type {
  AnswerResult,
  BootstrapState,
  ImportSummary,
  MistakeFilter,
  MistakeRow,
  Question,
  StatisticsSummary,
  SubmitAnswerRequest,
  TrainingRequest,
  TrainingSession,
} from "../features/questions/questionTypes";
import { HomePage } from "../pages/HomePage";
import { ImportPage } from "../pages/ImportPage";
import { MistakesPage } from "../pages/MistakesPage";
import { SettingsPage } from "../pages/SettingsPage";
import { StatisticsPage } from "../pages/StatisticsPage";
import { TrainingPage } from "../pages/TrainingPage";
import type { PageKey } from "./routes";
import { pageTitles } from "./routes";

const navItems: { key: PageKey; icon: typeof Home }[] = [
  { key: "home", icon: Home },
  { key: "mistakes", icon: Repeat },
  { key: "statistics", icon: BarChart3 },
  { key: "import", icon: Upload },
  { key: "settings", icon: Settings },
];

export function App() {
  const [page, setPage] = useState<PageKey>("home");
  const [bootstrap, setBootstrap] = useState<BootstrapState | null>(null);
  const [questions, setQuestions] = useState<Question[]>([]);
  const [stats, setStats] = useState<StatisticsSummary | null>(null);
  const [mistakes, setMistakes] = useState<MistakeRow[]>([]);
  const [mistakeFilter, setMistakeFilter] = useState<MistakeFilter>({ status: "all" });
  const [selectedDiscipline, setSelectedDiscipline] = useState("");
  const [session, setSession] = useState<TrainingSession | null>(null);
  const [answerResult, setAnswerResult] = useState<AnswerResult | null>(null);
  const [importSummary, setImportSummary] = useState<ImportSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [importing, setImporting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const disciplines = bootstrap?.disciplines ?? [];

  const currentQuestion = useMemo(() => {
    if (!session || session.currentIndex >= session.questionIds.length) {
      return null;
    }
    const id = session.questionIds[session.currentIndex];
    return questions.find((question) => question.id === id) ?? null;
  }, [questions, session]);

  async function refreshAll() {
    const boot = await questionService.getBootstrapState();
    const [questionList, statSnapshot, mistakeRows] = await Promise.all([
      questionService.listQuestions(),
      questionService.getStatistics(),
      questionService.listMistakes(mistakeFilter),
    ]);
    setBootstrap(boot);
    setImportSummary(boot.import);
    setQuestions(questionList);
    setStats(statSnapshot);
    setMistakes(mistakeRows);
  }

  useEffect(() => {
    refreshAll()
      .catch((reason: unknown) => setError(String(reason)))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    questionService
      .listMistakes(mistakeFilter)
      .then(setMistakes)
      .catch((reason: unknown) => setError(String(reason)));
  }, [mistakeFilter]);

  async function startTraining(request: TrainingRequest) {
    setError(null);
    const nextSession = await questionService.startTraining(request);
    setSession({ ...nextSession, currentIndex: 0 });
    setAnswerResult(null);
    setPage("training");
  }

  async function handleSubmit(payload: Omit<SubmitAnswerRequest, "questionId">) {
    if (!currentQuestion) {
      return;
    }
    setError(null);
    const result = await questionService.submitAnswer({
      ...payload,
      sessionId: session?.id ?? null,
      questionId: currentQuestion.id,
    });
    setAnswerResult(result);
    const [statSnapshot, mistakeRows] = await Promise.all([
      questionService.getStatistics(),
      questionService.listMistakes(mistakeFilter),
    ]);
    setStats(statSnapshot);
    setMistakes(mistakeRows);
  }

  async function handleNext() {
    setAnswerResult(null);
    setSession((previous) => {
      if (!previous) {
        return previous;
      }
      const nextIndex = previous.currentIndex + 1;
      if (nextIndex >= previous.total) {
        setPage("statistics");
        return { ...previous, currentIndex: nextIndex, finished: true };
      }
      return { ...previous, currentIndex: nextIndex };
    });
    await refreshAll();
  }

  async function handleSkip() {
    if (!session || !currentQuestion) {
      return;
    }
    await questionService.skipQuestion(session.id, currentQuestion.id);
    setAnswerResult(null);
    setSession((previous) => {
      if (!previous) {
        return previous;
      }
      const nextIndex = previous.currentIndex + 1;
      if (nextIndex >= previous.total) {
        setPage("statistics");
        return { ...previous, currentIndex: nextIndex, finished: true };
      }
      return { ...previous, currentIndex: nextIndex };
    });
  }

  async function handleFinish() {
    if (session) {
      await questionService.finishTraining(session.id).catch(() => undefined);
    }
    setAnswerResult(null);
    setSession(null);
    await refreshAll();
    setPage("statistics");
  }

  async function handleReimport() {
    setImporting(true);
    setError(null);
    try {
      const summary = await questionService.importSeedDocx(true);
      setImportSummary(summary);
      await refreshAll();
    } catch (reason) {
      setError(String(reason));
    } finally {
      setImporting(false);
    }
  }

  const content = (() => {
    if (loading) {
      return <div className="loading">Загрузка и автоматический импорт вопросов...</div>;
    }

    switch (page) {
      case "home":
        return (
          <HomePage
            bootstrap={bootstrap}
            stats={stats}
            disciplines={disciplines}
            selectedDiscipline={selectedDiscipline}
            onDisciplineChange={setSelectedDiscipline}
            onStartAll={() => startTraining({ mode: "all" }).catch((reason) => setError(String(reason)))}
            onStartMistakes={() => startTraining({ mode: "mistakes" }).catch((reason) => setError(String(reason)))}
            onStartDiscipline={() =>
              startTraining({ mode: "discipline", discipline: selectedDiscipline }).catch((reason) => setError(String(reason)))
            }
            onNavigate={(nextPage) => setPage(nextPage)}
          />
        );
      case "training":
        return (
          <TrainingPage
            session={session}
            question={currentQuestion}
            result={answerResult}
            onSubmit={handleSubmit}
            onNext={() => void handleNext()}
            onSkip={() => void handleSkip()}
            onFinish={() => void handleFinish()}
          />
        );
      case "mistakes":
        return (
          <MistakesPage
            mistakes={mistakes}
            disciplines={disciplines}
            filter={mistakeFilter}
            onFilterChange={setMistakeFilter}
            onStartMistakes={() =>
              startTraining({
                mode: "mistakes",
                discipline: mistakeFilter.discipline || null,
                questionType: mistakeFilter.questionType || null,
              }).catch((reason) => setError(String(reason)))
            }
          />
        );
      case "statistics":
        return <StatisticsPage stats={stats} />;
      case "import":
        return <ImportPage summary={importSummary} importing={importing} onReimport={handleReimport} />;
      case "settings":
        return <SettingsPage bootstrap={bootstrap} />;
      default:
        return null;
    }
  })();

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <Database size={26} />
          <div>
            <strong>Билеты</strong>
            <span>Windows trainer</span>
          </div>
        </div>
        <nav>
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <button
                key={item.key}
                className={page === item.key ? "nav-item nav-item--active" : "nav-item"}
                type="button"
                onClick={() => setPage(item.key)}
              >
                <Icon size={18} /> {pageTitles[item.key]}
              </button>
            );
          })}
        </nav>
      </aside>
      <main>
        {error && (
          <div className="error-banner">
            <strong>Ошибка</strong>
            <span>{error}</span>
            <button type="button" onClick={() => setError(null)}>Закрыть</button>
          </div>
        )}
        {content}
      </main>
    </div>
  );
}
