import { difference, maxZoom, pan } from './mandelbrot_web.js';

// Zooms in continuously by stretching keyframes, each rendered AHEAD times deeper than the
// last while the one before it is on screen. The view heads for a target that eases into the
// center, and the target follows the most detailed part of each new keyframe.
//
// The path is a list of segments, each heading for its own target from some zoom on. A new
// target starts a segment at the deepest keyframe, where the path already passes through its
// center, so every keyframe keeps covering the path deeper than it as long as the view moves
// slower than the keyframe's edges grow. Each segment blends in from the one it replaces with
// a smoothstep, so neither the position nor the direction of motion jumps.

const AHEAD = 2;
// Zooming this much further than a keyframe stalls until the next one is ready. A little past
// AHEAD, so the zoom can always reach the depth where the next keyframe starts to cover it.
const MAX_STRETCH = AHEAD * 1.05;
// In keyframe pixels, so rounding never makes a keyframe miss the view it was rendered for
const COVER_TOLERANCE = 1;
// Keyframes that take longer than a doubling of the zoom come out smaller, down to this
// fraction of the canvas, so heavy regions slow the zoom less
const MIN_KEYFRAME_SCALE = 0.35;
// Iteration needs change slowly with depth, and choosing them can cost more than a keyframe
const CHOOSE_ITERATIONS_EVERY = 3;
// Keyframes closer than this would barely move the zoom along
const MIN_KEYFRAME_STEP = 2 ** (1 / 8);
const CROSSFADE_MS = 250;
// The target's offset from the center shrinks as zoom^-(centering - 1). Keyframes only cover
// the path while the view moves by less than their edges grow, so targets far off center are
// centered more slowly, down to not at all, like zooming in at the cursor.
const CENTERING = 2;
const COVER_MARGIN = 0.9;
// A new path takes over from the old one gradually over this much zoom
const BLEND = 4;
// Seconds for the zoom rate to settle, so it eases in and out of speed changes and stalls
const EASE_SECONDS = 0.5;
const DETAIL_COLUMNS = 64;
const SEARCH_RADIUS = 0.15;
// Interior is white; next to it escapes take the most iterations, so steering avoids it
const WHITE = 250;
const WHITE_NEIGHBORHOOD = 4;
const WHITE_PENALTY = 4;
// After a click, the zoom keeps its target for this many times deeper
const MANUAL_ZOOM = 16;
const UI_INTERVAL_MS = 250;

/**
 * `host` connects the zoom to the page: `view()`, `show(view, withControls)`, `size()`,
 * `pixelSize(view)` on the canvas,
 * `finished()`, `renderKeyframe(view, width, height, chooseIterations)`, `rectIn(source, view)`, `clear()`,
 * `drawFrom(source, view, alpha)`, `keyframeScale()` at most, `lastRenderSeconds()` of the
 * last full render at the canvas size, `speed()`, `onChange(active)` and
 * `onEnd()`, called on reaching the deepest zoom.
 */
