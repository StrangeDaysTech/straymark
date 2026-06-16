//! Axonometric / BIM exploded view (Loom A3, Spec 002 §12 north star): the
//! architecture model in 3D — each layer is a stacked "floor", each component a
//! box on it, colored by the §4 "you are here" status. A real WebGL scene
//! (Three.js) with an **orthographic** camera (true axonometric — parallel
//! projection) and orbit controls.
//!
//! A3.1 adds the **explode** (a slider that separates the floors vertically —
//! the BIM "peel apart") and the **dependency lines** (the model's `links` as
//! 3D connectors between boxes, redrawn as floors move). Labels + click-detail
//! land in A3.2.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { stateColor } from './plan';
import { t } from './i18n';

interface ArchComponent {
  id: string;
  label: string;
  layer: string;
  states: string[];
}
interface ArchLayer {
  id: string;
  label: string;
  order: number;
}
interface ArchEdge {
  source: string;
  target: string;
}
interface ArchResponse {
  model_present: boolean;
  layers: ArchLayer[];
  components: ArchComponent[];
  edges: ArchEdge[];
}

const PRIORITY = ['active', 'in-progress', 'implemented', 'has-debt', 'wiring-gap', 'uncharted'];

// World units.
const BOX_W = 4;
const BOX_H = 1.4;
const BOX_D = 3;
const GAP_X = 2.4;
const SLAB_PAD = 2;
const MIN_FLOOR_GAP = 3.2; // compact
const MAX_FLOOR_GAP = 17; // fully exploded

interface Floor {
  group: THREE.Group;
  index: number; // 0..n-1 bottom→top
}
interface Conn {
  line: THREE.Line;
  a: THREE.Object3D;
  b: THREE.Object3D;
}

interface Scene3D {
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;
  camera: THREE.OrthographicCamera;
  controls: OrbitControls;
  frame: number;
  onResize: () => void;
  floors: Floor[];
  conns: Conn[];
  layerCount: number;
}

let current: Scene3D | null = null;

/** (Re)render the axonometric view into `container`. Idempotent. */
export async function renderAxon(container: HTMLElement, explode = 0.25): Promise<void> {
  let arch: ArchResponse;
  try {
    arch = await fetch('/api/architecture').then((r) => r.json() as Promise<ArchResponse>);
  } catch {
    showMessage(container, t('plan.error'));
    return;
  }
  if (!arch.model_present) {
    showMessage(container, t('plan.empty'));
    return;
  }

  dispose();
  container.textContent = '';

  const width = container.clientWidth || 800;
  const height = container.clientHeight || 600;

  const scene = new THREE.Scene();
  scene.background = new THREE.Color('#14161c');

  const frustum = 26;
  const aspect = width / height;
  const camera = new THREE.OrthographicCamera(
    -frustum * aspect,
    frustum * aspect,
    frustum,
    -frustum,
    0.1,
    1000,
  );
  camera.position.set(40, 34, 40);
  camera.lookAt(0, 0, 0);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(width, height);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  container.appendChild(renderer.domElement);

  scene.add(new THREE.AmbientLight(0xffffff, 1.1));
  const key = new THREE.DirectionalLight(0xffffff, 1.4);
  key.position.set(20, 40, 20);
  scene.add(key);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.1;
  controls.target.set(0, 0, 0);

  const { floors, boxByComp } = buildFloors(scene, arch);
  const conns = buildConnections(scene, arch, boxByComp);

  const onResize = () => {
    const w = container.clientWidth || width;
    const h = container.clientHeight || height;
    const a = w / h;
    camera.left = -frustum * a;
    camera.right = frustum * a;
    camera.top = frustum;
    camera.bottom = -frustum;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h);
  };
  window.addEventListener('resize', onResize);

  const s: Scene3D = {
    renderer,
    scene,
    camera,
    controls,
    frame: 0,
    onResize,
    floors,
    conns,
    layerCount: floors.length,
  };
  current = s;
  applyExplode(s, explode);

  const tick = () => {
    controls.update();
    renderer.render(scene, camera);
    s.frame = requestAnimationFrame(tick);
  };
  tick();
}

