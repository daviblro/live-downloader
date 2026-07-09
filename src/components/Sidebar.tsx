import { Clock3, Download, FolderOpen, LayoutDashboard, ListVideo, Settings2 } from "lucide-react";

export type View = "overview" | "watch-list" | "history" | "settings";

const items: Array<{ id: View; label: string; icon: typeof LayoutDashboard }> = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "watch-list", label: "Watch list", icon: ListVideo },
  { id: "history", label: "History", icon: Clock3 },
  { id: "settings", label: "Settings", icon: Settings2 },
];

interface SidebarProps {
  active: View;
  onNavigate: (view: View) => void;
  onOpenDownloads: () => void;
  downloadDirectory: string;
}

export function Sidebar({ active, onNavigate, onOpenDownloads, downloadDirectory }: SidebarProps) {
  return <aside className="sidebar">
    <div className="brand"><span className="brand-mark"><Download size={17} /></span><span>Live <strong>Downloader</strong></span></div>
    <nav aria-label="Primary navigation">
      {items.map(({ id, label, icon: Icon }) => (
        <button key={id} type="button" className={`nav-item ${active === id ? "active" : ""}`} onClick={() => onNavigate(id)}>
          <Icon size={19} aria-hidden="true" />{label}
        </button>
      ))}
    </nav>
    <div className="sidebar-bottom">
      <div className="disk-summary">
        <div className="disk-heading"><span>Disk health</span><span className="healthy"><i />Good</span></div>
        <p>Library is ready for recordings</p>
        <div className="meter"><span style={{ width: "52%" }} /></div>
      </div>
      <button type="button" className="path-button" onClick={onOpenDownloads} title={downloadDirectory}>
        <FolderOpen size={16} /><span><small>Recording path</small>{downloadDirectory}</span>
      </button>
    </div>
  </aside>;
}
