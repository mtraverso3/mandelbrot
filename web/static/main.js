import init, { difference, maxZoom, normalizeCoordinate, pan, presetNames, presetView, zoomAt as zoomView } from './mandelbrot_web.js';
import { createAutoZoom } from './autozoom.js';

const BASE_VIEW_WIDTH = 3;
const PREVIEW_DIVISOR = 4;
const BAND_PIXELS = 1 << 16;
const DRAG_THRESHOLD = 5;
const CLICK_ZOOM = 2;
const WHEEL_ZOOM_PER_PIXEL = 1.0025;
const WHEEL_SETTLE_MS = 150;
const MAX_ITERATIONS = 10000000;
const DEFAULT_ITERATIONS = 1500;
const DEEP_ZOOM = 1e10;
const GPU_START_TIMEOUT_MS = 5000;
const EXPORT_SCALES = [1, 2, 4, 8];
// Browsers refuse canvases with a longer side than this
const MAX_CANVAS_SIDE = 16384;
const KEY_PAN_FRACTION = 0.1;
// Keyframes are rendered sharper than the screen where the GPU makes that cheap
const GPU_KEYFRAME_SCALE = 1.5;
// Renders that finish sooner go straight to full resolution without flashing the preview
const PREVIEW_DELAY_MS = 150;

const $ = (id) => document.getElementById(id);
const canvas = $('view');
const ctx = canvas.getContext('2d');
const snapshot = document.createElement('canvas');
// The last finished render, which zooming stretches until the next one is ready
const rendered = { canvas: document.createElement('canvas'), view: null };
let lastRenderSeconds = 0;
const selection = $('selection');

const state = {
    view: null,
    preset: 'mandelbrot',
    iterations: DEFAULT_ITERATIONS,
    autoIterations: true,
    shading: 'normal',
    backend: 'gpu',
    historyIndex: 0,
    historyLength: 1,
};

function showError(message) {
    const error = $('error');
    error.textContent = message;
    error.hidden = false;
}

const workers = [];
const idle = [];
let queue = [];
let generation = 0;
let job = null;

function startWorkers() {
    const count = Math.min(navigator.hardwareConcurrency || 4, 16);
    for (let i = 0; i < count; i++) {
        const worker = new Worker(new URL('worker.js', import.meta.url), { type: 'module' });
        worker.onmessage = ({ data }) => onBand(worker, data);
        worker.onerror = (event) => showError(`Render worker failed: ${event.message}`);
        workers.push(worker);
        idle.push(worker);
    }
}

// Renders through WebGPU in its own worker when the browser supports it
const gpu = { worker: null, adapter: '', ready: false };

function startGpu() {
    if (!('gpu' in navigator)) return Promise.resolve();
    return new Promise((resolve) => {
        const worker = new Worker(new URL('gpu-worker.js', import.meta.url), { type: 'module' });
        const settle = (message) => {
            clearTimeout(timer);
            if (message?.kind === 'ready') {
                Object.assign(gpu, { worker, adapter: message.adapter, ready: true });
                worker.onmessage = ({ data }) => onGpuMessage(data);
            } else {
                if (message?.reason) console.warn(`WebGPU unavailable: ${message.reason}`);
                worker.terminate();
            }
            resolve();
        };
        const timer = setTimeout(settle, GPU_START_TIMEOUT_MS);
        worker.onmessage = ({ data }) => settle(data);
        worker.onerror = () => settle();
    });
}

function usesGpu() {
    return gpu.ready && state.backend === 'gpu';
}

function onGpuMessage(message) {
    if (message.generation !== generation) return;
    if (message.kind === 'band') {
        drawBand(message);
    } else if (message.kind === 'iterations') {
        onIterations(message.iterations);
    } else if (message.kind === 'failed') {
        console.warn(`GPU render failed, switching to the CPU: ${message.reason}`);
        gpu.ready = false;
        updateControls();
        if (job.choosing) {
            chooseIterations();
        } else {
            job.onGpu = false;
            dispatchJob();
        }
    }
}

