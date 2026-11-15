# BUILD.md — budowa Concat z źródeł (MVP short-form)

Stan: 2026-10-23, Concat v0.2.8, commit `1d04d14`. Zweryfikowana budowa: **macOS 27.2
arm64 (Apple M1)**. Instrukcje dla Windows x64 i Linux wg dokumentacji projektu —
nie testowane tutaj, ale oparte na README/CI upstreamu.

## Wymagania

| Narzędzie | Wersja | Uwagi |
|---|---|---|
| Rust toolchain | ≥ 1.93 (`rust-version` w `src/Cargo.toml`) | `cargo --version` |
| FFmpeg | ≥ 7 **biblioteki deweloperskie** (nagłówki + .dylib/.so/.dll) | linkowane przez `ffmpeg-the-third` |
| cmake | dowolna nowa | wymagany przez whisper.cpp (cmake w build script) |
| Kompilator C/C++ | clang (macOS), MSVC (Windows) | wymagany przez whisper.cpp/sherpa-onnx |
| Python 3 | ≥ 3.9 | tylko do testów MVP (`testy/*.py`) |

GPU: opcjonalna; bez niej silnik używa renderera programowego. Na M1 wykryto Metal
(`wgpu`), dekodowanie sprzętowe VideoToolbox. Slint pobiera prebuilt Skia przy budowie.

## macOS (zweryfikowane krok po kroku)

```bash
brew install ffmpeg cmake   # ffmpeg 9.0.2 + cmake 4.4.4 — działa
git clone https://github.com/jub0t/concat.git
cd concat/src
cargo build                 # ~5 min na M1; pierwsza budowa dłuższa (pobieranie zależności)
./target/debug/concat       # okno edytora
```

**Znany problem środowiskowy nr 1:** brak `cmake` objawia się wysypaniem build scriptu
`whisper-rs-sys` komunikatem `failed to execute command: No such file or directory (os error 2) / is cmake not installed?`.
Rozwiązanie: `brew install cmake` i ponowna budowa. Kompilacja `cmake` crate nie zgłasza
czytelnego błędu — sprawdź `which cmake` przed budową.

**Znany problem nr 2 (kosmetyczny):** linker warning `__eh_frame section too large …
compact unwind table` przy linkowaniu binarki `concat` na macOS arm64 — nieszkodliwy.

## Testy jednostkowe

```bash
cd src && cargo test        # 860 testów przechodzi; concat-render ~2 min (GPU)
```

Test transkrypcji na prawdziwym audio (pomijany domyślnie):
```bash
# wymaga pobranego modelu w ~/Library/Application Support/app.concat.editor/whisper-models/
CONCAT_TRANSCRIBE_AUDIO=/path/audio.wav cargo test -p concat-speech -- --ignored real_audio
```

## Modele AI (pobierane osobno, lokalnie)

Aplikacja pobiera modele przy pierwszym użyciu (Settings). Ręcznie: umieść plik w
katalogu danych (`<app data>/whisper-models/ggml-<id>.bin`). Manifest z rozmiarami
i sha256: `models/manifest.toml`.

| Model | Rozmiar | Zastosowanie |
|---|---|---|
| `ggml-tiny.bin` / `ggml-tiny.en.bin` | **77 MB** | transkrypcja (tiny.en tylko angielski; tiny wielojęzyczny) — najmniejszy wystarczający do testu |
| `ggml-base(.en).bin` | 148 MB | transkrypcja lepsza jakościowo |
| `ggml-small(.en).bin` | 488 MB | transkrypcja najlepsza w ofercie |
| Kokoro TTS int8 | 132 MB | text-to-speech |
| cutout (RVM/ISNet/SlimSAM) | 15–179 MB | usuwanie tła |

Wybór do pierwszego testu napisów: **`ggml-tiny.bin` (77 MB)** — potwierdzono transkrypcję
polską na M1 w ~10 s dla 45 s audio.

## Windows x64 (priorytetowa platforma produktu — wg dokumentacji, nie testowane tutaj)

1. Rust: https://rustup.rs (x86_64-pc-windows-msvc).
2. FFmpeg: z https://ffmpeg.org/download.html#build-windows wziąć **BtbN `win64-gpl-shared`**
   (takiego używa CI), rozpakować i ustawić `FFMPEG_DIR=<folder>` (zawierający `include/` i `lib/`).
3. Visual Studio Build Tools (MSVC + Windows SDK) — cmake i C++.
4. `cd src && cargo build --release -p concat` — binarka `target\release\concat.exe`.
   Instalator gotowy: GitHub Releases (setup + .msi, buildy niepodpisane — SmartScreen:
   „More info › Run anyway").
5. Uwaga z README: buildy Windows ARM wstrzymane (issue #326) — priorytet x64 jest zgodny.

## Linux

Dystrybucyjny FFmpeg ≥ 7 wystarcza; starszy → build BtbN `shared` + `FFMPEG_DIR`
jak na Windows. Skia wymaga nagłówków fontconfig/freetype (lista pakietów w
`.github/workflows/build-app.yml`). Alternatywnie flake Nix: `nix build`.

## Profil „shipping"

```bash
cargo build --profile app -p concat   # fat LTO, stripped, panic=abort
```

## Uruchomienie bez budowania (dla agentów/skryptów)

```bash
./target/debug/concat-cli api '<json-rpc>'   # jedno wywołanie JSON-RPC
./target/debug/concat-cli api < skrypt.jsonl # wiele wywołań + eventy
./target/debug/concat --mcp headless         # serwer MCP (Kimi/Claude)
```
