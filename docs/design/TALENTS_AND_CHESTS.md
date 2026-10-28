# Talenti dei compagni, monete, forzieri e reliquie (proposta)

Proposta del game-designer, 2026-10-28. I talenti persistenti sono una scelta dell'utente (oltre
l'MVP dell'handoff §1). Niente è ancora in `game-data/`: il gameplay-engineer porta i valori nei
`.ron`, con i campi nuovi dello schema, e li tara sulla sonda. **Nessun valore è stato provato
in gioco**: i conti usano le formule dei `.ron` di oggi e le misure del GDD, e sono stime.

## 0. Basi dei conti

- Run di 15 minuti, orde a 5:00 e 10:00, boss a 15:00 (~1 min di combattimento: fine ~16:00).
- Sconfitti al minuto con le ondate di oggi (nella run da 10 minuti muore ogni nemico generato,
  1592 nel GDD): 35, 48, 85, 116, 145, 174, 203, 232, 261, 290. Per 15 minuti stimo **~2700**:
  35–50 al minuto all'inizio, 100–200 a metà, 215–275 alla fine, ~100 e ~150 in più nelle orde.
- **Calca di riferimento** delle stime di danno al secondo (DPS), senza carte: 5 nemici
  nell'onda, 2 sull'anello delle foglie, catena da 6, 2 nemici entro 24 px da un bersaglio o
  da un colpo periodico (2,5 entro 30 px), 3 nel cono da 72 px (3,5 a 88). DPS base: Fiammetto
  10,9, Ondino 20, Foglin 18, Tuonny 34 (31 su 5); squadra ~50–70. Sul boss: 10,9, ~4, ~9, 9,2.

## 1. Talenti persistenti

### 1.1 Regole

- Livello ed esperienza per compagno, salvati sul dispositivo (file su desktop, `localStorage`
  sul web, nessun account); le run con `--god`, `--autoplay` o `--time-scale` non salvano.
- Un punto per livello dal 2 al 7 (6 punti, 6 nodi); un nodo richiede il precedente del suo
  ramo. I talenti valgono solo per il loro compagno, in ogni run in cui è in squadra; le
  statistiche (a) si sommano alle carte come le carte tra loro.
- Meccaniche (**a** statistica, **b** esplosione, **c** cono ogni N attacchi, **d** cura per
  colpito, **e** eco, **f** colpo periodico, **g** bruciatura), regole comuni:
  - b, c, e, f, g partono dal danno dell'attacco **già modificato** da carte e talenti; raggi,
    portate, N, periodi e ritardi dei talenti sono fissi (le carte di gittata non li toccano).
  - Un attacco è una raffica (tre proiettili insieme = un attacco); l'eco non conta per N.
  - L'esplosione nasce dai colpi diretti dei proiettili e ferisce gli altri nemici nel raggio,
    non il colpito. Bruciatura e cura nascono solo dai colpi dell'attacco base, eco esclusa.
  - Nessun effetto secondario ne attiva un altro; tutti contano come danni e sconfitti del
    compagno. La validazione dei limiti fisici va fatta con carte, talenti e reliquie al massimo.

### 1.2 Alberi

Stime nella calca di riferimento / su bersaglio singolo, **cumulative nel ramo**, in % del DPS
base del compagno. Gittata = raggio dell'onda per Ondino, dell'orbita per Foglin.

**Fiammetto**, «Scoppio» (A) e «Soffio» (B): il più debole nella calca, i rami gli danno l'area.

| Nodo | Nome | Testo UI | Meccanica e valori | Calca / singolo |
|---|---|---|---|---|
| ember_a1 | Brace che scoppia | Le palle di fuoco esplodono attorno al colpito | b: 24 px, 25% del colpo | +50% / 0 |
| ember_a2 | Scoppio largo | Esplosione più ampia e più forte | b: 24 → 30 px, 25 → 30% | +75% / 0 |
| ember_a3 | Brace che cova | Chi è colpito brucia per 3 s | g: 30% del colpo al secondo, 3 s | +85% / +30% |
| ember_b1 | Soffio rovente | Ogni 6 attacchi soffia un cono di fuoco | c: N 6, 25° per lato, 72 px, 80% | +40% / +13% |
| ember_b2 | Fiato lungo | Soffio ogni 5 attacchi e più lungo | c: N 6 → 5, 72 → 88 px | +56% / +16% |
| ember_b3 | Soffio ardente | Il soffio fa tutto il danno del colpo | c: 80 → 100% | +70% / +20% |

