import init, { GpuViewer } from './mandelbrot_web.js';

let viewer = null;

const ready = (async () => {
    await init();
    try {
        viewer = await GpuViewer.create();
        self.postMessage({ kind: 'ready', adapter: viewer.adapterName });
    } catch (error) {
        self.postMessage({ kind: 'unavailable', reason: error.message ?? String(error) });
    }
})();

// Renders a job's passes in order; a newer job cancels whatever is still running
self.onmessage = async ({ data: job }) => {
    await ready;
    if (job.kind === 'cancel') {
        viewer.cancel();
        return;
    }
    const { generation, view, iterations, normal } = job;
    try {
        for (const { pass, width, height } of job.passes) {
            const finished = await viewer.render(
                view.x, view.y, view.zoom,
                width, height, iterations, normal,
                (firstRow, pixels) => {
                    const band = { kind: 'band', generation, pass, pixels, width, firstRow, rowCount: pixels.length / 4 / width };
                    self.postMessage(band, [pixels.buffer]);
                },
            );
            if (!finished) return;
        }
    } catch (error) {
        self.postMessage({ kind: 'failed', generation, reason: error.message ?? String(error) });
    }
};