function dispatch() {
    while (idle.length && queue.length) {
        idle.pop().postMessage(queue.shift());
    }
}

// Enough bands for every worker even in small renders, which are often the slow ones
const BANDS_PER_WORKER = 4;

function bandTasks(pass, width, height) {
    const fewest = Math.ceil(height / (workers.length * BANDS_PER_WORKER));
    const rowsPerBand = Math.max(1, Math.min(Math.floor(BAND_PIXELS / width), fewest));
    const tasks = [];
    for (let firstRow = 0; firstRow < height; firstRow += rowsPerBand) {
        tasks.push({
            generation,
            pass,
            view: job.view,
            width,
            height,
            iterations: state.iterations,
            normal: state.shading === 'normal',
            firstRow,
            rowCount: Math.min(rowsPerBand, height - firstRow),
        });
    }
    const distance = (t) => Math.abs(t.firstRow + t.rowCount / 2 - height / 2);
    return tasks.sort((a, b) => distance(a) - distance(b));
}

// Drops whatever is rendering, so no band of an old view lands on a new one
function stopRendering() {
    generation++;
    queue = [];
    job = null;
    if (gpu.ready) gpu.worker.postMessage({ kind: 'cancel' });
}

function render() {
    stopAutoZoom();
    stopRendering();
    withIterations(state.view, canvas.width, canvas.height, renderBands);
}

// Chooses the iteration limit for `view` first when it is automatic, then calls `then`
function withIterations(view, width, height, then) {
    if (!state.autoIterations) {
        then();
        return;
    }
    job = { choosing: true, onGpu: usesGpu(), view, width, height, then };
    chooseIterations();
    updateStatus();
}

function chooseIterations() {
    const { view, width, height } = job;
    const task = { generation, kind: 'iterations', view, width, height };
    if (usesGpu()) {
        gpu.worker.postMessage(task);
    } else {
        queue = [task];
        dispatch();
    }
}

function onIterations(iterations) {
    state.iterations = iterations;
    updateControls();
    job.then();
}

function renderBands() {
    const { width, height } = canvas;
    const preview = document.createElement('canvas');
    preview.width = Math.max(1, Math.ceil(width / PREVIEW_DIVISOR));
    preview.height = Math.max(1, Math.ceil(height / PREVIEW_DIVISOR));
    const passes = [{ pass: 'full', width, height }];
    if (previewHelps()) passes.unshift({ pass: 'preview', width: preview.width, height: preview.height });
    startJob({ preview, previewRows: 0, fullBands: [] }, passes);
}

// Renders the view at `scale` times the canvas size, off screen, and downloads it
function exportImage(scale) {
    const width = canvas.width * scale;
    const height = canvas.height * scale;
    const target = document.createElement('canvas');
    target.width = width;
    target.height = height;
    const interrupted = job && !job.exporting && job.elapsed === undefined;
    stopRendering();
    startJob({ exporting: true, target, interrupted }, [{ pass: 'export', width, height }]);
}

// Renders `view` off screen for the automatic zoom, resolving to the finished keyframe
function renderKeyframe(view, width, height, chooseIterations) {
    return new Promise((resolve) => {
        stopRendering();
        const target = document.createElement('canvas');
        target.width = width;
        target.height = height;
        const start = () => startJob({ view, keyframe: true, target, resolve }, [{ pass: 'keyframe', width, height }]);
        if (chooseIterations) {
            withIterations(view, width, height, start);
        } else {
            start();
        }
    });
}

function startJob(fields, passes) {
    const last = passes[passes.length - 1];
    job = {
        view: state.view,
        ...fields,
        started: performance.now(),
        onGpu: usesGpu(),
        passes,
        width: last.width,
        height: last.height,
        rows: 0,
    };
    dispatchJob();
    updateStatus();
}

