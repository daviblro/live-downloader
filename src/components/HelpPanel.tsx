import { CircleHelp, ExternalLink, RadioTower, ShieldCheck } from "lucide-react";
import { useI18n } from "../i18n";

interface HelpPanelProps {
  onOpenIssues: () => void;
}

export function HelpPanel({ onOpenIssues }: HelpPanelProps) {
  const { translation: t } = useI18n();

  return <section className="help-view">
    <header className="topbar help-heading">
      <div><h1>{t.help.title}</h1><p>{t.help.description}</p></div>
      <button type="button" className="secondary-action" onClick={onOpenIssues}><ExternalLink size={17} />{t.help.openIssue}</button>
    </header>
    <div className="help-grid">
      <article className="faq-card"><div className="faq-icon"><RadioTower size={20} /></div><h2>{t.help.youtubeTitle}</h2><p>{t.help.youtubeBody}</p></article>
      <article className="faq-card"><div className="faq-icon"><CircleHelp size={20} /></div><h2>{t.help.startTitle}</h2><p>{t.help.startBody}</p></article>
      <article className="faq-card"><div className="faq-icon"><ShieldCheck size={20} /></div><h2>{t.help.accessTitle}</h2><p>{t.help.accessBody}</p></article>
      <article className="faq-card faq-support"><div><h2>{t.help.moreTitle}</h2><p>{t.help.moreBody}</p></div><button type="button" className="primary-action" onClick={onOpenIssues}><ExternalLink size={17} />{t.help.openIssue}</button></article>
    </div>
  </section>;
}
