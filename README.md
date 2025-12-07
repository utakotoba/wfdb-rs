### wfdb-rs

Pure Rust implementation of the [Waveform Database (WFDB)](https://physionet.org/content/wfdb) specification. This implementation is based on the [official specification](https://github.com/wfdb/wfdb.github.io/tree/ec26a8201e5279b2eca4805f944124dda82dedc2/spec).

### Status

The basic decoding of Waveform Database (WFDB) format files is implemented.

> [!NOTE]
> Working on supporting more formats and bringing the library to WebAssembly.

> [!WARNING]
> Versioning strategy:
> - v0.1.x: breaking changes may occur in patch releases.
> - v0.x.x (except v0.1.x): breaking changes may occur in minor releases.

> [!CAUTION]
> This library is not stable; it may not handle all kinds of waveform database files. I'm working on testing the library on more datasets in WFDB format, but it still takes some time to finish. If you find some problem processing your datasets, feel free to open an issue or PR.

### License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Acknowledgments

- [WFDB](https://physionet.org/content/wfdb)
- [WFDB C Library](https://github.com/bemoody/wfdb)
- [WFDB Python Library](https://github.com/MIT-LCP/wfdb-python)
