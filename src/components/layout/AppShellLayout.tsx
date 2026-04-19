import type { ReactNode } from "react";
import { AnimatePresence, motion } from "motion/react";

const VIEW_MOTION = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -6 },
};

const MOTION_TRANSITION = { duration: 0.18, ease: "easeOut" } as const;

export function AppShellLayout({
  appIconSrc,
  activeSection,
  sections,
  sidebarStatusLabel,
  sidebarStatusDetail,
  reportPreviewText,
  activeKicker,
  activeTitle,
  activeDescription,
  onSectionChange,
  children,
}: {
  appIconSrc: string;
  activeSection: string;
  sections: Array<{ id: string; kicker: string; title: string }>;
  sidebarStatusLabel: string;
  sidebarStatusDetail: string;
  reportPreviewText: string;
  activeKicker: string;
  activeTitle: string;
  activeDescription: string;
  onSectionChange: (section: string) => void;
  children: ReactNode;
}) {
  return (
    <main className="shell" data-theme="activityping">
      <section className="app-frame home-shell card border border-base-300 bg-base-100 shadow-xl">
        <aside className="sidebar">
          <div className="sidebar-brand">
            <div className="sidebar-brand-head">
              <img className="sidebar-brand-icon" src={appIconSrc} alt="ActivityPing icon" />
              <div>
                <p className="eyebrow">ActivityPing</p>
                <h1>Activity relay</h1>
              </div>
            </div>
          </div>

          <nav aria-label="Primary navigation">
            <ul className="menu menu-sm sidebar-menu rounded-box bg-base-200/70 p-2">
              {sections.map((section) => (
                <li key={section.id}>
                  <button className={activeSection === section.id ? "menu-active" : ""} onClick={() => onSectionChange(section.id)} type="button">
                    <span className="sidebar-menu-copy">
                      <span>{section.kicker}</span>
                      <strong>{section.title}</strong>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          </nav>

          <div className="sidebar-guide card border border-base-300 bg-base-200/70 p-3 shadow-none">
            <div className="sidebar-guide-head">
              <span className="eyebrow">Status</span>
              <strong>{sidebarStatusLabel}</strong>
            </div>
            <p className="sidebar-guide-note">{sidebarStatusDetail}</p>
            <div className="sidebar-guide-report">
              <span className="eyebrow">Outgoing report</span>
              <p className="sidebar-guide-preview">{reportPreviewText}</p>
            </div>
          </div>
        </aside>

        <section className="content">
          <header className="content-header">
            <div>
              <p className="eyebrow">{activeKicker}</p>
              <h2>{activeTitle}</h2>
              <p>{activeDescription}</p>
            </div>
          </header>
          <div className="content-body">
            <AnimatePresence mode="wait">
              <motion.div key={activeSection} className="section-motion" {...VIEW_MOTION} transition={MOTION_TRANSITION}>
                {children}
              </motion.div>
            </AnimatePresence>
          </div>
        </section>
      </section>
    </main>
  );
}
