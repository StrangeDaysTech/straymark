import type {ReactNode} from 'react';
import styles from './styles.module.css';
import flow from './flowAnimations.module.css';

export type Point = {x: number; y: number};

export type NodeShape = 'rect' | 'diamond' | 'capsule' | 'terminal';
export type NodeKind = 'main' | 'decision' | 'terminal' | 'output';

type NodeProps = {
  x: number;
  y: number;
  w: number;
  h: number;
  shape?: NodeShape;
  kind?: NodeKind;
  children: ReactNode;
  labelLines?: number;
};

export function Node({
  x,
  y,
  w,
  h,
  shape = 'rect',
  kind = 'main',
  children,
  labelLines = 1,
}: NodeProps) {
  const cx = x + w / 2;
  const cy = y + h / 2;
  const kindClass =
    kind === 'decision'
      ? styles.shapeDecision
      : kind === 'terminal'
        ? styles.shapeTerminal
        : kind === 'output'
          ? styles.shapeOutput
          : styles.shapeMain;

  let shapeEl: ReactNode;
  if (shape === 'diamond') {
    const points = `${cx},${y} ${x + w},${cy} ${cx},${y + h} ${x},${cy}`;
    shapeEl = <polygon points={points} className={`${styles.nodeShape} ${kindClass}`} />;
  } else if (shape === 'capsule') {
    shapeEl = (
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={h / 2}
        ry={h / 2}
        className={`${styles.nodeShape} ${kindClass}`}
      />
    );
  } else if (shape === 'terminal') {
    shapeEl = (
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={12}
        ry={12}
        className={`${styles.nodeShape} ${kindClass}`}
      />
    );
  } else {
    shapeEl = (
      <rect
        x={x}
        y={y}
        width={w}
        height={h}
        rx={3}
        ry={3}
        className={`${styles.nodeShape} ${kindClass}`}
      />
    );
  }

  const lineHeight = 14;
  const textY = cy - ((labelLines - 1) * lineHeight) / 2 + 4;

  return (
    <g className={styles.node}>
      {shapeEl}
      <text x={cx} y={textY} className={styles.nodeLabel} textAnchor="middle">
        {children}
      </text>
    </g>
  );
}

type EdgeProps = {
  from: Point;
  to: Point;
  via?: Point;
  label?: string;
  flow?: boolean;
  draw?: boolean;
  markerId: string;
};

export function Edge({from, to, via, label, flow: hasFlow = true, draw = true, markerId}: EdgeProps) {
  let d: string;
  if (via) {
    d = `M ${from.x} ${from.y} Q ${via.x} ${via.y} ${to.x} ${to.y}`;
  } else {
    const midX = (from.x + to.x) / 2;
    const midY = (from.y + to.y) / 2;
    const dx = Math.abs(to.x - from.x);
    const dy = Math.abs(to.y - from.y);
    if (dy < 6 || dx < 6) {
      d = `M ${from.x} ${from.y} L ${to.x} ${to.y}`;
    } else {
      d = `M ${from.x} ${from.y} C ${midX} ${from.y}, ${midX} ${to.y}, ${to.x} ${to.y}`;
    }
  }

  const classes = [styles.edge];
  if (hasFlow) classes.push(flow.edgeFlow);
  if (draw) classes.push(flow.edgeDraw);

  let labelEl: ReactNode = null;
  if (label) {
    const lx = via ? via.x : (from.x + to.x) / 2;
    const ly = via ? via.y : (from.y + to.y) / 2;
    labelEl = (
      <text x={lx} y={ly - 6} className={styles.edgeLabel} textAnchor="middle">
        {label}
      </text>
    );
  }

  return (
    <g>
      <path
        d={d}
        className={classes.join(' ')}
        fill="none"
        markerEnd={`url(#${markerId})`}
        pathLength={1}
      />
      {labelEl}
    </g>
  );
}

type DefsProps = {
  id: string;
};

export function BlueprintDefs({id}: DefsProps) {
  const gridId = `${id}-grid`;
  const arrowId = `${id}-arrow`;
  return (
    <defs>
      <pattern id={gridId} width="22" height="22" patternUnits="userSpaceOnUse">
        <path
          d="M 22 0 L 0 0 0 22"
          fill="none"
          stroke="var(--bp-grid)"
          strokeWidth="0.5"
        />
      </pattern>
      <pattern id={`${id}-grid-major`} width="110" height="110" patternUnits="userSpaceOnUse">
        <rect width="110" height="110" fill={`url(#${gridId})`} />
        <path
          d="M 110 0 L 0 0 0 110"
          fill="none"
          stroke="var(--bp-grid-major)"
          strokeWidth="0.6"
        />
      </pattern>
      <marker
        id={arrowId}
        viewBox="0 0 10 10"
        refX="9"
        refY="5"
        markerWidth="6"
        markerHeight="6"
        orient="auto-start-reverse"
      >
        <path d="M 0 0 L 10 5 L 0 10 z" className={styles.arrowHead} />
      </marker>
    </defs>
  );
}

type FrameProps = {
  index: number;
  total: number;
  title: ReactNode;
  caption?: ReactNode;
  version: string;
  children: ReactNode;
};

export function BlueprintFrame({index, total, title, caption, version, children}: FrameProps) {
  const indexStr = `${String(index + 1).padStart(2, '0')}/${String(total).padStart(2, '0')}`;
  return (
    <div className={styles.frame}>
      <div className={styles.frameHeader}>
        <span className={styles.frameStamp}>STRAYMARK</span>
        <span className={styles.frameSeparator}>·</span>
        <span className={styles.frameIndex}>{indexStr}</span>
        <span className={styles.frameTitle}>{title}</span>
        <span className={styles.frameVersion}>{version}</span>
      </div>
      <div className={styles.frameCanvas}>
        <span className={`${styles.corner} ${styles.cornerTL}`} aria-hidden="true" />
        <span className={`${styles.corner} ${styles.cornerTR}`} aria-hidden="true" />
        <span className={`${styles.corner} ${styles.cornerBL}`} aria-hidden="true" />
        <span className={`${styles.corner} ${styles.cornerBR}`} aria-hidden="true" />
        {children}
      </div>
      {caption ? <div className={styles.frameFooter}>{caption}</div> : null}
    </div>
  );
}
