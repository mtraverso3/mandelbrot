import init, { renderRows } from './mandelbrot_web.js';

const ready = init();

self.onmessage = async ({ data: task }) => {
    await ready;
    const { view, width, height, iterations, normal, firstRow, rowCount } = task;
    const pixels = renderRows(
        view.x, view.y, view.zoom,
        width, height, iterations, normal,
        firstRow, rowCount,
    );
    self.postMessage({ ...task, pixels }, [pixels.buffer]);
};
