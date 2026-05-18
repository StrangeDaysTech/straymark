import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const GITHUB_REPO = 'https://github.com/StrangeDaysTech/straymark';
const GITHUB_EDIT_BASE = `${GITHUB_REPO}/edit/main`;

const config: Config = {
  title: 'StrayMark',
  tagline: 'Cognitive discipline for AI-assisted engineering',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://strangedaystech.github.io',
  baseUrl: '/straymark/',

  organizationName: 'StrangeDaysTech',
  projectName: 'straymark',
  trailingSlash: false,

  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

  markdown: {
    format: 'detect',
    mermaid: true,
  },

  themes: ['@docusaurus/theme-mermaid'],

  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es'],
    localeConfigs: {
      en: {label: 'English'},
      es: {label: 'Español'},
    },
  },

  presets: [
    [
      'classic',
      {
        docs: {
          path: '../docs',
          routeBasePath: 'docs',
          sidebarPath: './sidebars.ts',
          include: ['intro.md', 'adopters/**', 'contributors/**', 'decisions/**'],
          exclude: ['**/i18n/**', '**/proposals/**'],
          editUrl: ({versionDocsDirPath, docPath, locale}) => {
            if (locale && locale !== 'en') {
              return `${GITHUB_EDIT_BASE}/docs/i18n/${locale}/${docPath}`;
            }
            return `${GITHUB_EDIT_BASE}/docs/${docPath}`;
          },
        },
        blog: {
          path: 'blog',
          routeBasePath: 'blog',
          blogTitle: 'Blog',
          blogDescription: 'The StrayMark chronicle — how the framework emerged, decision by decision.',
          showReadingTime: true,
          feedOptions: {
            type: ['rss', 'atom'],
            title: 'StrayMark Blog',
            description: 'The StrayMark chronicle.',
            copyright: `Copyright © ${new Date().getFullYear()} Strange Days Tech.`,
          },
          editUrl: ({locale, blogDirPath, blogPath}) => {
            if (locale && locale !== 'en') {
              return `${GITHUB_EDIT_BASE}/website/i18n/${locale}/docusaurus-plugin-content-blog/${blogPath}`;
            }
            return `${GITHUB_EDIT_BASE}/website/${blogDirPath}/${blogPath}`;
          },
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/docusaurus-social-card.jpg',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'StrayMark',
      logo: {
        alt: 'StrayMark',
        src: 'img/logo.svg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {to: '/blog', label: 'Blog', position: 'left'},
        {
          type: 'localeDropdown',
          position: 'right',
        },
        {
          href: GITHUB_REPO,
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Adoption Guide', to: '/docs/adopters/ADOPTION-GUIDE'},
            {label: 'CLI Reference', to: '/docs/adopters/CLI-REFERENCE'},
            {label: 'Workflows', to: '/docs/adopters/WORKFLOWS'},
          ],
        },
        {
          title: 'Project',
          items: [
            {label: 'Blog', to: '/blog'},
            {label: 'GitHub', href: GITHUB_REPO},
            {label: 'Releases', href: `${GITHUB_REPO}/releases`},
            {label: 'Issues', href: `${GITHUB_REPO}/issues`},
          ],
        },
        {
          title: 'Strange Days Tech',
          items: [
            {label: 'Organization', href: 'https://github.com/StrangeDaysTech'},
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Strange Days Tech. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['bash', 'rust', 'toml', 'yaml', 'json'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
