import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import {translate} from '@docusaurus/Translate';
import Hero from '@site/src/components/Hero';
import WorkflowDiagram from '@site/src/components/WorkflowDiagram';
import WhyExists from '@site/src/components/WhyExists';
import FeatureGrid from '@site/src/components/FeatureGrid';

export default function Home(): ReactNode {
  const description = translate({
    id: 'hero.tagline',
    message: 'Cognitive discipline for AI-assisted engineering',
    description: 'Hero tagline (H1 on the landing)',
  });
  return (
    <Layout description={description}>
      <Hero />
      <WorkflowDiagram />
      <WhyExists />
      <FeatureGrid />
    </Layout>
  );
}
