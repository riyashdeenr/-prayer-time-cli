import wasmModule from './mawaqit_worker.wasm';

let wasmInstance = null;

const fallbackProxy = new Proxy({}, {
  get: () => () => 0
});

const importObjectProxy = new Proxy({}, {
  get: () => fallbackProxy
});

async function getWasm() {
  if (!wasmInstance) {
    const instance = await WebAssembly.instantiate(wasmModule, importObjectProxy);
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

function ansiToHtml(str) {
  // Convert ANSI color escape codes into styled HTML spans
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\x1b\[38;2;46;204;113m/g, '<span class="text-emerald-400 font-bold">')
    .replace(/\x1b\[1;32m/g, '<span class="text-emerald-400 font-bold">')
    .replace(/\x1b\[1;33m/g, '<span class="text-amber-400 font-bold">')
    .replace(/\x1b\[1;36m/g, '<span class="text-cyan-400 font-bold">')
    .replace(/\x1b\[90m/g, '<span class="text-zinc-500">')
    .replace(/\x1b\[37m/g, '<span class="text-zinc-200">')
    .replace(/\x1b\[0m/g, '</span>');
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

      const nowSec = BigInt(Math.floor(Date.now() / 1000));

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
      const isRawQuery = url.searchParams.get('raw') === 'true' || request.headers.get('x-terminal') === 'true';
      const isTerminal = isRawQuery || /curl|httpie|wget|fetch|aria2|powershell|invoke-webrequest|invoke-restmethod|irm|iwr/i.test(userAgent);
      
      // If requested via terminal or internal fetch, return raw text/plain
      if (isTerminal) {

        return new Response(body, {
          status: 200,
          headers: {
            'content-type': 'text/plain; charset=utf-8',
            'cache-control': 'public, max-age=60, s-maxage=60',
            'access-control-allow-origin': '*',
            'x-powered-by': 'mawaqit-wasm-edge'
          }
        });
      }

      // If requested via browser, render Option 5 Interactive Web TUI Simulator (bio.rahmanr.com aesthetic)
      const initialHtmlOutput = ansiToHtml(body);
      const initialCity = city || 'Bedok New Town';

      const html = `<!DOCTYPE html>
<html lang="en" class="bg-black text-zinc-100">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>mawaqit (مواقيت) - Universal Terminal Prayer Engine</title>
    <!-- Tailwind CSS -->
    <script src="https://cdn.tailwindcss.com"></script>
    <script>
        tailwind.config = {
            theme: {
                extend: {
                    colors: {
                        pitchBlack: '#000000',
                    }
                }
            }
        };
    </script>
    <style>
        :root {
            --bg-pitch: #000000;
            --bg-card: rgba(10, 10, 14, 0.95);
            --border-subtle: rgba(255, 255, 255, 0.08);
            --border-hover: rgba(245, 158, 11, 0.5);
        }
        body {
            background-color: var(--bg-pitch);
            font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", "Segoe UI Symbol", monospace;
            font-feature-settings: "tnum" 1;
        }
        pre {
            font-family: inherit;
            font-feature-settings: inherit;
        }
        .pill-btn {
            background: rgba(24, 24, 27, 0.7);
            border: 1px solid rgba(63, 63, 70, 0.5);
            color: #d4d4d8;
            transition: all 0.15s ease;
            cursor: pointer;
            user-select: none;
        }
        .pill-btn:hover {
            border-color: rgba(245, 158, 11, 0.6);
            color: #fbbf24;
            background: rgba(39, 39, 42, 0.8);
        }
        .term-scroll::-webkit-scrollbar {
            width: 6px;
        }
        .term-scroll::-webkit-scrollbar-track {
            background: #09090b;
        }
        .term-scroll::-webkit-scrollbar-thumb {
            background: #27272a;
            border-radius: 3px;
        }
    </style>
</head>
<body class="min-h-screen flex flex-col justify-between p-4 sm:p-6 md:p-8 selection:bg-amber-500/20 selection:text-amber-300">
    <div class="max-w-4xl w-full mx-auto space-y-6">

        <!-- Top Header & Copy Banner -->
        <div class="text-center space-y-3">
            <h1 class="text-xl sm:text-2xl font-extrabold tracking-tight text-white flex items-center justify-center gap-2">
                <span>MAWAQIT</span>
                <span class="text-amber-400 font-normal text-lg sm:text-xl">مواقيت</span>
                <span class="text-xs px-2 py-0.5 rounded bg-zinc-900 border border-zinc-800 text-zinc-400 font-mono">EDGE WASM</span>
            </h1>
            <p class="text-xs text-zinc-400 font-mono">Universal Terminal Prayer & Celestial Ephemeris Engine</p>

            <!-- Dynamic Copy curl Banner (matching bio.rahmanr.com hero tag) -->
            <div class="inline-flex items-center gap-3 px-3.5 py-1.5 rounded-lg bg-zinc-950 border border-zinc-800 text-zinc-300 text-xs font-mono max-w-full overflow-hidden shadow-lg">
                <span class="text-emerald-400 font-bold">$</span>
                <span id="curl-cmd-display" class="truncate text-zinc-300">curl https://mawaqit.rahmanr.com</span>
                <button id="copy-btn" onclick="copyCurlCommand()" class="text-amber-400 hover:text-amber-300 font-bold shrink-0 ml-2 px-2 py-0.5 rounded bg-zinc-900 border border-amber-500/30 text-[11px] transition-colors">
                    📋 Copy
                </button>
            </div>
        </div>

        <!-- Interactive Terminal Simulator Window -->
        <div class="rounded-xl bg-black border border-zinc-800 shadow-2xl overflow-hidden font-mono text-xs">
            <!-- Window Bar -->
            <div class="flex items-center justify-between bg-zinc-950 px-4 py-3 border-b border-zinc-800 select-none">
                <div class="flex items-center gap-2">
                    <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
                    <div class="w-3 h-3 rounded-full bg-yellow-500/80"></div>
                    <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
                    <span class="ml-2 text-zinc-500 text-[11px] hidden sm:inline">mawaqit@cloudflare-edge:~</span>
                </div>
                <div class="flex items-center gap-2">
                    <span class="inline-block w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                    <span class="text-emerald-400 text-[10px] font-bold tracking-wider">ONLINE (BEDOK EDGE)</span>
                </div>
            </div>

            <!-- Terminal Output Buffer -->
            <div id="term-output" class="term-scroll p-4 sm:p-6 h-[420px] overflow-y-auto space-y-4 text-zinc-300 leading-relaxed">
                <div class="text-zinc-500 text-[11px] border-b border-zinc-900 pb-2">
                    ⚡ Connected to Cloudflare Edge. Location auto-resolved: <span class="text-amber-400">${initialCity}</span>. Type <strong class="text-amber-400">help</strong> for commands.
                </div>
                <pre class="leading-relaxed whitespace-pre">${initialHtmlOutput}</pre>
            </div>

            <!-- Command Line Input -->
            <div class="p-3.5 bg-zinc-950 border-t border-zinc-900 flex items-center gap-2">
                <span class="text-amber-400 font-bold select-none">mawaqit:~$</span>
                <input id="term-input" type="text" autocomplete="off" spellcheck="false"
                       placeholder="Type city, method, fasting, night, convert, or help..."
                       class="flex-1 bg-transparent border-none outline-none text-zinc-100 placeholder-zinc-600 font-mono text-xs caret-amber-400">
                <button onclick="handleInputSubmit()" class="text-zinc-400 hover:text-white px-2 py-0.5 rounded text-[11px] bg-zinc-900 border border-zinc-800">
                    Run ↵
                </button>
            </div>

            <!-- Action Pills Toolbar -->
            <div class="px-4 py-2.5 bg-zinc-950/80 border-t border-zinc-900/80 flex flex-wrap items-center gap-2 text-[11px]">
                <span class="text-zinc-500 select-none mr-1">Quick:</span>
                <button onclick="executeCommand('help')" class="pill-btn px-2.5 py-1 rounded">[help]</button>
                <button onclick="executeCommand('makkah')" class="pill-btn px-2.5 py-1 rounded">[makkah]</button>
                <button onclick="executeCommand('madhab hanafi')" class="pill-btn px-2.5 py-1 rounded text-amber-300/90 border-amber-500/20">[madhab: hanafi]</button>
                <button onclick="executeCommand('method muis')" class="pill-btn px-2.5 py-1 rounded">[method: muis]</button>
                <button onclick="executeCommand('method egyptian')" class="pill-btn px-2.5 py-1 rounded">[method: egyptian]</button>
                <button onclick="executeCommand('convert 1')" class="pill-btn px-2.5 py-1 rounded">[convert-hijri]</button>
                <button onclick="executeCommand('fasting')" class="pill-btn px-2.5 py-1 rounded text-emerald-300/90 border-emerald-500/20">[fasting]</button>
                <button onclick="executeCommand('night')" class="pill-btn px-2.5 py-1 rounded">[night]</button>
                <button onclick="executeCommand('white-days')" class="pill-btn px-2.5 py-1 rounded">[white-days]</button>
                <button onclick="executeCommand('prohibited')" class="pill-btn px-2.5 py-1 rounded">[duha]</button>
                <button onclick="executeCommand('format %location: %next in %remaining (%moon_symbol %moon_pct)')" class="pill-btn px-2.5 py-1 rounded text-cyan-300/90 border-cyan-500/20">[tmux format]</button>
                <button onclick="executeCommand('clear')" class="pill-btn px-2.5 py-1 rounded text-red-400/80 border-red-500/20">[clear]</button>
            </div>
        </div>

        <!-- Footer Documentation Info -->
        <div class="text-center font-mono text-[11px] text-zinc-600 space-y-1">
            <p>Designed for terminals & status bars: <code>curl mawaqit.rahmanr.com</code></p>
            <p>© 2026 RIYASHDEEN A R | Pure Rust & WebAssembly</p>
        </div>
    </div>

    <!-- Terminal Simulator Logic with Tab-Completion & History -->
    <script>
        const termOutput = document.getElementById('term-output');
        const termInput = document.getElementById('term-input');
        const curlCmdDisplay = document.getElementById('curl-cmd-display');
        const copyBtn = document.getElementById('copy-btn');

        const validCommands = [
            'help', 'makkah', 'fasting', 'night', 'white-days', 'prohibited', 'observances',
            'convert', 'madhab hanafi', 'madhab shafi', 'method muis', 'method mwl',
            'method egyptian', 'method isna', 'method karachi', 'clear'
        ];

        let cmdHistory = [];
        let historyIndex = -1;

        function updateCurlBanner(cmd) {
            let clean = cmd.trim();
            if (!clean || clean === 'clear' || clean === 'help') {
                curlCmdDisplay.textContent = 'curl https://mawaqit.rahmanr.com';
            } else if (clean.startsWith('format ')) {
                const fmt = clean.substring(7);
                curlCmdDisplay.textContent = 'curl -s "https://mawaqit.rahmanr.com?format=' + encodeURIComponent(fmt) + '"';
            } else if (clean.startsWith('method ') || clean.startsWith('madhab ')) {
                const parts = clean.split(' ');
                curlCmdDisplay.textContent = 'curl https://mawaqit.rahmanr.com?' + parts[0] + '=' + parts[1];
            } else {
                curlCmdDisplay.textContent = 'curl https://mawaqit.rahmanr.com/' + clean;
            }
        }

        function copyCurlCommand() {
            const text = curlCmdDisplay.textContent;
            navigator.clipboard.writeText(text).then(() => {
                copyBtn.textContent = '✓ Copied!';
                copyBtn.classList.add('text-emerald-400', 'border-emerald-500');
                setTimeout(() => {
                    copyBtn.textContent = '📋 Copy';
                    copyBtn.classList.remove('text-emerald-400', 'border-emerald-500');
                }, 2000);
            });
        }

        function ansiToHtmlClient(str) {
            return str
                .replace(/&/g, '&amp;')
                .replace(/</g, '&lt;')
                .replace(/>/g, '&gt;')
                .replace(/\\x1b\\[38;2;46;204;113m/g, '<span class="text-emerald-400 font-bold">')
                .replace(/\\x1b\\[1;32m/g, '<span class="text-emerald-400 font-bold">')
                .replace(/\\x1b\\[1;33m/g, '<span class="text-amber-400 font-bold">')
                .replace(/\\x1b\\[1;36m/g, '<span class="text-cyan-400 font-bold">')
                .replace(/\\x1b\\[90m/g, '<span class="text-zinc-500">')
                .replace(/\\x1b\\[37m/g, '<span class="text-zinc-200">')
                .replace(/\\x1b\\[0m/g, '</span>');
        }

        async function executeCommand(cmd) {
            const clean = cmd.trim();
            if (!clean) return;

            // Append command echo to terminal
            const cmdEcho = document.createElement('div');
            cmdEcho.className = 'flex items-center gap-2 text-amber-400 font-bold';
            cmdEcho.innerHTML = '<span class="text-zinc-500">mawaqit:~$</span> <span>' + clean + '</span>';
            termOutput.appendChild(cmdEcho);

            updateCurlBanner(clean);

            if (clean === 'clear' || clean === 'cls') {
                termOutput.innerHTML = '';
                return;
            }

            if (clean === 'save' || clean === '--save') {
                const note = document.createElement('pre');
                note.className = 'text-amber-300 whitespace-pre';
                note.textContent = '[!] Config saving is a local CLI feature (writes to ~/.config/mawaqit/config.toml).\\nRun \\'mawaqit --save\\' in your terminal.';
                termOutput.appendChild(note);
                termOutput.scrollTop = termOutput.scrollHeight;
                return;
            }

            // Build request URL for Wasm edge
            let requestUrl = '/';
            if (clean === 'help') {
                requestUrl = '/:help?raw=true';
            } else if (clean.startsWith('format ')) {
                const fmt = clean.substring(7);
                requestUrl = '/?format=' + encodeURIComponent(fmt) + '&raw=true';
            } else if (clean.startsWith('method ') || clean.startsWith('madhab ')) {
                const parts = clean.split(' ');
                requestUrl = '/?' + parts[0] + '=' + parts[1] + '&raw=true';
            } else if (clean.startsWith('convert')) {
                const parts = clean.split(' ');
                if (parts.length > 1) {
                    requestUrl = '/convert/' + parts[1] + '?raw=true';
                } else {
                    requestUrl = '/convert?raw=true';
                }
            } else {
                const separator = clean.includes('?') ? '&' : '?';
                requestUrl = '/' + clean + separator + 'raw=true';
            }

            try {
                // Fetch directly from edge asking for raw text output
                const resp = await fetch(requestUrl, {
                    headers: { 'x-terminal': 'true' }
                });
                const text = await resp.text();


                const resultBlock = document.createElement('pre');
                resultBlock.className = 'leading-relaxed whitespace-pre';
                resultBlock.innerHTML = ansiToHtmlClient(text);
                termOutput.appendChild(resultBlock);
            } catch (err) {
                const errBlock = document.createElement('pre');
                errBlock.className = 'text-red-400 whitespace-pre';
                errBlock.textContent = 'Error: ' + err.toString();
                termOutput.appendChild(errBlock);
            }

            termOutput.scrollTop = termOutput.scrollHeight;
        }

        function handleInputSubmit() {
            const val = termInput.value.trim();
            if (val) {
                cmdHistory.push(val);
                historyIndex = cmdHistory.length;
                termInput.value = '';
                executeCommand(val);
            }
        }

        termInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                handleInputSubmit();
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                if (historyIndex > 0) {
                    historyIndex--;
                    termInput.value = cmdHistory[historyIndex];
                }
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                if (historyIndex < cmdHistory.length - 1) {
                    historyIndex++;
                    termInput.value = cmdHistory[historyIndex];
                } else {
                    historyIndex = cmdHistory.length;
                    termInput.value = '';
                }
            } else if (e.key === 'Tab') {
                e.preventDefault();
                const cur = termInput.value.trim().toLowerCase();
                if (cur) {
                    const match = validCommands.find(c => c.startsWith(cur));
                    if (match) termInput.value = match;
                }
            }
        });

        // Focus input on click anywhere on terminal
        termOutput.addEventListener('click', () => {
            termInput.focus();
        });
    </script>
</body>
</html>`;

      return new Response(html, {
        status: 200,
        headers: {
          'content-type': 'text/html; charset=utf-8',
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
