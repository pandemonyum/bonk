# GDD — Pixel Monster Survivor (MVP)

Documento pratico del game-designer. Fonte dei requisiti: handoff §1 e §7. I valori vivono in
`game-data/*.ron`; questo documento ne spiega il perché e registra le misure.

## Fantasia e loop

Un giovane evocatore corre nella "Prateria dei Germogli" con compagni mostriciattolo
originali che attaccano da soli. Run di 10 minuti: muoviti, raccogli XP, scegli 1 carta su 3
a ogni livello, sopravvivi alle orde, batti il boss. Fine run: vittoria o morte, risultati,
restart immediato.

## Comandi

WASD/frecce: movimento 8 direzioni. Esc o P: pausa. F5 (o Invio da sconfitti): ricomincia.
1/2/3 o clic: carta del draft. F3: overlay debug. F12: screenshot. Nessun tasto d'attacco: gli
attacchi sono automatici.

## Valori attuali

| Parametro | Valore | Motivo |
|---|---|---|
| Evocatore | 75 px/s, 100 HP, 0,6 s di invulnerabilità dopo un colpo | attraversa lo schermo in ~6 s; il tempo di uscire dalla calca |
| Arena | 4096×4096 px, bordo di siepe | ~55 s per attraversarla: spazio per fuggire senza perdersi |
| Simulazione | 60 Hz fissi | riproducibile con seed |
| Compagno di fuoco | 12 danni, 1 colpo ogni 1,1 s, gittata 150 px, proiettile 220 px/s per 0,85 s | valori dell'handoff §7 |
| Compagno: scia | 18 px dietro il player, max 120 px/s | non intralcia la lettura del player |
| Ostacoli | densità 1,2%, zona libera 128 px, 60% alberi sull'erba | ~650 per arena, mai un vicolo cieco |

**Nemici (`enemies.ron`)**

| Archetipo | HP | Velocità | Danno a contatto | Ruolo |
|---|---|---|---|---|
| Gelatina | 30 (3 colpi) | 40 px/s | 3 | base, arriva in gruppo |
| Sciame | 12 (1 colpo) | 68 px/s | 2 | quasi veloce come il player: non si semina, va abbattuto |
| Corazzato | 120 (10 colpi) | 28 px/s | 6 | lento, blocca la strada mentre gli altri arrivano |

**Danni al player:** a ogni finestra di invulnerabilità si somma il danno di tutti i nemici a
contatto. Essere circondati costa caro, uscirne in tempo no: muoversi è la difesa principale.

**Progressione (`progression.ron`, P0.4)**

| Parametro | Valore | Motivo |
|---|---|---|
| XP per nemico | gelatina 1, sciame 1, corazzato 6 | il corazzato costa 10 colpi: vale la deviazione |
| XP per livello | 5 + 3 × (livello − 1): 5, 8, 11, 14, ... | primo livello entro 30–45 s, poi un ritmo che rallenta senza fermarsi |
| Raccolta | raggio 28 px dai piedi; poi la gemma vola a 220 px/s e si prende a 6 px | bisogna avvicinarsi alle gemme: muoversi rende |
| Gemme a terra | al massimo 300, oltre l'XP si somma alla gemma più vicina | nessuna XP persa, nessuna esplosione di entità |
| Draft | 3 carte, pesi di rarità comune 10, non comune 5, rara 2 | scelte frequenti, le rare si notano |

**Upgrade (`upgrades.ron`).** Le percentuali si sommano sul valore base (2 × +15% = +30%) e
tutto si ricalcola dai dati base più gli stack a ogni scelta.

| Carta | Effetto per stack | Rarità | Max | Requisiti |
|---|---|---|---|---|
| Fiamma viva | danno dei compagni +15% | comune | 5 | — |
| Respiro rapido | intervallo tra gli attacchi −10% | comune | 5 | — |
| Scintille gemelle | +1 proiettile a ventaglio | rara | 3 | Fiamma viva |
| Fuoco perforante | perforazione +1 | non comune | 3 | — |
| Vista lunga | gittata +20% (la durata del proiettile cresce con lei) | comune | 3 | — |
| Vampa veloce | velocità dei proiettili +20% | comune | 2 | — |
| Scorza robusta | HP massimi +20, cura 20 | comune | 5 | — |
| Germoglio | rigenera 0,5 HP/s | non comune | 3 | — |
| Passo leggero | movimento +10% | non comune | 3 | — |
| Richiamo | raggio di raccolta +25% | comune | 3 | — |
| Merenda | cura 30 HP, nessuno stack permanente | comune | 99 | — (il draft non resta vuoto) |

