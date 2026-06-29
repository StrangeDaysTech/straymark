import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  featuresSidebar: [
    {
      type: 'category',
      label: 'Features',
      link: {type: 'doc', id: 'index'},
      items: [
        'cognitive-discipline',
        'repo-native',
        'agent-governance',
        'evidence-byproduct',
        'cli',
        'tde',
        'skills',
        'multi-model-audit',
        'emergent-observation',
      ],
    },
  ],
};

export default sidebars;
