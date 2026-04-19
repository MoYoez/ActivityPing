import { motion } from "motion/react";

const CARD_MOTION = {
  initial: { opacity: 0, y: 8 },
  animate: { opacity: 1, y: 0 },
};

const MOTION_TRANSITION = { duration: 0.18, ease: "easeOut" } as const;

export function AboutView({
  appIconSrc,
  githubUrl,
  openGithubBusy,
  primaryButtonClass,
  onOpenGithub,
}: {
  appIconSrc: string;
  githubUrl: string;
  openGithubBusy: boolean;
  primaryButtonClass: string;
  onOpenGithub: () => void;
}) {
  return (
    <motion.section className="about-page" {...CARD_MOTION} transition={MOTION_TRANSITION}>
      <div className="about-identity">
        <img className="about-icon" src={appIconSrc} alt="ActivityPing icon" />
        <div className="about-copy">
          <p className="eyebrow">About</p>
          <h3>ActivityPing</h3>
          <p>Desktop activity monitor for Discord Rich Presence and webhook reporting.</p>
        </div>
      </div>

      <div className="about-repo">
        <span className="eyebrow">Repository</span>
        <strong>MoYoez/ActivityPing</strong>
        <span>{githubUrl}</span>
      </div>

      <div className="about-actions">
        <button className={primaryButtonClass} type="button" disabled={openGithubBusy} onClick={onOpenGithub}>
          {openGithubBusy ? "Opening..." : "Open GitHub"}
        </button>
      </div>
    </motion.section>
  );
}
