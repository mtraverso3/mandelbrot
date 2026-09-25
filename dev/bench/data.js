window.BENCHMARK_DATA = {
  "lastUpdate": 1790372418173,
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
      },
      {
        "commit": {
          "author": {
            "email": "marcostraverso2003@gmail.com",
            "name": "Marcos Traverso",
            "username": "mtraverso3"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "47d05e4247bb58e64662ed717b1fc3c55d5d74e4",
          "message": "Add interactive web explorer link to README\n\nAdded link to interactive web explorer for Mandelbrot CLI.",
          "timestamp": "2026-09-25T12:24:34-05:00",
          "tree_id": "847c6d7b4258429bd3b167cc29a46041cc90cc49",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/47d05e4247bb58e64662ed717b1fc3c55d5d74e4"
        },
        "date": 1790357157282,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 58.49,
            "range": "± 6.83",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 55.91 ms, max 79.21 ms, 17 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 42.24,
            "range": "± 2.93",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 41.72 ms, max 54.52 ms, 24 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 779.46,
            "range": "± 10.99",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 765.70 ms, max 791.20 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 586.5,
            "range": "± 12.04",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 585.43 ms, max 617.04 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 717.08,
            "range": "± 4.97",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 715.14 ms, max 728.45 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 487.72,
            "range": "± 5.29",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 485.94 ms, max 500.38 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 128.5,
            "range": "± 0.46",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 128.27 ms, max 129.62 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 704.4,
            "range": "± 3.03",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 703.24 ms, max 710.17 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 55.34,
            "range": "± 0.36",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 55.20 ms, max 56.60 ms, 19 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 56.49,
            "range": "± 0.22",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 56.14 ms, max 57.27 ms, 18 samples; AMD EPYC 7763 64-Core Processor"
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
          "id": "0fe4de00b0d240a5047a6f49443a23cd9204a324",
          "message": "Support deep zoom in the CLI and web viewer",
          "timestamp": "2026-09-25T13:32:19-05:00",
          "tree_id": "8ca012407fda00ce7aaff5f481045cac61fa81ec",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/0fe4de00b0d240a5047a6f49443a23cd9204a324"
        },
        "date": 1790361321433,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 60.5,
            "range": "± 7.52",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 58.49 ms, max 88.17 ms, 16 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 42.67,
            "range": "± 0.62",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 42.52 ms, max 44.86 ms, 24 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 820.97,
            "range": "± 1.57",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 820.12 ms, max 824.31 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 601.34,
            "range": "± 14.83",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 600.76 ms, max 638.32 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 767.87,
            "range": "± 2.43",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 763.78 ms, max 770.83 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 530.83,
            "range": "± 5.88",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 519.62 ms, max 532.86 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 130.72,
            "range": "± 0.24",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 130.53 ms, max 131.24 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 736.84,
            "range": "± 0.68",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 735.53 ms, max 737.06 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 5497.91,
            "range": "± 62.90",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 5366.41 ms, max 5501.68 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 602.81,
            "range": "± 2.01",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 601.07 ms, max 606.06 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 54.59,
            "range": "± 0.43",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 54.36 ms, max 56.20 ms, 19 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 58.64,
            "range": "± 0.35",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 58.06 ms, max 59.55 ms, 18 samples; AMD EPYC 7763 64-Core Processor"
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
          "id": "4f128d71c765467da5e02c5dc8f9628bc46856b9",
          "message": "Fit color bands to the view at deep zoom",
          "timestamp": "2026-09-25T13:50:55-05:00",
          "tree_id": "be1ceec13140f260cf4fd8429fc1c33f4b726ed8",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/4f128d71c765467da5e02c5dc8f9628bc46856b9"
        },
        "date": 1790362423523,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 48.03,
            "range": "± 4.64",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 46.09 ms, max 61.03 ms, 21 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 34.84,
            "range": "± 0.64",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 34.15 ms, max 35.74 ms, 29 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 652.01,
            "range": "± 22.91",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 633.57 ms, max 699.53 ms, 5 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 478.89,
            "range": "± 1.24",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 476.55 ms, max 479.67 ms, 5 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 590.6,
            "range": "± 2.04",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 589.88 ms, max 595.26 ms, 5 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 409.92,
            "range": "± 5.41",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 403.22 ms, max 416.10 ms, 5 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 104.16,
            "range": "± 0.91",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 103.97 ms, max 106.99 ms, 10 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 584.88,
            "range": "± 2.66",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 579.84 ms, max 585.95 ms, 3 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 4787.63,
            "range": "± 3.34",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 4786.70 ms, max 4794.21 ms, 3 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 510.79,
            "range": "± 0.87",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 510.11 ms, max 512.63 ms, 5 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 45.41,
            "range": "± 0.19",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 45.29 ms, max 46.24 ms, 22 samples; AMD EPYC 9V74 80-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 48.18,
            "range": "± 0.36",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 47.86 ms, max 49.67 ms, 21 samples; AMD EPYC 9V74 80-Core Processor"
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
          "id": "a790303a3911278cb52f1101c778ae528e95f10f",
          "message": "Trim comments to the non-obvious",
          "timestamp": "2026-09-25T14:13:54-05:00",
          "tree_id": "66bc55809de6e72c59bf4d2fdc9362798f6931e8",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/a790303a3911278cb52f1101c778ae528e95f10f"
        },
        "date": 1790363778663,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 59.89,
            "range": "± 7.77",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 58.22 ms, max 81.82 ms, 16 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 44.52,
            "range": "± 0.97",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 42.70 ms, max 45.34 ms, 23 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 789.47,
            "range": "± 48.09",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 786.94 ms, max 910.57 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 588.01,
            "range": "± 2.24",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 586.74 ms, max 591.82 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 732.18,
            "range": "± 7.25",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 731.48 ms, max 750.01 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 506.78,
            "range": "± 4.07",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 498.55 ms, max 510.65 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 131.05,
            "range": "± 0.44",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 130.97 ms, max 132.34 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 726.45,
            "range": "± 5.06",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 719.41 ms, max 731.77 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 746.66,
            "range": "± 3.67",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 741.94 ms, max 750.92 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 152.85,
            "range": "± 3.4",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 150.07 ms, max 159.29 ms, 7 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 54.81,
            "range": "± 0.28",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 54.65 ms, max 55.81 ms, 19 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 58.83,
            "range": "± 0.49",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 58.52 ms, max 60.75 ms, 17 samples; AMD EPYC 7763 64-Core Processor"
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
          "id": "14b8336b9529e3be0ebfc59a9362ebeb372d2af6",
          "message": "Choose iteration limits automatically",
          "timestamp": "2026-09-25T14:38:16-05:00",
          "tree_id": "e94022f11c586453388aac76227105685757f775",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/14b8336b9529e3be0ebfc59a9362ebeb372d2af6"
        },
        "date": 1790365233833,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 60.3,
            "range": "± 7.03",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 57.46 ms, max 81.44 ms, 16 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 43.12,
            "range": "± 0.97",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 42.85 ms, max 45.69 ms, 23 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 794.34,
            "range": "± 22.15",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 791.90 ms, max 849.57 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 600.01,
            "range": "± 1.58",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 598.07 ms, max 601.75 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 739.32,
            "range": "± 1.44",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 737.98 ms, max 741.95 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 503.65,
            "range": "± 5.64",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 502.29 ms, max 517.45 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 128.63,
            "range": "± 0.08",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 128.52 ms, max 128.75 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 722.29,
            "range": "± 1.11",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 721.14 ms, max 723.84 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 699.51,
            "range": "± 7.29",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 699.01 ms, max 714.72 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 150.78,
            "range": "± 1.09",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 148.60 ms, max 151.54 ms, 7 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 55.58,
            "range": "± 0.36",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 55.43 ms, max 56.68 ms, 18 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 58.83,
            "range": "± 0.48",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 58.43 ms, max 60.60 ms, 17 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "auto-iterations/mini-mandelbrot",
            "value": 34.53,
            "range": "± 2.92",
            "unit": "ms",
            "extra": "1024x820 view, chose 96000 iterations; min 33.67 ms, max 47.59 ms, 29 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "auto-iterations/deep-seahorse",
            "value": 114.8,
            "range": "± 0.93",
            "unit": "ms",
            "extra": "1024x820 view, chose 384000 iterations; min 113.64 ms, max 116.34 ms, 9 samples; AMD EPYC 7763 64-Core Processor"
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
          "id": "0765c59f8e1e6e19d4e8e11ef65ea4c67b9c87dd",
          "message": "Fix CI workflow syntax",
          "timestamp": "2026-09-25T15:25:33-05:00",
          "tree_id": "3aed9572295d1eddcdb1c8342c53998126036673",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/0765c59f8e1e6e19d4e8e11ef65ea4c67b9c87dd"
        },
        "date": 1790368072603,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 58.52,
            "range": "± 8.1",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 57.36 ms, max 81.57 ms, 16 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 42.72,
            "range": "± 0.78",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 42.59 ms, max 44.89 ms, 24 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 792.71,
            "range": "± 45.27",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 792.24 ms, max 905.79 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 595.01,
            "range": "± 1.49",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 594.37 ms, max 598.44 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/spirals/normal",
            "value": 741.22,
            "range": "± 1.61",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 738.94 ms, max 743.34 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 503.44,
            "range": "± 6.46",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 502.12 ms, max 516.33 ms, 5 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 128.57,
            "range": "± 0.11",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 128.49 ms, max 128.80 ms, 8 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 723.49,
            "range": "± 2.75",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 723.03 ms, max 729.07 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 713.25,
            "range": "± 4.8",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 703.12 ms, max 713.37 ms, 3 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 150.97,
            "range": "± 0.84",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 150.06 ms, max 153.02 ms, 7 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "encode/png/full-res",
            "value": 55.35,
            "range": "± 0.54",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 55.16 ms, max 57.29 ms, 19 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 56.84,
            "range": "± 0.29",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 56.52 ms, max 57.59 ms, 18 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "auto-iterations/mini-mandelbrot",
            "value": 34.08,
            "range": "± 0.56",
            "unit": "ms",
            "extra": "1024x820 view, chose 96000 iterations; min 33.60 ms, max 35.78 ms, 30 samples; AMD EPYC 7763 64-Core Processor"
          },
          {
            "name": "auto-iterations/deep-seahorse",
            "value": 113.05,
            "range": "± 0.87",
            "unit": "ms",
            "extra": "1024x820 view, chose 384000 iterations; min 112.40 ms, max 115.56 ms, 9 samples; AMD EPYC 7763 64-Core Processor"
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
          "id": "ae4d71632c7a326efddc3e6a2f6975ad88b8ea36",
          "message": "Add GPU benchmark scenarios",
          "timestamp": "2026-09-25T16:15:05-05:00",
          "tree_id": "27eb64669b8e81e781a863afc9257c98774c8bb6",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/ae4d71632c7a326efddc3e6a2f6975ad88b8ea36"
        },
        "date": 1790371051330,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 41.7,
            "range": "± 4.11",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 38.34 ms, max 54.75 ms, 24 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 28.43,
            "range": "± 2.41",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 27.62 ms, max 36.44 ms, 34 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 510.59,
            "range": "± 0.97",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 510.11 ms, max 512.50 ms, 5 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 391.13,
            "range": "± 12.86",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 382.38 ms, max 418.34 ms, 5 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/spirals/normal",
            "value": 478.45,
            "range": "± 21.35",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 472.44 ms, max 529.78 ms, 5 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 328.92,
            "range": "± 4.72",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 324.65 ms, max 336.22 ms, 5 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 94.69,
            "range": "± 5.96",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 91.13 ms, max 110.66 ms, 11 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 495.47,
            "range": "± 15.99",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 481.26 ms, max 519.97 ms, 3 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 514.09,
            "range": "± 2.77",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 510.94 ms, max 517.73 ms, 3 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 113.29,
            "range": "± 6.45",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 109.94 ms, max 128.14 ms, 9 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "encode/png/full-res",
            "value": 44.96,
            "range": "± 2.4",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 44.66 ms, max 52.07 ms, 22 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 43.74,
            "range": "± 2.86",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 43.07 ms, max 54.41 ms, 23 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "auto-iterations/mini-mandelbrot",
            "value": 26.06,
            "range": "± 1.49",
            "unit": "ms",
            "extra": "1024x820 view, chose 96000 iterations; min 25.55 ms, max 30.07 ms, 38 samples; Intel(R) Xeon(R) 6973P-C"
          },
          {
            "name": "auto-iterations/deep-seahorse",
            "value": 83.62,
            "range": "± 0.65",
            "unit": "ms",
            "extra": "1024x820 view, chose 384000 iterations; min 82.48 ms, max 84.50 ms, 12 samples; Intel(R) Xeon(R) 6973P-C"
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
          "id": "016b3b7b52731c3090d38f37bdef6fcf632f64ce",
          "message": "Render the web viewer on the GPU through WebGPU",
          "timestamp": "2026-09-25T16:38:12-05:00",
          "tree_id": "9a6c0385919e1f3a66d59c4193dc201fd9eb9ac9",
          "url": "https://github.com/mtraverso3/mandelbrot/commit/016b3b7b52731c3090d38f37bdef6fcf632f64ce"
        },
        "date": 1790372417598,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "render/mandelbrot/normal",
            "value": 44.47,
            "range": "± 5.17",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 44.01 ms, max 66.25 ms, 22 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/mandelbrot/flat",
            "value": 32.77,
            "range": "± 4.45",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 32.12 ms, max 44.57 ms, 29 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/mini-mandelbrot/normal",
            "value": 608.3,
            "range": "± 1.82",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 604.60 ms, max 608.75 ms, 5 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/mini-mandelbrot/flat",
            "value": 438.82,
            "range": "± 1.59",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 438.15 ms, max 442.18 ms, 5 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/spirals/normal",
            "value": 563.98,
            "range": "± 0.9",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 563.62 ms, max 566.05 ms, 5 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/quad-spiral/normal",
            "value": 395.8,
            "range": "± 1.66",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 4 threads; min 391.73 ms, max 396.06 ms, 5 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/mandelbrot/normal/1-thread",
            "value": 101.27,
            "range": "± 0.23",
            "unit": "ms",
            "extra": "1024x820, 1500 iterations, 1 threads; min 101.12 ms, max 101.99 ms, 10 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/mandelbrot/normal/full-res",
            "value": 552.49,
            "range": "± 3.82",
            "unit": "ms",
            "extra": "4096x3280, 1500 iterations, 4 threads; min 550.47 ms, max 559.39 ms, 3 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/deep-seahorse/normal",
            "value": 611.57,
            "range": "± 2.33",
            "unit": "ms",
            "extra": "320x256, 30000 iterations, 4 threads; min 610.48 ms, max 615.87 ms, 3 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "render/misiurewicz-1e100/normal",
            "value": 129.65,
            "range": "± 0.94",
            "unit": "ms",
            "extra": "1024x820, 3000 iterations, 4 threads; min 128.46 ms, max 130.67 ms, 8 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "encode/png/full-res",
            "value": 54.61,
            "range": "± 0.33",
            "unit": "ms",
            "extra": "4096x3280, 5369 KiB; min 54.35 ms, max 55.90 ms, 19 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "downsample/lanczos3/full-res",
            "value": 49.94,
            "range": "± 0.31",
            "unit": "ms",
            "extra": "4096x3280 -> 2048x1640; min 49.42 ms, max 50.53 ms, 21 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "auto-iterations/mini-mandelbrot",
            "value": 31.46,
            "range": "± 0.41",
            "unit": "ms",
            "extra": "1024x820 view, chose 96000 iterations; min 30.86 ms, max 32.80 ms, 32 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          },
          {
            "name": "auto-iterations/deep-seahorse",
            "value": 100.61,
            "range": "± 1.42",
            "unit": "ms",
            "extra": "1024x820 view, chose 384000 iterations; min 99.27 ms, max 104.76 ms, 10 samples; INTEL(R) XEON(R) PLATINUM 8573C"
          }
        ]
      }
    ]
  }
}