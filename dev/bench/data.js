window.BENCHMARK_DATA = {
  "lastUpdate": 1790356456707,
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
      },
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
          "id": "4013e486c92e2e4ade4b254d6319afc31c2c1f03",
          "message": "Use as_chunks_mut for fixed-size pixel chunks",
          "timestamp": "2026-09-25T11:21:14-05:00",
          "tree_id": "8a76a7aaeaee05b9dd82d7c7dc75b599249d219d",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/4013e486c92e2e4ade4b254d6319afc31c2c1f03"
        },
        "date": 1790353386895,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 60.66,
            "range": "± 7.84",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 56.93 ms, max 78.19 ms, 5 samples"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 48.63,
            "range": "± 6.42",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 42.39 ms, max 57.71 ms, 5 samples"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 789.57,
            "range": "± 31.52",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 786.63 ms, max 869.34 ms, 5 samples"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 593.51,
            "range": "± 54.59",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 588.67 ms, max 729.45 ms, 5 samples"
          },
          {
            "name": "render/spirals/normal",
            "value": 731.97,
            "range": "± 14.32",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 730.91 ms, max 768.23 ms, 5 samples"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 509.46,
            "range": "± 1.09",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 508.02 ms, max 511.20 ms, 5 samples"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 129.75,
            "range": "± 0.64",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 129.71 ms, max 131.36 ms, 5 samples"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 741.59,
            "range": "± 21.9",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 716.00 ms, max 769.62 ms, 3 samples"
          },
          {
            "name": "encode/png/full-res",
            "value": 56.24,
            "range": "± 0.43",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 55.78 ms, max 56.87 ms, 5 samples"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 58.56,
            "range": "± 0.17",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 58.32 ms, max 58.81 ms, 5 samples"
          }
        ]
      },
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
          "id": "3a26fb8c258d3c8a9964b243222487726b3b4375",
          "message": "Benchmark base and head on the same runner",
          "timestamp": "2026-09-25T11:32:01-05:00",
          "tree_id": "8a77f8070a6da64ef2b61ffbde84b3c64157f1a4",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/3a26fb8c258d3c8a9964b243222487726b3b4375"
        },
        "date": 1790354052934,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 58.64,
            "range": "± 2.16",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 55.93 ms, max 64.01 ms, 17 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 43.86,
            "range": "± 7.01",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 43.28 ms, max 59.87 ms, 21 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 767.46,
            "range": "± 1.91",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 766.49 ms, max 771.92 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 615.45,
            "range": "± 1.07",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 615.09 ms, max 617.96 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 715.35,
            "range": "± 8.12",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 713.65 ms, max 735.32 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 487.41,
            "range": "± 0.45",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 487.12 ms, max 488.41 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 129.16,
            "range": "± 3.25",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 128.96 ms, max 138.99 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 705.64,
            "range": "± 0.86",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 704.06 ms, max 706.06 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 55.23,
            "range": "± 0.49",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 55.08 ms, max 57.13 ms, 19 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 56.21,
            "range": "± 0.58",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 55.84 ms, max 57.97 ms, 18 samples; AMD EPYC 7763 64-Core Processor"
          }
        ]
      },
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
          "id": "304ffdda9b06b4942f762e346cd200eb4a258dbc",
          "message": "Add web CI, deploy and PR preview workflows",
          "timestamp": "2026-09-25T12:12:09-05:00",
          "tree_id": "b7c54633fc73d79ba5565aaf1436015d867f27a5",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/304ffdda9b06b4942f762e346cd200eb4a258dbc"
        },
        "date": 1790356455785,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 58.95,
            "range": "± 4.1",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 56.58 ms, max 74.01 ms, 17 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 44.78,
            "range": "± 6.44",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 41.83 ms, max 58.66 ms, 21 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 770.07,
            "range": "± 2.5",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 766.33 ms, max 773.75 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 587.36,
            "range": "± 1.43",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 585.65 ms, max 589.65 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 731.06,
            "range": "± 14.37",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 714.27 ms, max 752.69 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 486.34,
            "range": "± 7.32",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 485.54 ms, max 501.75 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 128.49,
            "range": "± 0.36",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 128.45 ms, max 129.40 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 704.56,
            "range": "± 3.74",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 704.03 ms, max 712.21 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 55.87,
            "range": "± 0.34",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 55.71 ms, max 57.31 ms, 18 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 58.96,
            "range": "± 0.52",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 58.50 ms, max 60.41 ms, 17 samples; AMD EPYC 7763 64-Core Processor"
          }
        ]
      }
    ]
  }
}