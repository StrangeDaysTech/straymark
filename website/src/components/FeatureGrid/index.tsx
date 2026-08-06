import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Translate, {translate} from '@docusaurus/Translate';
import styles from './styles.module.css';

type Feature = {
  titleId: string;
  titleDefault: string;
  bodyId: string;
  bodyDefault: string;
  to: string;
};

const FEATURES: Feature[] = [
  {
    titleId: 'features.discipline.title',
    titleDefault: 'Structured cognitive discipline',
    bodyId: 'features.discipline.body',
    bodyDefault:
      'Charters define purpose and limits before code. AILOGs capture human/agent exchange. AIDECs record decisions and tradeoffs.',
    to: '/features/cognitive-discipline',
  },
  {
    titleId: 'features.repo.title',
    titleDefault: 'Repo-native by design',
    bodyId: 'features.repo.body',
    bodyDefault:
      'Everything lives in your git repo: artifacts, governance rules, agent directives. No external platform, no second source of truth.',
    to: '/features/repo-native',
  },
  {
    titleId: 'features.agents.title',
    titleDefault: 'Declarative agent governance',
    bodyId: 'features.agents.body',
    bodyDefault:
      'Versioned rules in STRAYMARK.md and AGENT-RULES bind agent behavior at the workflow level — not at runtime, not after the fact.',
    to: '/features/agent-governance',
  },
  {
    titleId: 'features.evidence.title',
    titleDefault: 'Evidence as a byproduct',
    bodyId: 'features.evidence.body',
    bodyDefault:
      'EU AI Act, ISO 42001, NIST AI RMF, and GDPR mappings emerge from the same artifacts the team already produces. No parallel paper trail.',
    to: '/features/evidence-byproduct',
  },
  {
    titleId: 'features.cli.title',
    titleDefault: 'A CLI that does the work',
    bodyId: 'features.cli.body',
    bodyDefault:
      'init, validate, audit, analyze, compliance, metrics — one binary, eleven commands, deterministic outputs you can grep and pipe.',
    to: '/features/cli',
  },
  {
    titleId: 'features.tde.title',
    titleDefault: 'TDE: drift detection',
    bodyId: 'features.tde.body',
    bodyDefault:
      'The Transversal Debt Engine surfaces hidden coupling between charters before it compounds into incidents.',
    to: '/features/tde',
  },
  {
    titleId: 'features.skills.title',
    titleDefault: 'Skills for AI agents',
    bodyId: 'features.skills.body',
    bodyDefault:
      'Eleven slash-commands wrap the rituals: /straymark-charter-new, /straymark-ailog, /straymark-audit-prompt, /straymark-status. The agent drives the framework, not you.',
    to: '/features/skills',
  },
  {
    titleId: 'features.audit.title',
    titleDefault: 'Multi-model external audit',
    bodyId: 'features.audit.body',
    bodyDefault:
      'Three auditor CLIs (e.g. claude, copilot, agy) read the same prompt and audit the Charter independently at the closure gate — before it ships. A calibrator deduplicates, reclassifies severity, and merges signed evidence into telemetry.',
    to: '/features/multi-model-audit',
  },
  {
    titleId: 'features.eod.title',
    titleDefault: 'Emergent observation by design',
    bodyId: 'features.eod.body',
    bodyDefault:
      'Mandatory cross-references between documents let the agent spot stale specs and inter-charter drift on its own. Cognitive discipline raises the floor without tightening the prompt.',
    to: '/features/emergent-observation',
  },
];

export default function FeatureGrid(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.inner}>
        <h2 className={styles.title}>
          <Translate id="features.title" description="Feature grid title">
            What's in the box
          </Translate>
        </h2>
        <div className={styles.grid}>
          {FEATURES.map((f) => (
            <Link key={f.titleId} to={f.to} className={styles.card}>
              <h3 className={styles.cardTitle}>
                {translate({id: f.titleId, message: f.titleDefault})}
              </h3>
              <p className={styles.cardBody}>
                {translate({id: f.bodyId, message: f.bodyDefault})}
              </p>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
}
