//! Axonometric / BIM exploded view (Loom A3, Spec 002 §12 north star): the
//! architecture model rendered in 3D — each layer is a stacked "floor", each
//! component a box on it, colored by the §4 "you are here" status. A real WebGL
//! scene (Three.js) with an **orthographic** camera (true axonometric — parallel
//! projection, no perspective) and orbit controls. Explode (separating the
//! floors) and labels land in A3.1/A3.2.
//!
//! Same model as the 2D plan (`/api/architecture`); this is just a second view
//! of it (the BIM "one model, many views"). Auto-laid-out per floor — the 3D
//! view doesn't need the human `plan.drawio` geometry.

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
interface ArchResponse {
  model_present: boolean;
  layers: ArchLayer[];
  components: ArchComponent[];
}

const PRIORITY = ['active', 'in-progress', 'implemented', 'has-debt', 'wiring-gap', 'uncharted'];

// World units.
const BOX_W = 4;
const BOX_H = 1.4;
const BOX_D = 3;
const GAP_X = 2.4;
const FLOOR_GAP = 6; // vertical spacing between floors (A3.1 makes this a slider)
const SLAB_PAD = 2;

interface Scene3D {
  renderer: THREE.WebGLRenderer;
  scene: THREE.Scene;
  camera: THREE.OrthographicCamera;
  controls: OrbitControls;
  frame: number;
  onResize: () => void;
}

let current: Scene3D | null = null;

/** (Re)render the axonometric view into `container`. Idempotent. */
export async function renderAxon(container: HTMLElement): Promise<void> {
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

  // Orthographic camera = true axonometric (parallel) projection.
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

  buildFloors(scene, arch);

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

  const s: Scene3D = { renderer, scene, camera, controls, frame: 0, onResize };
  const tick = () => {
    controls.update();
    renderer.render(scene, camera);
    s.frame = requestAnimationFrame(tick);
  };
  tick();
  current = s;
}

/** Build a stacked floor per layer with a box per component (centered). */
function buildFloors(scene: THREE.Scene, arch: ArchResponse): void {
  const layers = [...arch.layers].sort((a, b) => a.order - b.order);
  // Center the whole stack vertically.
  const yOffset = ((layers.length - 1) * FLOOR_GAP) / 2;

  layers.forEach((layer, i) => {
    const comps = arch.components.filter((c) => c.layer === layer.id);
    if (comps.length === 0) return;
    const y = i * FLOOR_GAP - yOffset;

    // Row of boxes, centered on x.
    const rowWidth = comps.length * BOX_W + (comps.length - 1) * GAP_X;
    let x = -rowWidth / 2 + BOX_W / 2;

    // Translucent slab = the "floor".
    const slab = new THREE.Mesh(
      new THREE.BoxGeometry(rowWidth + SLAB_PAD * 2, 0.25, BOX_D + SLAB_PAD * 2),
      new THREE.MeshStandardMaterial({
        color: 0x2a2e3a,
        transparent: true,
        opacity: 0.35,
        roughness: 0.9,
      }),
    );
    slab.position.set(0, y - BOX_H / 2 - 0.4, 0);
    scene.add(slab);

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
      box.position.set(x, y, 0);
      box.userData = { componentId: comp.id, label: comp.label };
      // Crisp edge outline in the component's stroke color.
      const edges = new THREE.LineSegments(
        new THREE.EdgesGeometry(box.geometry),
        new THREE.LineBasicMaterial({ color: new THREE.Color(palette.stroke) }),
      );
      box.add(edges);
      scene.add(box);
      x += BOX_W + GAP_X;
    }
  });
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
    if (obj instanceof THREE.Mesh || obj instanceof THREE.LineSegments) {
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
