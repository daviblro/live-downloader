import { CircleHelp, Clock3, Download, FolderOpen, HardDrive, LayoutDashboard, ListVideo, Settings2 } from "lucide-react";
import { useI18n } from "../i18n";
import type { DiskUsage } from "../types";

export type View = "overview" | "watch-list" | "history" | "settings" | "help";

const items: Array<{ id: View; label: "overview" | "watchList" | "history" | "settings" | "help"; icon: typeof LayoutDashboard }> = [
  { id: "overview", label: "overview", icon: LayoutDashboard },
  { id: "watch-list", label: "watchList", icon: ListVideo },
  { id: "history", label: "history", icon: Clock3 },
  { id: "settings", label: "settings", icon: Settings2 },
  { id: "help", label: "help", icon: CircleHelp },
];

interface SidebarProps {
  active: View;
  onNavigate: (view: View) => void;
  onOpenDownloads: () => void;
  downloadDirectory: string;
  diskUsage: DiskUsage | null;
}

function formatDiskSize(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1_000)), units.length - 1);
  const value = bytes / 1_000 ** index;
  return `${new Intl.NumberFormat(undefined, { maximumFractionDigits: index >= 3 ? 1 : 0 }).format(value)} ${units[index]}`;
}

export function Sidebar({ active, onNavigate, onOpenDownloads, downloadDirectory, diskUsage }: SidebarProps) {
  const { translation: t } = useI18n();
  const usedBytes = diskUsage ? Math.max(diskUsage.totalBytes - diskUsage.availableBytes, 0) : 0;
  const usedPercent = diskUsage?.totalBytes ? Math.round((usedBytes / diskUsage.totalBytes) * 100) : 0;
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
        <div className="disk-heading"><span className="disk-title"><HardDrive size={17} aria-hidden="true" />{t.sidebar.diskHealth}</span><span className="healthy"><i />{t.sidebar.good}</span></div>
        <p>{diskUsage ? t.sidebar.usedOfTotal(formatDiskSize(usedBytes), formatDiskSize(diskUsage.totalBytes), usedPercent) : t.sidebar.libraryReady}</p>
        <div className="meter" aria-label={diskUsage ? t.sidebar.usedOfTotal(formatDiskSize(usedBytes), formatDiskSize(diskUsage.totalBytes), usedPercent) : undefined}><span style={{ width: `${usedPercent}%` }} /></div>
      </div>
      <button type="button" className="path-button" onClick={onOpenDownloads} title={downloadDirectory}>
        <FolderOpen size={16} /><span><small>{t.sidebar.recordingPath}</small>{downloadDirectory}</span>
      </button>
    </div>
  </aside>;
}
