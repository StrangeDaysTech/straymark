/**
 * One-shot migrator: Aparador/Para blog/ -> website/blog/ + i18n/es/.
 *
 * Source files live outside the repo's public tree (Aparador/ is gitignored).
 * Each post has an EN + ES pair, named `NN-<locale-slug>.<en|es>.md`, paired
 * by the `NN-` numeric prefix. Frontmatter is normalized to Docusaurus blog
 * conventions; the source files in Aparador stay as the editorial archive.
 *
 * zh-CN is intentionally NOT handled here. zh-CN posts are translations of
 * the editorial work (not original authoring), so they live directly under
 * `website/i18n/zh-CN/docusaurus-plugin-content-blog/` with no migrator
 * round-trip. See the feat/website-zh-CN PR for that workflow.
 *
 * Run once: `npm run migrate:blog` (from website/).
 * Idempotent: only deletes .md files in output dirs, preserving authors.yml.
 */
import {readdirSync, readFileSync, writeFileSync, mkdirSync, rmSync, existsSync, unlinkSync} from 'node:fs';
import {join, resolve} from 'node:path';
import matter from 'gray-matter';

const ROOT = resolve(__dirname, '..');
const APARADOR = resolve(ROOT, '../Aparador/Para blog');
const OUT_EN = resolve(ROOT, 'blog');
const OUT_ES = resolve(ROOT, 'i18n/es/docusaurus-plugin-content-blog');
const EXCERPTS_PATH = resolve(__dirname, 'blog-excerpts.json');

type Excerpt = {en: string; es: string};
const excerpts: Record<string, Excerpt> = JSON.parse(readFileSync(EXCERPTS_PATH, 'utf8'));

const FILE_RE = /^(\d{2})-(.+)\.(en|es)\.md$/;

type Post = {
  prefix: string;
  enSlug?: string;
  esSlug?: string;
  enPath?: string;
  esPath?: string;
};

function collect(): Map<string, Post> {
  const posts = new Map<string, Post>();
  for (const name of readdirSync(APARADOR)) {
    const m = name.match(FILE_RE);
    if (!m) continue;
    const [, prefix, slug, locale] = m;
    const post = posts.get(prefix) ?? {prefix};
    const path = join(APARADOR, name);
    if (locale === 'en') {
      post.enSlug = slug;
      post.enPath = path;
    } else {
      post.esSlug = slug;
      post.esPath = path;
    }
    posts.set(prefix, post);
  }
  return posts;
}

function stripLeadingH1(body: string, title: string): string {
  // Strip a leading `# <title>` if it duplicates the frontmatter title.
  const lines = body.split('\n');
  let i = 0;
  while (i < lines.length && lines[i].trim() === '') i++;
  if (i < lines.length) {
    const m = lines[i].match(/^#\s+(.+?)\s*$/);
    if (m && m[1].trim() === title.trim()) {
      // Drop the H1 line and one blank line after it.
      lines.splice(i, lines[i + 1]?.trim() === '' ? 2 : 1);
      return lines.join('\n').replace(/^\n+/, '');
    }
  }
  return body;
}

function transformFrontmatter(fm: Record<string, unknown>, slug: string): Record<string, unknown> {
  const out: Record<string, unknown> = {
    slug,
    title: fm.title,
    authors: ['jose'],
    tags: fm.tags ?? [],
    date: fm.date,
  };
  if (fm.subtitle) out.description = fm.subtitle;
  return out;
}

function prependExcerpt(body: string, excerpt: string | undefined): string {
  if (!excerpt) return body;
  // Lead paragraph (italics, since it's an editorial excerpt) + truncate
  // marker. Everything above the marker is what Docusaurus shows on the
  // /blog and /es/blog listings.
  return `*${excerpt}*\n\n<!-- truncate -->\n\n${body.replace(/^\n+/, '')}`;
}

function emit(srcPath: string, slug: string, outDir: string, canonicalSlug: string, locale: 'en' | 'es'): {date: string; filename: string} {
  const raw = readFileSync(srcPath, 'utf8');
  const parsed = matter(raw);
  const fm = parsed.data;
  const title = String(fm.title ?? '');
  const date = fm.date instanceof Date
    ? fm.date.toISOString().slice(0, 10)
    : String(fm.date);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(date)) {
    throw new Error(`Invalid date "${date}" in ${srcPath}`);
  }
  let body = stripLeadingH1(parsed.content, title);
  body = prependExcerpt(body, excerpts[canonicalSlug]?.[locale]);
  const outFm = transformFrontmatter(fm, slug);
  const filename = `${date}-${slug}.md`;
  const outPath = join(outDir, filename);
  const yaml = matter.stringify(body, outFm);
  writeFileSync(outPath, yaml);
  return {date, filename};
}

function main() {
  if (!existsSync(APARADOR)) {
    throw new Error(`Source not found: ${APARADOR}`);
  }
  // Clean only the migrator's own output (.md files) so curated assets like
  // authors.yml / tags.yml in these directories are preserved across re-runs.
  mkdirSync(OUT_EN, {recursive: true});
  mkdirSync(OUT_ES, {recursive: true});
  for (const dir of [OUT_EN, OUT_ES]) {
    for (const name of readdirSync(dir)) {
      if (name.endsWith('.md')) unlinkSync(join(dir, name));
    }
  }

  const posts = collect();
  const sorted = [...posts.values()].sort((a, b) => a.prefix.localeCompare(b.prefix));
  let enCount = 0;
  let esCount = 0;

  for (const post of sorted) {
    if (!post.enPath || !post.esPath || !post.enSlug || !post.esSlug) {
      console.warn(`[migrate-blog] post ${post.prefix} is missing a locale pair; skipping`);
      continue;
    }
    // EN file under blog/, using EN slug both as filename and frontmatter slug.
    const en = emit(post.enPath, post.enSlug, OUT_EN, post.enSlug, 'en');
    // ES file under i18n/es/, using EN slug as filename (Docusaurus pairs by
    // filename) but ES slug in frontmatter for a clean Spanish URL.
    emit(post.esPath, post.esSlug, OUT_ES, post.enSlug, 'es');
    // Rename the ES output to match the EN filename for i18n pairing.
    const esFilename = `${en.date}-${post.enSlug}.md`;
    const esWritten = join(OUT_ES, `${en.date}-${post.esSlug}.md`);
    const esTarget = join(OUT_ES, esFilename);
    if (esWritten !== esTarget) {
      const content = readFileSync(esWritten, 'utf8');
      writeFileSync(esTarget, content);
      rmSync(esWritten);
    }
    enCount++;
    esCount++;
    console.log(`[migrate-blog] ${post.prefix} -> ${en.filename}`);
  }

  console.log(`[migrate-blog] done. EN: ${enCount}, ES: ${esCount}`);
}

main();
