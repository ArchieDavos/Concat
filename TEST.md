# TEST.md — wykonane testy funkcjonalne MVP

Data: 2026-10-23. Platforma testowa: macOS 27.2 arm64, Apple M1, 8 wątków,
Metal 4, FFmpeg 9.0.2 (homebrew), Rust 1.97.1. Concat v0.2.8, commit `1d04d14`,
**kod bez modyfikacji** (zmieniono wyłącznie środowisko: doinstalowano `cmake`).

## Materiał testowy (legalny, wygenerowany lokalnie)

`test-media/zrodlo_poziome.mp4` (+ remux `.mov`) — skrypt `test-media/generuj_materiat.sh`:
- 45,02 s, 1280×720 (16:9), 30 fps, H.264 High + AAC 48 kHz stereo;
- 5 scen co ~9 s o radykalnie różnych wzorcach: `testsrc2` (ze wbudowanym zegarem),
  `smptehdbars`, `mandelbrot`, `life`, `testsrc` — wyraźne zmiany obrazu;
- narracja: macOS `say -v Zosia` (pl_PL), osobne zdanie w każdej scenie (do napisów
  i weryfikacji audio); `narracja.wav` = ścieżka 16 kHz mono.

## Jak czytać wyniki

- **obecna w kodzie** — funkcja istnieje w źródłach;
- **uruchomiona** — wykonana na prawdziwym procesie/binarnce;
- **przetestowana** — weryfikacja wyniku na powyższym materiale (nie „zadziałało bez błędu”,
  tylko sprawdzony efekt).

## 1. Budowa i testy jednostkowe — PRZECHODZI

- `cargo build` — sukces po instalacji cmake (pierwsza próba: jawny błąd
  `is cmake not installed?` w `whisper-rs-sys`; naprawione środowiskowo, bez zmian w kodzie).
- `cargo test` — **860 passed / 0 failed** (m.in. concat-project 163, concat-media 118,
  concat-render 101 — ten ostatni ~2 min na GPU; export 16, server 28, mcp e2e 3).
- Uruchomienie okna `./target/debug/concat` — uruchomione; ekran startowy wyrenderowany
  poprawnie (zrzut `test-media/gui_start.png`); log: GPU Apple M1 (Metal), dekodowanie
  VideoToolbox. Tryb `--mcp headless` — handshake MCP poprawny (tools + instructions).
- `concat-cli probe/render` — dostępne w kodzie; `render` nie był wymagany do MVP
  (eksport przetestowany przez API).

## 2. Główny test montażu (`testy/test_mvp.py`, 20/20 PASS) — PRZETESTOWANY

Interaktywna sesja JSON-RPC na prawdziwym silniku (`concat-cli api`), projekt
`test-projects/MVP` (1080×1920@30), źródło 45 s:

| # | Krok | Wynik |
|---|---|---|
| 1–2 | `version`, utworzenie projektu 9:16 30 fps | settings zwrocone 1080×1920@30 ✓ |
| 3–4 | import MP4 i MOV do koszyka | mediaId m1/m2 ✓ |
| 5–7 | save, close, reopen | ✓ |
| 8 | klip MP4 na oś | clip c5, 45 s ✓ |
| 9–10 | split w 10 s i 20 s | 3 klipy: c5 0–10, c6 10–20, c7 20–45,02 ✓ |
| 11 | usunięcie środkowego (ripple) | bez luki: c5 0–10, c7 10–35,02 ✓ |
| 12 | zamiana kolejności | 3 osobne moveClips; kolejność odwrócona ✓ |
| 13 | tekst „Concat MVP — test" 0–4 s na torze nad obrazem | ✓ |
| 14 | volume=0.3 (klip A), muted=true (klip C) | ✓ |
| 15 | stan osi po montażu | wszystkie atrybuty zgodne ✓ |
| 16–18 | save → close → reopen | montaż w pełni zachowany ✓ |
| 19–20 | eksport (job j1) + event `export.done` | 1080×1920 ✓ |

Uwaga: zamiana kolejności wymagała 3 wywołań — wsadowy `moveClips` zakładających się
klipów jest przez silnik **po cichu ignorowany** (`resolve_moves` → brak ruchu,
`src/crates/concat-project/src/commands/clips.rs:240-259`). To semantyka silnika,
nie błąd; UI przeciąga klipy pojedynczo.

## 3. Weryfikacja pliku wyeksportowanego (`verify_export.py`, 11/11 PASS)

`eksport_mvp.mp4`: H.264 ✓, 1080×1920 ✓, 30 fps ✓, AAC 48 kHz stereo ✓, 35,033 s ✓
(10 + 25,02 po montażu). Ponadto:
- **kolejność scen** — porównanie ramek (MSE po wykryciu pasma treści): t=2 → źródło 22 s
  (mandelbrot), t=15 → źródło 35 s (life), t=30 → źródło 5 s (testsrc2) — wszystkie MSE
  33–235 (próg 300) ✓. Oznacza to poprawne odwzorowanie montażu w pikselach.
- **audio** — klip wyciszony: mean −91 dB; klip ze ściszoną narracją: −24,8 dB ✓.
- **tekst** — nakładka widoczna w pierwszych 4 s (różnica górnej ćwiartki MSE 3241) ✓;
  ramki do wglądu: `test-media/verify/eksport_t*.png` (9:16, pillarbox czarny, treść 16:9
  dopasowana i wyśrodkowana).
- odtwarzanie pliku: sprawdzane przez dekodowanie ffmpeg (wszystkie ramki/audio powyżej);
  obejrzane ramki — zgodne z oczekiwaniem.

## 4. Przycinanie i podgląd (`testy/test_trim.py`, 8/8 PASS) — PRZETESTOWANY

