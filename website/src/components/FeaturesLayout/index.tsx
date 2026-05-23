import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import {useLocation} from '@docusaurus/router';
import {translate} from '@docusaurus/Translate';
import styles from './styles.module.css';

type Props = {
  children: ReactNode;
};

type FeatureLink = {
  slug: string;
  titleId: string;
  titleDefault: string;
};

const FEATURES: FeatureLink[] = [
  {
    slug: 'cognitive-discipline',
    titleId: 'features.discipline.title',
    titleDefault: 'Structured cognitive discipline',
  },
  {
    slug: 'repo-native',
    titleId: 'features.repo.title',
    titleDefault: 'Repo-native by design',
  },
  {
    slug: 'agent-governance',
    titleId: 'features.agents.title',
    titleDefault: 'Declarative agent governance',
  },
  {
    slug: 'evidence-byproduct',
    titleId: 'features.evidence.title',
    titleDefault: 'Evidence as a byproduct',
  },
  {slug: 'cli', titleId: 'features.cli.title', titleDefault: 'A CLI that does the work'},
  {slug: 'tde', titleId: 'features.tde.title', titleDefault: 'TDE: drift detection'},
  {
    slug: 'skills',
    titleId: 'features.skills.title',
    titleDefault: 'Skills for AI agents',
  },
  {
    slug: 'multi-model-audit',
    titleId: 'features.audit.title',
    titleDefault: 'Multi-model external audit',
  },
  {
    slug: 'emergent-observation',
    titleId: 'features.eod.title',
    titleDefault: 'Emergent observation by design',
  },
];

function isActive(pathname: string, slug: string): boolean {
  const target = `/features/${slug}`;
  return (
    pathname.endsWith(target) ||
    pathname.endsWith(`${target}/`) ||
    pathname.includes(`${target}/`)
  );
}

function isOverviewActive(pathname: string): boolean {
  // True only on the /features index — must NOT match /features/<slug>.
  return /\/features\/?$/.test(pathname);
}

export default function FeaturesLayout({children}: Props): ReactNode {
  const {pathname} = useLocation();
  const sidebarTitle = translate({
    id: 'features.sidebar.title',
    message: "What's in the box",
    description: 'Title for the left sidebar shared across feature pages',
  });
  const overviewLabel = translate({
    id: 'features.sidebar.overview',
    message: 'Overview',
    description: 'Sidebar link back to the /features index page from any feature subpage',
  });
  const overviewActive = isOverviewActive(pathname);

  return (
    <div className={styles.container}>
      <aside className={styles.sidebar} aria-label={sidebarTitle}>
        <h3 className={styles.sidebarTitle}>{sidebarTitle}</h3>
        <ul className={styles.sidebarList}>
          <li>
            <Link
              to="/features"
              className={`${styles.sidebarLink} ${overviewActive ? styles.sidebarLinkActive : ''}`}
              aria-current={overviewActive ? 'page' : undefined}
            >
              {overviewLabel}
            </Link>
          </li>
          {FEATURES.map((f) => {
            const active = isActive(pathname, f.slug);
            return (
              <li key={f.slug}>
                <Link
                  to={`/features/${f.slug}`}
                  className={`${styles.sidebarLink} ${active ? styles.sidebarLinkActive : ''}`}
                  aria-current={active ? 'page' : undefined}
                >
                  {translate({id: f.titleId, message: f.titleDefault})}
                </Link>
              </li>
            );
          })}
        </ul>
      </aside>
      <main className={styles.content}>
        <article className={styles.article}>{children}</article>
      </main>
    </div>
  );
}
