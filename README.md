# Exam Ticket Trainer

Windows desktop-приложение для заучивания экзаменационных билетов из `.docx`.

## Запуск

```powershell
npm install
npm run tauri dev
```

При первом запуске приложение автоматически ищет `fos_gia_vo_bak_09_03_02_rsob.docx`, импортирует вопросы и создает SQLite-базу в:

```text
%LOCALAPPDATA%\com.local.exam-ticket-trainer\exam-trainer.sqlite3
```

## Проверки

```powershell
npm run lint
npm test
npm run build
```

## Windows-сборка

```powershell
npm run build:win
```

Готовые артефакты:

```text
src-tauri\target\release\bundle\msi\
src-tauri\target\release\bundle\nsis\
```