function dispatchJob() {
    if (job.onGpu) {
        gpu.worker.postMessage({
            generation,
            view: job.view,
            iterations: state.iterations,
            normal: state.shading === 'normal',
            passes: job.passes,
        });
    } else {
        queue = job.passes.flatMap(({ pass, width, height }) => bandTasks(pass, width, height));
        dispatch();
    }
}

function onBand(worker, band) {
    idle.push(worker);
    if (band.generation === generation && band.kind === 'iterations') {
        onIterations(band.iterations);
    } else if (band.generation === generation) {
        drawBand(band);
    }
    dispatch();
}

function drawBand({ pass, pixels, width, firstRow, rowCount }) {
    const image = new ImageData(new Uint8ClampedArray(pixels.buffer), width, rowCount);
    if (pass === 'preview') {
        job.preview.getContext('2d').putImageData(image, 0, firstRow);
        job.previewRows += rowCount;
        if (job.previewRows === job.preview.height) {
            const previewed = job;
            const wait = PREVIEW_DELAY_MS - (performance.now() - job.started);
            setTimeout(() => showPreview(previewed), Math.max(0, wait));
        }
        return;
    }
    const target = pass === 'full' ? ctx : job.target.getContext('2d');
    target.putImageData(image, 0, firstRow);
    job.rows += rowCount;
    if (pass === 'full') job.fullBands.push({ image, firstRow });
    if (job.rows === job.height) {
        job.elapsed = performance.now() - job.started;
        if (pass === 'full') {
            keepRendered(state.view);
            lastRenderSeconds = job.elapsed / 1000;
        }
        if (pass === 'export') finishExport();
        if (pass === 'keyframe') job.resolve({ canvas: job.target, view: job.view });
    }
    updateStatus();
}

function finishExport() {
    const { target, interrupted } = job;
    target.toBlob((blob) => {
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        const name = [state.view.x, state.view.y].map((c) => shorten(c, 12)).join('_');
        link.download = `mandelbrot_${name}_${formatZoom(state.view.zoom)}_${target.width}x${target.height}.png`;
        link.click();
        URL.revokeObjectURL(link.href);
    }, 'image/png');
    if (interrupted) render();
}

function showPreview(previewed) {
    if (previewed !== job || job.elapsed !== undefined) return;
    ctx.imageSmoothingEnabled = true;
    ctx.drawImage(job.preview, 0, 0, canvas.width, canvas.height);
    for (const band of job.fullBands) {
        ctx.putImageData(band.image, 0, band.firstRow);
    }
}

// Centers are exact decimal strings, so arithmetic on them happens in wasm

function pixelSize(view) {
    return BASE_VIEW_WIDTH / view.zoom / canvas.width;
}

function offsetFromCenter(view, px, py) {
    const size = pixelSize(view);
    return { dx: (px - canvas.width / 2) * size, dy: (py - canvas.height / 2) * size };
}

function clientToCanvas(clientX, clientY) {
    const rect = canvas.getBoundingClientRect();
    return {
        px: ((clientX - rect.left) * canvas.width) / rect.width,
        py: ((clientY - rect.top) * canvas.height) / rect.height,
    };
}

function zoomAt(px, py, factor) {
    const { view } = state;
    const { dx, dy } = offsetFromCenter(view, px, py);
    const [x, y, zoom] = zoomView(view.x, view.y, view.zoom, dx, dy, factor);
    return { x, y, zoom: Number(zoom) };
}

function zoomToRect(px, py, factor) {
    const { view } = state;
    const { dx, dy } = offsetFromCenter(view, px, py);
    const [x, y] = pan(view.x, view.y, view.zoom, dx, dy);
    return { x, y, zoom: Math.min(view.zoom * factor, maxZoom()) };
}

function keepRendered(view) {
    rendered.canvas.width = canvas.width;
    rendered.canvas.height = canvas.height;
    rendered.canvas.getContext('2d').drawImage(canvas, 0, 0);
    rendered.view = view;
}