/** Build a floor group per layer with a box per component; index the boxes. */
function buildFloors(
  scene: THREE.Scene,
  arch: ArchResponse,
): { floors: Floor[]; boxByComp: Map<string, THREE.Object3D> } {
  const layers = [...arch.layers]
    .sort((a, b) => a.order - b.order)
    .filter((l) => arch.components.some((c) => c.layer === l.id));
  const boxByComp = new Map<string, THREE.Object3D>();
  const floors: Floor[] = [];

  layers.forEach((layer, i) => {
    const comps = arch.components.filter((c) => c.layer === layer.id);
    const group = new THREE.Group();

    const rowWidth = comps.length * BOX_W + (comps.length - 1) * GAP_X;
    let x = -rowWidth / 2 + BOX_W / 2;

    const slab = new THREE.Mesh(
      new THREE.BoxGeometry(rowWidth + SLAB_PAD * 2, 0.25, BOX_D + SLAB_PAD * 2),
      new THREE.MeshStandardMaterial({
        color: 0x2a2e3a,
        transparent: true,
        opacity: 0.35,
        roughness: 0.9,
      }),
    );
    slab.position.set(0, -BOX_H / 2 - 0.4, 0);
    group.add(slab);

    for (const comp of comps) {
      const state = pickState(comp.states);
      const palette = stateColor(state);
      const box = new THREE.Mesh(
        new THREE.BoxGeometry(BOX_W, BOX_H, BOX_D),
        new THREE.MeshStandardMaterial({
          color: new THREE.Color(palette.fill),
          emissive: new THREE.Color(state === 'active' ? palette.stroke : '#000000'),
          emissiveIntensity: state === 'active' ? 0.35 : 0,
          roughness: 0.55,
          metalness: 0.1,
        }),
      );
      box.position.set(x, 0, 0);
      box.userData = { componentId: comp.id, label: comp.label };
      box.add(
        new THREE.LineSegments(
          new THREE.EdgesGeometry(box.geometry),
          new THREE.LineBasicMaterial({ color: new THREE.Color(palette.stroke) }),
        ),
      );
      group.add(box);
      boxByComp.set(comp.id, box);
      x += BOX_W + GAP_X;
    }

    scene.add(group);
    floors.push({ group, index: i });
  });

  return { floors, boxByComp };
}

/** One line per model edge (source → target component box). */
function buildConnections(
  scene: THREE.Scene,
  arch: ArchResponse,
  boxByComp: Map<string, THREE.Object3D>,
): Conn[] {
  const conns: Conn[] = [];
  const material = new THREE.LineBasicMaterial({ color: 0x6b7280, transparent: true, opacity: 0.75 });
  for (const edge of arch.edges) {
    const a = boxByComp.get(edge.source);
    const b = boxByComp.get(edge.target);
    if (!a || !b) continue;
    const geom = new THREE.BufferGeometry().setFromPoints([new THREE.Vector3(), new THREE.Vector3()]);
    const line = new THREE.Line(geom, material);
    scene.add(line);
    conns.push({ line, a, b });
  }
  return conns;
}

/** Position the floors at the given explode amount (0 compact … 1 exploded)
 * and re-route the dependency lines to the boxes' new world positions. */
function applyExplode(s: Scene3D, amount: number): void {
  const gap = MIN_FLOOR_GAP + amount * (MAX_FLOOR_GAP - MIN_FLOOR_GAP);
  const yOffset = ((s.layerCount - 1) * gap) / 2;
  for (const f of s.floors) {
    f.group.position.y = f.index * gap - yOffset;
  }
  s.scene.updateMatrixWorld(true);
  const pa = new THREE.Vector3();
  const pb = new THREE.Vector3();
  for (const c of s.conns) {
    c.a.getWorldPosition(pa);
    c.b.getWorldPosition(pb);
    c.line.geometry.setFromPoints([pa.clone(), pb.clone()]);
  }
}

/** Set the explode amount on the live scene (called by the slider). */
export function setExplode(amount: number): void {
  if (current) applyExplode(current, Math.max(0, Math.min(1, amount)));
}

function pickState(states: string[]): string {
  for (const p of PRIORITY) {
    if (states.includes(p)) return p;
  }
  return 'uncharted';
}

/** Tear down the current scene (GL context, listeners, geometries). */
export function dispose(): void {
  if (!current) return;
  cancelAnimationFrame(current.frame);
  window.removeEventListener('resize', current.onResize);
  current.controls.dispose();
  current.scene.traverse((obj) => {
    if (obj instanceof THREE.Mesh || obj instanceof THREE.LineSegments || obj instanceof THREE.Line) {
      obj.geometry.dispose();
      const m = obj.material;
      if (Array.isArray(m)) m.forEach((mm) => mm.dispose());
      else m.dispose();
    }
  });
  current.renderer.dispose();
  current.renderer.domElement.remove();
  current = null;
}

function showMessage(container: HTMLElement, msg: string): void {
  dispose();
  container.innerHTML = `<div class="plan-empty">${msg}</div>`;
}
