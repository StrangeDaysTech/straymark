import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Translate from '@docusaurus/Translate';
import styles from './styles.module.css';

export default function GettingStarted(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.inner}>
        <h2 className={styles.title}>
          <Translate id="gettingStarted.title" description="Bottom getting-started section title">
            Ready to try it?
          </Translate>
        </h2>
        <p className={styles.body}>
          <Translate
            id="gettingStarted.body"
            description="Bottom getting-started section body paragraph"
          >
            The quickstart guide walks you from a fresh terminal to a closed Charter with one external-audit cycle on top — six short sections, plain copy-paste-able commands, about ten minutes of reading. If you've read this far, that's where to go next.
          </Translate>
        </p>
        <Link className={`button button--primary button--lg ${styles.cta}`} to="/quickstart">
          <Translate id="gettingStarted.cta" description="Bottom getting-started CTA button">
            Read the quickstart →
          </Translate>
        </Link>
        <p className={styles.meta}>
          <Translate
            id="gettingStarted.oss"
            description="OSS/MIT mention under the bottom CTA, with {github} interpolating to a link to the repo"
            values={{
              github: (
                <Link to="https://github.com/StrangeDaysTech/straymark">
                  GitHub →
                </Link>
              ),
            }}
          >
            {'Open source · MIT-licensed · source on {github}'}
          </Translate>
        </p>
      </div>
    </section>
  );
}