// Where `view` lies in `source`, a finished render `{ canvas, view }`, in its pixels
function rectIn(source, view) {
    const image = source.canvas;
    const oldSize = BASE_VIEW_WIDTH / source.view.zoom / image.width;
    const ratio = pixelSize(view) / oldSize;
    const topLeft = offsetFromCenter(view, 0, 0);
    return {
        x: (difference(view.x, source.view.x) + topLeft.dx) / oldSize + image.width / 2,
        y: (difference(view.y, source.view.y) + topLeft.dy) / oldSize + image.height / 2,
        width: canvas.width * ratio,
        height: canvas.height * ratio,
    };
}

function renderedRect(view) {
    return rectIn(rendered, view);
}

function clearView() {
    ctx.globalAlpha = 1;
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
}

// Draws `view` by stretching the finished render `source` over it
function drawFrom(source, view, alpha = 1) {
    const rect = rectIn(source, view);
    ctx.globalAlpha = alpha;
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';
    ctx.drawImage(source.canvas, rect.x, rect.y, rect.width, rect.height, 0, 0, canvas.width, canvas.height);
    ctx.globalAlpha = 1;
}

// Stretches the last finished render over the new view, so repeated zooms never resample
// an already stretched image
function reproject(from, to) {
    if (!rendered.view) keepRendered(from);
    clearView();
    drawFrom(rendered, to);
}

// The preview only beats the stretched last render where that is magnified further than the
// preview is, or leaves part of the view uncovered
function previewHelps() {
    if (!rendered.view) return true;
    const rect = renderedRect(state.view);
    const covered = rect.x >= 0 && rect.y >= 0
        && rect.x + rect.width <= rendered.canvas.width
        && rect.y + rect.height <= rendered.canvas.height;
    return !covered || canvas.width / rect.width > PREVIEW_DIVISOR;
}

function presetFor(name) {
    const view = presetView(name);
    return view && { x: view[0], y: view[1], zoom: Number(view[2]) };
}

function readHash() {
    const params = new URLSearchParams(location.hash.slice(1));
    const number = (key) => {
        const value = params.get(key);
        return value === null || value === '' ? NaN : Number(value);
    };

    const preset = presetNames().includes(params.get('p')) ? params.get('p') : 'mandelbrot';
    const view = {
        x: normalizeCoordinate(params.get('x') ?? ''),
        y: normalizeCoordinate(params.get('y') ?? ''),
        zoom: number('z'),
    };
    const validView = view.x !== undefined && view.y !== undefined && view.zoom > 0 && view.zoom <= maxZoom();
    const iterations = Math.round(number('it'));
    const fixed = iterations >= 1 && iterations <= MAX_ITERATIONS;

    return {
        preset,
        view: validView ? view : presetFor(preset),
        iterations: fixed ? iterations : DEFAULT_ITERATIONS,
        autoIterations: !fixed,
        shading: params.get('s') === 'flat' ? 'flat' : 'normal',
    };
}

function hash() {
    const { view, iterations, autoIterations, shading, preset } = state;
    return '#' + new URLSearchParams({
        x: view.x,
        y: view.y,
        z: view.zoom,
        it: autoIterations ? 'auto' : iterations,
        s: shading,
        p: preset,
    });
}

function writeUrl(push) {
    if (push) {
        state.historyIndex++;
        state.historyLength = state.historyIndex + 1;
        history.pushState({ index: state.historyIndex }, '', hash());
    } else {
        history.replaceState({ index: state.historyIndex }, '', hash());
    }
    updateControls();
}

// Moves to `view` right away by stretching the last render; the caller renders it
function showView(view) {
    stopAutoZoom();
    const from = state.view;
    state.view = view;
    reproject(from, view);
}

function navigate(view, { push = true } = {}) {
    showView(view);
    writeUrl(push);
    render();
}

window.addEventListener('popstate', (event) => {
    stopAutoZoom();
    const from = state.view;
    Object.assign(state, readHash());
    state.historyIndex = event.state?.index ?? 0;
    reproject(from, state.view);
    updateControls();
    render();
});

function formatZoom(zoom) {
    return zoom >= 1e4 ? zoom.toExponential(3) : zoom.toPrecision(4);
}

