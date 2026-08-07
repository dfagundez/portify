#!/usr/bin/env node
/**
 * Generates the Portify source icons.
 *
 * Written from scratch rather than pulled from a design file so the mark is
 * reproducible in CI and on any machine with Node — no ImageMagick, no Python
 * imaging stack, no binary blob whose provenance nobody remembers.
 *
 *   node scripts/generate-icon.mjs
 *
 * Produces:
 *   assets/icon.png       1024×1024 full-colour app icon (source for `tauri icon`)
 *   assets/tray-mono.png  512×512 monochrome mark for template-style tray icons
 *
 * The shape is a squircle with a white "P" built from a stem rectangle and a
 * half-annulus bowl whose centre sits exactly on the stem's right edge, so the
 * two join without a seam. Everything is rendered with 4× supersampling.
 */

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SAMPLES = 4; // per axis, so 16 samples per pixel

/* ---------- geometry, expressed in a 1024×1024 space ---------- */

const CANVAS = 1024;
const CORNER_RADIUS = 210;

// Stroke weight is the whole game here. A tray icon is rendered at 16px, so a
// stroke thinner than ~12% of the canvas lands under 2px and the letter turns
// to mush. These give a 140/1024 = 13.7% stroke: about 2.2px in the tray, still
// balanced at 1024. The glyph is centred as a whole: 370×610 inside 1024².
const STROKE = 140;
const STEM = { x0: 327, x1: 327 + STROKE, y0: 207, y1: 817 };
// The bowl's centre sits exactly on the stem's right edge, so the ring meets the
// stem with no seam and no overlap artefact.
const BOWL = { cx: STEM.x1, cy: 437, inner: 90, outer: 90 + STROKE };

/** Signed coverage test for the rounded square. */
function inSquircle(x, y) {
  const r = CORNER_RADIUS;
  const nx = Math.min(Math.max(x, r), CANVAS - r);
  const ny = Math.min(Math.max(y, r), CANVAS - r);
  const dx = x - nx;
  const dy = y - ny;
  return dx * dx + dy * dy <= r * r;
}

/** The letter P: stem plus the right half of an annulus. */
function inLetter(x, y) {
  if (x >= STEM.x0 && x <= STEM.x1 && y >= STEM.y0 && y <= STEM.y1) return true;

  if (x >= BOWL.cx) {
    const dx = x - BOWL.cx;
    const dy = y - BOWL.cy;
    const distance = Math.sqrt(dx * dx + dy * dy);
    if (distance >= BOWL.inner && distance <= BOWL.outer) return true;
  }
  return false;
}

/**
 * Vertical gradient.
 *
 * Both stops sit on either side of the interface accent (#2f6fed), so the icon
 * and the app read as the same product. The earlier blue→violet ramp was
 * inherited from the Python version and matched nothing in the UI.
 */
const GRADIENT = { from: [0x4d, 0x84, 0xf5], to: [0x1b, 0x4f, 0xd1] };

function background(y) {
  const t = y / CANVAS;
  return GRADIENT.from.map((from, channel) =>
    Math.round(from + (GRADIENT.to[channel] - from) * t),
  );
}

/* ---------- rasteriser ---------- */

/**
 * @param {number} size output size in pixels
 * @param {"color"|"mono"} mode colour icon, or black mark on transparency
 */
function render(size, mode) {
  const scale = CANVAS / size;
  const pixels = Buffer.alloc(size * size * 4);

  for (let py = 0; py < size; py++) {
    for (let px = 0; px < size; px++) {
      let insideShape = 0;
      let insideLetter = 0;

      for (let sy = 0; sy < SAMPLES; sy++) {
        for (let sx = 0; sx < SAMPLES; sx++) {
          const x = (px + (sx + 0.5) / SAMPLES) * scale;
          const y = (py + (sy + 0.5) / SAMPLES) * scale;
          if (inLetter(x, y)) insideLetter++;
          else if (inSquircle(x, y)) insideShape++;
        }
      }

      const total = SAMPLES * SAMPLES;
      const letter = insideLetter / total;
      const shape = insideShape / total;
      const offset = (py * size + px) * 4;

      if (mode === "mono") {
        // Template icons are a single colour plus alpha; the OS recolours them.
        pixels[offset] = 0;
        pixels[offset + 1] = 0;
        pixels[offset + 2] = 0;
        pixels[offset + 3] = Math.round(letter * 255);
        continue;
      }

      const [r, g, b] = background((py + 0.5) * scale);
      const alpha = letter + shape;
      if (alpha <= 0) continue;

      // Composite the white letter over the gradient, weighted by coverage.
      pixels[offset] = Math.round((letter * 255 + shape * r) / alpha);
      pixels[offset + 1] = Math.round((letter * 255 + shape * g) / alpha);
      pixels[offset + 2] = Math.round((letter * 255 + shape * b) / alpha);
      pixels[offset + 3] = Math.round(Math.min(alpha, 1) * 255);
    }
  }

  return encodePng(size, size, pixels);
}

/* ---------- minimal PNG encoder ---------- */

const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(buffer) {
  let crc = -1;
  for (const byte of buffer) crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  return (crc ^ -1) >>> 0;
}

function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length);
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body));
  return Buffer.concat([length, body, crc]);
}

function encodePng(width, height, rgba) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header[8] = 8; // bit depth
  header[9] = 6; // colour type: RGBA
  header[10] = 0; // deflate
  header[11] = 0; // adaptive filtering
  header[12] = 0; // no interlace

  // One filter byte (0 = None) per scanline.
  const raw = Buffer.alloc(height * (width * 4 + 1));
  for (let y = 0; y < height; y++) {
    raw[y * (width * 4 + 1)] = 0;
    rgba.copy(raw, y * (width * 4 + 1) + 1, y * width * 4, (y + 1) * width * 4);
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", header),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

/* ---------- entry point ---------- */

const assets = join(ROOT, "assets");
mkdirSync(assets, { recursive: true });

const targets = [
  ["icon.png", 1024, "color"],
  ["tray-mono.png", 512, "mono"],
];

for (const [name, size, mode] of targets) {
  const png = render(size, mode);
  writeFileSync(join(assets, name), png);
  console.log(`wrote assets/${name} (${size}×${size}, ${(png.length / 1024).toFixed(1)} KB)`);
}