**Ondino**, «Marea» (A) e «Sorgente» (B): il difensore; A rallenta, B cura e dà portata.

| Nodo | Nome | Testo UI | Meccanica e valori | Calca / singolo |
|---|---|---|---|---|
| tide_a1 | Onda di ritorno | Mezzo secondo dopo parte una seconda onda | e: 0,5 s, 35%; rallenta anche lei | +35% / +35%; rallentamento quasi continuo (oggi 1,5 s ogni 2) |
| tide_a2 | Marea lenta | Chi è bagnato va ancora più piano | a: rallentamento 0,55 → 0,45, 1,5 → 2,0 s | +35%; chi ti insegue nell'onda perde un altro 18% |
| tide_a3 | Marea alta | Onda più ampia | a: gittata +15% (56 → 64 px, ~+20% bersagli) | +62% / +35% |
| tide_b1 | Acqua che ristora | Ogni nemico colpito dall'onda ti cura | d: 0,2 HP a colpito, max 0,6 a onda | cura ≤ 0,3 HP/s (un Germoglio) |
| tide_b2 | Getto dal profondo | Ogni 6 s un getto d'acqua sul nemico più vicino | f: 6 s, entro 120 px, 24 px, 150% | +20% / +50% |
| tide_b3 | Sorgente viva | L'onda cura di più | d: 0,2 → 0,25 HP, max 0,6 → 1,0 a onda | cura ≤ 0,5 HP/s (30 HP al minuto) |

**Foglin**, «Rovo» (A) e «Radici» (B): A difende da vicino, B colpisce chi è ancora lontano.

| Nodo | Nome | Testo UI | Meccanica e valori | Calca / singolo |
|---|---|---|---|---|
| sprout_a1 | Linfa urticante | Le foglie lasciano una linfa che brucia | g: 25% del colpo al secondo, 2 s | +25% / +25% |
| sprout_a2 | Quarta foglia | Una foglia in più in orbita | a: +1 foglia | +50% / +39% |
| sprout_a3 | Rovo largo | Orbita più ampia e più veloce | a: gittata +15% (34 → 39 px), velocità angolare +15% | +65% / +39% |
| sprout_b1 | Radici che afferrano | Ogni 6 s spuntano radici sotto il nemico più vicino | f: 6 s, entro 130 px, 24 px, 150% | +17% / +17% |
| sprout_b2 | Radici fitte | Radici più spesso | f: 6 → 5 s | +20% / +20% |
| sprout_b3 | Radici spinose | Radici più larghe e più forti | f: 24 → 30 px, 150 → 200% | +33% / +27% |

**Tuonny**, «Catena» (A) e «Temporale» (B): già il migliore nella calca, A cresce poco in
percentuale; B gli dà il colpo pesante su élite e boss.

| Nodo | Nome | Testo UI | Meccanica e valori | Calca / singolo |
|---|---|---|---|---|
| spark_a1 | Eco del lampo | 0,4 s dopo, il fulmine riparte | e: 0,4 s, 20%; scossa anche lei | +20% / +20% |
| spark_a2 | Catena salda | Ogni salto perde meno danno | a: calo per salto 0,80 → 0,85 | +32–35% / +20% |
| spark_a3 | Salto in più | Il fulmine salta su un nemico in più | a: +1 salto (5 → 6; conta con 7+ nemici) | +32–47% / +20% |
| spark_b1 | Lampo dal cielo | Ogni 7 s un lampo cade sul nemico più vicino | f: 7 s, entro 160 px, 24 px, 100% | +10% / +19% |
| spark_b2 | Cielo carico | Lampi più frequenti | f: 7 → 5 s | +14% / +26% |
| spark_b3 | Lampo grande | Lampo più largo e più forte | f: 24 → 28 px, 100 → 150% | +24% / +39% |

