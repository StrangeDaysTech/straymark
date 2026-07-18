// Generates RSS 2.0 + Atom 1.0 feeds for the blog, per locale, into build/.
//
// The blog is a docs-plugin instance (#340), not the classic blog plugin, so
// Docusaurus does not emit feeds for it. This post-build script fills that gap:
// it reads the published posts (canonical EN + the es / zh-CN translations),
// sorts them newest-first, and writes:
//
//   build/blog/rss.xml        build/blog/atom.xml         (en)
//   build/es/blog/rss.xml     build/es/blog/atom.xml      (es)
//   build/zh-CN/blog/rss.xml  build/zh-CN/blog/atom.xml   (zh-CN)
//
// It runs from `postbuild` (feeds are written into an already-built build/),
// mirroring the generated-artifact discipline of scripts/gen-llms-full.ts.
// Autodiscovery <link> tags are declared in docusaurus.config.ts headTags.
import {readdirSync, readFileSync, writeFileSync, existsSync, mkdirSync} from 'node:fs';
import {join, resolve} from 'node:path';
import matter from 'gray-matter';

const ROOT = resolve(__dirname, '..');
const BUILD = join(ROOT, 'build');
const SITE_URL = 'https://straymark.dev';
const AUTHOR_NAME = readAuthorName();

interface Locale {
  code: string; // '' for en (root), 'es', 'zh-CN'
  postsDir: string;
  title: string;
  description: string;
}

// Per-locale post sources. EN is canonical (website/blog); the translations
// live under the docs-blog i18n instance, same filenames/slugs.
const LOCALES: Locale[] = [
  {
    code: '',
    postsDir: join(ROOT, 'blog'),
    title: 'StrayMark Blog',
    description: 'The StrayMark chronicle — how the framework emerged, decision by decision.',
  },
  {
    code: 'es',
    postsDir: join(ROOT, 'i18n/es/docusaurus-plugin-content-docs-blog/current'),
    title: 'Blog de StrayMark',
    description: 'La crónica de StrayMark — cómo emergió el framework, decisión por decisión.',
  },
  {
    code: 'zh-CN',
    postsDir: join(ROOT, 'i18n/zh-CN/docusaurus-plugin-content-docs-blog/current'),
    title: 'StrayMark 博客',
    description: 'StrayMark 编年史 —— 这个框架如何随着一次次决策逐步成形。',
  },
];

interface Post {
  slug: string;
  title: string;
  date: Date;
  description: string;
}

function readAuthorName(): string {
  try {
    const yml = readFileSync(join(ROOT, 'blog/authors.yml'), 'utf8');
    const m = yml.match(/^\s*name:\s*(.+)\s*$/m);
    if (m) return m[1].trim();
  } catch {
    /* fall through */
  }
  return 'Strange Days Tech';
}

function xmlEscape(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

function readPosts(dir: string): Post[] {
  if (!existsSync(dir)) return [];
  const posts: Post[] = [];
  for (const name of readdirSync(dir)) {
    if (!/\.mdx?$/.test(name)) continue;
    if (/^index\./.test(name)) continue; // the landing page, not a post
    const {data} = matter(readFileSync(join(dir, name), 'utf8'));
    if (data.draft === true) continue;
    if (!data.slug || data.slug === '/') continue;
    if (!data.date) continue;
    posts.push({
      slug: String(data.slug).replace(/^\//, ''),
      title: String(data.title ?? data.slug),
      date: new Date(data.date),
      description: String(data.description ?? '').trim(),
    });
  }
  return posts.sort((a, b) => b.date.getTime() - a.date.getTime());
}

function prefix(code: string): string {
  return code ? `/${code}` : '';
}

function renderRss(loc: Locale, posts: Post[]): string {
  const base = `${SITE_URL}${prefix(loc.code)}`;
  const items = posts
    .map((p) => {
      const url = `${base}/blog/${p.slug}`;
      return [
        '    <item>',
        `      <title>${xmlEscape(p.title)}</title>`,
        `      <link>${url}</link>`,
        `      <guid isPermaLink="true">${url}</guid>`,
        `      <pubDate>${p.date.toUTCString()}</pubDate>`,
        `      <dc:creator>${xmlEscape(AUTHOR_NAME)}</dc:creator>`,
        `      <description>${xmlEscape(p.description)}</description>`,
        '    </item>',
      ].join('\n');
    })
    .join('\n');
  const lastBuild = (posts[0]?.date ?? new Date()).toUTCString();
  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/">',
    '  <channel>',
    `    <title>${xmlEscape(loc.title)}</title>`,
    `    <link>${base}/blog</link>`,
    `    <description>${xmlEscape(loc.description)}</description>`,
    `    <language>${loc.code || 'en'}</language>`,
    `    <lastBuildDate>${lastBuild}</lastBuildDate>`,
    `    <atom:link href="${base}/blog/rss.xml" rel="self" type="application/rss+xml"/>`,
    items,
    '  </channel>',
    '</rss>',
    '',
  ].join('\n');
}

function renderAtom(loc: Locale, posts: Post[]): string {
  const base = `${SITE_URL}${prefix(loc.code)}`;
  const updated = (posts[0]?.date ?? new Date()).toISOString();
  const entries = posts
    .map((p) => {
      const url = `${base}/blog/${p.slug}`;
      return [
        '  <entry>',
        `    <title>${xmlEscape(p.title)}</title>`,
        `    <link href="${url}"/>`,
        `    <id>${url}</id>`,
        `    <published>${p.date.toISOString()}</published>`,
        `    <updated>${p.date.toISOString()}</updated>`,
        `    <author><name>${xmlEscape(AUTHOR_NAME)}</name></author>`,
        `    <summary>${xmlEscape(p.description)}</summary>`,
        '  </entry>',
      ].join('\n');
    })
    .join('\n');
  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    '<feed xmlns="http://www.w3.org/2005/Atom">',
    `  <title>${xmlEscape(loc.title)}</title>`,
    `  <subtitle>${xmlEscape(loc.description)}</subtitle>`,
    `  <link href="${base}/blog/atom.xml" rel="self"/>`,
    `  <link href="${base}/blog"/>`,
    `  <id>${base}/blog</id>`,
    `  <updated>${updated}</updated>`,
    entries,
    '</feed>',
    '',
  ].join('\n');
}

function main(): void {
  if (!existsSync(BUILD)) {
    console.error('[gen-feeds] build/ not found — run after `docusaurus build`.');
    process.exit(1);
  }
  let total = 0;
  for (const loc of LOCALES) {
    const posts = readPosts(loc.postsDir);
    if (posts.length === 0) {
      console.warn(`[gen-feeds] no posts for '${loc.code || 'en'}' — skipped`);
      continue;
    }
    const outDir = join(BUILD, ...(loc.code ? [loc.code] : []), 'blog');
    if (!existsSync(outDir)) mkdirSync(outDir, {recursive: true});
    writeFileSync(join(outDir, 'rss.xml'), renderRss(loc, posts));
    writeFileSync(join(outDir, 'atom.xml'), renderAtom(loc, posts));
    console.log(`[gen-feeds] ${loc.code || 'en'}: ${posts.length} posts → ${outDir}/{rss,atom}.xml`);
    total += posts.length;
  }
  console.log(`[gen-feeds] done (${total} entries across ${LOCALES.length} locales).`);
}

main();