function shorten(coordinate, digits) {
    const point = coordinate.indexOf('.');
    return point === -1 ? coordinate : coordinate.slice(0, point + 1 + digits);
}

function updateControls() {
    const { view } = state;
    const preset = presetFor(state.preset);
    const atPreset = preset.x === view.x && preset.y === view.y && preset.zoom === view.zoom;
    $('preset').value = atPreset ? state.preset : 'custom';
    $('shading').value = state.shading;
    const gpuOption = $('backend').options[0];
    gpuOption.disabled = !gpu.ready;
    gpuOption.textContent = gpu.ready ? `GPU${gpu.adapter ? ` · ${gpu.adapter}` : ''}` : 'GPU (unavailable)';
    $('backend').options[1].textContent = `CPU · ${workers.length} workers`;
    $('backend').value = gpu.ready ? state.backend : 'cpu';
    $('iterations').value = state.iterations;
    $('auto-iterations').checked = state.autoIterations;
    $('back').disabled = state.historyIndex === 0;
    $('forward').disabled = state.historyIndex >= state.historyLength - 1;
    const digits = Math.max(15, Math.ceil(Math.log10(view.zoom)) + 5);
    const imaginary = view.y.startsWith('-') ? `− ${shorten(view.y.slice(1), digits)}` : `+ ${shorten(view.y, digits)}`;
    $('center').textContent = `${shorten(view.x, digits)} ${imaginary}i`;
    $('zoom').textContent = `${formatZoom(view.zoom)}×`;
    $('zoom-limit').hidden = view.zoom < maxZoom();
    $('deep-hint').hidden = state.autoIterations || view.zoom < DEEP_ZOOM;
}

function updateStatus() {
    const status = $('status');
    if (!job) {
        status.textContent = '';
        return;
    }
    if (job.choosing) {
        status.textContent = `Choosing iteration limit on ${job.onGpu ? 'GPU' : 'CPU'}…`;
        return;
    }
    const size = `${job.width}×${job.height}`;
    const device = job.onGpu ? 'GPU' : 'CPU';
    const seconds = `${((job.elapsed ?? 0) / 1000).toFixed(2)} s`;
    const percent = Math.floor((100 * job.rows) / job.height);
    if (job.exporting) {
        status.textContent = job.elapsed === undefined
            ? `Exporting ${size}… ${percent}% on ${device}`
            : `Exported ${size} in ${seconds} on ${device}`;
    } else if (job.keyframe) {
        status.textContent = `Auto zoom · next frame ${percent}% on ${device}`;
    } else {
        status.textContent = job.elapsed === undefined
            ? `Rendering… ${percent}% on ${device}`
            : `${seconds} at ${size} on ${device}`;
    }
}

function resolutionScale() {
    const choice = $('resolution').value;
    return choice === 'device' ? Math.min(window.devicePixelRatio || 1, 2) : Number(choice);
}

function updateExportScales() {
    const select = $('export-scale');
    const chosen = Number(select.value) || 1;
    select.replaceChildren(...EXPORT_SCALES
        .filter((scale) => Math.max(canvas.width, canvas.height) * scale <= MAX_CANVAS_SIDE)
        .map((scale) => new Option(`${canvas.width * scale}×${canvas.height * scale}`, scale)));
    select.value = [...select.options].some((option) => Number(option.value) === chosen) ? chosen : 1;
}

function resizeCanvas() {
    const scale = resolutionScale();
    const width = Math.max(1, Math.round(canvas.clientWidth * scale));
    const height = Math.max(1, Math.round(canvas.clientHeight * scale));
    if (width === canvas.width && height === canvas.height) {
        return false;
    }
    snapshot.width = canvas.width;
    snapshot.height = canvas.height;
    snapshot.getContext('2d').drawImage(canvas, 0, 0);
    canvas.width = width;
    canvas.height = height;
    ctx.drawImage(snapshot, 0, 0, width, height);
    updateExportScales();
    return true;
}

