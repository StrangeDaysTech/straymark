import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Hero from '@site/src/components/Hero';
import WorkflowDiagram from '@site/src/components/WorkflowDiagram';
import WhyExists from '@site/src/components/WhyExists';
import FeatureGrid from '@site/src/components/FeatureGrid';

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title={siteConfig.title} description={siteConfig.tagline}>
      <Hero />
      <WorkflowDiagram />
      <WhyExists />
      <FeatureGrid />
    </Layout>
  );
}