### 1.3 Effetto atteso

| Al massimo | DPS calca | DPS singolo | Effetto sulla squadra |
|---|---|---|---|
| Fiammetto | 10,9 → ~28 (×2,6) | 10,9 → ~16 (×1,5) | +25–35% nella calca, ~+20% sul boss |
| Ondino | 20 → ~36 (×1,8) | ~4 → ~7 | +25–30% nella calca, cura ≤ 0,5 HP/s, rallentamento più forte |
| Foglin | 18 → ~36 (×2,0) | ~9 → ~15 (×1,65) | +25–35% nella calca, ~+20% sul boss |
| Tuonny | 31–34 → ~49–58 (×1,6–1,7) | 9,2 → ~15 (×1,6) | +25–35% nella calca, ~+20% sul boss |

Sulla sonda (da misurare; riferimento **70–85% di vittorie a zero talenti**, bot che va ai
forzieri): un compagno al massimo +5–10 punti di vittorie, boss ~15% più rapido; tre al massimo
(calca ×1,8–2, boss ×1,6) 90–97% di vittorie, boss in 35–60 s, danni subiti al 50–70%. È il
premio del veterano dopo ~20 run, non toglie il pericolo: oltre il 97% di vittorie o sotto il
50% dei danni si taglia di un terzo il valore dei nodi 3. Le carte restano la crescita
principale (danno e cadenza al massimo già ×3,5): i talenti pesano nei primi minuti e sul boss.

### 1.4 Esperienza dei compagni

**esperienza = ⌊secondi in squadra ÷ 15⌋ + ⌊sconfitti dal compagno ÷ 40⌋ + 20 se vittoria**

- Tempo, 4 al minuto (~60%): giusto per ogni ruolo (Ondino rallenta e finisce meno nemici,
  Tuonny ne finisce tanti). Sconfitti, 1 ogni 40 (~25%): premia la run giocata bene, un divario
  ×2 tra ruoli vale ~10 punti; lo sconfitto è di chi dà l'ultimo colpo. Vittoria, +20 (~15%).
- Esclusi i danni (doppione degli sconfitti, numeri da migliaia) e la cura (solo col ramo
  Sorgente: Ondino crescerebbe più in fretta); restano nel riepilogo come informazione.
- Contano anche run perse o interrotte (F5, M); da fermi si muore a 0:18 con ~1 punto.
- Riepilogo, una riga per compagno: «Fiammetto +109 esperienza (tempo 64, nemici 25, vittoria
  20)», con la barra del livello e «Livello 2: nuovo talento!» quando sale.

| Caso | Tempo | Nemici | Vittoria | Totale |
|---|---|---|---|---|
| Iniziale, vittoria a ~16:00, ~1000 sconfitti | 64 | 25 | 20 | **109** |
| 2° compagno (livello 4, ~1:15), vittoria, ~900 | 59 | 22 | 20 | **101** |
| 3° compagno (livello 10, ~3:15), vittoria, ~800 | 51 | 20 | 20 | **91** |
| Iniziale, sconfitta a 9:00, ~450 | 36 | 11 | 0 | **47** |

Con il 70–85% di vittorie un compagno in squadra prende **~85–90 a run**. Per salire dal livello
n al n + 1 servono **100 + 50 × (n − 1)** (forma della XP della run); massimo livello 7:

| Livello | Per salire | Cumulata | Run a 88 | Run a 65–110 |
|---|---|---|---|---|
| 2 (1° talento) | 100 | 100 | 1,1 | 0,9–1,5 |
| 3 | 150 | 250 | 2,8 | 2,3–3,8 |
| 4 | 200 | 450 | 5,1 | 4,1–6,9 |
| 5 | 250 | 700 | 8,0 | 6,4–10,8 |
| 6 | 300 | 1000 | 11,4 | 9,1–15,4 |
| 7 (albero completo) | 350 | 1350 | 15,3 | 12,3–20,8 |