const autoZoom = createAutoZoom({
    view: () => state.view,
    show: (view, withControls) => {
        state.view = view;
        if (withControls) writeUrl(false);
    },
    size: () => ({ width: canvas.width, height: canvas.height }),
    pixelSize,
    finished: () => {
        if (!rendered.view) return null;
        const copy = document.createElement('canvas');
        copy.width = rendered.canvas.width;
        copy.height = rendered.canvas.height;
        copy.getContext('2d').drawImage(rendered.canvas, 0, 0);
        return { canvas: copy, view: rendered.view };
    },
    renderKeyframe,
    rectIn,
    clear: clearView,
    drawFrom,
    keyframeScale: () => (usesGpu() ? GPU_KEYFRAME_SCALE : 1),
    lastRenderSeconds: () => lastRenderSeconds,
    speed: () => Number($('zoom-speed').value),
    onChange: (active) => {
        $('autozoom').textContent = active ? '⏸ Stop zooming' : '▶ Auto zoom';
        $('autozoom').setAttribute('aria-pressed', String(active));
    },
    onEnd: () => {
        stopAutoZoom();
        writeUrl(false);
        render();
    },
});

// The keyframe on screen becomes the render later zooms stretch
function stopAutoZoom() {
    const frame = autoZoom.stop();
    if (frame) {
        rendered.canvas = frame.canvas;
        rendered.view = frame.view;
    }
}

function toggleAutoZoom() {
    if (autoZoom.active) {
        stopAutoZoom();
        writeUrl(false);
        render();
    } else {
        stopRendering();
        autoZoom.start();
    }
}

function setPanelCollapsed(collapsed) {
    const toggle = $('toggle-panel');
    $('panel').classList.toggle('collapsed', collapsed);
    toggle.textContent = collapsed ? '+' : '–';
    toggle.title = collapsed ? 'Show controls' : 'Hide controls';
    toggle.setAttribute('aria-expanded', String(!collapsed));
}

function setupControls() {
    const presetSelect = $('preset');
    for (const name of presetNames()) {
        presetSelect.add(new Option(name, name));
    }
    presetSelect.add(new Option('Custom view', 'custom', false, false));
    presetSelect.lastElementChild.disabled = true;

    presetSelect.addEventListener('change', () => {
        state.preset = presetSelect.value;
        navigate(presetFor(state.preset));
    });
    $('shading').addEventListener('change', (event) => {
        state.shading = event.target.value;
        writeUrl(false);
        render();
    });
    $('backend').addEventListener('change', (event) => {
        state.backend = event.target.value;
        render();
    });
    $('iterations').addEventListener('change', (event) => {
        const value = Math.round(Number(event.target.value));
        state.iterations = Math.min(Math.max(Number.isFinite(value) ? value : DEFAULT_ITERATIONS, 1), MAX_ITERATIONS);
        state.autoIterations = false;
        writeUrl(false);
        render();
    });
    $('auto-iterations').addEventListener('change', (event) => {
        state.autoIterations = event.target.checked;
        writeUrl(false);
        render();
    });
    $('resolution').addEventListener('change', () => {
        if (resizeCanvas()) render();
    });
    $('back').addEventListener('click', () => history.back());
    $('forward').addEventListener('click', () => history.forward());
    $('zoom-out').addEventListener('click', () => navigate(zoomAt(canvas.width / 2, canvas.height / 2, 1 / CLICK_ZOOM)));
    $('reset').addEventListener('click', () => navigate(presetFor(state.preset)));
    $('download').addEventListener('click', () => exportImage(Number($('export-scale').value)));
    $('autozoom').addEventListener('click', toggleAutoZoom);

    $('toggle-panel').addEventListener('click', () => {
        setPanelCollapsed(!$('panel').classList.contains('collapsed'));
    });
    setPanelCollapsed(window.matchMedia('(max-width: 600px)').matches);

    let resizeTimer;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            if (resizeCanvas()) render();
        }, WHEEL_SETTLE_MS);
    });
}

