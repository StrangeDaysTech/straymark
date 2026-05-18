import {cpSync, rmSync, existsSync, mkdirSync} from 'node:fs';
import {join, resolve, dirname} from 'node:path';

const ROOT = resolve(__dirname, '..');
const DOCS_I18N = resolve(ROOT, '../docs/i18n');
const SITE_I18N = resolve(ROOT, 'i18n');

const LOCALES = ['es'];
const SUBDIRS = ['adopters', 'contributors', 'decisions'];

for (const locale of LOCALES) {
  const src = join(DOCS_I18N, locale);
  if (!existsSync(src)) {
    console.log(`[sync-docs-i18n] skip ${locale} (no source)`);
    continue;
  }
  const dst = join(SITE_I18N, locale, 'docusaurus-plugin-content-docs', 'current');
  rmSync(dst, {recursive: true, force: true});
  mkdirSync(dst, {recursive: true});

  for (const sub of SUBDIRS) {
    const subSrc = join(src, sub);
    if (existsSync(subSrc)) {
      cpSync(subSrc, join(dst, sub), {recursive: true});
    }
  }

  const introSrc = join(src, 'intro.md');
  if (existsSync(introSrc)) {
    cpSync(introSrc, join(dst, 'intro.md'));
  }

  console.log(`[sync-docs-i18n] synced ${locale} -> ${dst}`);
}
