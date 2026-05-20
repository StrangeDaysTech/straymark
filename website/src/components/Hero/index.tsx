import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Translate from '@docusaurus/Translate';
import CodeBlock from '@theme/CodeBlock';
import ThemedImage from '@theme/ThemedImage';
import styles from './styles.module.css';

const INSTALL = 'curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh';

export default function Hero(): ReactNode {
  return (
    <header className={styles.hero}>
      <div className={styles.inner}>
        <ThemedImage
          className={styles.banner}
          alt="StrayMark — by Strange Days Tech"
          sources={{
            light: '/img/straymark-banner-light.svg',
            dark: '/img/straymark-banner-dark.svg',
          }}
        />
        <h1 className={styles.tagline}>
          <Translate id="hero.tagline" description="Hero tagline (H1 on the landing)">
            Cognitive discipline for AI-assisted engineering
          </Translate>
        </h1>
        <ul className={styles.pillars}>
          <li>
            <Translate id="hero.pillar1" description="First hero pillar">
              Track every decision
            </Translate>
          </li>
          <li>
            <Translate id="hero.pillar2" description="Second hero pillar">
              Detect drift empirically
            </Translate>
          </li>
          <li>
            <Translate id="hero.pillar3" description="Third hero pillar">
              Audit-ready by default
            </Translate>
          </li>
        </ul>
        <div className={styles.cta}>
          <CodeBlock language="bash" className={styles.install}>
            {INSTALL}
          </CodeBlock>
          <div className={styles.buttons}>
            <Link className="button button--primary button--lg" to="/docs/">
              <Translate id="hero.cta.docs" description="Hero CTA — read the docs">
                Read the docs
              </Translate>
            </Link>
            <Link className="button button--secondary button--lg" to="/blog">
              <Translate id="hero.cta.blog" description="Hero CTA — read the blog">
                Read the chronicle
              </Translate>
            </Link>
          </div>
        </div>
      </div>
    </header>
  );
}
