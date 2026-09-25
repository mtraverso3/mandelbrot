import init, { presetNames, presetView } from './mandelbrot_web.js';

const BASE_VIEW_WIDTH = 3;
const PRECISION_LIMIT = 1e13;
const PREVIEW_DIVISOR = 4;
const BAND_PIXELS = 1 << 16;
const DRAG_THRESHOLD = 5;
const CLICK_ZOOM = 2;
const WHEEL_ZOOM_PER_PIXEL = 1.0025;
const WHEEL_SETTLE_MS = 150;
const MAX_ITERATIONS = 100000;
const DEFAULT_ITERATIONS = 1500;

const $ = (id) => document.getElementById(id);
const canvas = $('view');
const ctx = canvas.getContext('2d');
const snapshot = document.createElement('canvas');
const selection = $('selection');

const state = {
    view: null,
    preset: 'mandelbrot',
    iterations: DEFAULT_ITERATIONS,
    shading: 'normal',
    historyIndex: 0,
    historyLength: 1,
};

function showError(message) {
    const error = $('error');
    error.textContent = message;
    error.hidden = false;
}

// Rendering: bands of rows are farmed out to a pool of workers, each running the wasm renderer.
// A quarter-resolution preview pass goes first, then full-resolution bands from the center out.

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

function dispatch() {
    while (idle.length && queue.length) {
        idle.pop().postMessage(queue.shift());
    }
}

