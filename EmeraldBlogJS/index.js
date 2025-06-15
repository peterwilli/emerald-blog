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
const postprocessing = {
  enabled: true,
};
init();

function getFrustumWorldSize(camera, depth = 1000) {
  const fov = camera.fov * (Math.PI / 180); // convert to radians
  const height = Math.tan(fov / 2) * depth;
  const width = height * camera.aspect;
  return { width, height, depth };
}

function init() {
  container = document.querySelector(".emerald-bg");
  camera = new THREE.PerspectiveCamera(
    70,
    window.innerWidth / window.innerHeight,
    1,
    3000,
  );
  camera.position.y = 0;
  camera.position.z = 450;
  scene = new THREE.Scene();
  scene.add(camera);
  renderer = new THREE.WebGLRenderer();
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setAnimationLoop(animate);
  renderer.autoClear = false;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;

  container.appendChild(renderer.domElement);
  const renderScene = new RenderPass(scene, camera);

  bloomPass = new UnrealBloomPass(
    new THREE.Vector2(window.innerWidth, window.innerHeight),
    1.0,
    0.9,
    0.6,
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
    window.innerWidth,
    window.innerHeight,
    parameters,
  );
  composer = new EffectComposer(renderer, renderTarget);
  composer.addPass(renderScene);
  composer.addPass(bokehPass);
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
    const rotSpeed = () => {
      return Math.random() * 0.005 - 0.0025;
    };
    const colors = [new THREE.Color(1, 1, 1), new THREE.Color(1, 1, 3)];
    for (let layer = 0; layer < 3; layer++) {
      for (let i = 0; i < 10; i++) {
        const color = colors[Math.round(Math.random() * (colors.length - 1))];
        const ior = 2.4;
        const clientWidth = window.innerWidth;
        const clientHeight = window.innerHeight;

        const material = new THREE.ShaderMaterial({
          uniforms: {
            envMap: { value: textureCube },
            bvh: { value: new MeshBVHUniformStruct() },
            bounces: { value: 3 },
            color: { value: color },
            ior: { value: ior },
            correctMips: { value: true },
            projectionMatrixInv: { value: camera.projectionMatrixInverse },
            viewMatrixInv: { value: camera.matrixWorld },
            chromaticAberration: { value: false },
            aberrationStrength: { value: 0.01 },
            resolution: { value: new THREE.Vector2(clientWidth, clientHeight) },
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

        const mergedGeometry = diamond.geometry;
        mergedGeometry.boundsTree = new MeshBVH(mergedGeometry.toNonIndexed(), {
          lazyGeneration: false,
          strategy: SAH,
        });
        const collider = new THREE.Mesh(mergedGeometry);
        collider.material.wireframe = true;
        collider.material.opacity = 0.5;
        collider.material.transparent = true;
        collider.visible = false;
        collider.boundsTree = mergedGeometry.boundsTree;
        scene.add(collider);

        const mesh = new THREE.Mesh(diamond.geometry, material);
        mesh.castShadow = true;
        mesh.receiveShadow = true;
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
}

function onWindowResize() {
  camera.aspect = window.innerWidth / window.innerHeight;
  camera.updateProjectionMatrix();
  windowHalfX = window.innerWidth / 2;
  windowHalfY = window.innerHeight / 2;
  postprocessing.rtTextureDepth.setSize(window.innerWidth, window.innerHeight);
  postprocessing.rtTextureColor.setSize(window.innerWidth, window.innerHeight);
  postprocessing.bokeh_uniforms["textureWidth"].value = window.innerWidth;
  postprocessing.bokeh_uniforms["textureHeight"].value = window.innerHeight;
  renderer.setSize(window.innerWidth, window.innerHeight);
}

function crystalsRotate(delta) {
  crystals.forEach((crystal) => {
    crystal.mesh.rotation.x += crystal.rotationSpeed.x;
    crystal.mesh.rotation.y += crystal.rotationSpeed.y;
    crystal.mesh.rotation.z += crystal.rotationSpeed.z;
  });
}

function animate() {
  const delta = clock.getDelta();
  crystalsRotate(delta);
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
