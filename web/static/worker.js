import init, { autoIterations, lastBands, renderRows } from './mandelbrot_web.js';

const ready = init();

self.onmessage = async ({ data: task }) => {
    await ready;
    if (task.kind === 'iterations') {
        const { view, width, height } = task;
        self.postMessage({ ...task, iterations: autoIterations(view.x, view.y, view.zoom, width, height) });
        return;
    }
    const { view, width, height, samples, iterations, normal, palette, firstRow, rowCount, bands } = task;
    const pixels = renderRows(
        view.x, view.y, view.zoom,
        width, height, samples ?? 1, iterations, normal, palette,
        firstRow, rowCount,
        bands?.[0] ?? NaN, bands?.[1] ?? NaN,
    );
    self.postMessage({ ...task, pixels, bands: Array.from(lastBands()) }, [pixels.buffer]);
};
