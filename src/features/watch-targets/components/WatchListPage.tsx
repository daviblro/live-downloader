import { Plus, Search } from "lucide-react";
import type { WatchTarget } from "../../../shared/contracts";
import { useI18n } from "../../../shared/i18n";
import { WatchTable } from "./WatchTable";

interface WatchListPageProps {
  targets: WatchTarget[];
  selectedId: string | null;
  filter: string;
  onFilterChange: (value: string) => void;
  onAdd: () => void;
  onSelect: (target: WatchTarget) => void;
  onCheck: (target: WatchTarget) => void;
  onToggle: (target: WatchTarget) => void;
  onRemove: (target: WatchTarget) => void;
}

export function WatchListPage({
  targets,
  selectedId,
  filter,
  onFilterChange,
  onAdd,
  onSelect,
  onCheck,
  onToggle,
  onRemove,
}: WatchListPageProps) {
  const { translation: t } = useI18n();
  return (
    <section className="list-view">
      <header className="topbar">
        <div>
          <h1>{t.list.title}</h1>
          <p>{t.list.description}</p>
        </div>
        <button type="button" className="primary-action" onClick={onAdd}>
          <Plus size={18} />
          {t.common.addStream}
        </button>
      </header>
      <div className="list-toolbar">
        <label className="search-field">
          <Search size={17} />
          <input
            value={filter}
            onChange={(event) => onFilterChange(event.target.value)}
            placeholder={t.list.filterPlaceholder}
          />
        </label>
        <span>{t.list.sources(targets.length)}</span>
      </div>
      <WatchTable
        targets={targets}
        selectedId={selectedId}
        onSelect={onSelect}
        onCheck={onCheck}
        onToggle={onToggle}
        onRemove={onRemove}
      />
    </section>
  );
}