Ogni run vale dal 15% a più del 100% di un livello: la barra si muove sempre. Con 4 compagni e 3
posti, chi li alterna (ognuno in squadra ~3 run su 4) completa tutti e quattro in ~20 run.

### 1.5 Azzeramento

**Sì, gratis, solo nel menu** («Azzera» sul compagno libera tutti i punti; mai a metà run). Al
livello 7 si hanno tutti i nodi: l'unica scelta vera è l'ordine dei rami e non deve punire chi
ha scelto male; provare l'abilità nuova è il divertimento; non c'è una valuta da proteggere.

## 2. Monete, forzieri e reliquie (solo nella run)

Scopo: far girare il giocatore per l'arena di 4096×4096 invece di girare attorno ai mostri.

### 2.1 Monete

Cadono a 6 px dalla gemma; stessa calamita (28 px, poi volo a 220 px/s); non spariscono; al
massimo 100 a terra, oltre si sommano alla più vicina. HUD: monete sotto gli sconfitti.

| Nemico | Monete | Motivo |
|---|---|---|
| Gelatina | 3% | 1–2 monete al minuto all'inizio (35–50 sconfitti), 6–8 alla fine |
| Sciame | 3% | arriva in massa: stessa probabilità |
| Corazzato | 15% | 10 colpi e lento: vale la deviazione, come la sua XP (6) |
| Gelatina Gigante | 6 sicure, sparse attorno | una ogni 75 s; la prima (2:30) paga da sola più di metà del primo forziere: affrontarla rende |
| Gran Corazzato | 0 | sconfiggerlo chiude la run: le monete non servirebbero più |

### 2.2 Forzieri

| Parametro | Valore | Motivo |
|---|---|---|
| Sulla mappa | 2 insieme | una scelta di direzione, al massimo due mete |
| Nascita | 600–1000 px dal player (8–13 s a 75 px/s), mai in vista, ≥ 160 px dalla siepe, ≥ 600 px dall'altro, su un punto libero; senza posto 400–1400 px | lontano abbastanza da attraversare l'orda, vicino abbastanza da tornare |
| Quando | 2 all'inizio; aperto uno, ne nasce subito un altro | niente timer: il ritmo lo danno le monete |
| Spostamento | oltre 1800 px dal player (~24 s di cammino) rinasce nella fascia, fuori vista | chi è stato spinto lontano dall'orda non resta senza meta |
| Prezzo | **10 monete, +5 a ogni forziere aperto**: 10, 15, 20, 25, 30, 35, 40 | il primo poco dopo la prima élite, poi cresce con gli sconfitti al minuto |
| Apertura | ci cammini sopra con le monete: pausa e 1 reliquia su 3; senza, «Mancano N monete» | come il draft, nessun tasto nuovo |
| Freccia | al bordo dello schermo verso il forziere più vicino, col prezzo; dorata se basta, grigia se no | come il teschio del boss, ma non si confonde |

### 2.3 Un forziere ogni ~2 minuti

Nemici comuni 0,95 × 3% + 0,05 × 15% = **3,6%** a sconfitto; 10 élite (2:30, poi ogni 75 s) × 6.

| Minuti | Sconfitti stimati | Monete (3,6%) | Élite | Cumulate | Forziere (prezzo; cumulata) |
|---|---|---|---|---|---|
| 0–2 | 85 | 3 | — | 3 | — |
| 2–4 | 180 | 6,5 | 2 (12) | 22 | 1° (10) a ~2:40 |
| 4–6 | 360 (orda a 5:00) | 13 | 1 (6) | 41 | 2° (15; 25) a ~4:20 |
| 6–8 | 325 | 11,7 | 2 (12) | 64 | 3° (20; 45) a ~6:20 |
| 8–10 | 385 | 13,9 | 1 (6) | 84 | 4° (25; 70) a ~8:35 |
| 10–12 | 595 (orda a 10:00) | 21,4 | 2 (12) | 118 | 5° (30; 100) a ~11:00 |
| 12–14 | 505 | 18,2 | 2 (12) | 148 | 6° (35; 135) a ~13:10 |
| 14–15 | 275 | 9,9 | — | 158 | il 7° (175) non arriva |

