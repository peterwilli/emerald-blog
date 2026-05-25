import * as THREE from "three";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";
// import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { RenderPass } from "three/examples/jsm/postprocessing/RenderPass.js";
// import Stats from "three/examples/jsm/libs/stats.module.js";
import {
  MeshBVH,
  MeshBVHUniformStruct,
  shaderStructs,
  shaderIntersectFunction,
  SAH,
} from "three-mesh-bvh";
import { BokehPass } from "three/examples/jsm/postprocessing/BokehPass.js";
import { UnrealBloomPass } from "three/examples/jsm/postprocessing/UnrealBloomPass.js";
import { EffectComposer } from "three/examples/jsm/postprocessing/EffectComposer.js";
import { OutputPass } from "three/examples/jsm/postprocessing/OutputPass.js";

let container, stats, bokehPass, composer;
let camera, scene, renderer, controls;
let clock = new THREE.Clock();
let frontMostCrystal = null;

let bloomPass = null;
const crystals = [];
const prefersReducedMotion = window.matchMedia(
  "(prefers-reduced-motion: reduce)",
).matches;
const isSmallScreen = window.matchMedia("(max-width: 768px)").matches;
const isLowPowerDevice =
  prefersReducedMotion ||
  isSmallScreen ||
  (navigator.hardwareConcurrency ?? 8) <= 4;
const targetFps = prefersReducedMotion ? 12 : isLowPowerDevice ? 24 : 30;
const frameInterval = 1 / targetFps;
const maxPixelRatio = isLowPowerDevice ? 1 : 1.5;
const crystalsPerLayer = isLowPowerDevice ? 5 : 8;
const bounceCount = isLowPowerDevice ? 2 : 3;
const useBokeh = !prefersReducedMotion && !isSmallScreen;
let frameAccumulator = 0;
let isVisible = true;
let isInViewport = true;

const copyrightYear = document.querySelector("#copyright-year");
if (copyrightYear) {
  copyrightYear.textContent = new Date().getFullYear().toString();
}

init();

function viewportWidth() {
  return container?.clientWidth || window.innerWidth;
}

function viewportHeight() {
  return container?.clientHeight || window.innerHeight;
}

function renderPixelRatio() {
  return Math.min(window.devicePixelRatio || 1, maxPixelRatio);
}

function renderResolution() {
  const ratio = renderPixelRatio();
  return {
    width: Math.max(1, Math.floor(viewportWidth() * ratio)),
    height: Math.max(1, Math.floor(viewportHeight() * ratio)),
  };
}

function shouldAnimate() {
  return isVisible && isInViewport;
}

function updateAnimationLoop() {
  renderer.setAnimationLoop(shouldAnimate() ? animate : null);
  if (shouldAnimate()) {
    frameAccumulator = 0;
    clock.start();
    render();
  }
}

function updateMaterialResolutions() {
  const resolution = renderResolution();
  crystals.forEach((crystal) => {
    crystal.mesh.material.uniforms.resolution.value.set(
      resolution.width,
      resolution.height,
    );
  });
}

function getFrustumWorldSize(camera, depth = 1000) {
  const fov = camera.fov * (Math.PI / 180); // convert to radians
  const height = Math.tan(fov / 2) * depth;
  const width = height * camera.aspect;
  return { width, height, depth };
}

