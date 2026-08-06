import Translate from '@docusaurus/Translate';
import type {ReactNode} from 'react';
import styles from './styles.module.css';
import {BlueprintDefs, Edge, Node, type Point} from './primitives';

const VB_W = 900;
const VB_H = 380;

/* Standard 5-column grid: 5 nodes of width 160 with 20px gaps and 10px margins. */
const COLS = [10, 190, 370, 550, 730];

function Canvas({id, children}: {id: string; children: ReactNode}) {
  return (
    <svg
      viewBox={`0 0 ${VB_W} ${VB_H}`}
      preserveAspectRatio="xMidYMid meet"
      className={styles.svg}
      role="img"
      aria-hidden="true"
    >
      <BlueprintDefs id={id} />
      <rect width={VB_W} height={VB_H} fill={`url(#${id}-grid-major)`} />
      {children}
    </svg>
  );
}

const R = {w: 160, h: 46};
const D = {w: 122, h: 78};

const left = (x: number, y: number, h = R.h): Point => ({x, y: y + h / 2});
const right = (x: number, y: number, w = R.w, h = R.h): Point => ({x: x + w, y: y + h / 2});
const top = (x: number, y: number, w = R.w): Point => ({x: x + w / 2, y});
const bottom = (x: number, y: number, w = R.w, h = R.h): Point => ({x: x + w / 2, y: y + h});

const dLeft = (x: number, y: number): Point => ({x, y: y + D.h / 2});
const dRight = (x: number, y: number): Point => ({x: x + D.w, y: y + D.h / 2});
const dTop = (x: number, y: number): Point => ({x: x + D.w / 2, y});
const dBottom = (x: number, y: number): Point => ({x: x + D.w / 2, y: y + D.h});

