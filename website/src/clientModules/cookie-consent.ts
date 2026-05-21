/**
 * Cookie consent + Google Consent Mode v2 bridge.
 *
 * Runs only in the browser. Initialises vanilla-cookieconsent v3, and on
 * accept/reject calls gtag('consent', 'update', ...) so GA respects the
 * decision. The default-deny gtag('consent', 'default', ...) call lives in
 * docusaurus.config.ts > headTags and runs before gtag.js loads.
 *
 * Public API:
 *   window.openCookieConsent()  - reopen the preferences panel
 *                                 (wired into the footer "Privacy settings"
 *                                 link via theme-classic footer JS).
 */

import type {} from 'vanilla-cookieconsent';

// Declare gtag on window for TS.
declare global {
  interface Window {
    dataLayer?: unknown[];
    gtag?: (...args: unknown[]) => void;
    openCookieConsent?: () => void;
  }
}

// Strings shared across locales. Docusaurus's i18n doesn't reach this far
// (clientModules run outside the React tree), so we read the active locale
// from the <html lang> attribute and pick the matching copy ourselves.
const COPY = {
  en: {
    title: 'We use anonymous analytics',
    description:
      'StrayMark uses Google Analytics (with IP anonymisation) to understand which docs are most useful. Nothing personal is collected. You can change your mind any time from the footer.',
    acceptAll: 'Accept',
    acceptNecessary: 'Reject',
    showPreferences: 'Manage preferences',
    preferencesTitle: 'Cookie preferences',
    necessaryTitle: 'Strictly necessary',
    necessaryDescription:
      'Required for the site to function (locale switcher, theme preference). Always on.',
    analyticsTitle: 'Analytics',
    analyticsDescription:
      'Anonymous page-view stats via Google Analytics. Used to learn which sections of the docs are read and which are ignored.',
    savePreferences: 'Save preferences',
    close: 'Close',
    learnMore: 'Read the full privacy notice',
    learnMoreHref: '/privacy',
  },
  es: {
    title: 'Usamos analíticas anónimas',
    description:
      'StrayMark usa Google Analytics (con IP anonimizada) para entender qué documentos resultan más útiles. No se recopila nada personal. Puedes cambiar de opinión en cualquier momento desde el pie de página.',
    acceptAll: 'Aceptar',
    acceptNecessary: 'Rechazar',
    showPreferences: 'Gestionar preferencias',
    preferencesTitle: 'Preferencias de cookies',
    necessaryTitle: 'Estrictamente necesarias',
    necessaryDescription:
      'Requeridas para que el sitio funcione (selector de idioma, preferencia de tema). Siempre activas.',
    analyticsTitle: 'Analíticas',
    analyticsDescription:
      'Estadísticas anónimas de vistas de página vía Google Analytics. Sirven para aprender qué secciones de la documentación se leen y cuáles se ignoran.',
    savePreferences: 'Guardar preferencias',
    close: 'Cerrar',
    learnMore: 'Leer el aviso de privacidad completo',
    learnMoreHref: '/privacy',
  },
  'zh-CN': {
    title: '我们使用匿名分析',
    description:
      'StrayMark 使用 Google Analytics（已匿名化 IP）来了解哪些文档最有用。不会收集任何个人信息。你随时可以从页脚改变主意。',
    acceptAll: '接受',
    acceptNecessary: '拒绝',
    showPreferences: '管理偏好',
    preferencesTitle: 'Cookie 偏好',
    necessaryTitle: '严格必需',
    necessaryDescription:
      '站点正常运行所需（语言切换器、主题偏好）。始终开启。',
    analyticsTitle: '分析',
    analyticsDescription:
      '通过 Google Analytics 收集匿名页面浏览统计。用于了解文档的哪些部分被阅读、哪些被忽略。',
    savePreferences: '保存偏好',
    close: '关闭',
    learnMore: '阅读完整的隐私声明',
    learnMoreHref: '/privacy',
  },
} as const;

type LocaleKey = keyof typeof COPY;

function detectLocale(): LocaleKey {
  if (typeof document === 'undefined') return 'en';
  const lang = document.documentElement.lang || 'en';
  if (lang.startsWith('zh')) return 'zh-CN';
  if (lang.startsWith('es')) return 'es';
  return 'en';
}

function updateGtagConsent(granted: boolean) {
  // gtag is defined in the inline headTag script that initialises dataLayer.
  // Call it through window so we don't depend on the closure.
  if (typeof window === 'undefined' || typeof window.gtag !== 'function') {
    // dataLayer-only fallback: push directly.
    window.dataLayer = window.dataLayer || [];
    window.dataLayer.push([
      'consent',
      'update',
      {analytics_storage: granted ? 'granted' : 'denied'},
    ]);
    return;
  }
  window.gtag('consent', 'update', {
    analytics_storage: granted ? 'granted' : 'denied',
  });
}