function init() {
  container = document.querySelector(".emerald-bg");
  if (!container) {
    return;
  }
  camera = new THREE.PerspectiveCamera(
    70,
    viewportWidth() / viewportHeight(),
    1,
    3000,
  );
  camera.position.y = 0;
  camera.position.z = 450;
  scene = new THREE.Scene();
  scene.add(camera);
  renderer = new THREE.WebGLRenderer({
    antialias: false,
    powerPreference: "low-power",
  });
  renderer.setPixelRatio(renderPixelRatio());
  renderer.setSize(viewportWidth(), viewportHeight());
  renderer.autoClear = false;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;

  container.appendChild(renderer.domElement);
  const renderScene = new RenderPass(scene, camera);

  bloomPass = new UnrealBloomPass(
    new THREE.Vector2(viewportWidth(), viewportHeight()),
    isLowPowerDevice ? 0.45 : 0.8,
    0.8,
    0.65,
  );
  bokehPass = new BokehPass(scene, camera, {
    focus: 30,
    aperture: 0.00001,
    maxblur: 0.01,
  });

  const outputPass = new OutputPass();
  const parameters = {
    minFilter: THREE.LinearFilter,
    magFilter: THREE.LinearFilter,
    format: THREE.RGBAFormat,
    type: THREE.FloatType,
  };

  const renderTarget = new THREE.WebGLRenderTarget(
    viewportWidth(),
    viewportHeight(),
    parameters,
  );
  composer = new EffectComposer(renderer, renderTarget);
  composer.setPixelRatio(renderPixelRatio());
  composer.addPass(renderScene);
  if (useBokeh) {
    composer.addPass(bokehPass);
  }
  composer.addPass(bloomPass);
  composer.addPass(outputPass);

  // controls = new OrbitControls(camera, renderer.domElement);

  function randomRot() {
    return Math.random() * Math.PI * 2;
  }

  function showBgGradient(colorSides, colorMiddle, w, h, intensity) {
    // Create a canvas element
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext("2d");

    // Parse hex color to RGB
    function hexToRgb(hex) {
      const r = parseInt(hex.slice(1, 3), 16);
      const g = parseInt(hex.slice(3, 5), 16);
      const b = parseInt(hex.slice(5, 7), 16);
      return { r, g, b };
    }

    // Convert RGB to hex
    function rgbToHex(r, g, b) {
      return `#${[r, g, b].map((x) => x.toString(16).padStart(2, "0")).join("")}`;
    }

    // Convert RGB to HSV
    function rgbToHsv(r, g, b) {
      (r /= 255), (g /= 255), (b /= 255);
      const max = Math.max(r, g, b),
        min = Math.min(r, g, b);
      let h,
        s,
        v = max;
      const d = max - min;
      s = max === 0 ? 0 : d / max;

      if (max === min) {
        h = 0; // achromatic
      } else {
        switch (max) {
          case r:
            h = (g - b) / d + (g < b ? 6 : 0);
            break;
          case g:
            h = (b - r) / d + 2;
            break;
          case b:
            h = (r - g) / d + 4;
            break;
        }
        h /= 6;
      }
      return { h, s, v };
    }

    // Convert HSV to RGB
    function hsvToRgb(h, s, v) {
      let r, g, b;
      const i = Math.floor(h * 6);
      const f = h * 6 - i;
      const p = v * (1 - s);
      const q = v * (1 - f * s);
      const t = v * (1 - (1 - f) * s);

      switch (i % 6) {
        case 0:
          (r = v), (g = t), (b = p);
          break;
        case 1:
          (r = q), (g = v), (b = p);
          break;
        case 2:
          (r = p), (g = v), (b = t);
          break;
        case 3:
          (r = p), (g = q), (b = v);
          break;
        case 4:
          (r = t), (g = p), (b = v);
          break;
        case 5:
          (r = v), (g = p), (b = q);
          break;
      }

      return {
        r: Math.round(r * 255),
        g: Math.round(g * 255),
        b: Math.round(b * 255),
      };
    }

    // Lighten color in HSV space (increases Value while preserving Hue and Saturation)
    function lightenHsv(rgb, factor) {
      const hsv = rgbToHsv(rgb.r, rgb.g, rgb.b);
      // Increase value (brightness) while keeping hue and saturation
      hsv.v = Math.min(1, hsv.v + (1 - hsv.v) * factor);
      return hsvToRgb(hsv.h, hsv.s, hsv.v);
    }

    // Convert middle color to RGB
    const middleRgb = hexToRgb(colorMiddle);

    for (let y = 0; y < h; y++) {
      // Calculate lightness factor (0 at top, 1 at bottom)
      const lightnessFactor = (y / h) * intensity;

      // Lighten the middle color in HSV space
      const lightenedRgb = lightenHsv(middleRgb, lightnessFactor);
      const currentMiddleColor = rgbToHex(
        lightenedRgb.r,
        lightenedRgb.g,
        lightenedRgb.b,
      );

      // Create gradient for this scanline
      const gradient = ctx.createLinearGradient(0, y, w, y);
      gradient.addColorStop(0, colorSides);
      gradient.addColorStop(0.5, currentMiddleColor);
      gradient.addColorStop(1, colorSides);

      // Draw this scanline
      ctx.fillStyle = gradient;
      ctx.fillRect(0, y, w, 1);
    }

    return canvas;
  }

  const colors = [];
  for (let i = 0; i < 6; i++) {
    const color = showBgGradient("#122804", "#404040", 512, 512, 1 / (i + 1));
    colors.push(color);
  }
  const bgColor = showBgGradient("#080215", "#09011b", 512, 512, 0.025);
  const textureCube = new THREE.CubeTexture(colors);
  textureCube.needsUpdate = true;
  scene.background = new THREE.CanvasTexture(bgColor);
  scene.background.needsUpdate = true;
  const loader = new GLTFLoader();
  loader.load("/dflat.glb", function (gltf) {
    let diamond = null;
    gltf.scene.traverse(function (child) {
      if (child.name == "Diamond_1_0") {
        diamond = child;
      }
    });
    if (!diamond) {
      return;
    }

    const diamondGeometry = diamond.geometry;
    const boundsTree = new MeshBVH(diamondGeometry.toNonIndexed(), {
      lazyGeneration: false,
      strategy: SAH,
    });
    const collider = new THREE.Mesh(diamondGeometry);
    collider.visible = false;
    collider.boundsTree = boundsTree;

    const rotSpeed = () => {
      return Math.random() * 0.005 - 0.0025;
    };
    const colors = [new THREE.Color(1, 1, 1), new THREE.Color(1, 1, 3)];
    for (let layer = 0; layer < 3; layer++) {
      for (let i = 0; i < crystalsPerLayer; i++) {
        const color = colors[Math.round(Math.random() * (colors.length - 1))];
        const ior = 2.4;
        const resolution = renderResolution();

        const material = new THREE.ShaderMaterial({
          uniforms: {
            envMap: { value: textureCube },
            bvh: { value: new MeshBVHUniformStruct() },
            bounces: { value: bounceCount },
            color: { value: color },
            ior: { value: ior },
            correctMips: { value: true },
            projectionMatrixInv: { value: camera.projectionMatrixInverse },
            viewMatrixInv: { value: camera.matrixWorld },
            chromaticAberration: { value: false },
            aberrationStrength: { value: 0.01 },
            resolution: {
              value: new THREE.Vector2(resolution.width, resolution.height),
            },
          },
          vertexShader: /*glsl*/ `
            varying vec3 vWorldPosition;
            varying vec3 vNormal;
            uniform mat4 viewMatrixInv;
            void main() {
                vWorldPosition = (modelMatrix * vec4(position, 1.0)).xyz;
                vNormal = (viewMatrixInv * vec4(normalMatrix * normal, 0.0)).xyz;
                gl_Position = projectionMatrix * viewMatrix * modelMatrix * vec4(position, 1.0);
            }
            `,
          fragmentShader: /*glsl*/ `
            precision highp isampler2D;
            precision highp usampler2D;
            varying vec3 vWorldPosition;
            varying vec3 vNormal;
            uniform samplerCube envMap;
            uniform float bounces;
            ${shaderStructs}
            ${shaderIntersectFunction}
            uniform BVH bvh;
            uniform float ior;
            uniform vec3 color;
            uniform bool correctMips;
            uniform bool chromaticAberration;
            uniform mat4 projectionMatrixInv;
            uniform mat4 viewMatrixInv;
            uniform mat4 modelMatrix;
            uniform vec2 resolution;
            uniform bool chromaticAbberation;
            uniform float aberrationStrength;
            vec3 totalInternalReflection(vec3 ro, vec3 rd, vec3 normal, float ior, mat4 modelMatrixInverse) {
                vec3 rayOrigin = ro;
                vec3 rayDirection = rd;
                rayDirection = refract(rayDirection, normal, 1.0 / ior);
                rayOrigin = vWorldPosition + rayDirection * 0.001;
                rayOrigin = (modelMatrixInverse * vec4(rayOrigin, 1.0)).xyz;
                rayDirection = normalize((modelMatrixInverse * vec4(rayDirection, 0.0)).xyz);
                for(float i = 0.0; i < bounces; i++) {
                    uvec4 faceIndices = uvec4( 0u );
                    vec3 faceNormal = vec3( 0.0, 0.0, 1.0 );
                    vec3 barycoord = vec3( 0.0 );
                    float side = 1.0;
                    float dist = 0.0;
                    bvhIntersectFirstHit( bvh, rayOrigin, rayDirection, faceIndices, faceNormal, barycoord, side, dist );
                    vec3 hitPos = rayOrigin + rayDirection * max(dist - 0.001, 0.0);
                    faceNormal *= side;
                    vec3 tempDir = refract(rayDirection, faceNormal, ior);
                    if (length(tempDir) != 0.0) {
                        rayDirection = tempDir;
                        break;
                    }
                    rayDirection = reflect(rayDirection, faceNormal);
                    rayOrigin = hitPos + rayDirection * 0.01;
                }
                rayDirection = normalize((modelMatrix * vec4(rayDirection, 0.0)).xyz);
                return rayDirection;
            }
            void main() {
                mat4 modelMatrixInverse = inverse(modelMatrix);
                vec2 uv = gl_FragCoord.xy / resolution;
                vec3 directionCamPerfect = (projectionMatrixInv * vec4(uv * 2.0 - 1.0, 0.0, 1.0)).xyz;
                directionCamPerfect = (viewMatrixInv * vec4(directionCamPerfect, 0.0)).xyz;
                directionCamPerfect = normalize(directionCamPerfect);
                vec3 normal = vNormal;
                vec3 rayOrigin = cameraPosition;
                vec3 rayDirection = normalize(vWorldPosition - cameraPosition);
                vec3 finalColor;
                if (chromaticAberration) {
                vec3 rayDirectionR = totalInternalReflection(rayOrigin, rayDirection, normal, max(ior * (1.0 - aberrationStrength), 1.0), modelMatrixInverse);
                vec3 rayDirectionG = totalInternalReflection(rayOrigin, rayDirection, normal, max(ior, 1.0), modelMatrixInverse);
                vec3 rayDirectionB = totalInternalReflection(rayOrigin, rayDirection, normal, max(ior * (1.0 + aberrationStrength), 1.0), modelMatrixInverse);
                float finalColorR = textureGrad(envMap, rayDirectionR, dFdx(correctMips ? directionCamPerfect: rayDirection), dFdy(correctMips ? directionCamPerfect: rayDirection)).r;
                float finalColorG = textureGrad(envMap, rayDirectionG, dFdx(correctMips ? directionCamPerfect: rayDirection), dFdy(correctMips ? directionCamPerfect: rayDirection)).g;
                float finalColorB = textureGrad(envMap, rayDirectionB, dFdx(correctMips ? directionCamPerfect: rayDirection), dFdy(correctMips ? directionCamPerfect: rayDirection)).b;
                finalColor = vec3(finalColorR, finalColorG, finalColorB) * color;
                } else {
                    rayDirection = totalInternalReflection(rayOrigin, rayDirection, normal, max(ior, 1.0), modelMatrixInverse);
                    finalColor = textureGrad(envMap, rayDirection, dFdx(correctMips ? directionCamPerfect: rayDirection), dFdy(correctMips ? directionCamPerfect: rayDirection)).rgb;
                    finalColor *= color;
                }
                gl_FragColor = vec4(vec3(finalColor), 1.0);
            }
            `,
        });

        const mesh = new THREE.Mesh(diamondGeometry, material);
        mesh.material.uniforms.bvh.value.updateFrom(collider.boundsTree);

        const camDist = 400; // place diamonds in front of camera at approx this depth
        const frustum = getFrustumWorldSize(camera, camDist);

        mesh.position.x = (Math.random() - 0.5) * frustum.width * 1.5;
        mesh.position.y = (Math.random() - 0.5) * frustum.height * 1.5;
        mesh.position.z = -camDist / 2 + layer * 0.5 * camDist; // spread them out along Z axis in front of camera

        mesh.rotation.x = randomRot();
        mesh.rotation.y = randomRot();
        mesh.rotation.z = randomRot();
        mesh.scale.multiplyScalar(10 + Math.random() * 20);

        if (frontMostCrystal == null && layer == 2) {
          frontMostCrystal = mesh;
        }

        crystals.push({
          rotationSpeed: {
            x: rotSpeed(),
            y: rotSpeed(),
            z: rotSpeed(),
          },
          mesh,
        });
        scene.add(mesh);
      }
    }
  });

  // stats = new Stats();
  // container.appendChild(stats.dom);
  // container.style.touchAction = "none";
  window.addEventListener("resize", onWindowResize);
  document.addEventListener("visibilitychange", () => {
    isVisible = document.visibilityState === "visible";
    updateAnimationLoop();
  });
  if ("IntersectionObserver" in window) {
    new IntersectionObserver(
      ([entry]) => {
        isInViewport = entry?.isIntersecting ?? true;
        updateAnimationLoop();
      },
      { threshold: 0 },
    ).observe(container);
  }
  updateAnimationLoop();
}

function onWindowResize() {
  const width = viewportWidth();
  const height = viewportHeight();
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
  renderer.setPixelRatio(renderPixelRatio());
  composer.setPixelRatio(renderPixelRatio());
  renderer.setSize(width, height);
  composer.setSize(width, height);
  updateMaterialResolutions();
}

function crystalsRotate(delta) {
  crystals.forEach((crystal) => {
    const frameScale = delta * 60;
    crystal.mesh.rotation.x += crystal.rotationSpeed.x * frameScale;
    crystal.mesh.rotation.y += crystal.rotationSpeed.y * frameScale;
    crystal.mesh.rotation.z += crystal.rotationSpeed.z * frameScale;
  });
}

function animate() {
  const delta = clock.getDelta();
  frameAccumulator += delta;
  if (frameAccumulator < frameInterval) {
    return;
  }
  const elapsed = frameAccumulator;
  frameAccumulator = 0;
  crystalsRotate(elapsed);
  render();
  // stats.update();
}

function updateBokeh() {
  if (frontMostCrystal != null) {
    const targetDistance = frontMostCrystal.position.distanceTo(
      camera.position,
    );
    bokehPass.uniforms["focus"].value = targetDistance;
  }
}

function render() {
  updateBokeh();
  composer.render();
}