function selectionRect(start, current) {
    const rect = canvas.getBoundingClientRect();
    const aspect = rect.width / rect.height;
    const dx = current.x - start.x;
    const dy = current.y - start.y;
    let width = Math.abs(dx);
    let height = Math.abs(dy);
    if (width / aspect > height) {
        height = width / aspect;
    } else {
        width = height * aspect;
    }
    return {
        left: dx < 0 ? start.x - width : start.x,
        top: dy < 0 ? start.y - height : start.y,
        width,
        height,
    };
}

function setupPointer() {
    let drag = null;

    const endDrag = () => {
        drag = null;
        selection.hidden = true;
    };

    canvas.addEventListener('pointerdown', (event) => {
        if (event.button !== 0 || event.pointerType === 'touch') return;
        canvas.setPointerCapture(event.pointerId);
        drag = { start: { x: event.clientX, y: event.clientY }, rect: null };
    });

    canvas.addEventListener('pointermove', (event) => {
        if (!drag) return;
        const current = { x: event.clientX, y: event.clientY };
        if (!drag.rect && Math.hypot(current.x - drag.start.x, current.y - drag.start.y) < DRAG_THRESHOLD) {
            return;
        }
        drag.rect = selectionRect(drag.start, current);
        Object.assign(selection.style, {
            left: `${drag.rect.left}px`,
            top: `${drag.rect.top}px`,
            width: `${drag.rect.width}px`,
            height: `${drag.rect.height}px`,
        });
        selection.hidden = false;
    });

    canvas.addEventListener('pointerup', (event) => {
        if (!drag) return;
        const { start, rect } = drag;
        endDrag();
        if (rect) {
            const center = clientToCanvas(rect.left + rect.width / 2, rect.top + rect.height / 2);
            const factor = canvas.getBoundingClientRect().width / rect.width;
            navigate(zoomToRect(center.px, center.py, factor));
        } else if (autoZoom.active) {
            const { px, py } = clientToCanvas(start.x, start.y);
            autoZoom.retarget(px, py);
        } else {
            const { px, py } = clientToCanvas(start.x, start.y);
            navigate(zoomAt(px, py, event.shiftKey ? 1 / CLICK_ZOOM : CLICK_ZOOM));
        }
    });

    canvas.addEventListener('pointercancel', (event) => {
        if (event.pointerType !== 'touch') endDrag();
    });
    window.addEventListener('keydown', (event) => {
        if (event.key === 'Escape') endDrag();
    });

    canvas.addEventListener('contextmenu', (event) => {
        event.preventDefault();
        const { px, py } = clientToCanvas(event.clientX, event.clientY);
        navigate(zoomAt(px, py, 1 / CLICK_ZOOM));
    });

    let wheelTimer = null;
    canvas.addEventListener('wheel', (event) => {
        event.preventDefault();
        const pixels = event.deltaMode === WheelEvent.DOM_DELTA_LINE ? event.deltaY * 16 : event.deltaY;
        const { px, py } = clientToCanvas(event.clientX, event.clientY);
        const view = zoomAt(px, py, WHEEL_ZOOM_PER_PIXEL ** -pixels);
        stopRendering();
        showView(view);
        writeUrl(wheelTimer === null);
        clearTimeout(wheelTimer);
        wheelTimer = setTimeout(() => {
            wheelTimer = null;
            render();
        }, WHEEL_SETTLE_MS);
    }, { passive: false });
}