/* -------------------------------------------------------------- */
/* 1. CHARTER                                                     */
/* -------------------------------------------------------------- */
export function CharterDiagram() {
  const id = 'bp-charter';
  const arrow = `${id}-arrow`;
  const row1y = 60;
  const n1 = {x: COLS[0], y: row1y};
  const n2 = {x: COLS[1], y: row1y};
  const n3 = {x: COLS[2], y: row1y};
  const n4 = {x: COLS[3], y: row1y};
  const n5 = {x: COLS[4], y: row1y};
  // Decision diamond centered under n5 (column center at 730+80=810; diamond w=122 → x=749)
  const dec = {x: 749, y: 200};
  // Bottom branch row
  const recon = {x: 320, y: 215};
  const close = {x: 730, y: 300};
  return (
    <Canvas id={id}>
      <Node x={n1.x} y={n1.y} w={R.w} h={R.h} shape="terminal" kind="terminal">
        <Translate id="workflow.charter.node.scaffold">Scaffold charter</Translate>
      </Node>
      <Node x={n2.x} y={n2.y} w={R.w} h={R.h}>
        <Translate id="workflow.charter.node.scope">Define scope &amp; files</Translate>
      </Node>
      <Node x={n3.x} y={n3.y} w={R.w} h={R.h}>
        <Translate id="workflow.charter.node.verification">Write verification</Translate>
      </Node>
      <Node x={n4.x} y={n4.y} w={R.w} h={R.h}>
        <Translate id="workflow.charter.node.risks">List risks</Translate>
      </Node>
      <Node x={n5.x} y={n5.y} w={R.w} h={R.h}>
        <Translate id="workflow.charter.node.execute">Execute &amp; emit AILOG</Translate>
      </Node>
      <Node x={dec.x} y={dec.y} w={D.w} h={D.h} shape="diamond" kind="decision">
        <Translate id="workflow.charter.node.drift">Drift detected?</Translate>
      </Node>
      <Node x={recon.x} y={recon.y} w={R.w} h={R.h}>
        <Translate id="workflow.charter.node.reconcile">Reconcile in PR</Translate>
      </Node>
      <Node x={close.x} y={close.y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.charter.node.close">Close charter</Translate>
      </Node>

      <Edge from={right(n1.x, n1.y)} to={left(n2.x, n2.y)} markerId={arrow} />
      <Edge from={right(n2.x, n2.y)} to={left(n3.x, n3.y)} markerId={arrow} />
      <Edge from={right(n3.x, n3.y)} to={left(n4.x, n4.y)} markerId={arrow} />
      <Edge from={right(n4.x, n4.y)} to={left(n5.x, n5.y)} markerId={arrow} />
      <Edge
        from={bottom(n5.x, n5.y)}
        to={dTop(dec.x, dec.y)}
        via={{x: n5.x + R.w / 2, y: 160}}
        markerId={arrow}
      />
      <Edge
        from={dLeft(dec.x, dec.y)}
        to={right(recon.x, recon.y)}
        via={{x: (dec.x + recon.x + R.w) / 2, y: dec.y + D.h / 2 - 10}}
        label="yes"
        markerId={arrow}
      />
      <Edge
        from={dBottom(dec.x, dec.y)}
        to={top(close.x, close.y)}
        via={{x: dec.x + D.w / 2 + 10, y: dec.y + D.h + 8}}
        label="no"
        markerId={arrow}
      />
      <Edge
        from={bottom(recon.x, recon.y)}
        to={left(close.x, close.y)}
        via={{x: (recon.x + close.x) / 2, y: close.y + R.h / 2 + 18}}
        markerId={arrow}
      />
    </Canvas>
  );
}

/* -------------------------------------------------------------- */
/* 2. AILOG                                                       */
/* -------------------------------------------------------------- */
export function AilogDiagram() {
  const id = 'bp-ailog';
  const arrow = `${id}-arrow`;
  const row1y = 80;
  const row2y = 240;
  const n = [
    {x: COLS[0], y: row1y},
    {x: COLS[1], y: row1y},
    {x: COLS[2], y: row1y},
    {x: COLS[3], y: row1y},
    {x: COLS[4], y: row1y},
    {x: COLS[2], y: row2y}, // Record risks (under Load template)
    {x: COLS[1], y: row2y}, // Commit (under Compute sequence)
  ];
  return (
    <Canvas id={id}>
      <Node x={n[0].x} y={n[0].y} w={R.w} h={R.h} shape="terminal" kind="terminal">
        <Translate id="workflow.ailog.node.context">Gather context</Translate>
      </Node>
      <Node x={n[1].x} y={n[1].y} w={R.w} h={R.h}>
        <Translate id="workflow.ailog.node.sequence">Compute sequence #</Translate>
      </Node>
      <Node x={n[2].x} y={n[2].y} w={R.w} h={R.h}>
        <Translate id="workflow.ailog.node.template">Load template</Translate>
      </Node>
      <Node x={n[3].x} y={n[3].y} w={R.w} h={R.h}>
        <Translate id="workflow.ailog.node.metadata">Fill metadata</Translate>
      </Node>
      <Node x={n[4].x} y={n[4].y} w={R.w} h={R.h}>
        <Translate id="workflow.ailog.node.changes">Document changes</Translate>
      </Node>
      <Node x={n[5].x} y={n[5].y} w={R.w} h={R.h}>
        <Translate id="workflow.ailog.node.risks">Record risks</Translate>
      </Node>
      <Node x={n[6].x} y={n[6].y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.ailog.node.commit">Commit with AILOG ref</Translate>
      </Node>

      <Edge from={right(n[0].x, n[0].y)} to={left(n[1].x, n[1].y)} markerId={arrow} />
      <Edge from={right(n[1].x, n[1].y)} to={left(n[2].x, n[2].y)} markerId={arrow} />
      <Edge from={right(n[2].x, n[2].y)} to={left(n[3].x, n[3].y)} markerId={arrow} />
      <Edge from={right(n[3].x, n[3].y)} to={left(n[4].x, n[4].y)} markerId={arrow} />
      {/* Drop down + over: from Document changes bottom → Record risks right */}
      <Edge
        from={bottom(n[4].x, n[4].y)}
        to={right(n[5].x, n[5].y)}
        via={{x: n[4].x + R.w / 2, y: row2y + R.h / 2}}
        markerId={arrow}
      />
      <Edge from={left(n[5].x, n[5].y)} to={right(n[6].x, n[6].y)} markerId={arrow} />
    </Canvas>
  );
}

/* -------------------------------------------------------------- */
/* 3. TDE — Technical Debt Entry                                  */
/* -------------------------------------------------------------- */
export function TdeDiagram() {
  const id = 'bp-tde';
  const arrow = `${id}-arrow`;
  const row1y = 80;
  const row2y = 215;
  const identify = {x: COLS[0], y: row1y};
  // Diamond centered at column 1 (190 + 80 = 270; diamond w=122 → x=209)
  const dec = {x: 209, y: 65};
  const create = {x: COLS[2], y: row1y};
  const score = {x: COLS[3], y: row1y};
  const prioritize = {x: COLS[4], y: row1y};
  const drop = {x: COLS[1], y: row2y};
  const remediate = {x: COLS[3], y: row2y};
  const resolve = {x: COLS[4], y: row2y};
  return (
    <Canvas id={id}>
      <Node x={identify.x} y={identify.y} w={R.w} h={R.h} shape="terminal" kind="terminal">
        <Translate id="workflow.tde.node.identify">Identify debt</Translate>
      </Node>
      <Node x={dec.x} y={dec.y} w={D.w} h={D.h} shape="diamond" kind="decision">
        <Translate id="workflow.tde.node.eligible">Eligible?</Translate>
      </Node>
      <Node x={drop.x} y={drop.y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.tde.node.drop">Track as R&lt;N&gt;</Translate>
      </Node>
      <Node x={create.x} y={create.y} w={R.w} h={R.h}>
        <Translate id="workflow.tde.node.create">Create TDE doc</Translate>
      </Node>
      <Node x={score.x} y={score.y} w={R.w} h={R.h}>
        <Translate id="workflow.tde.node.score">Score impact × effort</Translate>
      </Node>
      <Node x={prioritize.x} y={prioritize.y} w={R.w} h={R.h}>
        <Translate id="workflow.tde.node.prioritize">Human prioritization</Translate>
      </Node>
      <Node x={remediate.x} y={remediate.y} w={R.w} h={R.h}>
        <Translate id="workflow.tde.node.remediate">Remediate</Translate>
      </Node>
      <Node x={resolve.x} y={resolve.y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.tde.node.resolved">Mark resolved</Translate>
      </Node>

      <Edge from={right(identify.x, identify.y)} to={dLeft(dec.x, dec.y)} markerId={arrow} />
      <Edge
        from={dRight(dec.x, dec.y)}
        to={left(create.x, create.y)}
        label="yes"
        markerId={arrow}
      />
      <Edge
        from={dBottom(dec.x, dec.y)}
        to={top(drop.x, drop.y)}
        via={{x: dec.x + D.w / 2, y: 180}}
        label="no"
        markerId={arrow}
      />
      <Edge from={right(create.x, create.y)} to={left(score.x, score.y)} markerId={arrow} />
      <Edge from={right(score.x, score.y)} to={left(prioritize.x, prioritize.y)} markerId={arrow} />
      <Edge
        from={bottom(prioritize.x, prioritize.y)}
        to={top(remediate.x, remediate.y)}
        markerId={arrow}
      />
      <Edge from={right(remediate.x, remediate.y)} to={left(resolve.x, resolve.y)} markerId={arrow} />
    </Canvas>
  );
}

/* -------------------------------------------------------------- */
/* 4. COMPLIANCE                                                  */
/* -------------------------------------------------------------- */
export function ComplianceDiagram() {
  const id = 'bp-compliance';
  const arrow = `${id}-arrow`;
  const row1y = 80;
  const row2y = 230;
  const load = {x: COLS[0], y: row1y};
  const discover = {x: COLS[1], y: row1y};
  const parse = {x: COLS[2], y: row1y};
  const scan = {x: COLS[3], y: row1y};
  // Diamond centered at col 4: 730 + 80 = 810 → x = 749
  const dec = {x: 749, y: 65};
  const pass = {x: COLS[4], y: row2y};
  const suggest = {x: COLS[2], y: row2y};
  return (
    <Canvas id={id}>
      <Node x={load.x} y={load.y} w={R.w} h={R.h} shape="terminal" kind="terminal">
        <Translate id="workflow.compliance.node.scope">Load regional scope</Translate>
      </Node>
      <Node x={discover.x} y={discover.y} w={R.w} h={R.h}>
        <Translate id="workflow.compliance.node.discover">Discover .straymark/</Translate>
      </Node>
      <Node x={parse.x} y={parse.y} w={R.w} h={R.h}>
        <Translate id="workflow.compliance.node.parse">Parse frontmatter</Translate>
      </Node>
      <Node x={scan.x} y={scan.y} w={R.w} h={R.h}>
        <Translate id="workflow.compliance.node.scan">Scan standards</Translate>
      </Node>
      <Node x={dec.x} y={dec.y} w={D.w} h={D.h} shape="diamond" kind="decision">
        <Translate id="workflow.compliance.node.gaps">Gaps?</Translate>
      </Node>
      <Node x={pass.x} y={pass.y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.compliance.node.pass">Report pass</Translate>
      </Node>
      <Node x={suggest.x} y={suggest.y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.compliance.node.suggest">Suggest DPIA/MCARD/ETH</Translate>
      </Node>

      <Edge from={right(load.x, load.y)} to={left(discover.x, discover.y)} markerId={arrow} />
      <Edge from={right(discover.x, discover.y)} to={left(parse.x, parse.y)} markerId={arrow} />
      <Edge from={right(parse.x, parse.y)} to={left(scan.x, scan.y)} markerId={arrow} />
      <Edge from={right(scan.x, scan.y)} to={dLeft(dec.x, dec.y)} markerId={arrow} />
      <Edge
        from={dBottom(dec.x, dec.y)}
        to={top(pass.x, pass.y)}
        via={{x: dec.x + D.w / 2, y: dec.y + D.h + 15}}
        label="no"
        markerId={arrow}
      />
      <Edge
        from={dLeft(dec.x, dec.y)}
        to={right(suggest.x, suggest.y)}
        via={{x: (dec.x + suggest.x + R.w) / 2, y: dec.y + D.h / 2 + 70}}
        label="yes"
        markerId={arrow}
      />

      {/* Stack hint above the scan step */}
      <text x={scan.x + R.w / 2} y={scan.y - 14} className={styles.edgeLabel} textAnchor="middle">
        EU AI · ISO 42001 · NIST · TC260
      </text>
    </Canvas>
  );
}

/* -------------------------------------------------------------- */
/* 5. AUDIT TRAIL                                                 */
/* -------------------------------------------------------------- */
export function AuditDiagram() {
  const id = 'bp-audit';
  const arrow = `${id}-arrow`;
  const row1y = 80;
  const row2y = 230;
  const resolve = {x: COLS[0], y: row1y};
  const prompt = {x: COLS[1], y: row1y};
  const run = {x: COLS[2], y: row1y};
  const collect = {x: COLS[3], y: row1y};
  const verify = {x: COLS[4], y: row1y};
  const reclass = {x: COLS[3], y: row2y};
  const plan = {x: COLS[1], y: row2y}; // gap = COLS[2] is empty in bottom row to let the wire breathe
  const merge = {x: COLS[0], y: row2y};
  return (
    <Canvas id={id}>
      <Node x={resolve.x} y={resolve.y} w={R.w} h={R.h} shape="terminal" kind="terminal">
        <Translate id="workflow.audit.node.resolve">Resolve charter</Translate>
      </Node>
      <Node x={prompt.x} y={prompt.y} w={R.w} h={R.h}>
        <Translate id="workflow.audit.node.prompt">Generate audit prompt</Translate>
      </Node>
      <Node x={run.x} y={run.y} w={R.w} h={R.h}>
        <Translate id="workflow.audit.node.run">Run N auditors</Translate>
      </Node>
      <Node x={collect.x} y={collect.y} w={R.w} h={R.h}>
        <Translate id="workflow.audit.node.collect">Collect reports</Translate>
      </Node>
      <Node x={verify.x} y={verify.y} w={R.w} h={R.h}>
        <Translate id="workflow.audit.node.verify">Verify findings</Translate>
      </Node>
      <Node x={reclass.x} y={reclass.y} w={R.w} h={R.h}>
        <Translate id="workflow.audit.node.reclassify">Reclassify severity</Translate>
      </Node>
      <Node x={plan.x} y={plan.y} w={R.w} h={R.h}>
        <Translate id="workflow.audit.node.plan">Build remediation plan</Translate>
      </Node>
      <Node x={merge.x} y={merge.y} w={R.w} h={R.h} shape="terminal" kind="output">
        <Translate id="workflow.audit.node.merge">Merge into telemetry</Translate>
      </Node>

      <Edge from={right(resolve.x, resolve.y)} to={left(prompt.x, prompt.y)} markerId={arrow} />
      <Edge from={right(prompt.x, prompt.y)} to={left(run.x, run.y)} markerId={arrow} />
      <Edge from={right(run.x, run.y)} to={left(collect.x, collect.y)} markerId={arrow} />
      <Edge from={right(collect.x, collect.y)} to={left(verify.x, verify.y)} markerId={arrow} />
      <Edge
        from={bottom(verify.x, verify.y)}
        to={right(reclass.x, reclass.y)}
        via={{x: verify.x + R.w / 2, y: reclass.y + R.h / 2}}
        markerId={arrow}
      />
      <Edge from={left(reclass.x, reclass.y)} to={right(plan.x, plan.y)} markerId={arrow} />
      <Edge from={left(plan.x, plan.y)} to={right(merge.x, merge.y)} markerId={arrow} />

      {/* Parallel fan-out hint at "Run N auditors" */}
      <text x={run.x + R.w / 2} y={run.y - 14} className={styles.edgeLabel} textAnchor="middle">
        agy · claude · copilot
      </text>
    </Canvas>
  );
}
