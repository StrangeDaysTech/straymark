import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import Translate, {translate} from '@docusaurus/Translate';
import FeatureGrid from '@site/src/components/FeatureGrid';
import styles from './index.module.css';

export default function FeaturesIndex(): ReactNode {
  const metaTitle = translate({
    id: 'meta.features.title',
    message: "What's in the box — all StrayMark features",
    description:
      'OG/Twitter title and browser tab for the /features index (the " | StrayMark" suffix is appended by Docusaurus automatically)',
  });
  const metaDescription = translate({
    id: 'meta.features.description',
    message:
      'Every StrayMark capability on one page: charters, governance, multi-model audit, TDE drift detection, the CLI, agent skills, and emergent observation.',
    description: 'OG/Twitter description and meta description for the /features index',
  });

  return (
    <Layout title={metaTitle} description={metaDescription}>
      <main>
        <header className={styles.header}>
          <div className={styles.headerInner}>
            <p className={styles.eyebrow}>
              <Translate
                id="features.index.eyebrow"
                description="Small eyebrow label above the /features H1"
              >
                Features
              </Translate>
            </p>
            <h1 className={styles.title}>
              <Translate
                id="features.index.title"
                description="H1 of the /features index page"
              >
                What's in the box
              </Translate>
            </h1>
            <p className={styles.intro}>
              <Translate
                id="features.index.intro"
                description="Intro paragraph above the FeatureGrid on the /features index page"
              >
                Nine capabilities that compose StrayMark — from cognitive
                discipline and repo-native governance to multi-model audit
                and emergent observation. Pick the one that maps to the
                problem you're solving; each links to a focused page.
              </Translate>
            </p>
          </div>
        </header>
        <FeatureGrid />
      </main>
    </Layout>
  );
}
