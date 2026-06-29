import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import Translate, {translate} from '@docusaurus/Translate';
import styles from './styles.module.css';

const INSTALL = 'curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/straymark/main/install.sh | sh';
const HIGHLIGHT_TERMS = ['Cognitive', 'cognitiva', '认知'];

function renderHighlightedTagline(): ReactNode {
  const tagline = translate({
    id: 'hero.tagline',
    message: 'Cognitive discipline for AI-assisted engineering',
    description: 'Hero tagline (H1 on the landing)',
  });
  const lowerTagline = tagline.toLocaleLowerCase();
  const term = HIGHLIGHT_TERMS.find((candidate) =>
    lowerTagline.includes(candidate.toLocaleLowerCase()),
  );

  if (!term) return tagline;

  const start = lowerTagline.indexOf(term.toLocaleLowerCase());
  const end = start + term.length;
  return (
    <>
      {tagline.slice(0, start)}
      <span className={styles.highlight}>{tagline.slice(start, end)}</span>
      {tagline.slice(end)}
    </>
  );
}

export default function Hero(): ReactNode {
  return (
    <header className={styles.hero}>
      <div className={styles.inner}>
        <div className={styles.copy}>
          <p className={styles.eyebrow}>
            <Translate id="hero.eyebrow" description="Small hero eyebrow">
              StrayMark · documentation
            </Translate>
          </p>
          <h1 className={styles.tagline}>
            {renderHighlightedTagline()}
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
        </div>
        <div className={styles.cta}>
          <p className={styles.installLabel}>
            <Translate
              id="hero.installLabel"
              description="Eyebrow label above the install command in the hero"
            >
              Install the CLI in one line
            </Translate>
          </p>
          <pre className={styles.install} aria-label="Install command">
            <code>
              <span className={styles.prompt}>$</span> {INSTALL}
              {'\n'}
              <span className={styles.comment}># creates repo-native governance artifacts</span>
              {'\n'}
              <span className={styles.comment}># ready for your first Charter</span>
            </code>
          </pre>
          <p className={styles.quickstartHint}>
            <Translate
              id="hero.quickstartHint"
              description="Subtle link under the install command pointing to the quickstart guide"
              values={{
                link: (
                  <Link to="/quickstart" className={styles.quickstartLink}>
                    <Translate id="hero.quickstartHint.link" description="Link text in the quickstart hint">
                      quickstart guide
                    </Translate>
                  </Link>
                ),
              }}
            >
              {'…then follow our short {link} →'}
            </Translate>
          </p>
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
