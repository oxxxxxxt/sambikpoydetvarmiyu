export function formatQuestionType(type: string) {
  switch (type) {
    case "single_choice":
      return "Один ответ";
    case "multiple_choice":
      return "Несколько ответов";
    case "open":
      return "Открытый";
    case "self_check":
      return "Самопроверка";
    default:
      return type;
  }
}

export function formatStatus(status: string) {
  return status === "learned" ? "Выучено" : "В изучении";
}
