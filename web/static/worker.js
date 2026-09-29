import init, { autoIterations, lastBands, renderPixels, renderRows } from './mandelbrot_web.js';

const ready = init();

self.onmessage = async ({ data: task }) => {
    await ready;
    if (task.kind === 'iterations') {
        const { view, width, height } = task;
        self.postMessage({ ...task, iterations: autoIterations(view.x, view.y, view.zoom, width, height) });
        return;
    }
    const { view, width, height, iterations, normal, palette, bands } = task;
    if (task.kind === 'refine') {
        const pixels = renderPixels(
            view.x, view.y, view.zoom,
            width, height, task.samples, iterations, normal, palette,
            task.pixels, task.round,
            bands[0], bands[1],
        );
        self.postMessage({ ...task, pixels }, [pixels.buffer]);
        return;
    }
    const pixels = renderRows(
        view.x, view.y, view.zoom,
        width, height, iterations, normal, palette,
        task.firstRow, task.rowCount,
        bands?.[0] ?? NaN, bands?.[1] ?? NaN,
    );
    self.postMessage({ ...task, pixels, bands: Array.from(lastBands()) }, [pixels.buffer]);
};
