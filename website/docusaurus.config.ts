import {themes as prismThemes} from 'prism-react-renderer';
import type {Config, I18n} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const GITHUB_REPO = 'https://github.com/StrangeDaysTech/straymark';
const GITHUB_EDIT_BASE = `${GITHUB_REPO}/edit/main`;

const config: Config = {
  title: 'StrayMark',
  tagline: 'Cognitive discipline for AI-assisted engineering',
  favicon: 'favicon.ico',

  headTags: [
    // ── Google Analytics 4 + Consent Mode v2 ─────────────────────────────────
    // Critical: these five tags must appear in this exact order. They sit at
    // the top of headTags so they precede everything else Docusaurus injects.
    //
    // 1. Bootstrap dataLayer + gtag, push consent defaults to DENY analytics
    //    storage. gtag.js will read these from dataLayer when it boots and
    //    refuse to write cookies / fire hits until consent is updated.
    {
      tagName: 'script',
      attributes: {},
      innerHTML: `
        window.dataLayer = window.dataLayer || [];
        function gtag(){dataLayer.push(arguments);}
        gtag('consent', 'default', {
          ad_storage: 'denied',
          ad_user_data: 'denied',
          ad_personalization: 'denied',
          analytics_storage: 'denied',
          functionality_storage: 'granted',
          security_storage: 'granted',
          wait_for_update: 500
        });
      `.trim(),
    },
    // 2-3. Preconnect to Google's endpoints to save a round-trip when gtag.js
    //      loads. Cheap and standard.
    {
      tagName: 'link',
      attributes: {
        rel: 'preconnect',
        href: 'https://www.googletagmanager.com',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'preconnect',
        href: 'https://www.google-analytics.com',
      },
    },
    // 4. Load gtag.js async. When it executes, it reads the dataLayer queue
    //    above (consent default first, then config below) and applies
    //    consent BEFORE attempting to write cookies or fire events.
    {
      tagName: 'script',
      attributes: {
        async: 'true',
        src: 'https://www.googletagmanager.com/gtag/js?id=G-M8NW21WY2M',
      },
    },
    // 5. Initialise the GA tracker. anonymize_ip truncates the last octet of
    //    visitor IPs before transmission. Cookies are still gated by consent
    //    default above.
    {
      tagName: 'script',
      attributes: {},
      innerHTML: `
        gtag('js', new Date());
        gtag('config', 'G-M8NW21WY2M', {anonymize_ip: true});
      `.trim(),
    },
    // ── End GA setup ────────────────────────────────────────────────────────

    {
      tagName: 'link',
      attributes: {
        rel: 'icon',
        type: 'image/svg+xml',
        href: '/favicon.svg',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'icon',
        type: 'image/png',
        sizes: '32x32',
        href: '/favicon-32x32.png',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'icon',
        type: 'image/png',
        sizes: '16x16',
        href: '/favicon-16x16.png',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'apple-touch-icon',
        sizes: '180x180',
        href: '/apple-touch-icon.png',
      },
    },
    {
      tagName: 'link',
      attributes: {
        rel: 'manifest',
        href: '/site.webmanifest',
      },
    },
    {
      tagName: 'meta',
      attributes: {
        name: 'theme-color',
        content: '#0E0E10',
      },
    },
  ],

  future: {
    v4: true,
    // Required for sitemap <lastmod> to be populated from git: the sitemap
    // plugin reads each route's last update timestamp via this VCS layer.
    // 'git-eager' pre-reads the whole repo once at build start, then answers
    // per-file queries in memory — faster than 'git-ad-hoc' for a build that
    // covers ~80 routes × 3 locales. See VcsPreset in @docusaurus/types.
    experimental_vcs: 'git-eager',
  },

  url: 'https://straymark.dev',
  baseUrl: '/',

  organizationName: 'StrangeDaysTech',
  projectName: 'straymark',
  trailingSlash: false,

  onBrokenLinks: 'warn',

  markdown: {
    format: 'detect',
    mermaid: true,
    hooks: {
      onBrokenMarkdownLinks: 'warn',
    },
  },

  themes: ['@docusaurus/theme-mermaid'],

  clientModules: ['./src/clientModules/cookie-consent.ts'],

  i18n: {
    defaultLocale: 'en',
    locales: ['en', 'es', 'zh-CN'],
    localeConfigs: {
      en: {label: 'English'},
      es: {label: 'Español'},
      'zh-CN': {label: '简体中文'},
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
          include: ['intro.md', 'adopters/**', 'contributors/**'],
          exclude: ['**/i18n/**', '**/proposals/**', '**/decisions/**'],
          // Populates each route's metadata.lastUpdatedAt from git so the
          // sitemap plugin can emit <lastmod> per doc URL.
          showLastUpdateTime: true,
        },
        blog: {
          path: 'blog',
          routeBasePath: 'blog',
          blogTitle: 'Blog',
          blogDescription: 'The StrayMark chronicle — how the framework emerged, decision by decision.',
          blogSidebarCount: 'ALL',
          blogSidebarTitle: 'All posts',
          showReadingTime: true,
          // Populates metadata.lastUpdatedAt from git for every post so the
          // sitemap plugin can emit <lastmod> per post URL. Same lever the
          // docs plugin uses above.
          showLastUpdateTime: true,
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
        // Sitemap config. The classic preset bundles @docusaurus/plugin-sitemap;
        // by default it emits one sitemap.xml per locale build (en at root,
        // /es/sitemap.xml, /zh-CN/sitemap.xml) with no lastmod and no
        // hreflang alternates — meaning search engines treat each locale's
        // copy of the same content as unrelated documents.
        //
        // We enrich the default emitter via createSitemapItems:
        //   - `lastmod: 'date'` makes the plugin attach <lastmod> per URL,
        //     pulled from git for files with a known sourceFilePath.
        //   - createSitemapItems wraps defaultCreateSitemapItems and adds an
        //     `xhtml:link rel="alternate" hreflang="..."` entry per item for
        //     every locale we ship, plus an x-default pointing at the EN
        //     canonical. Google uses this to cluster the per-locale URLs as
        //     translations of the same page rather than as duplicates.
        //
        // The static counterparts (/sitemap-index.xml that aggregates the
        // three per-locale sitemaps, and /robots.txt that points crawlers at
        // the index) live under website/static/ — Docusaurus copies them
        // verbatim to the build root.
        sitemap: {
          lastmod: 'date',
          createSitemapItems: async (params) => {
            const {defaultCreateSitemapItems, ...rest} = params;
            const items = await defaultCreateSitemapItems(rest);
            const {siteConfig} = params;
            const siteUrl = siteConfig.url;
            // Docusaurus types `siteConfig.i18n` as I18nConfig (the input
            // shape), but at build time the loader hydrates it into the full
            // I18n shape, which carries `currentLocale`. Cast accordingly.
            const i18n = siteConfig.i18n as unknown as I18n;
            const {defaultLocale, locales, currentLocale} = i18n;

            // URL prefix for the current build. Default locale has no prefix
            // (URLs live at root); other locales are namespaced as
            // /<locale>/... — matches Docusaurus's i18n routing.
            const prefixFor = (locale: string): string =>
              locale === defaultLocale ? '' : `/${locale}`;
            const currentPrefix = prefixFor(currentLocale);

            // Strip the current-locale prefix from a path to recover the
            // locale-agnostic canonical path. Used to build the per-locale
            // alternate URLs below.
            const stripCurrentPrefix = (path: string): string => {
              if (!currentPrefix) return path;
              if (path === currentPrefix) return '/';
              if (path.startsWith(`${currentPrefix}/`)) {
                return path.slice(currentPrefix.length);
              }
              return path;
            };

            return items.map((item) => {
              const path = item.url.replace(siteUrl, '');
              const canonicalPath = stripCurrentPrefix(path) || '/';

              // One hreflang entry per shipped locale + x-default. The lib
              // (sitemap@7.x) emits each as
              //   <xhtml:link rel="alternate" hreflang="<lang>" href="<url>" />
              const links = [
                ...locales.map((locale) => ({
                  lang: locale,
                  url: `${siteUrl}${prefixFor(locale)}${canonicalPath}`,
                })),
                {
                  lang: 'x-default',
                  url: `${siteUrl}${canonicalPath}`,
                },
              ];

              return {...item, links};
            });
          },
        },
        // Note: we do NOT use the preset's `gtag` option. The plugin injects
        // gtag.js + `gtag('config', ...)` BEFORE user-config headTags render,
        // which means `gtag('consent', 'default', 'denied')` arrives after
        // config — too late to gate the initial cookie writes. We replicate
        // the plugin's behaviour manually in `headTags` (further down) so we
        // control the order, and re-implement SPA route tracking in the
        // cookie-consent client module via `onRouteUpdate`.
      } satisfies Preset.Options,
    ],
  ],

  plugins: ['plugin-image-zoom'],

  themeConfig: {
    imageZoom: {
      selector: '.markdown img, .zoomable img',
      options: {
        margin: 24,
        background: 'rgba(0, 0, 0, 0.85)',
        scrollOffset: 0,
      },
    },
    // Sitewide Open Graph / Twitter Card image. 1200×630 PNG, dark theme,
    // matches the brand and reads cleanly when embedded in Slack, Twitter/X,
    // LinkedIn, GitHub previews, etc. SVG source kept next to the PNG so the
    // card can be re-rendered when the brand evolves: see
    // `static/img/og/straymark.svg` and run `inkscape straymark.svg
    // --export-type=png --export-filename=straymark.png --export-width=1200`.
    image: 'img/og/straymark.png',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'StrayMark',
      logo: {
        alt: 'StrayMark',
        src: 'img/logo.svg',
        srcDark: 'img/logo-dark.svg',
        // Explicit dimensions prevent a navbar reflow before the SVG loads
        // (Lighthouse `unsized-images`).
        width: 32,
        height: 32,
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
            {label: 'GitHub', href: 'https://github.com/StrangeDaysTech'},
            {label: 'LinkedIn', href: 'https://www.linkedin.com/company/strangedaystech/'},
            {label: 'Website', href: 'https://strangedays.tech/'},
          ],
        },
        {
          title: 'Legal',
          items: [
            {label: 'Privacy', to: '/privacy'},
            // Intercepted by the cookie-consent client module
            // (src/clientModules/cookie-consent.ts) — it reads the
            // ?cookies=open query param on mount AND on click, opening the
            // preferences panel. Without JS, the user simply lands on
            // /privacy, which already explains how to opt out.
            {label: 'Cookie preferences', to: '/privacy?cookies=open'},
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
