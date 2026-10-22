# GDD — Pixel Monster Survivor (MVP)

Documento pratico del game-designer. Fonte dei requisiti: handoff §1 e §7.

## Fantasia e loop

Un giovane evocatore corre nella "Prateria dei Germogli" con compagni mostriciattolo
originali che attaccano da soli. Run di 10 minuti: muoviti, raccogli XP, scegli 1 carta su 3
a ogni livello, sopravvivi alle orde, batti il boss. Fine run: vittoria o morte, risultati,
restart immediato.

## Comandi

WASD/frecce: movimento 8 direzioni. Esc o P: pausa. F3: overlay debug. F12: screenshot.
Nessun tasto d'attacco: gli attacchi sono automatici.

## Valori attuali (in `game-data/*.ron` dalla P0.0; questa tabella ne spiega il perché)

| Parametro | Valore | Motivo |
|---|---|---|
| Velocità evocatore | 75 px/s | attraversa lo schermo (480 px) in ~6 s: lettura comoda delle orde |
| Velocità nemico base | 40 px/s | ~53% del player: si possono distanziare, non ignorare |
| Ondata iniziale | 10 nemici | primo risultato utile dell'handoff |
| Anello di spawn | 170–240 px dal player | appena fuori dalla mezza altezza dello schermo (135 px) |
| Distanza di contatto | 10 px | dove partiranno i danni da contatto (P0.3) |
| Separazione nemici | 14 px | sprite da 16 px quasi a contatto senza sovrapporsi |
| Arena | 4096×4096 px, bordo di siepe | ~55 s per attraversarla: spazio per fuggire senza perdersi |
| Simulazione | 60 Hz fissi | riproducibile con seed |
| HP gelatina | 30 | 3 colpi del compagno di fuoco |
| Compagno di fuoco | 12 danni, 1 colpo ogni 1,1 s, gittata 150 px, proiettile 220 px/s per 0,85 s (~187 px) | valori dell'handoff §7; il volo oltre la gittata raggiunge chi si sposta |
| Ostacoli | densità 1,2%, zona libera 128 px, 60% alberi sull'erba | ~650 per arena, mai un vicolo cieco |
| Compagno: distanza di scia | 18 px dietro il player, max 120 px/s | non intralcia la lettura del player |
| Popolazione nemici | 10, rimpiazzati a 290–340 px (fuori schermo) | finché non c'è lo spawn director (P0.3) |

**Misurato (P0.2, `tests/balance_probe.rs`, 60 s, 5 seed):** 54–55 colpi, 100% a segno,
18 nemici sconfitti al minuto da fermo, 14–16 in movimento (alcuni nemici restano fuori
gittata alle spalle). La mira predittiva non serve con queste velocità.

## Contenuti MVP (bozza, da bilanciare in P0)

**Compagni iniziali** (nomi di lavorazione, l'art director propone quelli definitivi):

| Compagno | Elemento | Attacco base | Parametri chiave |
|---|---|---|---|
| Brasa | fuoco | proiettile mirato al nemico più vicino | danno, cadenza, n. proiettili, perforazione |
| Gocci | acqua | onda in area attorno al player | danno, cadenza, raggio, rallentamento |
| Frond | natura | foglie orbitanti | danno, n. foglie, raggio orbita, velocità |

**Nemici:** gelatina (base, lenta, in gruppo), sciame veloce (poca vita), corazzato (lento, molta
vita); 1 élite (versione grande con telegraph di carica); 1 boss al minuto 10 con attacco
telegrafato in rosso.

**Upgrade (~10, tutti con effetto runtime testabile):** danno +15%, cadenza −10%, proiettili +1,
area +20%, raggio raccolta +25%, velocità +10%, HP massimi +20, rigenerazione, perforazione +1,
nuovo slot compagno. Ogni upgrade: ID, rarità, prerequisiti, max stack, formula, testo UI.

## Obiettivi di bilanciamento (test case da scrivere in P0.4–P0.7)

- Senza upgrade, il giocatore che resta fermo muore entro 90 s (non invincibile).
- Il primo level-up arriva entro 30–45 s (progressione percepibile).
- A metà run almeno due build elementali diverse sono praticabili (scelte significative).
- Il boss è sconfiggibile da una build media in 60–120 s.
