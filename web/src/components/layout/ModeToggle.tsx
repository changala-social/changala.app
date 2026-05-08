import { useMode } from '../../context/ModeContext';
import type { Mode } from '../../generated/types';

const modes: { value: Mode; label: string }[] = [
  { value: 'academic', label: 'Academic' },
  { value: 'brain', label: 'Brain' },
  { value: 'all', label: 'All' },
];

export function ModeToggle() {
  const { mode, setMode } = useMode();

  return (
    <div className="flex bg-surface-alt rounded-lg p-0.5 border border-border">
      {modes.map((m) => (
        <button
          key={m.value}
          onClick={() => setMode(m.value)}
          className={`px-3 py-1 text-xs font-medium rounded-md transition-all ${
            mode === m.value
              ? m.value === 'academic'
                ? 'bg-academic text-white shadow-sm'
                : m.value === 'brain'
                ? 'bg-brain text-white shadow-sm'
                : 'bg-text text-surface shadow-sm'
              : 'text-text-secondary hover:text-text'
          }`}
        >
          {m.label}
        </button>
      ))}
    </div>
  );
}