**6 forzieri a run, uno ogni 1:40–2:25 (media ~2:05)** per chi raccoglie e ci va. Con sconfitti
reali diversi si cambia solo la probabilità di gelatina e sciame: p ≈ (135 − monete delle élite
fino a 13:00) ÷ sconfitti fino a 13:00 (con queste stime (135 − 54) ÷ 2175 ≈ 3,7%).

### 2.4 Reliquie

Pausa e 3 reliquie a caso, a pesi uguali, tra le 8 non ancora prese (una volta per run, mai nel
draft; con ~6 forzieri la scelta conta fino all'ultimo). Meno di 3 rimaste: si mostrano quelle;
finite: cura 50 HP (`Heal`). Si sommano alle carte come le carte tra loro.

| ID | Nome | Testo UI | Effetto | Contro le carte |
|---|---|---|---|---|
| relic_courage | Pietra del coraggio | Danno dei compagni +50% | `CompanionDamagePct(50)` | Fiamma viva +15% a stack: più di 3 carte |
| relic_hourglass | Clessidra svelta | Intervallo tra gli attacchi −20% | `CompanionIntervalPct(-20)` | Respiro rapido −10%; con 5 carte −70% (vedi rischi) |
| relic_nest | Nido di scintille | +2 palle di fuoco, foglie o salti del fulmine | `CompanionProjectiles(2)` | Scintille gemelle +1, rara, max 3 |
| relic_spyglass | Cannocchiale di corteccia | Gittata, onda e orbita +40% | `CompanionRangePct(40)` | Vista lunga +20% |
| relic_oak | Cuore di quercia | HP massimi +50 (e cura 50) | `MaxHp(50)` | Scorza robusta +20 |
| relic_dew | Rugiada perenne | Rigeneri 0,6 HP al secondo | `RegenPerSecond(0.6)` | Germoglio 0,3 |
| relic_sandals | Sandali di vento | Velocità di movimento +20% | `MoveSpeedPct(20)` | Passo leggero +10%; con tutto 75 × 1,5 = 112 px/s, sotto i 120 dei compagni |
| relic_piggy | Salvadanaio di coccio | Ogni moneta raccolta vale doppio | **nuovo** `CoinGainPct(100)` | nessuna carta |

**Un solo effetto nuovo, `CoinGainPct(f32)`**: monete × (1 + valore ÷ 100), decimali tenuti,
mostrate per difetto. È l'unica reliquia che alimenta i forzieri (meno forza subito, ~8–9
forzieri invece di 6): una scelta vera. Niente «XP +%»: la XP si ritara per i 15 minuti e oggi
finisce già a 10:00 (BACKLOG). Escluse: `PickupRadiusPct` (toglie il motivo di muoversi),
`CompanionPierce` e `ProjectileSpeedPct` (solo Fiammetto), `Heal` da solo (`MaxHp` cura già).

## 3. Livelli dei compagni nella run

| Posto | Livello del player | Quando (stima, giocatore medio) | Regola che deve restare vera |
|---|---|---|---|
| 2° | **4** (24 XP) | ~1:00–1:30 | prima dello sciame (oggi 1:40) |
| 3° | **10** (153 XP) | ~3:00–3:40 | almeno 90 s prima della prima orda (5:00): tempo per 2–3 carte |

- **Stima** con le ondate di oggi e il giocatore medio della P0.7 (primo livello 0:19–0:44,
  livello 14–18 a 5:00): un compagno da solo sconfigge 18–30 nemici al minuto (P0.2: 14–18 con
  Fiammetto), livello 4 verso 1:00–1:30; due fanno ~50–70 XP al minuto più 25 dalla prima élite
  (2:30), da 24 a 153 XP in ~2 minuti; con tre dalle ~3:20, livello ~14–17 a 5:00.