async function initConsentUI() {
  if (typeof window === 'undefined') return;

  // vanilla-cookieconsent injects its own CSS; we import it explicitly so
  // webpack bundles it. The CSS file path is stable since v3.
  await import('vanilla-cookieconsent/dist/cookieconsent.css');
  const CookieConsent = await import('vanilla-cookieconsent');

  const lc = detectLocale();
  const t = COPY[lc];

  CookieConsent.run({
    guiOptions: {
      consentModal: {
        layout: 'box inline',
        position: 'bottom right',
        equalWeightButtons: true,
        flipButtons: false,
      },
      preferencesModal: {
        layout: 'box',
        position: 'right',
        equalWeightButtons: true,
        flipButtons: false,
      },
    },
    categories: {
      necessary: {readOnly: true, enabled: true},
      analytics: {},
    },
    language: {
      default: lc,
      translations: {
        [lc]: {
          consentModal: {
            title: t.title,
            description: `${t.description} <a href="${t.learnMoreHref}">${t.learnMore}</a>.`,
            acceptAllBtn: t.acceptAll,
            acceptNecessaryBtn: t.acceptNecessary,
            showPreferencesBtn: t.showPreferences,
          },
          preferencesModal: {
            title: t.preferencesTitle,
            acceptAllBtn: t.acceptAll,
            acceptNecessaryBtn: t.acceptNecessary,
            savePreferencesBtn: t.savePreferences,
            closeIconLabel: t.close,
            sections: [
              {
                title: t.necessaryTitle,
                description: t.necessaryDescription,
                linkedCategory: 'necessary',
              },
              {
                title: t.analyticsTitle,
                description: t.analyticsDescription,
                linkedCategory: 'analytics',
              },
            ],
          },
        },
      },
    },
    onConsent: ({cookie}) => {
      updateGtagConsent(cookie.categories?.includes('analytics') ?? false);
    },
    onChange: ({cookie}) => {
      updateGtagConsent(cookie.categories?.includes('analytics') ?? false);
    },
  });

  // Expose a reopen handle for footer links / external triggers.
  window.openCookieConsent = () => {
    CookieConsent.showPreferences();
  };

  // Intercept clicks on any link whose href contains ?cookies=open — the
  // footer "Cookie preferences" link uses that query so the URL works without
  // JS (it just lands on /privacy). With JS we hijack the click and open the
  // panel inline. One listener on document survives client-side navigation.
  document.addEventListener('click', (e) => {
    const target = (e.target as HTMLElement | null)?.closest?.(
      'a[href*="cookies=open"]',
    );
    if (target) {
      e.preventDefault();
      CookieConsent.showPreferences();
    }
  });

  // If the user landed via a deep-link like /privacy?cookies=open (no JS at
  // click time, or shared URL), open the panel after mount and clean the URL.
  if (
    typeof URLSearchParams !== 'undefined' &&
    new URLSearchParams(window.location.search).get('cookies') === 'open'
  ) {
    CookieConsent.showPreferences();
    const cleanUrl =
      window.location.pathname +
      window.location.hash;
    window.history.replaceState({}, '', cleanUrl);
  }
}

// Fire-and-forget on module load. Docusaurus imports client modules during
// hydration and we want the banner mounted as soon as possible.
initConsentUI();

type RouteUpdateArgs = {
  location: {pathname: string};
  previousLocation: {pathname: string} | null;
};

// Docusaurus calls this on every SPA route change. Replaces the page-view
// tracking that the @docusaurus/plugin-google-gtag client module would do —
// we removed the gtag preset option because its scripts injected before our
// consent default, defeating Consent Mode v2. See docusaurus.config.ts.
export default {
  onRouteUpdate({location, previousLocation}: RouteUpdateArgs) {
    if (typeof window === 'undefined') return;
    if (!previousLocation || location.pathname === previousLocation.pathname) return;
    if (typeof window.gtag !== 'function') return;
    // gtag respects the current consent state — if analytics_storage is still
    // 'denied', this becomes a no-op (or a cookieless ping under Consent Mode
    // v2). When the user has accepted, a normal page_view event fires.
    setTimeout(() => {
      window.gtag!('event', 'page_view', {
        page_title: document.title,
        page_location: window.location.href,
        page_path: location.pathname,
      });
    }, 50);
  },
};
