type DisciplineFilterProps = {
  disciplines: string[];
  value: string;
  onChange: (value: string) => void;
  includeAll?: boolean;
};

export function DisciplineFilter({
  disciplines,
  value,
  onChange,
  includeAll = true,
}: DisciplineFilterProps) {
  return (
    <label className="field">
      <span>Дисциплина</span>
      <select value={value} onChange={(event) => onChange(event.target.value)}>
        {includeAll && <option value="">Все дисциплины</option>}
        {disciplines.map((discipline) => (
          <option key={discipline} value={discipline}>
            {discipline}
          </option>
        ))}
      </select>
    </label>
  );
}
