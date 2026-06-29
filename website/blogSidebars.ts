import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  blogSidebar: [
    {
      type: 'category',
      label: 'Blog',
      link: {type: 'doc', id: 'index'},
      items: [
        {
          type: 'category',
          label: '2026',
          collapsible: false,
          collapsed: false,
          items: [
            '2026-06-17-what-the-open-format-left-to-the-producer',
            '2026-06-16-where-the-debt-actually-was',
            '2026-06-14-what-the-graph-couldnt-draw-yet',
            '2026-06-12-what-the-second-reader-demanded',
            '2026-06-04-what-the-bash-script-said-was-in-sync',
            '2026-05-31-what-the-feature-flag-compiled-away',
            '2026-05-23-what-the-binary-couldnt-hide',
            '2026-05-16-pattern-1-and-pattern-2-chain-evolution',
            '2026-05-16-emergent-observation-design',
            '2026-05-15-opening-the-framework',
            '2026-05-15-agents-md-as-a-universal-standard',
            '2026-05-14-manual-discipline-before-the-pattern',
            '2026-05-13-the-audit-prompt-was-the-outlier',
            '2026-05-12-tde-and-transversal-debt',
            '2026-05-11-validate-and-schemas-as-a-formal-layer',
            '2026-05-09-the-rebrand-to-straymark',
            '2026-05-09-charters-invisible-to-agents',
            '2026-05-06-charters-and-the-external-audit-cycle',
            '2026-04-30-six-plans-and-the-rename-to-charter',
            '2026-04-27-exploring-the-framework',
            '2026-04-05-four-names-in-four-months',
          ],
        },
      ],
    },
  ],
};

export default sidebars;
