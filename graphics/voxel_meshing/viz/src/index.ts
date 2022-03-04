/**
 * https://threejs.org/
 * https://threejs.org/docs/index.html
 * https://threejs.org/examples/
 * https://threejs.org/manual/#en/primitives
 * https://sbcode.net/threejs/dat-gui/
 */

import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { CSS2DObject, CSS2DRenderer } from 'three/examples/jsm/renderers/CSS2DRenderer.js';
import { Vector3 } from "three/src/math/Vector3";

const cellsInChunkRow = 4;
const halfCellsInChunkRow = cellsInChunkRow / 2.0;
const maxIndexInChunkRow = cellsInChunkRow - 1;

const fullSize = 1;
const halfSize = 0.5;

function init() {
  const canvas = document.querySelector('#canvas') as HTMLElement;
  renderer = new THREE.WebGLRenderer({ canvas, antialias: true });

  const labelRendererDiv = document.querySelector('#labelRenderer') as HTMLElement;
  labelRenderer = new CSS2DRenderer({ element: labelRendererDiv });

  const fov = 75;
  const aspect = canvas.clientWidth / canvas.clientHeight;
  const near = 0.1;
  const far = 1000;
  camera = new THREE.PerspectiveCamera(fov, aspect, near, far);
  camera.position.set(halfCellsInChunkRow, halfCellsInChunkRow, cellsInChunkRow);

  const controls = new OrbitControls(camera, canvas);
  controls.target.set(halfCellsInChunkRow, halfCellsInChunkRow, -halfCellsInChunkRow);
  controls.update();

  scene = new THREE.Scene();
  scene.background = new THREE.Color(0xFFFFFF);

  const blackLineMaterial = new THREE.LineBasicMaterial({
    color: 0x000000,
  });
  const blueLoLineMaterial = new THREE.LineBasicMaterial({
    color: 0x003166,
  });
  const blueHiLineMaterial = new THREE.LineBasicMaterial({
    color: 0x006ee6,
  });

  const origin = new THREE.Object3D();
  origin.scale.z = -1;
  scene.add(origin);

  makeLowResolutionChunkBorders(origin, blackLineMaterial, "Main");
  const loZChunk = makeHighResolutionChunkBorder(origin, blueLoLineMaterial, "Lo-Z (0, 0)");
  loZChunk.position.z = -halfSize
  const hiZChunk00 = makeHighResolutionChunkBorder(origin, blueHiLineMaterial, "Hi-Z (0, 0)");
  hiZChunk00.position.z = cellsInChunkRow
}

function makeLowResolutionChunkBorders(
  origin: THREE.Object3D,
  material: THREE.Material,
  name: string
): THREE.Object3D {
  const chunk = new THREE.Object3D();
  origin.add(chunk)
  for(let x = 0; x < cellsInChunkRow; x++) {
    for(let y = 0; y < cellsInChunkRow; y++) {
      for(let z = 0; z < cellsInChunkRow; z++) {
        if(x === 0 || x === maxIndexInChunkRow || y === 0 || y === maxIndexInChunkRow || z === 0 || z === maxIndexInChunkRow) {
          makeBox(chunk, material, fullSize, new Vector3(x, y, z));
        }
      }
    }
  }
  chunk.add(makeAxes(halfCellsInChunkRow, new Vector3(halfCellsInChunkRow, halfCellsInChunkRow, halfCellsInChunkRow)));
  chunk.add(makeLabel(name));
  return chunk;
}

function makeHighResolutionChunkBorder(
  origin: THREE.Object3D,
  material: THREE.Material,
  name: string
): THREE.Object3D {
  const chunk = new THREE.Object3D();
  origin.add(chunk)
  for(let x = 0; x < cellsInChunkRow; x++) {
    for(let y = 0; y < cellsInChunkRow; y++) {
      makeBox(chunk, material, halfSize, new Vector3(x / 2.0, y / 2.0, 0));
    }
  }
  chunk.add(makeLabel(name));
  return chunk;
}

function makeLabel(text: string): CSS2DObject {
  const div = document.createElement('div');
  div.className = 'label';
  div.textContent = text;
  div.style.marginTop = '1em';
  return new CSS2DObject(div);
}

function makeAxes(size: number, offset: Vector3): THREE.AxesHelper {
  const axes = new THREE.AxesHelper(size);
  axes.position.set(offset.x, offset.y, offset.z);
  axes.renderOrder = 1;
  return axes
}

function makeBox(
  origin: THREE.Object3D,
  material: THREE.Material,
  size: number,
  position: Vector3
) {
  const boxGeometry = new THREE.BoxGeometry(size, size, size);
  const geometry = new THREE.EdgesGeometry(boxGeometry);
  const cube = new THREE.LineSegments(geometry, material);
  const halfSize = size / 2.0;
  cube.position.set(position.x + halfSize, position.y + halfSize, position.z + halfSize);
  origin.add(cube);
}

function render(_time: number) {
  if(resizeRendererToDisplaySize()) {
    const canvas = renderer.domElement;
    camera.aspect = canvas.clientWidth / canvas.clientHeight;
    camera.updateProjectionMatrix();
  }
  renderer.render(scene, camera);
  labelRenderer.render(scene, camera);
  requestAnimationFrame(render);
}

function resizeRendererToDisplaySize() {
  const canvas = renderer.domElement;
  const pixelRatio = window.devicePixelRatio;
  const width = canvas.clientWidth * pixelRatio | 0;
  const height = canvas.clientHeight * pixelRatio | 0;
  const needResize = canvas.width !== width || canvas.height !== height;
  if(needResize) {
    renderer.setSize(width, height, false);
    labelRenderer.setSize(width, height);
  }
  return needResize;
}

let camera: THREE.PerspectiveCamera, scene: THREE.Scene, renderer: THREE.Renderer, labelRenderer: CSS2DRenderer;

init();
render(0);