function bandTasks(pass, width, height) {
    const rowsPerBand = Math.max(1, Math.floor(BAND_PIXELS / width));
    const tasks = [];
    for (let firstRow = 0; firstRow < height; firstRow += rowsPerBand) {
        tasks.push({
            generation,
            pass,
            view: state.view,
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

function render() {
    generation++;
    const { width, height } = canvas;
    const preview = document.createElement('canvas');
    preview.width = Math.max(1, Math.ceil(width / PREVIEW_DIVISOR));
    preview.height = Math.max(1, Math.ceil(height / PREVIEW_DIVISOR));

    const previewTasks = bandTasks('preview', preview.width, preview.height);
    const fullTasks = bandTasks('full', width, height);
    job = {
        started: performance.now(),
        preview,
        previewRemaining: previewTasks.length,
        fullBands: [],
        fullTotal: fullTasks.length,
    };
    queue = [...previewTasks, ...fullTasks];
    updateStatus();
    dispatch();
}

function onBand(worker, band) {
    idle.push(worker);
    if (band.generation === generation) {
        drawBand(band);
    }
    dispatch();
}

function drawBand({ pass, pixels, width, firstRow, rowCount }) {
    const image = new ImageData(new Uint8ClampedArray(pixels.buffer), width, rowCount);
    if (pass === 'preview') {
        job.preview.getContext('2d').putImageData(image, 0, firstRow);
        if (--job.previewRemaining === 0) {
            ctx.imageSmoothingEnabled = true;
            ctx.drawImage(job.preview, 0, 0, canvas.width, canvas.height);
            for (const band of job.fullBands) {
                ctx.putImageData(band.image, 0, band.firstRow);
            }
        }
    } else {
        ctx.putImageData(image, 0, firstRow);
        job.fullBands.push({ image, firstRow });
        if (job.fullBands.length === job.fullTotal) {
            job.elapsed = performance.now() - job.started;
        }
    }
    updateStatus();
}

// Geometry, matching the Rust renderer: pixel (px, py) maps to
// center + (p - size / 2) * pixelSize, with the view BASE_VIEW_WIDTH / zoom wide.

function pixelSize(view) {
    return BASE_VIEW_WIDTH / view.zoom / canvas.width;
}

function toComplex(view, px, py) {
    const size = pixelSize(view);
    return {
        x: view.x + (px - canvas.width / 2) * size,
        y: view.y + (py - canvas.height / 2) * size,
    };
}

function clientToCanvas(clientX, clientY) {
    const rect = canvas.getBoundingClientRect();
    return {
        px: ((clientX - rect.left) * canvas.width) / rect.width,
        py: ((clientY - rect.top) * canvas.height) / rect.height,
    };
}

function zoomAt(px, py, factor) {
    const anchor = toComplex(state.view, px, py);
    return {
        x: anchor.x + (state.view.x - anchor.x) / factor,
        y: anchor.y + (state.view.y - anchor.y) / factor,
        zoom: state.view.zoom * factor,
    };
}

// Redraws the current image as it would appear in `to`, for instant feedback while rendering.
function reproject(from, to) {
    const { width, height } = canvas;
    snapshot.width = width;
    snapshot.height = height;
    snapshot.getContext('2d').drawImage(canvas, 0, 0);

    const oldSize = pixelSize(from);
    const ratio = pixelSize(to) / oldSize;
    const topLeft = toComplex(to, 0, 0);
    const sx = (topLeft.x - from.x) / oldSize + width / 2;
    const sy = (topLeft.y - from.y) / oldSize + height / 2;

    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, width, height);
    ctx.imageSmoothingEnabled = ratio > 1;
    ctx.drawImage(snapshot, sx, sy, width * ratio, height * ratio, 0, 0, width, height);
}

// State, URL and history

function presetFor(name) {
    const view = presetView(name);
    return view && { x: view[0], y: view[1], zoom: view[2] };
}

function readHash() {
    const params = new URLSearchParams(location.hash.slice(1));
    const number = (key) => {
        const value = params.get(key);
        return value === null || value === '' ? NaN : Number(value);
    };

    const preset = presetNames().includes(params.get('p')) ? params.get('p') : 'mandelbrot';
    const view = { x: number('x'), y: number('y'), zoom: number('z') };
    const validView = Number.isFinite(view.x) && Number.isFinite(view.y) && view.zoom > 0 && Number.isFinite(view.zoom);
    const iterations = Math.round(number('it'));

    return {
        preset,
        view: validView ? view : presetFor(preset),
        iterations: iterations >= 1 && iterations <= MAX_ITERATIONS ? iterations : DEFAULT_ITERATIONS,
        shading: params.get('s') === 'flat' ? 'flat' : 'normal',
    };
}

function hash() {
    const { view, iterations, shading, preset } = state;
    return '#' + new URLSearchParams({
        x: view.x,
        y: view.y,
        z: view.zoom,
        it: iterations,
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

function navigate(view, { push = true } = {}) {
    const from = state.view;
    state.view = view;
    reproject(from, view);
    writeUrl(push);
    render();
}

window.addEventListener('popstate', (event) => {
    const from = state.view;
    Object.assign(state, readHash());
    state.historyIndex = event.state?.index ?? 0;
    reproject(from, state.view);
    updateControls();
    render();
});

// Controls

function formatZoom(zoom) {
    return zoom >= 1e4 ? zoom.toExponential(3) : zoom.toPrecision(4);
}

function updateControls() {
    const { view } = state;
    const preset = presetFor(state.preset);
    const atPreset = preset.x === view.x && preset.y === view.y && preset.zoom === view.zoom;
    $('preset').value = atPreset ? state.preset : 'custom';
    $('shading').value = state.shading;
    $('iterations').value = state.iterations;
    $('back').disabled = state.historyIndex === 0;
    $('forward').disabled = state.historyIndex >= state.historyLength - 1;
    $('center').textContent = `${view.x.toPrecision(15)} ${view.y < 0 ? '−' : '+'} ${Math.abs(view.y).toPrecision(15)}i`;
    $('zoom').textContent = `${formatZoom(view.zoom)}×`;
    $('precision-warning').hidden = view.zoom < PRECISION_LIMIT;
}

function updateStatus() {
    if (!job) return;
    const status = $('status');
    const size = `${canvas.width}×${canvas.height}`;
    if (job.elapsed !== undefined) {
        status.textContent = `${(job.elapsed / 1000).toFixed(2)} s at ${size}`;
    } else {
        const percent = Math.floor((100 * job.fullBands.length) / job.fullTotal);
        status.textContent = `Rendering… ${percent}% (${workers.length} workers)`;
    }
}

function resolutionScale() {
    const choice = $('resolution').value;
    return choice === 'device' ? Math.min(window.devicePixelRatio || 1, 2) : Number(choice);
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
    return true;
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
    $('iterations').addEventListener('change', (event) => {
        const value = Math.round(Number(event.target.value));
        state.iterations = Math.min(Math.max(Number.isFinite(value) ? value : DEFAULT_ITERATIONS, 1), MAX_ITERATIONS);
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
    $('download').addEventListener('click', () => {
        canvas.toBlob((blob) => {
            const link = document.createElement('a');
            link.href = URL.createObjectURL(blob);
            link.download = `mandelbrot_${state.view.x}_${state.view.y}_${formatZoom(state.view.zoom)}.png`;
            link.click();
            URL.revokeObjectURL(link.href);
        }, 'image/png');
    });

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

// Pointer interaction: drag a rectangle to zoom into it, click to zoom in, shift+click or
// right-click to zoom out, scroll to zoom around the cursor.

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
        if (event.button !== 0) return;
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
            const target = toComplex(state.view, center.px, center.py);
            const factor = canvas.getBoundingClientRect().width / rect.width;
            navigate({ x: target.x, y: target.y, zoom: state.view.zoom * factor });
        } else {
            const { px, py } = clientToCanvas(start.x, start.y);
            navigate(zoomAt(px, py, event.shiftKey ? 1 / CLICK_ZOOM : CLICK_ZOOM));
        }
    });

    canvas.addEventListener('pointercancel', endDrag);
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
        const from = state.view;
        state.view = view;
        reproject(from, view);
        writeUrl(wheelTimer === null);
        clearTimeout(wheelTimer);
        wheelTimer = setTimeout(() => {
            wheelTimer = null;
            render();
        }, WHEEL_SETTLE_MS);
    }, { passive: false });
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
    startWorkers();
    resizeCanvas();
    updateControls();
    render();
}

main();
