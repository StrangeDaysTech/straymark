import type {ReactNode} from 'react';
import Layout from '@theme/Layout';
import {translate} from '@docusaurus/Translate';
import Hero from '@site/src/components/Hero';
import CognitivePairing from '@site/src/components/CognitivePairing';
import WorkflowDiagram from '@site/src/components/WorkflowDiagram';
import AsciinemaDemo from '@site/src/components/AsciinemaDemo';
import WhyExists from '@site/src/components/WhyExists';
import FeatureGrid from '@site/src/components/FeatureGrid';
import GettingStarted from '@site/src/components/GettingStarted';

export default function Home(): ReactNode {
  // SEO/Open-Graph title and description for the landing page. Kept separate
  // from the H1/hero copy (which stays short and punchy) because share
  // previews and search engines reward longer, keyword-rich strings — the
  // ~50-60 char title and ~110-160 char description ranges are the
  // generally-accepted sweet spot.
  // Docusaurus auto-appends ` | ${siteTitle}` to any Layout title, so we
  // only provide the tagline portion — final rendered title becomes
  // "Cognitive discipline for AI-assisted engineering | StrayMark".
  const metaTitle = translate({
    id: 'meta.landing.title',
    message: 'Cognitive discipline for AI-assisted engineering',
    description: 'OG/Twitter title and browser tab for the landing page (the " | StrayMark" suffix is appended by Docusaurus automatically)',
  });
  const metaDescription = translate({
    id: 'meta.landing.description',
    message:
      'Open-source framework + Rust CLI to externalize scope, decisions, and risks so AI agents stay coherent across turns. Maps to ISO 42001, EU AI Act, NIST AI RMF.',
    description: 'OG/Twitter description and meta description for the landing page',
  });
  return (
    <Layout title={metaTitle} description={metaDescription}>
      {/* Lighthouse flagged the homepage as missing a <main> landmark — the
          Docusaurus Layout wraps children in a generic <div>. Wrap our
          sections in <main> so screen readers and search engines can locate
          primary content. */}
      <main>
        <Hero />
        <CognitivePairing />
        <WorkflowDiagram />
        <AsciinemaDemo />
        <WhyExists />
        <FeatureGrid />
        <GettingStarted />
      </main>
    </Layout>
  );
}
