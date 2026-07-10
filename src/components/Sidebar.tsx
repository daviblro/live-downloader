import { Clock3, Download, FolderOpen, LayoutDashboard, ListVideo, Settings2 } from "lucide-react";
import { useI18n } from "../i18n";

export type View = "overview" | "watch-list" | "history" | "settings";

const items: Array<{ id: View; label: "overview" | "watchList" | "history" | "settings"; icon: typeof LayoutDashboard }> = [
  { id: "overview", label: "overview", icon: LayoutDashboard },
  { id: "watch-list", label: "watchList", icon: ListVideo },
  { id: "history", label: "history", icon: Clock3 },
  { id: "settings", label: "settings", icon: Settings2 },
];

interface SidebarProps {
  active: View;
  onNavigate: (view: View) => void;
  onOpenDownloads: () => void;
  downloadDirectory: string;
}

export function Sidebar({ active, onNavigate, onOpenDownloads, downloadDirectory }: SidebarProps) {
  const { translation: t } = useI18n();
  return <aside className="sidebar">
    <div className="brand"><span className="brand-mark"><Download size={17} /></span><span>Live <strong>Downloader</strong></span></div>
    <nav aria-label={t.sidebar.primaryNavigation}>
      {items.map(({ id, label, icon: Icon }) => (
        <button key={id} type="button" className={`nav-item ${active === id ? "active" : ""}`} onClick={() => onNavigate(id)}>
          <Icon size={19} aria-hidden="true" />{t.nav[label]}
        </button>
      ))}
    </nav>
    <div className="sidebar-bottom">
      <div className="disk-summary">
        <div className="disk-heading"><span>{t.sidebar.diskHealth}</span><span className="healthy"><i />{t.sidebar.good}</span></div>
        <p>{t.sidebar.libraryReady}</p>
        <div className="meter"><span style={{ width: "52%" }} /></div>
      </div>
      <button type="button" className="path-button" onClick={onOpenDownloads} title={downloadDirectory}>
        <FolderOpen size={16} /><span><small>{t.sidebar.recordingPath}</small>{downloadDirectory}</span>
      </button>
    </div>
  </aside>;
}
