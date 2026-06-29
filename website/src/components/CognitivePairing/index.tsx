import type {ReactNode} from 'react';
import Translate from '@docusaurus/Translate';
import styles from './styles.module.css';

export default function CognitivePairing(): ReactNode {
  return (
    <section className={styles.section} aria-labelledby="cognitive-pairing-title">
      <div className={styles.inner}>
        <p className={styles.eyebrow}>
          <Translate id="cognitivePairing.eyebrow" description="Eyebrow for the cognitive pairing section">
            Cognitive pairing
          </Translate>
        </p>
        <h2 id="cognitive-pairing-title" className={styles.title}>
          <Translate id="cognitivePairing.title" description="Title for the cognitive pairing section">
            Knowledge humans can stand inside
          </Translate>
        </h2>
        <div className={styles.copy}>
          <p>
            <Translate id="cognitivePairing.body1" description="First paragraph explaining cognitive pairing">
              StrayMark is a cognitive pairing tool: it turns project information into situated
              knowledge. It does not dump data for a machine to query; it builds a map a person can
              stand inside, see what is being decided and why, what is in motion, and where the work
              is going.
            </Translate>
          </p>
          <p>
            <Translate id="cognitivePairing.body2" description="Second paragraph explaining human control and agent discipline">
              It also keeps humans from being outrun by the speed at which agents code and decide.
              For AI-augmented engineering, StrayMark keeps you in command of design and
              implementation decisions with verifiable context, while giving AI agents the cognitive
              discipline they need to stay coherent in medium and large projects.
            </Translate>
          </p>
        </div>
      </div>
    </section>
  );
}
