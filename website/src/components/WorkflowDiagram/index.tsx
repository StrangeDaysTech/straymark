import {
  useCallback,
  useEffect,
  useId,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from 'react';
import Translate, {translate} from '@docusaurus/Translate';
import useEmblaCarousel from 'embla-carousel-react';
import styles from './styles.module.css';
import {BlueprintFrame} from './primitives';
import {
  AilogDiagram,
  AuditDiagram,
  CharterDiagram,
  ComplianceDiagram,
  TdeDiagram,
} from './diagrams';

const FRAMEWORK_VERSION = 'fw-4.2.0';

type Workflow = {
  slug: string;
  short: ReactNode;
  title: ReactNode;
  caption: ReactNode;
  diagram: ReactNode;
};

function workflows(): Workflow[] {
  return [
    {
      slug: 'charter',
      short: <Translate id="workflow.charter.short">Charter</Translate>,
      title: <Translate id="workflow.charter.title">Charter — bounded unit of work</Translate>,
      caption: (
        <Translate id="workflow.charter.caption">
          Declare scope, files, verification and risks ex-ante. After execution, drift between
          declaration and reality is reconciled in the same PR or the charter cannot close.
        </Translate>
      ),
      diagram: <CharterDiagram />,
    },
    {
      slug: 'ailog',
      short: <Translate id="workflow.ailog.short">AILOG</Translate>,
      title: <Translate id="workflow.ailog.title">AILOG — AI action log</Translate>,
      caption: (
        <Translate id="workflow.ailog.caption">
          The execution ledger for every change: what was done, why, the risks discovered in flight,
          and the alternatives considered. One AILOG per commit, sequence-numbered per day.
        </Translate>
      ),
      diagram: <AilogDiagram />,
    },
    {
      slug: 'tde',
      short: <Translate id="workflow.tde.short">TDE</Translate>,
      title: (
        <Translate id="workflow.tde.title">TDE — transversal technical debt</Translate>
      ),
      caption: (
        <Translate id="workflow.tde.caption">
          When debt is heritage, crosses modules, needs its own charter or requires human
          prioritization, it stops being a per-charter risk and becomes a tracked, scored TDE.
        </Translate>
      ),
      diagram: <TdeDiagram />,
    },
    {
      slug: 'compliance',
      short: <Translate id="workflow.compliance.short">Compliance</Translate>,
      title: (
        <Translate id="workflow.compliance.title">
          Compliance — regulatory scanning
        </Translate>
      ),
      caption: (
        <Translate id="workflow.compliance.caption">
          The CLI walks .straymark/ documents, parses frontmatter, and scans them against the
          standards in scope (EU AI Act, ISO 42001, NIST, TC260, PIPL...). Gaps surface as
          actionable suggestions.
        </Translate>
      ),
      diagram: <ComplianceDiagram />,
    },
    {
      slug: 'audit',
      short: <Translate id="workflow.audit.short">Audit trail</Translate>,
      title: (
        <Translate id="workflow.audit.title">Audit trail — multi-model verification</Translate>
      ),
      caption: (
        <Translate id="workflow.audit.caption">
          Several auditor CLIs (agy, claude, copilot...) read the same prompt and audit a closed
          Charter. Findings are deduplicated, verified against code, reclassified and merged into
          telemetry as a signed evidence block.
        </Translate>
      ),
      diagram: <AuditDiagram />,
    },
  ];
}

export default function WorkflowDiagram(): ReactNode {
  const items = workflows();
  const total = items.length;
  const [emblaRef, emblaApi] = useEmblaCarousel({
    loop: false,
    containScroll: 'trimSnaps',
    align: 'start',
  });
  const [selected, setSelected] = useState(0);
  const [canPrev, setCanPrev] = useState(false);
  const [canNext, setCanNext] = useState(true);
  const tablistId = useId();

  const sync = useCallback(() => {
    if (!emblaApi) return;
    setSelected(emblaApi.selectedScrollSnap());
    setCanPrev(emblaApi.canScrollPrev());
    setCanNext(emblaApi.canScrollNext());
  }, [emblaApi]);

  useEffect(() => {
    if (!emblaApi) return;
    sync();
    emblaApi.on('select', sync);
    emblaApi.on('reInit', sync);
    return () => {
      emblaApi.off('select', sync);
      emblaApi.off('reInit', sync);
    };
  }, [emblaApi, sync]);

  const scrollTo = useCallback(
    (index: number) => {
      emblaApi?.scrollTo(index);
    },
    [emblaApi],
  );

  const onTabKey = useCallback(
    (e: KeyboardEvent<HTMLButtonElement>) => {
      if (e.key === 'ArrowRight') {
        e.preventDefault();
        scrollTo(Math.min(selected + 1, total - 1));
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        scrollTo(Math.max(selected - 1, 0));
      } else if (e.key === 'Home') {
        e.preventDefault();
        scrollTo(0);
      } else if (e.key === 'End') {
        e.preventDefault();
        scrollTo(total - 1);
      }
    },
    [scrollTo, selected, total],
  );

  return (
    <section className={styles.section}>
      <div className={styles.inner}>
        <h2 className={styles.title}>
          <Translate id="workflow.section.title" description="Workflow section title">
            Five workflows, one repo
          </Translate>
        </h2>
        <p className={styles.caption}>
          <Translate id="workflow.section.caption" description="Workflow section caption">
            Every step leaves a versioned artifact in your repo. No external systems, no implicit
            decisions.
          </Translate>
        </p>

        <ul className={styles.tablist} role="tablist" aria-label="StrayMark workflows">
          {items.map((w, i) => (
            <li key={w.slug} role="presentation">
              <button
                type="button"
                role="tab"
                id={`${tablistId}-tab-${w.slug}`}
                aria-controls={`${tablistId}-panel-${w.slug}`}
                aria-selected={i === selected}
                tabIndex={i === selected ? 0 : -1}
                className={styles.tabButton}
                onClick={() => scrollTo(i)}
                onKeyDown={onTabKey}
              >
                {w.short}
              </button>
            </li>
          ))}
        </ul>

        <div className={styles.viewport} ref={emblaRef}>
          <div className={styles.container}>
            {items.map((w, i) => (
              <div
                key={w.slug}
                className={styles.slide}
                role="tabpanel"
                id={`${tablistId}-panel-${w.slug}`}
                aria-labelledby={`${tablistId}-tab-${w.slug}`}
                aria-hidden={i !== selected}
              >
                <BlueprintFrame
                  index={i}
                  total={total}
                  title={w.title}
                  caption={w.caption}
                  version={FRAMEWORK_VERSION}
                >
                  {w.diagram}
                </BlueprintFrame>
              </div>
            ))}
          </div>
        </div>

        <div className={styles.controls} aria-hidden="false">
          <button
            type="button"
            className={styles.controlButton}
            onClick={() => emblaApi?.scrollPrev()}
            disabled={!canPrev}
            aria-label={translate({
              id: 'workflow.controls.previous',
              message: 'Previous workflow',
              description: 'Aria label for the previous-workflow button',
            })}
          >
            ‹
          </button>
          <div className={styles.dots} role="tablist" aria-label="Workflow indicators">
            {items.map((w, i) => (
              <button
                key={w.slug}
                type="button"
                role="tab"
                className={styles.dot}
                aria-selected={i === selected}
                aria-label={`${i + 1} / ${total}`}
                onClick={() => scrollTo(i)}
              />
            ))}
          </div>
          <button
            type="button"
            className={styles.controlButton}
            onClick={() => emblaApi?.scrollNext()}
            disabled={!canNext}
            aria-label={translate({
              id: 'workflow.controls.next',
              message: 'Next workflow',
              description: 'Aria label for the next-workflow button',
            })}
          >
            ›
          </button>
        </div>
      </div>
    </section>
  );
}