// One finger pans and two pinch; the view follows the fingers and renders once they lift
function setupTouch() {
    const touches = new Map();
    let gesture = null;

    // Positions are measured from where the current set of fingers went down
    const restart = () => {
        gesture = { view: state.view, start: new Map(touches), moved: gesture?.moved ?? false };
    };
    const at = (event) => {
        const { px, py } = clientToCanvas(event.clientX, event.clientY);
        return { x: px, y: py };
    };
    const middle = (points) => ({
        x: points.reduce((sum, p) => sum + p.x, 0) / points.length,
        y: points.reduce((sum, p) => sum + p.y, 0) / points.length,
    });

    canvas.addEventListener('pointerdown', (event) => {
        if (event.pointerType !== 'touch') return;
        canvas.setPointerCapture(event.pointerId);
        if (touches.size === 0) gesture = null;
        touches.set(event.pointerId, at(event));
        restart();
    });

    canvas.addEventListener('pointermove', (event) => {
        if (!touches.has(event.pointerId)) return;
        touches.set(event.pointerId, at(event));
        const ids = [...gesture.start.keys()].slice(0, 2);
        const from = ids.map((id) => gesture.start.get(id));
        const to = ids.map((id) => touches.get(id));
        const [start, now] = [middle(from), middle(to)];
        if (!gesture.moved && Math.hypot(now.x - start.x, now.y - start.y) < DRAG_THRESHOLD && ids.length === 1) {
            return;
        }
        if (!gesture.moved) stopRendering();
        gesture.moved = true;

        let view = gesture.view;
        if (ids.length === 2) {
            const spread = (points) => Math.hypot(points[0].x - points[1].x, points[0].y - points[1].y);
            const { dx, dy } = offsetFromCenter(view, start.x, start.y);
            const [x, y, zoom] = zoomView(view.x, view.y, view.zoom, dx, dy, spread(to) / Math.max(spread(from), 1));
            view = { x, y, zoom: Number(zoom) };
        }
        const size = pixelSize(view);
        const [x, y] = pan(view.x, view.y, view.zoom, (start.x - now.x) * size, (start.y - now.y) * size);
        showView({ x, y, zoom: view.zoom });
    });

    const lift = (event) => {
        if (!touches.delete(event.pointerId)) return;
        if (touches.size > 0) {
            restart();
            return;
        }
        if (gesture.moved) {
            writeUrl(true);
            render();
        } else if (event.type === 'pointerup') {
            const point = gesture.start.get(event.pointerId);
            if (autoZoom.active) {
                autoZoom.retarget(point.x, point.y);
            } else {
                navigate(zoomAt(point.x, point.y, CLICK_ZOOM));
            }
        }
    };
    canvas.addEventListener('pointerup', lift);
    canvas.addEventListener('pointercancel', lift);
}

function setupKeyboard() {
    const pans = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
    window.addEventListener('keydown', (event) => {
        if (event.target.closest('input, select, textarea, button') || event.ctrlKey || event.metaKey || event.altKey) return;
        const { width, height } = canvas;
        const push = !event.repeat;
        if (pans[event.key]) {
            const [sx, sy] = pans[event.key];
            const { view } = state;
            const size = pixelSize(view);
            const [x, y] = pan(view.x, view.y, view.zoom, sx * width * KEY_PAN_FRACTION * size, sy * height * KEY_PAN_FRACTION * size);
            navigate({ x, y, zoom: view.zoom }, { push });
        } else if (event.key === '+' || event.key === '=') {
            navigate(zoomAt(width / 2, height / 2, CLICK_ZOOM), { push });
        } else if (event.key === '-' || event.key === '_') {
            navigate(zoomAt(width / 2, height / 2, 1 / CLICK_ZOOM), { push });
        } else if (event.key === 'r' || event.key === 'R') {
            navigate(presetFor(state.preset));
        } else if (event.key === ' ') {
            toggleAutoZoom();
        } else {
            return;
        }
        event.preventDefault();
    });
}

async function main() {
    try {
        await init();
    } catch (error) {
        showError(`Could not load the renderer: ${error.message}`);
        return;
    }
    Object.assign(state, readHash());
    state.historyIndex = history.state?.index ?? 0;
    state.historyLength = state.historyIndex + 1;
    history.replaceState({ index: state.historyIndex }, '', hash());

    setupControls();
    setupPointer();
    setupTouch();
    setupKeyboard();
    startWorkers();
    await startGpu();
    resizeCanvas();
    updateExportScales();
    updateControls();
    render();
}

main();