"Nuovo slot compagno" arriva con i compagni della P0.5. La validazione controlla i limiti fisici
con tutti gli upgrade al massimo (proiettili che attraversano i nemici, corpi che saltano oltre
un tronco, cadenza sotto un passo, gemme più lente del player).

**Ondate (`stages.ron`, ritmo = base + crescita × minuti dall'inizio dell'ondata)**

| Ondata | Da | Base/min | Crescita/min² | Max vivi |
|---|---|---|---|---|
| Gelatina | 0:00 | 24 | 12 | 90 |
| Sciame | 1:00 | 20 | 15 | 110 |
| Corazzato | 2:30 | 4 | 2 | 25 |

5 gelatine all'avvio; al massimo 220 nemici insieme (obiettivo prestazioni P0); nascita
nell'anello 290–360 px dal player, sempre fuori vista.

## Misure

- **P0.2, colpi (`tests/sim/balance.rs`, 60 s, 5 seed):** 100% a segno, 14–18 nemici sconfitti
  al minuto con un compagno. La mira predittiva non serve.
- **P0.3, sopravvivenza senza upgrade (5 seed):** prima taratura (danni 5/3/10, invulnerabilità
  0,5 s, 8 nemici iniziali) troppo dura: fermi 0:10, autoplay 0:28–0:43. Dopo la taratura
  attuale: **fermi 0:13**, **autoplay 1:06–1:16**. L'autoplay è un movimento cieco (curva di
  Lissajous): un giocatore che schiva dura di più. La run completa di 10 minuti richiede gli
  upgrade della P0.4.
- **P0.3, carico:** 220 nemici a 59 FPS / 16,9 ms nel gioco (build dev, vsync); simulazione
  0,45–0,62 ms per passo con 50–500 nemici.
- **P0.4, livelli (5 seed, prima carta scelta in automatico):** primo level-up con l'autoplay a
  **0:30–0:41** (obiettivo 30–45 s: centrato). Con il player invulnerabile, per misurare il solo
  ritmo: livello 5 a ~2:20, livello 10 a 4:39–5:56, livello finale 14–25 a 10:00. Da fermi si
  muore a 0:13 con 2–3 nemici sconfitti, ancora al livello 1. Con l'autoplay e
  gli upgrade la run dura 1:08–1:24 (livello 2–3), contro 1:06–1:16 senza: la scelta casuale e
  il movimento cieco non bastano; la run di 10 minuti si riverifica con i compagni della P0.5 e
  la difficoltà della P0.6, e con un giocatore vero.

## Contenuti MVP (bozza, da bilanciare in P0)

**Compagni iniziali** (nomi di lavorazione, l'art director propone quelli definitivi):

| Compagno | Elemento | Attacco base | Parametri chiave |
|---|---|---|---|
| Brasa | fuoco | proiettile mirato al nemico più vicino | danno, cadenza, n. proiettili, perforazione |
| Gocci | acqua | onda in area attorno al player | danno, cadenza, raggio, rallentamento |
| Frond | natura | foglie orbitanti | danno, n. foglie, raggio orbita, velocità |

**Nemici ancora da fare:** 1 élite (versione grande con telegraph di carica); 1 boss al minuto
10 con attacco telegrafato in rosso (P0.6).

**Upgrade (~10, tutti con effetto runtime testabile, P0.4):** danno +15%, cadenza −10%,
proiettili +1, area +20%, raggio raccolta +25%, velocità +10%, HP massimi +20, rigenerazione,
perforazione +1, nuovo slot compagno. Ogni upgrade: ID, rarità, prerequisiti, max stack,
formula, testo UI.

## Obiettivi di bilanciamento

- Senza upgrade, il giocatore che resta fermo muore entro 90 s (non invincibile). **Oggi: 13 s.**
- Il primo level-up arriva entro 30–45 s (progressione percepibile). **Oggi: 0:30–0:41.**
- A metà run almeno due build elementali diverse sono praticabili (scelte significative).
- Il boss è sconfiggibile da una build media in 60–120 s.
