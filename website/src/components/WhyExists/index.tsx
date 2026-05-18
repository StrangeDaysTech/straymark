import type {ReactNode} from 'react';
import Translate from '@docusaurus/Translate';
import styles from './styles.module.css';

export default function WhyExists(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.inner}>
        <h2 className={styles.eyebrow}>
          <Translate id="why.eyebrow" description="Why this exists eyebrow">
            Why this exists
          </Translate>
        </h2>
        <p className={styles.lead}>
          <Translate id="why.body" description="Why this exists body paragraph">
            The industry is busy with models, guardrails, and compliance. The missing piece is upstream of all of them: the cognitive discipline of the team working with the agents. Without structure, agents drift, decisions are lost, risks go undocumented, and regulatory evidence becomes improvised. StrayMark structures the work itself — repo-native, agent-aware, audit-ready by construction.
          </Translate>
        </p>
      </div>
    </section>
  );
}
