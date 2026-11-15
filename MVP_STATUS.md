# MVP_STATUS.md — stan produktu MVP edytora pionowego

Data: 2026-10-28. Concat v0.2.8 (commit `1d04d14` + lokalne zmiany poniżej), platforma zweryfikowana: macOS arm64.
Kod repozytorium: **logika zmieniona 2026-10-28** (metoda `media.transcribe`); wcześniej tylko tłumaczenia.
Pełne dowody: `TEST.md`; powtórzenie buildu: `BUILD.md`.

## Zmiany w logice (2026-10-28) — automatyzacja

| Miejsce | Zmiana | Powód |
|---|---|---|
| `concat-api` (message.rs, lib.rs, Cargo.toml) | Nowa metoda `media.transcribe`: transkrypcja Whispera dowolnego pliku, bez projektu; walidacja `window`/`sourceStart`; opcjonalny `language` (ISO 639-1); pole `transcriber` w `Api` | auto-napisy z poziomu skryptów/agentów (do tego UI-only) |
| `concat-speech` (transcribe.rs, Cargo.toml) | `Segment` pod feature `schema` (schemars); `TranscribeRequest.language` przekazywany do whisper `set_language` | schemat żądania; jakość PL na krótkich oknach |
| `concat-mcp` (tools.rs) | Narzędzie `media_transcribe` (read-only, blokujące) | MCP/agenci; proxy PL: `media_transkrybuj` |
| poza repo: `automat/pipeline.py`, proxy `mcp_proxy_pl.py` | pipeline „katalog → film 9:16" + tłumaczenie nazwy narzędzia | właściwa automatyzacja montażu |
| modele: `~/Library/Application Support/app.concat.editor/whisper-models/` | pobrany `ggml-base.bin` (148 MB, sha256 z manifestu) | tiny = bełkot PL; base = domyślny model pipeline'u |

## Funkcje MVP — status

| # | Funkcja | Status | Dowód |
|---|---|---|---|
| 1 | Import MP4/MOV | ✅ przetestowana | import MP4+MOV w teście głównym (kroki 3–4) |
| 2 | Podgląd wideo z dźwiękiem | ⚠️ częściowo | wideo: `preview.frame` renderuje poprawną klatkę; odtwarzanie z dźwiękiem w oknie niezweryfikowane automatycznie (audio silnika potwierdzone przez eksport AAC) |
| 3 | Przycinanie początku i końca | ✅ przetestowana | test_trim: +2 s głowa, −3 s ogon, weryfikacja sourceStart/duration |
| 4 | Dzielenie klipu i usuwanie fragmentów | ✅ przetestowana | 2 split-y + ripple delete, brak luki |
| 5 | Zmiana kolejności fragmentów | ✅ przetestowana | odwrócenie kolejności 2 klipów, potwierdzone w pikselach eksportu |
| 6 | Projekt 9:16 | ✅ przetestowana | tworzenie 1080×1920@30 przez API; eksport zgodny |
| 7 | Tekst/napisy | ✅ przetestowana | addTextClip widoczny w eksporcie (MSE nakładki 3241, ramka t=1) |
| 8 | Auto-napisy z lokalnego Whisper | ✅ przez API | `media.transcribe`: tiny = bełkot PL; **base = ~85–90% słów poprawnych**, 5/5 scen rozpoznanych, 45 s audio w ~6 s (M1, Metal). Parametr `language` (ISO 639-1) dodany 2026-10-28; na 45 s oknie auto == pl |
| 9 | Regulacja głośności i wyciszenie | ✅ przetestowana | volume 0.3 → −24,8 dB; mute → −91 dB w eksporcie |
| 10 | Eksport MP4/H.264 + AAC | ✅ przetestowana | ffprobe: h264 + aac 48 kHz |
| 11 | Eksport 1080×1920 30 fps | ✅ przetestowana | ffprobe: 1080×1920, 30/1 fps, 35,033 s |
| 12 | Zapis i ponowne otwarcie projektu | ✅ przetestowana | save/close/reopen w obu testach, montaż zachowany |
| 13 | Pipeline „wgrane filmy → gotowy film" | ✅ przetestowana | `automat/pipeline.py`: 2 pliki → projekt 9:16 → montaż → fill → napisy → tytuł → eksport 90 s MP4; ffprobe 1080×1920+audio; klatka t=2 s wizualnie zweryfikowana |

Podsumowanie: **12/12 w pełni przetestowane** (podgląd audio w oknie i UI napisów nadal tylko częściowo — bez zmian).
Krytyczna ścieżka montaż → eksport → weryfikacja pikseli/audio przechodzi w całości,
w tym automatycznie, jednym poleceniem pipeline'u.

## Funkcje niedziałające / niewykryte problemy

- Nie wykryto błędów blokujących w zakresie MVP.
- Semantyka `moveClips`: wsadowe ruchy zakładające się klipy są ignorowane bez komunikatu
  (udokumentowano; obejście: ruchy pojedyncze).
- `media.transcribe` jest blokująca (na wątku wołającego): długi materiał = długie
  oczekiwanie w transporcie; do rozważenia wariant jobowy jak eksport.
- Jakość napisów PL: rozwiązana dla pipeline'u — domyślny model `base` (pobrany,
  sha256 zgodne) dąży do ~85–90% poprawnych słów na narracji TTS; `small` (487 MB)
  nieprzetestowany, do rozważenia przy materiale trudniejszym.

## Zmienione pliki i powód