- Non 6 e 12 (il ritmo dell'esempio): il 6 arriva a ~2:00–2:30, dopo lo sciame (P0.5–P0.6: si
  moriva lì con un compagno solo), il 12 a ~4:15–4:45, dentro l'orda. Se l'ingegnere ritarda lo
  sciame (es. 2:30), vanno bene 5 e 12: contano le due regole, non i numeri.
- Al livello di sblocco il draft propone solo i compagni che non hai (3, poi 2: il quarto resta
  fuori); altrove nessuna carta compagno. Sostituisce `companion_card_until_level: 3` (campo nuovo, per esempio
  `companion_unlock_levels: [4, 10]`, da chiedere al gameplay-engineer).

## 4. Test case per qa-performance

| # | Scenario | Seed | Atteso |
|---|---|---|---|
| T1 | Sonda giocatore medio, 15 min, zero talenti, va al forziere quando ha le monete (bot da estendere) | 1–8 × 4 compagni iniziali | 70–85% di vittorie; 5–7 forzieri a run, il primo fra 2:15 e 3:30 |
| T2 | Come T1, compagno iniziale al massimo | 1–8 × 4 | +5–10 punti di vittorie; danno del compagno nei primi 5 min ×1,5–2,6 rispetto a T1 |
| T3 | Come T1, tre compagni al massimo; poi player fermo | 1–8 × 4; fermo 1–5 | ≤ 97% di vittorie; danni subiti ≥ 50% di T1; boss in 35–70 s; da fermo sconfitta entro 90 s |
| T4 | Livelli della run (T1) | 1–8 × 4 | mediana del livello 4 prima dello sciame; del livello 10 almeno 90 s prima della prima orda |
| T5 | Esperienza e livelli | unità | 960 s, 1000 sconfitti, vittoria → 109; 540 s, 450, sconfitta → 47; cumulata 99 → L1, 100 → L2, 250 → L3, 1349 → L6, 1350 e oltre → L7 |
| T6 | Nodi, azzeramento, salvataggio | unità | nodo senza il precedente rifiutato; «Azzera» libera tutti i punti a livelli invariati; dati intatti al riavvio; con `--god`/`--autoplay` il file non cambia |
| T7 | Forzieri | 1–20 | sempre 2, a 600–1000 px, mai in vista né su ostacoli, ≥ 600 px tra loro; oltre 1800 px rinasce; prezzo 10, 15, 20, 25 dopo 0–3 aperture; con 9 monete non si apre |
| T8 | Monete e reliquie | unità | élite 6 monete, boss 0; `CoinGainPct(100)` raddoppia; reliquie mai nel draft, una per tipo a run, 3 proposte diverse, pool vuoto → cura 50 |
| T9 | Validazione con carte, talenti e reliquie al massimo | unità | nessun errore: proiettili che non attraversano i nemici, cadenza sopra un passo, compagni più veloci del player |

## 5. Rischi e cose da sentire in playtest umano

- **Cura**: il tetto di Ondino è per onda e cresce con la cadenza (5 Respiro rapido: 1,0 HP/s;
  con la Clessidra ~1,7). Rugiada più 3 Germogli = 1,5 HP/s, che nella P0.6 copriva quasi tutti
  i danni. Se serve: Rugiada 0,4, tetto di Sorgente viva 0,8.
- **Clessidra svelta**: le percentuali negative sommate pesano di più con tante carte (−70%:
  cadenza ×3,3 contro ×2). Se diventa la scelta obbligata, −15%.
- **Leggibilità** di esplosioni, coni, lampi, radici e getti tra 200 nemici (pixel-art-director).
- **Forzieri**: il giocatore ci va davvero, e 600–1000 px sono una deviazione giusta nell'orda?
  Se li ignora la sonda passa ma lo scopo («muoversi») no.
- **Equità e stime**: l'esperienza dipende per ~25% dagli sconfitti; le stime di DPS dalla calca
  di riferimento. La sonda misuri il danno per compagno (serve anche al riepilogo) prima di
  toccare i valori.
