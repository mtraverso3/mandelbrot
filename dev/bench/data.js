window.BENCHMARK_DATA = {
  "lastUpdate": 1790351185814,
  "repoUrl": "https://github.com/mtraverso3/mandelbrot",
  "entries": {
    "Mandelbrot benchmarks": [
      {
        "commit": {
          "author": {
            "email": "marcostraverso2003@gmail.com",
            "name": "Marcos Traverso",
            "username": "mtraverso3"
          },
          "committer": {
            "email": "marcostraverso2003@gmail.com",
            "name": "Marcos Traverso",
            "username": "mtraverso3"
          },
          "distinct": true,
          "id": "ef70eee4af08e40767a8e5ffebeb441fd8a4b74c",
          "message": "Bump workflow actions and keep master bench runs from cancelling",
          "timestamp": "2026-09-25T10:43:47-05:00",
          "tree_id": "6ab6aa09d86fe587f6253f76437f2614ba294cb8",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/ef70eee4af08e40767a8e5ffebeb441fd8a4b74c"
        },
        "date": 1790351185364,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 365.52,
            "range": "± 32.09",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 347.22 ms, max 437.97 ms, 5 samples"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 270.29,
            "range": "± 8.61",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 269.49 ms, max 291.92 ms, 5 samples"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 772.22,
            "range": "± 7.64",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 756.47 ms, max 773.84 ms, 5 samples"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 595.8,
            "range": "± 2.31",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 592.33 ms, max 597.55 ms, 5 samples"
          },
          {
            "name": "render/spirals/normal",
            "value": 729.43,
            "range": "± 11.52",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 707.73 ms, max 734.21 ms, 5 samples"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 583.98,
            "range": "± 7.78",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 580.28 ms, max 598.38 ms, 5 samples"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 1094.2,
            "range": "± 0.84",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 1093.71 ms, max 1096.06 ms, 5 samples"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 5860.39,
            "range": "± 153.45",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 5548.49 ms, max 5886.11 ms, 3 samples"
          },
          {
            "name": "encode/png/full-res",
            "value": 45.98,
            "range": "± 0.39",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 45.63 ms, max 46.65 ms, 5 samples"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 259.75,
            "range": "± 0.31",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 259.43 ms, max 260.26 ms, 5 samples"
          }
        ]
      }
    ]
  }
}