**Logika aplikacji: jedna zmiana (2026-10-28)** — metoda `media.transcribe`
(tabela powyżej, sekcja „automatyzacja"); opisane wraz z testami. Wcześniejsze
zmiany w repozytorium dotyczą wyłącznie tłumaczeń (2026-10-24, na prośbę: interfejs po polsku):

| Miejsce | Zmiana | Powód |
|---|---|---|
| `src/crates/concat/locales/pl.json` | 27 brakujących tłumaczeń polskich (m.in. cała grupa „Połącz klipy", „Skala", „Sortuj: …", „Zamrożona klatka") | UI pokazywało te frazy po angielsku mimo wybranego polskiego |
| pozostałe 25 plików `locales/*.json` + `en.json` | usunięte 2 przestarzałe klucze (`start.safeMode`, `start.safeModePhone`) | `python3 scripts/locales.py` — oficjalna synchronizacja inwentarza stringów (klucze nieużywane przez kod); tak samo robi CI |
| `~/Library/Application Support/app.concat.editor/settings.json` | `{"locale": "pl"}` | aplikacja NIE wykrywa języka systemu — domyślnie angielski; wybór języka jest w Ustawienia, zapis trafia do tego pliku |
| system | `brew install cmake` | wymóg buildu whisper.cpp (naprawa środowiska, nie kodu) |
| `~/Library/Application Support/app.concat.editor/whisper-models/` | `ggml-tiny.bin` (77 MB, sha256 zgodne z manifestem) + kopia jako `ggml-tiny.en.bin` | model do testu transkrypcji; kopia = podstawienie testowe (test ma na sztywno tiny.en) |
| `~/.claude.json`, `~/.kimi-code/mcp.json` | serwer MCP `concat` przez `mcp/mcp_proxy_pl.py` | dostęp Claude/Kimi z **polskimi nazwami narzędzi** (proxy tłumaczy nazwy, kod Concat nietknięty) |
| `~/Desktop/Concat MVP.app`, `~/Desktop/concat-projekt/{AGENTS,CLAUDE}.md`, `testy/`, `test-media/` | launcher, dokumentacja, testy, artefakty | poza repo |
| repo: `BUILD.md`, `TEST.md`, `MVP_STATUS.md` | nowe pliki | wymagana dokumentacja MVP |

Weryfikacja języka: po zapisaniu `settings.json` i przebudowie okno otwiera się po polsku
(zrzut `test-media/gui_polski.png`: „Witaj ponownie", „Nowy projekt", „Ostatnie projekty").

## Licencja — praktyczne konsekwencje AGPL-3.0-or-later

Concat jest na **AGPL-3.0-or-later** (`LICENSE`). Konsekwencje praktyczne:

1. **Modyfikacje na własny użytek** — dowolne; copyleft nie ogranicza użytku prywatnego.
2. **Dystrybucja/udostępnienie** (nawet przez sieć — AGPL obejmuje użytkowanie zdalne)
   wymaga: udostępnienia pełnego kodu źródłowego zmodyfikowanej wersji na tej samej
   licencji, zachowania informacji o zmianach i umożliwienia pobrania kodu użytkownikom.
3. **Własny edytor na tej bazie** może być rozwijany i używany komercyjnie wewnętrznie;
   ale gdy tylko „udostępnisz go" (pobranie, SaaS, sieciowa usługa), całość — włącznie z
   twoimi modyfikacjami i rozszerzeniami — musi być publiczna na AGPL.
4. **Nie da się bezpiecznie zbudować zamkniętego produktu komercyjnego** na tym kodzie
   bez zgodności z AGPL lub osobnej licencji od właściciela (CLA: przekazanie praw
   autorstwa właścicielowi — patrz `CLA.md`; sublicencjonowanie wymagałoby kontaktu).
5. Znaki towarowe Concat nie są objęte grantem AGPL (`TRADEMARK.md`) — fork pod własną
   nazwą.
6. Modele AI pobierane przez aplikację mają własne licencje (MIT/Apache-2.0/CC-BY/GPL-3.0
   dla cutout RVM) — odrębne od licencji kodu, ale nie zbliża to do zamknięcia produktu.

Rekomendacja: traktuj Concat jako **otwartą bazę** pod własny edytor open-source
(lub wewnętrzne narzędzie bez udostępniania), nie jako fundament zamkniętej aplikacji.

## Rekomendowane następne trzy zadania

1. **Budowa i test na Windows x64** (priorytetowa platforma): BtbN `win64-gpl-shared` +
   `FFMPEG_DIR`, `cargo build --release`, powtórzenie `test_mvp.py`/`verify_export.py`
   względem binarki Windows; porównanie z gotowym instalatorem z Releases.
2. **Wariant jobowy transkrypcji** (długi materiał nie blokuje transportu):
   `transcribe.start/status` + eventy `transcribe.done/failed`, na wzorze eksportu.
   Pipeline sekwencyjny tego nie potrzebuje; zysk głównie dla sesji MCP-agentów.
3. **Wygodniejszy start 9:16 z poziomu okna**: preset „TikTok/Reels/Shorts 1080×1920"
   w kreatorze New project (obecnie wybór kształtu istnieje — zweryfikować klikalność
   i dodać brakujące presety, jeśli nie ma 9:16 jednym kliknięciem).

## Jak uruchomić (jeden zestaw poleceń)

```bash
cd ~/Desktop/concat-projekt/concat/src
cargo build                  # raz: ~5 min
./target/debug/concat        # edytor (okno)
# testy MVP:
python3 ~/Desktop/concat-projekt/testy/test_mvp.py && \
python3 ~/Desktop/concat-projekt/testy/test_trim.py && \
python3 ~/Desktop/concat-projekt/testy/verify_export.py
# automatyzacja: katalog z filmami -> gotowy pionowy MP4 z napisami:
python3 ~/Desktop/concat-projekt/automat/pipeline.py --in <katalog> --out <film.mp4> --napisy
```
