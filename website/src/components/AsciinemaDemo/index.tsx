import {useEffect, useRef, type ReactNode} from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Translate from '@docusaurus/Translate';
import 'asciinema-player/dist/bundle/asciinema-player.css';
import styles from './styles.module.css';

function PlayerInner(): ReactNode {
  const ref = useRef<HTMLDivElement | null>(null);
  const castUrl = useBaseUrl('/asciinema/straymark-demo.cast');

  useEffect(() => {
    if (!ref.current) return;
    let player: {dispose?: () => void} | undefined;
    let disposed = false;

    import('asciinema-player').then(({create}) => {
      if (disposed || !ref.current) return;
      player = create(castUrl, ref.current, {
        autoPlay: true,
        loop: true,
        preload: true,
        controls: 'auto',
        idleTimeLimit: 4,
        fit: 'width',
        terminalFontSize: 'small',
      });
    });

    return () => {
      disposed = true;
      if (player && typeof player.dispose === 'function') {
        player.dispose();
      }
    };
  }, [castUrl]);

  return <div ref={ref} className={styles.player} />;
}

export default function AsciinemaDemo(): ReactNode {
  return (
    <section className={styles.section}>
      <div className={styles.inner}>
        <h2 className={styles.title}>
          <Translate id="asciinema.title" description="Asciinema section title">
            From a fresh terminal to your first Charter
          </Translate>
        </h2>
        <p className={styles.caption}>
          <Translate id="asciinema.caption" description="Asciinema section caption">
            A 15-second recording of the real CLI. Three commands; no edits, no orchestration.
          </Translate>
        </p>
        <BrowserOnly fallback={<div className={styles.fallback}>Loading demo…</div>}>
          {() => <PlayerInner />}
        </BrowserOnly>
      </div>
    </section>
  );
}
