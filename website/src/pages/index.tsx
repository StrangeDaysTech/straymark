import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title} description={siteConfig.tagline}>
      <main style={{maxWidth: 760, margin: '4rem auto', padding: '0 1.5rem'}}>
        <h1>{siteConfig.title}</h1>
        <p>{siteConfig.tagline}</p>
        <p>
          <Link className="button button--primary" to="/docs/">
            Read the docs
          </Link>
        </p>
      </main>
    </Layout>
  );
}