Projekt TRIM: trim głowy +2 s i ogona −3 s → klip 40,02 s, `sourceStart=2`,
`start=2` (tryb bez ripple zostawia treść w miejscu — semantyka silnika, potwierdzona
w `commands/clips.rs:261+`); zachowane po save/reopen ✓. `preview.frame` (t=3) zwróciło
nieczarną, poprawnie skomponowaną klatkę 9:16 (`verify/trim_preview.png` — scena testsrc2
z źródła t=1, czyli przesunięcie in-pointu zadziałało).

## 5. Automatyczne napisy (Whisper) — URUCHOMIONE I PRZETESTOWANE (poziom silnika)

- Transkrypcja jest **funkcją okna** (UI), nie ma jej w JSON-RPC ani MCP (potwierdzone
  w kodzie: `concat-api` nie ma takiej metody; UI → `concat-speech` → batch `addTextClip`).
- Przetestowano realną ścieżkę silnika: pomijany test `transcribes_a_file_with_the_installed_model`
  (`cargo test -p concat-speech -- --ignored real_audio`) na pliku `narracja.wav`.
  Model: **ggml-tiny (77 MB, wielojęzyczny)** — jako `ggml-tiny.en.bin`, bo test ma
  na sztywno id `tiny.en`; whisper.cpp odczytuje metadane z bajtów pliku, więc załadował
  się model wielojęzyczny. **Podstawienie czysto testowe, udokumentowane.**
- Wynik: **ok** — 45 s audio w ~10 s na M1 (Metal); w logu tokeny polskie
  („…regulujemy głośności…"), assertion niepustych segmentów przeszło.
- **Nie przetestowane:** pełny przepływ UI (Captions → Generate → napisy na osi)
  oraz dokładność timestamps — wymaga ręcznej weryfikacji w oknie.

## 6. Co NIE zostało przetestowane / ograniczenia

1. **Interaktywna praca w oknie** (mysz/klawiatura): okno uruchomione i wyrenderowane,
   ale montaż był prowadzony przez API — te same komendy co UI, lecz sam interfejs
   (timeline pod kursor, skróty) nie był klikany. Odtwarzanie podglądu **z dźwiękiem**
   w oknie niezweryfikowane (ścieżka audio silnika zweryfikowana przez eksport AAC).
2. **Windows x64**: priorytetowa platforma docelowa, ale ta maszyna to macOS — budowa
   Windows udokumentowana w BUILD.md, niewykonana. Zgodność kodu z Windows nie naruszono
   (zero zmian w źródłach).
3. **Napisy przez UI** — jak wyżej; silnik działa, UI nieklikane.
4. MOV przetestowany tylko w imporcie (probe + bin); eksport z MOV nie był osobno
   weryfikowany (tożsame demuxer).
5. Nie testowano: efekty/przejścia, keyframy, TTS, usuwanie tła, stabilizacja, proxy,
   eksport HEVC/AV1, HDR/10-bit, 4K — poza zakresem MVP.
6. Zależność od sha256 modelu: podstawiony plik `ggml-tiny.en.bin` nie zgadza się
   z manifestem (to bajty modelu `tiny`) — tylko lokalnie, do testu.

## Lista błędów napotkanych i rozwiązanych

| Błąd | Przyczyna | Rozwiązanie |
|---|---|---|
| `is cmake not installed?` przy budowie `whisper-rs-sys` | brak cmake w systemie | `brew install cmake` (środowisko, nie kod) |
| `unknown variant 'timeline.read'` | metoda nie istnieje w API (nazwa z MCP) | użyć `project.get` |
| `missing field 'trackId'` w moveClips | pole obowiązkowe | przekazać trackId klipu |
| Zamiana kolejności bez efektu | wsadowe ruchy zakładające się = cichy brak ruchu | 3 osobne moveClips |
| Podgląd czarny w t=1 | po trimie głowy oś zaczyna się w t=2 | żądać klatki w zakresie klipu |

## Test dodatkowy: wiele plików (wideo + zdjęcie) w jednym projekcie

Materiał: `test-media/zdjecie_test.png` (1280×720, PNG) + `test-media/film_test.mp4`
(45,02 s, 1280×720@30) — oba wygenerowane lokalnie ffmpeg.

Kroki (jedna sesja `concat-cli api`, projekt 1080×1920@30, `test-projects/WIELE`):
1. `media.import` PNG → `m1`, `media.import` MP4 → `m2`.
2. Zdjęcie na oś: `timeline.addClip` → `c3`, `kind=image`, `duration=5.0`, `start=0.0`
   (zdjęcia domyślnie dostają 5 s, można je przycinać/rozszerzać).
3. Wideo na oś: `c4`, `start=5.0` (od razu za zdjęciem).
4. `export.run` → **OK**, wynik: `test-media/eksport_wiele.mp4`.

Wynik weryfikacji (ffprobe + ramka t=2, `eksport_wiele_t2.png`):
- **PASS** — `duration=50,033 s` = 5 s zdjęcie + 45,02 s wideo (suma klipów się zgadza),
- **PASS** — ramka t=2 pokazuje zdjęcie (gradient żółto-zielony) w pionowej ramce 9:16
  z czarnymi pasami — zdjęcie faktycznie jest w eksporcie, nie tylko na osi.

Wniosek: edytor przyjmuje wiele plików różnego typu (wideo + zdjęcia) w jednym projekcie;
każdy dostaje własny klip na osi czasu i jest uwzględniany w eksporcie. Import wielu
plików naraz przez UI (przeciągnięcie) — funkcja obecna w kodzie (drag&drop), klikane
ręcznie nie było; ścieżka API przetestowana jak wyżej.