export function createAutoZoom(host) {
    let active = false;
    let frameRequest = 0;
    let lastTime = 0;
    let lastUi = 0;
    let zoom = 0;
    // Finished keyframes, shallowest first
    let keyframes = [];
    // Bumped whenever keyframes in flight stop matching the path
    let epoch = 0;
    let pending = false;
    let path = [];
    let manualUntil = 0;
    let shown = null;
    let fading = null;
    let scale = 1;
    let rate = 0;
    let rendered = 0;

    function segmentAt(z) {
        return path.findLast((segment) => segment.zoom <= z) ?? path[0];
    }

    function centerOf(segment, z) {
        const shrink = (segment.zoom / z) ** segment.centering;
        const own = pan(segment.target.x, segment.target.y, z, segment.dx * shrink, segment.dy * shrink);
        if (!segment.replaces || z >= segment.zoom * BLEND) return own;
        const t = Math.max(Math.log(z / segment.zoom) / Math.log(BLEND), 0);
        const weight = t * t * (3 - 2 * t);
        const old = centerOf(segment.replaces, z);
        return pan(old[0], old[1], z, weight * difference(own[0], old[0]), weight * difference(own[1], old[1]));
    }

    function viewAt(z) {
        const [x, y] = centerOf(segmentAt(z), z);
        return { x, y, zoom: z };
    }

    // The fastest the view moves while `segment` blends in, as a fraction of how fast the
    // edges of keyframes grow, which bounds how fast it may move and stay covered
    function fastest(segment) {
        const { width, height } = host.size();
        const steps = 24;
        let fastest = 0;
        let [x, y] = centerOf(segment, segment.zoom);
        for (let i = 1; i <= steps; i++) {
            const z = segment.zoom * BLEND ** (i / steps);
            const [nextX, nextY] = centerOf(segment, z);
            const size = host.pixelSize({ zoom: z }) * (Math.log(BLEND) / steps);
            const vx = Math.abs(difference(nextX, x)) / size / (width / 2);
            const vy = Math.abs(difference(nextY, y)) / size / (height / 2);
            fastest = Math.max(fastest, vx, vy);
            [x, y] = [nextX, nextY];
        }
        return fastest;
    }

    // Heads for `target` from zoom `from` on
    function aimFrom(from, target) {
        const view = path.length ? viewAt(from) : host.view();
        const replaces = path.length ? segmentAt(from) : null;
        const dx = difference(view.x, target.x);
        const dy = difference(view.y, target.y);
        const { width, height } = host.size();
        const size = host.pixelSize(view);
        const room = Math.min((width / 2) / Math.abs(dx / size), (height / 2) / Math.abs(dy / size));
        const segment = { target, zoom: from, dx, dy, centering: Math.max(Math.min(CENTERING, COVER_MARGIN * room), 1), replaces };
        // Blending into a sharp turn moves faster than either path, so it centers more gently
        // or, failing that, turns without blending
        while (segment.replaces && fastest(segment) > 1) {
            if (segment.centering > 1) segment.centering = Math.max(segment.centering - 0.25, 1);
            else segment.replaces = null;
        }
        path = path.filter((segment) => segment.zoom < from && segment.zoom >= 0);
        path.push(segment);
        // Segments the zoom has passed for good, and finished blends, are no longer needed
        const current = path.findLastIndex((segment) => segment.zoom <= zoom);
        if (current > 0) path = path.slice(current);
        for (const segment of path) {
            if (zoom >= segment.zoom * BLEND) segment.replaces = null;
        }
    }

    // Scales keyframes so they take about as long to render as the zoom takes to double
    function keyframeScale(seconds, current = 1) {
        const budget = 1 / host.speed();
        const scale = seconds > 0 ? current * Math.sqrt(budget / seconds) : host.keyframeScale();
        return Math.min(Math.max(scale, MIN_KEYFRAME_SCALE), host.keyframeScale());
    }

    function covers(frame, view) {
        const rect = host.rectIn(frame, view);
        return rect.x >= -COVER_TOLERANCE && rect.y >= -COVER_TOLERANCE
            && rect.x + rect.width <= frame.canvas.width + COVER_TOLERANCE
            && rect.y + rect.height <= frame.canvas.height + COVER_TOLERANCE;
    }


    function bestFor(view) {
        return keyframes.findLast((frame) => covers(frame, view)) ?? null;
    }

    function usable(z) {
        const best = bestFor(viewAt(z));
        return best && z / best.view.zoom <= MAX_STRETCH ? best : null;
    }

    // The deepest zoom up to `far` the keyframes draw
    function reach(far) {
        if (usable(far)) return far;
        let near = zoom;
        for (let i = 0; i < 12; i++) {
            const middle = Math.sqrt(near * far);
            if (usable(middle)) near = middle;
            else far = middle;
        }
        return near;
    }

    function frame(time) {
        const dt = Math.min((time - lastTime) / 1000, 0.1);
        lastTime = time;
        const easing = 1 - Math.exp(-dt / EASE_SECONDS);
        rate += (host.speed() - rate) * easing;
        // Eases towards where the keyframes run out, instead of stopping dead there
        const limit = reach(Math.min(zoom * 2 ** (2 * rate * EASE_SECONDS), maxZoom()));
        const next = Math.min(zoom * 2 ** (rate * dt), zoom * (limit / zoom) ** easing);
        if (dt > 0) rate = Math.min(rate, Math.log2(next / zoom) / dt);
        const best = usable(next) ?? usable(zoom) ?? shown;
        zoom = next;
        const view = viewAt(zoom);

        if (best !== shown) {
            fading = shown && { frame: shown, since: time };
            shown = best;
        }
        host.clear();
        const fade = fading ? Math.min((time - fading.since) / CROSSFADE_MS, 1) : 1;
        if (fading && fade < 1) host.drawFrom(fading.frame, view);
        if (shown) host.drawFrom(shown, view, fade);
        if (fade >= 1) fading = null;

        host.show(view, time - lastUi > UI_INTERVAL_MS);
        if (time - lastUi > UI_INTERVAL_MS) lastUi = time;
        schedule();
        if (zoom >= maxZoom()) {
            host.onEnd();
            return;
        }
        frameRequest = requestAnimationFrame(frame);
    }

    // As deep as the deepest keyframe still covers the path, so the zoom never stalls between
    // them, and no further than the zoom may stretch it
    function nextKeyframeZoom() {
        const deepest = keyframes.at(-1);
        let far = Math.min(deepest ? deepest.view.zoom * AHEAD : zoom, maxZoom());
        if (!deepest || covers(deepest, viewAt(far))) return far;
        let near = deepest.view.zoom;
        for (let i = 0; i < 8; i++) {
            const middle = Math.sqrt(near * far);
            if (covers(deepest, viewAt(middle))) near = middle;
            else far = middle;
        }
        return Math.max(near, deepest.view.zoom * MIN_KEYFRAME_STEP, Math.min(zoom, far));
    }

    function schedule() {
        const deepest = keyframes.at(-1)?.view.zoom ?? zoom / AHEAD;
        if (pending || deepest >= maxZoom() || deepest > zoom * AHEAD) return;
        pending = true;
        const started = epoch;
        const { width, height } = host.size();
        const view = viewAt(nextKeyframeZoom());
        const since = performance.now();
        const choose = rendered++ % CHOOSE_ITERATIONS_EVERY === 0;
        host.renderKeyframe(view, Math.round(width * scale), Math.round(height * scale), choose).then((finished) => {
            if (!active || started !== epoch) return;
            pending = false;
            // Render time grows with the pixel count, so with the square of the scale
            const seconds = (performance.now() - since) / 1000;
            scale = keyframeScale(seconds, scale);
            keyframes.push(finished);
            // Only keyframes at least as deep as the one covering the zoom are still useful
            const covering = keyframes.findLastIndex((frame) => frame.view.zoom <= zoom);
            keyframes = keyframes.slice(Math.max(covering, 0));
            if (zoom >= manualUntil) steer(finished);
        });
    }

    // Keyframes deeper than the current zoom were rendered for the old path
    function dropAhead() {
        keyframes = keyframes.filter((frame) => frame.view.zoom <= zoom || frame === shown);
        epoch++;
        pending = false;
    }

    function steer(frame) {
        const target = detailNear(frame, path.at(-1).target);
        if (target) aimFrom(Math.max(frame.view.zoom, zoom), target);
    }

    // The most detailed point of `frame` near `around`, scored on a coarse grid
    function detailNear(frame, around) {
        const columns = DETAIL_COLUMNS;
        const rows = Math.max(1, Math.round((columns * frame.canvas.height) / frame.canvas.width));
        const grid = document.createElement('canvas');
        grid.width = columns;
        grid.height = rows;
        const context = grid.getContext('2d', { willReadFrequently: true });
        context.imageSmoothingQuality = 'high';
        context.drawImage(frame.canvas, 0, 0, columns, rows);
        const pixels = context.getImageData(0, 0, columns, rows).data;
        const at = (x, y) => 4 * (y * columns + x);
        const white = (x, y) => Math.min(pixels[at(x, y)], pixels[at(x, y) + 1], pixels[at(x, y) + 2]) >= WHITE;
        const whiteNear = (x, y) => {
            let count = 0;
            let total = 0;
            for (let ny = Math.max(y - WHITE_NEIGHBORHOOD, 0); ny <= Math.min(y + WHITE_NEIGHBORHOOD, rows - 1); ny++) {
                for (let nx = Math.max(x - WHITE_NEIGHBORHOOD, 0); nx <= Math.min(x + WHITE_NEIGHBORHOOD, columns - 1); nx++) {
                    count += white(nx, ny);
                    total++;
                }
            }
            return count / total;
        };

        // Keyframes span the view's width at any resolution
        const cell = (host.pixelSize(frame.view) * host.size().width) / columns;
        const centerX = difference(around.x, frame.view.x) / cell + columns / 2;
        const centerY = difference(around.y, frame.view.y) / cell + rows / 2;
        const radius = SEARCH_RADIUS * columns;
        let best = null;
        let bestScore = 0;
        for (let y = 0; y < rows - 1; y++) {
            for (let x = 0; x < columns - 1; x++) {
                const distance = Math.hypot(x + 0.5 - centerX, y + 0.5 - centerY);
                if (distance > radius) continue;
                let contrast = 0;
                for (let channel = 0; channel < 3; channel++) {
                    const value = pixels[at(x, y) + channel];
                    contrast += Math.abs(value - pixels[at(x + 1, y) + channel]);
                    contrast += Math.abs(value - pixels[at(x, y + 1) + channel]);
                }
                const score = contrast * (1 - whiteNear(x, y)) ** WHITE_PENALTY * Math.exp(-(((2 * distance) / radius) ** 2));
                if (score > bestScore) {
                    bestScore = score;
                    best = { x: x + 0.5, y: y + 0.5 };
                }
            }
        }
        if (!best) return null;
        const [x, y] = pan(frame.view.x, frame.view.y, frame.view.zoom, (best.x - columns / 2) * cell, (best.y - rows / 2) * cell);
        return { x, y };
    }

    function start() {
        if (active) return;
        active = true;
        const view = host.view();
        zoom = view.zoom;
        path = [];
        aimFrom(zoom, { x: view.x, y: view.y });
        const finished = host.finished();
        keyframes = finished ? [finished] : [];
        shown = keyframes[0] ?? null;
        fading = null;
        pending = false;
        scale = keyframeScale(host.lastRenderSeconds());
        rate = 0;
        rendered = 0;
        epoch++;
        manualUntil = 0;
        if (finished) steer(finished);
        lastTime = lastUi = performance.now();
        frameRequest = requestAnimationFrame(frame);
        host.onChange(true);
    }

    // Leaves the view as it is and returns the keyframe it is drawn from
    function stop() {
        if (!active) return null;
        active = false;
        cancelAnimationFrame(frameRequest);
        epoch++;
        host.onChange(false);
        return shown;
    }

    // Heads for a point of the canvas instead, for a while
    function retarget(px, py) {
        const view = viewAt(zoom);
        const { width, height } = host.size();
        const size = host.pixelSize(view);
        const [x, y] = pan(view.x, view.y, view.zoom, (px - width / 2) * size, (py - height / 2) * size);
        aimFrom(zoom, { x, y });
        manualUntil = zoom * MANUAL_ZOOM;
        dropAhead();
    }

    return { start, stop, retarget, get active() { return active; } };
}
