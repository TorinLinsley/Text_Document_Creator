import fs from 'node:fs';
import zlib from 'node:zlib';
import path from 'node:path';

const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function paeth(a, b, c) {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  if (pa <= pb && pa <= pc) return a;
  return pb <= pc ? b : c;
}

function decodePng(buf) {
  let off = 8;
  let w = 0;
  let h = 0;
  let depth = 0;
  let ctype = 0;
  const idat = [];
  while (off < buf.length) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString('ascii', off + 4, off + 8);
    const data = buf.subarray(off + 8, off + 8 + len);
    if (type === 'IHDR') {
      w = data.readUInt32BE(0);
      h = data.readUInt32BE(4);
      depth = data[8];
      ctype = data[9];
    } else if (type === 'IDAT') {
      idat.push(data);
    }
    off += 12 + len;
  }
  if (depth !== 8 || (ctype !== 6 && ctype !== 2)) throw new Error('unsupported png');
  const bpp = ctype === 6 ? 4 : 3;
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const stride = w * bpp;
  const out = new Uint8Array(w * h * 4);
  let prev = new Uint8Array(stride);
  let pos = 0;
  for (let y = 0; y < h; y++) {
    const filter = raw[pos++];
    const line = new Uint8Array(stride);
    for (let x = 0; x < stride; x++) line[x] = raw[pos + x];
    pos += stride;
    for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? line[x - bpp] : 0;
      const b = prev[x];
      const c = x >= bpp ? prev[x - bpp] : 0;
      let v = line[x];
      if (filter === 1) v = (v + a) & 0xff;
      else if (filter === 2) v = (v + b) & 0xff;
      else if (filter === 3) v = (v + ((a + b) >> 1)) & 0xff;
      else if (filter === 4) v = (v + paeth(a, b, c)) & 0xff;
      line[x] = v;
    }
    for (let x = 0; x < w; x++) {
      const i = (y * w + x) * 4;
      const s = x * bpp;
      out[i] = line[s];
      out[i + 1] = line[s + 1];
      out[i + 2] = line[s + 2];
      out[i + 3] = bpp === 4 ? line[s + 3] : 255;
    }
    prev = line;
  }
  return { w, h, data: out };
}

function resize(src, sw, sh, dw, dh) {
  const out = new Uint8Array(dw * dh * 4);
  const rx = sw / dw;
  const ry = sh / dh;
  for (let y = 0; y < dh; y++) {
    const y0 = y * ry;
    const y1 = Math.min(sh, (y + 1) * ry);
    for (let x = 0; x < dw; x++) {
      const x0 = x * rx;
      const x1 = Math.min(sw, (x + 1) * rx);
      let r = 0, g = 0, b = 0, a = 0, wsum = 0;
      for (let sy = Math.floor(y0); sy < Math.ceil(y1); sy++) {
        const wy = Math.min(y1, sy + 1) - Math.max(y0, sy);
        for (let sx = Math.floor(x0); sx < Math.ceil(x1); sx++) {
          const wx = Math.min(x1, sx + 1) - Math.max(x0, sx);
          const wt = wx * wy;
          const i = (sy * sw + sx) * 4;
          const al = src[i + 3] / 255;
          r += src[i] * al * wt;
          g += src[i + 1] * al * wt;
          b += src[i + 2] * al * wt;
          a += src[i + 3] * wt;
          wsum += wt;
        }
      }
      const o = (y * dw + x) * 4;
      const alpha = a / wsum;
      const k = alpha > 0 ? 255 / alpha : 0;
      out[o] = Math.round(Math.min(255, (r / wsum) * k));
      out[o + 1] = Math.round(Math.min(255, (g / wsum) * k));
      out[o + 2] = Math.round(Math.min(255, (b / wsum) * k));
      out[o + 3] = Math.round(alpha);
    }
  }
  return out;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const body = Buffer.concat([Buffer.from(type, 'ascii'), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(body), 0);
  return Buffer.concat([len, body, crc]);
}

function png(buf, s) {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(s, 0);
  ihdr.writeUInt32BE(s, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  const raw = Buffer.alloc(s * (s * 4 + 1));
  for (let y = 0; y < s; y++) {
    raw[y * (s * 4 + 1)] = 0;
    Buffer.from(buf.buffer, buf.byteOffset + y * s * 4, s * 4).copy(raw, y * (s * 4 + 1) + 1);
  }
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', ihdr),
    chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

function dib(buf, s) {
  const xor = s * 4;
  const andRow = Math.ceil(s / 8) + ((4 - (Math.ceil(s / 8) % 4)) % 4);
  const b = Buffer.alloc(40 + xor * s + andRow * s);
  b.writeUInt32LE(40, 0);
  b.writeInt32LE(s, 4);
  b.writeInt32LE(s * 2, 8);
  b.writeUInt16LE(1, 12);
  b.writeUInt16LE(32, 14);
  b.writeUInt32LE(0, 16);
  b.writeUInt32LE(xor * s, 20);
  let o = 40;
  for (let y = s - 1; y >= 0; y--) {
    for (let x = 0; x < s; x++) {
      const i = (y * s + x) * 4;
      b[o++] = buf[i + 2];
      b[o++] = buf[i + 1];
      b[o++] = buf[i];
      b[o++] = buf[i + 3];
    }
  }
  return b;
}

const srcPath = process.argv[2];
const outPath = process.argv[3];
const src = decodePng(fs.readFileSync(srcPath));
const sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256];
const entries = sizes.map((s) => {
  const px = s === src.w ? src.data : resize(src.data, src.w, src.h, s, s);
  return { s, data: s >= 256 ? png(px, s) : dib(px, s) };
});

const header = Buffer.alloc(6);
header.writeUInt16LE(0, 0);
header.writeUInt16LE(1, 2);
header.writeUInt16LE(entries.length, 4);
const dir = Buffer.alloc(16 * entries.length);
let offset = 6 + 16 * entries.length;
entries.forEach((e, idx) => {
  const o = idx * 16;
  dir[o] = e.s >= 256 ? 0 : e.s;
  dir[o + 1] = e.s >= 256 ? 0 : e.s;
  dir.writeUInt16LE(1, o + 4);
  dir.writeUInt16LE(32, o + 6);
  dir.writeUInt32LE(e.data.length, o + 8);
  dir.writeUInt32LE(offset, o + 12);
  offset += e.data.length;
});
fs.writeFileSync(outPath, Buffer.concat([header, dir, ...entries.map((e) => e.data)]));
console.log('icon.ico ->', sizes.join(','));
