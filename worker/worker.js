import wasmModule from './mawaqit_worker.wasm';

let wasmInstance = null;

async function getWasm() {
  if (!wasmInstance) {
    const instance = await WebAssembly.instantiate(wasmModule, {});
    wasmInstance = instance.exports || instance;
  }
  return wasmInstance;
}

function passStringToWasm(wasm, str) {
  if (!str) return 0;
  const encoder = new TextEncoder();
  const bytes = encoder.encode(str + '\0');
  const ptr = wasm.mawaqit_alloc(bytes.length);
  const memory = new Uint8Array(wasm.memory.buffer);
  memory.set(bytes, ptr);
  return ptr;
}

function getStringFromWasm(wasm, ptr) {
  if (!ptr) return '';
  const memory = new Uint8Array(wasm.memory.buffer);
  let end = ptr;
  while (memory[end] !== 0) {
    end++;
  }
  const decoder = new TextDecoder('utf-8');
  const str = decoder.decode(memory.subarray(ptr, end));
  wasm.mawaqit_free_string(ptr);
  return str;
}

export default {
  async fetch(request, env, ctx) {
    try {
      const wasm = await getWasm();
      const url = new URL(request.url);
      const userAgent = request.headers.get('user-agent') || '';

      const cf = request.cf || {};
      const city = cf.city || request.headers.get('cf-ipcity') || null;
      const lat = cf.latitude ? parseFloat(cf.latitude) : 0.0;
      const lon = cf.longitude ? parseFloat(cf.longitude) : 0.0;
      const hasCoords = (cf.latitude && cf.longitude) ? 1 : 0;

      let tzOffsetHours = 8;
      try {
        if (cf.timezone) {
          const d = new Date();
          const str = d.toLocaleString('en-US', { timeZone: cf.timezone, timeZoneName: 'shortOffset' });
          const match = str.match(/GMT([+-]\d+)/);
          if (match) {
            tzOffsetHours = parseInt(match[1], 10);
          }
        }
      } catch (e) {
        tzOffsetHours = 8;
      }

      const nowSec = Date.now() / 1000.0;

      const pathPtr = passStringToWasm(wasm, url.pathname);
      const searchPtr = passStringToWasm(wasm, url.search);
      const uaPtr = passStringToWasm(wasm, userAgent);
      const cityPtr = city ? passStringToWasm(wasm, city) : 0;

      const resPtr = wasm.process_edge_request(
        pathPtr,
        searchPtr,
        uaPtr,
        cityPtr,
        hasCoords,
        lat,
        lon,
        tzOffsetHours,
        nowSec
      );

      const body = getStringFromWasm(wasm, resPtr);

      // Clean up input buffers
      if (pathPtr) wasm.mawaqit_free(pathPtr, url.pathname.length + 1);
      if (searchPtr) wasm.mawaqit_free(searchPtr, url.search.length + 1);
      if (uaPtr) wasm.mawaqit_free(uaPtr, userAgent.length + 1);
      if (cityPtr) wasm.mawaqit_free(cityPtr, city.length + 1);

      const isTerminal = /curl|httpie|wget|fetch|aria2/i.test(userAgent);
      const contentType = isTerminal
        ? 'text/plain; charset=utf-8'
        : 'text/html; charset=utf-8';

      return new Response(body, {
        status: 200,
        headers: {
          'content-type': contentType,
          'cache-control': 'public, max-age=60, s-maxage=60',
          'access-control-allow-origin': '*',
          'x-powered-by': 'mawaqit-wasm-edge'
        }
      });
    } catch (err) {
      return new Response('Edge runtime error: ' + err.toString(), {
        status: 500,
        headers: { 'content-type': 'text/plain; charset=utf-8' }
      });
    }
  }
};
