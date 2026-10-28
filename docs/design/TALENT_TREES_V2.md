# Alberi dei talenti v2 (proposta)

Proposta del game-designer, 2026-10-28, su richiesta dell'utente («alberi più grandi come
l'immagine»). Sostituisce i due rami da tre nodi di D-058. **Nessun valore è stato provato in
gioco** (task senza build): i conti usano le formule di `game-data/*.ron`, le misure del GDD e
lo stesso modello di stima della prima proposta (`TALENTS_AND_CHESTS.md`, «calca di
riferimento»). L'ingegnere li porta in `companions.ron` e `progression.ron` e li tara sulla sonda.

Nomi delle varianti come nello schema in lavorazione (`src/data/schema/talents.rs`): gli
effetti per il player sono `PlayerMaxHp`, `PlayerRegen`, `PlayerSpeedPct`, `PlayerPickupPct`,
`PlayerCoinPct` (nel messaggio: MaxHp, RegenPerSecond, MoveSpeedPct, PickupRadiusPct,
CoinGainPct). Se si tengono i nomi del messaggio basta sostituirli nel blocco RON.

## 1. Struttura e regole

- Per compagno tre rami: **elemento** (Fiamme, Onde, Foglie, Fulmini), **Supporto**,
  **Evoluzione**. Ogni ramo: due percorsi da tre nodi (col 0 e 1, righe 0–2), poi l'**unione**
  (col 2, riga 3, richiede l'ultimo nodo di entrambi i percorsi) e il **finale** (col 2, riga 4,
  richiede l'unione). 8 nodi a ramo, 24 a compagno, 96 in tutto.
- Gradi: in ogni percorso due nodi da 3 gradi e uno da 1; unione e finale 1/1. Ramo da 16 punti,
  tranne Tuonny (Fulmini 14, Evoluzione 14, Supporto 20: il fulmine ha poche statistiche che non
  pesano troppo sul danno). **Ogni compagno ha 48 punti**, quindi un solo `max_level` per tutti.
- ID: `<compagno>_<el|su|ev>_<a|b|c><n>`; a = col 0, b = col 1, c1 = unione, c2 = finale
  (uguale per i quattro compagni, invece di `ember_f_a1` dell'esempio).
- **Una meccanica (Explode, Cone, Echo, Strike, Burn, HealPerHit) e una statistica che
  sostituisce (SlowFactor, SlowSecs, ChainFalloff) sono nominate da un solo nodo per
  compagno**, che le fa crescere grado per grado. Unica eccezione: un nodo **1/1** può essere
  ripreso più forte dal finale dello stesso ramo (Cone di Fiammetto, Strike di Ondino, Foglin
  e Tuonny, SlowFactor di Ondino). Così nessun grado diventa inutile: con «vale l'ultimo
  nell'ordine dei dati», un nodo da 3 gradi superato da un altro nodo farebbe sprecare i punti
  (i requisiti chiedono un grado solo).
- Le statistiche e gli effetti `Player*` si sommano (tra gradi, nodi e compagni in squadra).
  Gli effetti `Player*` valgono finché il compagno è in squadra.
- Combinazioni meccanica/attacco usate solo dove il codice di oggi le gestisce: Explode, Cone e
  Burn sul proiettile; Echo sull'onda e sul fulmine; Burn sulle foglie; HealPerHit sull'onda;
  Strike per tutti.
- Profili salvati: l'esperienza resta e il livello si ricalcola con la curva nuova (un compagno
  al livello 7 di oggi, 1350 punti, arriva al livello 25 con 24 punti). Gli id dei nodi
  cambiano, quindi tutti i punti tornano da spendere.

### Due effetti nuovi (`TalentEffect`)

| Effetto | Cosa fa | Dove | Limite da validare |
|---|---|---|---|
| `PlayerDamageTakenPct(f32)` | danni subiti dal player in %, negativo = meno danni; si somma; vale per ogni colpo (contatto, carica, colpo del boss: in `strike_player`) | Foglin −4%, Ondino −1% | somma di tutti i talenti > −20% |
| `PlayerInvulnSecs(f32)` | secondi in più di invulnerabilità dopo un colpo (0,75 s di `player.ron`); si somma | Fiammetto +0,03 s | somma ≤ 0,15 s |

Servono perché il Supporto aiuti il player senza altri HP o cure, che spingono le vittorie verso
il 100% senza togliere pericolo. Valori piccoli: si sentono nella calca, non la annullano.

## 2. I dodici rami

| Compagno | Ramo | Tema | Percorso A (col 0) | Percorso B (col 1) | Unione | Finale |
|---|---|---|---|---|---|---|
| Fiammetto | **Fiamme** | la palla che scoppia e il fiato di fuoco | esplosione, danno, bruciatura | primo soffio, gittata, danno | gittata | **Soffio del vulcano**: soffio ogni 10 attacchi, più ampio |
| Fiammetto | **Supporto** | il focolare: calore, oro, riparo | HP, HP, raccolta | monete, monete, monete | HP | **Fiamma custode**: invulnerabilità +0,03 s |
| Fiammetto | **Evoluzione** | il cucciolo diventa adulto | danno, gittata, cadenza | gittata, danno, cadenza | danno | **Pioggia di braci**: una brace dal cielo ogni 16 s |
| Ondino | **Onde** | l'onda e il getto | getto (intro), danno, ampiezza | danno, ampiezza, danno | ampiezza | **Getto possente**: getto ogni 10 s, più largo |
| Ondino | **Supporto** | la sorgente che cura | cura a colpito, rigenerazione, danni −1% | raccolta, HP, HP | HP | **Sorgente viva**: rigenerazione |
| Ondino | **Evoluzione** | la marea che trattiene | durata del rallentamento, seconda onda, danno | rallentamento (intro), ampiezza, danno | ampiezza | **Marea che avvolge**: nemici al 46% |
| Foglin | **Foglie** | linfa e radici | bruciatura della linfa, danno, orbita | radici (intro), velocità foglie, orbita | velocità foglie | **Radici spinose**: radici ogni 10 s, più larghe |
| Foglin | **Supporto** | la corteccia che protegge | danni −1% a grado, rigenerazione, HP | HP, raccolta, rigenerazione | HP | **Corteccia viva**: danni −1% |
| Foglin | **Evoluzione** | il turbine di foglie | velocità foglie, danno, orbita | velocità foglie, orbita, velocità | velocità | **Quarta foglia**: +1 foglia |
| Tuonny | **Fulmini** | l'eco e il temporale | eco del fulmine, danno, portata | lampo dal cielo (intro), portata, portata | portata | **Temporale**: lampo ogni 10 s, più largo |
| Tuonny | **Supporto** | lo scatto e la calamita | velocità, raccolta, HP | monete, HP, raccolta | monete | **Lampo nei piedi**: velocità +2% |
| Tuonny | **Evoluzione** | la catena che non si spezza | calo per salto, portata, danno | portata, portata, danno | portata | **Salto in più**: +1 salto |

I nodi di riga 0 dei rami elemento danno subito qualcosa che si vede (esplosione, soffio, getto,
linfa, radici, eco, lampo) con numeri piccoli: è il «si sente ma poco» dei primi punti.

## 3. Punti, livelli, esperienza

- **48 punti per compagno**, uno per livello: **livello massimo 49**.
- Formula della run invariata (~85–90 a run per un compagno in squadra, D-058): ⌊s in squadra
  ÷ 15⌋ + ⌊sconfitti ÷ 40⌋ + 20 se vittoria. Cambia solo la curva: dal livello n al n + 1
  servono **20 + 3 × (n − 1)** (20, 23, 26, … 161), in tutto 4344.

```ron
companion_xp: (seconds_per_point: 15.0, kills_per_point: 40, victory: 20, first_level: 20, per_level: 3, max_level: 49),
```

| Run (88 a run) | 1 | 2 | 3 | 5 | 10 | 15 | 20 | 30 | 40 | 50 |
|---|---|---|---|---|---|---|---|---|---|---|
| Punti | 3 | 6 | 8 | 12 | 18 | **24** (metà) | 28 | 36 | 42 | **48** (completo) |

- Primo punto con 20 di esperienza: 5 minuti in squadra o una vittoria; dopo la prima run
  normale 3 punti; poi 2–3 a run fino alla quinta, ~1,3 fino alla quindicesima, ~0,7 alla fine.
- Metà albero in ~15 run (12–20 con 110–65 a run); completo in ~50 (40–67). Con 4 compagni e 3
  posti, chi li alterna completa tutti e quattro in ~65 run: è l'obiettivo a lungo termine.

## 4. Bilancio

**Da dove viene il tetto.** Sonda di oggi: 27/32 (84%) senza talenti, danni 70–260; con i sei
nodi di D-058 a tutti i compagni 32/32 e danni al ~37%. Con il modello della prima proposta
quei sei nodi valevano ~×1,8 di danno al secondo di squadra nella calca, quindi i danni subiti
scendono circa come (forza)^−1,7. Per restare **≥ 55% dei danni** la forza totale (talenti di
danno e Supporto insieme) deve restare **≤ ~×1,43**. Scelta: danno di squadra **×1,3–1,35**
con tre compagni al massimo, Supporto **≈ ×1,05–1,08**. L'albero completo da 48 punti vale
circa il 40% dei sei nodi di oggi: la forza cresce molto più piano.

Stime al massimo, senza carte, nella calca di riferimento (DPS base: Fiammetto 10,9, Ondino 20,
Foglin 18, Tuonny 34; squadra ~62) e su bersaglio singolo; «squadra» = DPS in più ÷ 62.

| Compagno | Ramo al massimo | DPS calca | DPS singolo | Squadra / player |
|---|---|---|---|---|
| Fiammetto | Fiamme | +27% (+3,0) | +14% (+1,6) | ~+5% |
| Fiammetto | Evoluzione | +24% (+2,6) | +21% (+2,3) | ~+4% |
| Fiammetto | Supporto | — | — | HP +11, monete +12%, raccolta +3%, invulnerabilità +0,03 s |
| Fiammetto | **tutto** | **×1,55 (+6)** | ×1,35 (+3,8) | ~+10%; il più debole oggi (5/8 da iniziale) |
| Ondino | Onde | +21% (+4,1) | +22% | ~+7% |
| Ondino | Evoluzione | +14% (+2,8) | +9% | ~+4,5%; nemici al 46% per 1,8 s (oggi 55% per 1,5) |
| Ondino | Supporto | — | — | cura ≤ 0,15 HP/s a onda, rigenera 0,10 HP/s, HP +10, danni −1%, raccolta +3% |
| Ondino | **tutto** | **×1,36 (+7)** | ×1,3 | ~+12%, più il rallentamento |
| Foglin | Foglie | +27% (+4,8) | +21% | ~+8% |
| Foglin | Evoluzione | +28% (+5,1) | +14% | ~+8% (la quarta foglia conta di più con le carte di cadenza) |
| Foglin | Supporto | — | — | danni −4%, rigenera 0,08 HP/s, HP +10, raccolta +3% |
| Foglin | **tutto** | **×1,4–1,5 (+7–9)** | ×1,3 | ~+12–14% |
| Tuonny | Fulmini | +14% (+4,6) | +15% | ~+7% |
| Tuonny | Evoluzione | +13% (+4,5) | +2% | ~+7% (+1 salto conta con 7+ nemici) |
| Tuonny | Supporto | — | — | velocità +5%, raccolta +9%, monete +9%, HP +9 |
| Tuonny | **tutto** | **×1,2–1,25 (+7–8)** | ×1,17 | ~+12% (parte da 34: ogni % pesa il triplo) |

- **Squadra di tre al massimo**: +21–24 DPS nella calca (×1,3–1,4), ~×1,25 sul boss.
  Supporto, caso peggiore (tre compagni qualsiasi): HP +31, rigenerazione 0,18 HP/s più cura
  ≤ 0,15 HP/s (in tutto quanto un Germoglio), danni −5%, invulnerabilità +0,03 s, velocità +5%,
  raccolta +15%, monete +21%.
- **Attese sulla sonda (da misurare)**: albero completo su tutti e quattro **92–97% di vittorie,
  danni al 55–62%** di quelli senza talenti; metà albero (24 punti) 88–94%. I primi 3 punti
  (dopo una run) valgono +2–4% di DPS di squadra.
- **Guardia e ordine dei tagli** (se le vittorie superano il 97% o i danni scendono sotto il
  55%): 1) i nodi `DamagePct` delle Evoluzioni a 0,5 per grado; 2) i finali dei rami elemento
  −25%; 3) `PlayerDamageTakenPct` di Foglin a −2 e `PlayerInvulnSecs` a 0,02. Se invece le
  vittorie con l'albero completo restano sotto il 92%, si alzano prima i finali.
- **Limiti fisici con carte, reliquie e talenti al massimo** (la validazione deve contare anche
  i talenti): player 75 × (1 + 0,30 + 0,20 + 0,05) = 116 px/s, sotto i 120 dei compagni;
  ampiezza dell'onda di Ondino +11% → onda di 0,74 s contro un intervallo minimo di 0,76 s
  (oltre +12% non entra: niente `IntervalPct` per Ondino); foglie +24% di velocità angolare e
  +9% di orbita, 4,6 px a passo contro 12; salti di Tuonny 5 + 3 + 2 + 1 = 11 e foglie 3 + 3 +
  2 + 1 = 9, sotto 32; `IntervalPct` di Fiammetto −4% in tutto (le percentuali negative pesano
  di più con tante carte).

## 5. Nodi e icone (per il pixel-art-director)

Una icona per nodo (16×16 come le altre del menu), nei colori del ramo: elemento, ambra,
blu. «a grado» = il valore si ripete a ogni grado. Mai simboli presi da altri giochi.

**Fiammetto, Fiamme**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| ember_el_a1 | Brace che scoppia | 3 | esplosione 16/18/20 px, 4/5/6% del colpo | palla di fuoco che si apre in quattro scintille |
| ember_el_a2 | Fuoco che cresce | 3 | danno +1% a grado | fiammella su tre gradini |
| ember_el_a3 | Brace che cova | 1 | bruciatura 4%/s per 2 s | brace rossa con un filo di fumo |
| ember_el_b1 | Soffio rovente | 1 | cono ogni 12 attacchi, 20°, 56 px, 40% | muso di profilo con un piccolo cono di fuoco |
| ember_el_b2 | Tiro lungo | 3 | gittata +2% a grado | palla di fuoco con scia lunga |
| ember_el_b3 | Brace viva | 3 | danno +1% a grado | tizzone con punti gialli |
| ember_el_c1 | Brace e soffio | 1 | gittata +4% | palla di fuoco e cono incrociati |
| ember_el_c2 | Soffio del vulcano | 1 | cono ogni 10 attacchi, 22°, 64 px, 40% | cono di fuoco largo dal bordo giallo |

**Fiammetto, Supporto**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| ember_su_a1 | Calore amico | 3 | HP massimi +2 a grado | cuore arancione con fiammella |
| ember_su_a2 | Fiamma nel petto | 3 | HP massimi +1 a grado | fiammella dentro un cuore |
| ember_su_a3 | Lume | 1 | raccolta +3% | lanterna piccola |
| ember_su_b1 | Luccichio | 3 | monete +2% a grado | moneta con riflesso |
| ember_su_b2 | Fiuto per l'oro | 3 | monete +1% a grado | muso che annusa una moneta |
| ember_su_b3 | Occhio d'oro | 1 | monete +3% | occhio con pupilla a moneta |
| ember_su_c1 | Focolare | 1 | HP massimi +2 | tre ceppi con fiamma |
| ember_su_c2 | Fiamma custode | 1 | invulnerabilità +0,03 s | scudo a forma di fiamma |

**Fiammetto, Evoluzione**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| ember_ev_a1 | Artiglio caldo | 3 | danno +1% a grado | artiglio con bagliore |
| ember_ev_a2 | Occhio di brace | 3 | gittata +3% a grado | occhio arancione con raggi |
| ember_ev_a3 | Fiato pronto | 1 | intervallo -2% | bocca aperta con sbuffo corto |
| ember_ev_b1 | Sguardo lontano | 3 | gittata +2% a grado | occhio socchiuso e orizzonte |
| ember_ev_b2 | Scaglie calde | 3 | danno +1% a grado | tre scaglie rosse |
| ember_ev_b3 | Cuore acceso | 1 | intervallo -2% | cuore con brace al centro |
| ember_ev_c1 | Fiammetto adulto | 1 | danno +2% | sagoma di Fiammetto più grande |
| ember_ev_c2 | Pioggia di braci | 1 | ogni 16 s una brace sul nemico più vicino entro 160 px: 20 px, 80% | braci che cadono su un cerchio |

**Ondino, Onde**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| tide_el_a1 | Getto dal profondo | 1 | ogni 12 s un getto entro 120 px: 20 px, 60% | getto d'acqua dal suolo |
| tide_el_a2 | Acqua pesante | 3 | danno +1% a grado | goccia grande e pesante |
| tide_el_a3 | Marea alta | 3 | onda +1% a grado | onda alta |
| tide_el_b1 | Onda piena | 3 | danno +1% a grado | cerchio d'onda pieno |
| tide_el_b2 | Marea larga | 3 | onda +1% a grado | due cerchi d'onda larghi |
| tide_el_b3 | Corrente | 1 | danno +1% | tre linee di corrente |
| tide_el_c1 | Onda e getto | 1 | onda +1% | onda con un getto al centro |
| tide_el_c2 | Getto possente | 1 | getto ogni 10 s entro 130 px: 22 px, 70% | getto alto con spruzzi |

**Ondino, Supporto**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| tide_su_a1 | Acqua che ristora | 3 | cura 0,05/0,08/0,1 HP a colpito, max 0,15/0,24/0,3 a onda | goccia con una croce chiara |
| tide_su_a2 | Sorsi d'acqua | 3 | rigenera 0,02 HP/s a grado | tre goccioline in fila |
| tide_su_a3 | Velo d'acqua | 1 | danni subiti -1% | velo azzurro trasparente |
| tide_su_b1 | Bolle leggere | 3 | raccolta +1% a grado | bolla con una gemma dentro |
| tide_su_b2 | Pelle di rugiada | 3 | HP massimi +2 a grado | pelle a squame con rugiada |
| tide_su_b3 | Acqua amica | 1 | HP massimi +2 | pinna che offre una goccia |
| tide_su_c1 | Fonte calma | 1 | HP massimi +2 | fonte di pietra con acqua ferma |
| tide_su_c2 | Sorgente viva | 1 | rigenera 0,04 HP/s | sorgente che zampilla con un cuore |

**Ondino, Evoluzione**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| tide_ev_a1 | Acqua che trattiene | 3 | rallentamento 1,6/1,7/1,8 s | anello di gocce attorno a un piede |
| tide_ev_a2 | Onda di ritorno | 3 | seconda onda dopo 0,6/0,55/0,5 s al 2/3/5% | due onde una dietro l'altra |
| tide_ev_a3 | Goccia dura | 1 | danno +1% | goccia dal bordo bianco |
| tide_ev_b1 | Acqua gelida | 1 | nemici al 52% della velocità | goccia con brina |
| tide_ev_b2 | Risacca | 3 | onda +1% a grado | onda che torna sulla riva |
| tide_ev_b3 | Corrente fredda | 3 | danno +1% a grado | corrente azzurro chiaro |
| tide_ev_c1 | Marea profonda | 1 | onda +1% | onda blu scuro |
| tide_ev_c2 | Marea che avvolge | 1 | nemici al 46% della velocità | vortice che avvolge una sagoma |

**Foglin, Foglie**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| sprout_el_a1 | Linfa urticante | 3 | bruciatura 3/4/5%/s per 2 s | foglia con goccia di linfa viola |
| sprout_el_a2 | Foglia robusta | 3 | danno +1% a grado | foglia spessa con nervatura |
| sprout_el_a3 | Giro largo | 1 | orbita +2% | foglie su un cerchio largo |
| sprout_el_b1 | Radici che afferrano | 1 | ogni 12 s radici entro 130 px: 20 px, 60% | radici che escono dal suolo |
| sprout_el_b2 | Vento tra le foglie | 3 | foglie +2% più veloci a grado | foglie con linee di vento |
| sprout_el_b3 | Rami lunghi | 3 | orbita +1% a grado | ramo lungo |
| sprout_el_c1 | Rovo e radice | 1 | foglie +2% più veloci | rovo intrecciato a una radice |
| sprout_el_c2 | Radici spinose | 1 | radici ogni 10 s entro 140 px: 22 px, 70% | ciuffo di radici spinose |

**Foglin, Supporto**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| sprout_su_a1 | Scorza di corteccia | 3 | danni subiti -1% a grado | scudo di corteccia |
| sprout_su_a2 | Linfa che nutre | 3 | rigenera 0,02 HP/s a grado | goccia di linfa con un cuore |
| sprout_su_a3 | Radice salda | 1 | HP massimi +3 | radice ancorata a una pietra |
| sprout_su_b1 | Foglia larga | 3 | HP massimi +2 a grado | foglia aperta come un ombrello |
| sprout_su_b2 | Rami che raccolgono | 3 | raccolta +1% a grado | rami attorno a una gemma |
| sprout_su_b3 | Seme di riserva | 1 | rigenera 0,02 HP/s | seme nel guscio |
| sprout_su_c1 | Bosco fitto | 1 | HP massimi +1 | tre alberelli |
| sprout_su_c2 | Corteccia viva | 1 | danni subiti -1% | corteccia con nodi che brillano |

**Foglin, Evoluzione**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| sprout_ev_a1 | Foglie svelte | 3 | foglie +2% più veloci a grado | foglia con linee di velocità |
| sprout_ev_a2 | Linfa forte | 3 | danno +1% a grado | goccia di linfa scura |
| sprout_ev_a3 | Nervature | 1 | orbita +1% | nervature in rilievo |
| sprout_ev_b1 | Giro veloce | 3 | foglie +2% più veloci a grado | cerchio di foglie che gira |
| sprout_ev_b2 | Orbita ampia | 3 | orbita +1% a grado | orbita larga punteggiata |
| sprout_ev_b3 | Nodo del ramo | 1 | foglie +2% più veloci | nodo di un ramo |
| sprout_ev_c1 | Foglin cresciuto | 1 | foglie +2% più veloci | sagoma di Foglin più grande |
| sprout_ev_c2 | Quarta foglia | 1 | +1 foglia | quattro foglie a croce |

**Tuonny, Fulmini**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| spark_el_a1 | Eco del lampo | 3 | il fulmine riparte dopo 0,45/0,42/0,4 s al 2/3/4% | due fulmini, il secondo più tenue |
| spark_el_a2 | Lampo vivo | 3 | danno +1% a grado | fulmine spesso e giallo |
| spark_el_a3 | Coda di stelle | 1 | portata +3% | coda a stella con scintille |
| spark_el_b1 | Lampo dal cielo | 1 | ogni 12 s un lampo entro 160 px: 20 px, 50% | nuvola con un lampo che scende |
| spark_el_b2 | Aria tesa | 3 | portata +1% a grado | aria con linee elettriche |
| spark_el_b3 | Nube bassa | 1 | portata +2% | nuvola bassa e grigia |
| spark_el_c1 | Lampo e catena | 1 | portata +2% | lampo e catena di punti |
| spark_el_c2 | Temporale | 1 | lampo ogni 10 s entro 160 px: 24 px, 60% | nuvola scura con tre lampi |

**Tuonny, Supporto**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| spark_su_a1 | Passo elettrico | 3 | velocità +1% a grado | zampa con una scintilla |
| spark_su_a2 | Pelo che attira | 3 | raccolta +2% a grado | pelo ritto che attira gemme |
| spark_su_a3 | Scossa amica | 3 | HP massimi +2 a grado | cuore con una scintilla rosa |
| spark_su_b1 | Monete attratte | 3 | monete +2% a grado | monete che volano verso un ciuffo di pelo |
| spark_su_b2 | Cuore che batte | 3 | HP massimi +1 a grado | cuore che pulsa |
| spark_su_b3 | Calamita viva | 3 | raccolta +1% a grado | calamita con due scintille |
| spark_su_c1 | Corrente amica | 1 | monete +3% | corrente a onda tra due stelle |
| spark_su_c2 | Lampo nei piedi | 1 | velocità +2% | piedi con un lampo |

**Tuonny, Evoluzione**

| Nodo | Nome | Gradi | Effetto | Icona |
|---|---|---|---|---|
| spark_ev_a1 | Catena salda | 3 | calo per salto 19/18,5/18% invece di 20% | catena di anelli elettrici |
| spark_ev_a2 | Salto lungo | 3 | portata +2% a grado | arco lungo tra due punti |
| spark_ev_a3 | Pelo carico | 1 | danno +1% | pelo pieno di scintille |
| spark_ev_b1 | Coda tesa | 1 | portata +2% | coda dritta e tesa |
| spark_ev_b2 | Scintille lontane | 3 | portata +1% a grado | scintille sparse lontano |
| spark_ev_b3 | Pelo dritto | 1 | danno +1% | pelo dritto sulla schiena |
| spark_ev_c1 | Tuonny cresciuto | 1 | portata +2% | sagoma di Tuonny più grande |
| spark_ev_c2 | Salto in più | 1 | +1 salto | catena con un anello dorato in più |

## 6. Icone d'intestazione dei rami (12)

| Compagno | Fiamme / Onde / Foglie / Fulmini | Supporto (ambra) | Evoluzione (blu) |
|---|---|---|---|
| Fiammetto | fiamma a goccia con brace al centro | focolare di tre ceppi | zampa artigliata con una scaglia |
| Ondino | onda arricciata | goccia con un cuore | spirale d'acqua |
| Foglin | foglia con nervature | scudo di corteccia | germoglio che diventa ramo |
| Tuonny | saetta a zigzag | stella con scia (la coda di Tuonny) | coda a stella grande con scintille |

## 7. RON da incollare (`companions.ron`, campo `talents` di ogni compagno)

```ron
// ember (Fiammetto)
talents: [
    (name: "Fiamme", theme: Element, nodes: [
        (id: "ember_el_a1", name: "Brace che scoppia", col: 0, row: 0, requires: [],
         description: "Le palle di fuoco esplodono: 4–6% del colpo entro 16–20 px",
         ranks: [[Explode(radius: 16.0, damage_pct: 4.0)], [Explode(radius: 18.0, damage_pct: 5.0)], [Explode(radius: 20.0, damage_pct: 6.0)]]),
        (id: "ember_el_a2", name: "Fuoco che cresce", col: 0, row: 1, requires: ["ember_el_a1"],
         description: "Danno di Fiammetto +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "ember_el_a3", name: "Brace che cova", col: 0, row: 2, requires: ["ember_el_a2"],
         description: "Chi è colpito brucia: 4% del colpo al secondo per 2 s",
         ranks: [[Burn(dps_pct: 4.0, duration_s: 2.0)]]),
        (id: "ember_el_b1", name: "Soffio rovente", col: 1, row: 0, requires: [],
         description: "Ogni 12 attacchi soffia un cono di fuoco (40% del colpo)",
         ranks: [[Cone(every: 12, half_angle_deg: 20.0, range: 56.0, damage_pct: 40.0)]]),
        (id: "ember_el_b2", name: "Tiro lungo", col: 1, row: 1, requires: ["ember_el_b1"],
         description: "Gittata delle palle di fuoco +2% a grado",
         ranks: [[RangePct(2.0)], [RangePct(2.0)], [RangePct(2.0)]]),
        (id: "ember_el_b3", name: "Brace viva", col: 1, row: 2, requires: ["ember_el_b2"],
         description: "Danno di Fiammetto +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "ember_el_c1", name: "Brace e soffio", col: 2, row: 3, requires: ["ember_el_a3", "ember_el_b3"],
         description: "Gittata delle palle di fuoco +4%",
         ranks: [[RangePct(4.0)]]),
        (id: "ember_el_c2", name: "Soffio del vulcano", col: 2, row: 4, requires: ["ember_el_c1"],
         description: "Il soffio arriva ogni 10 attacchi, più ampio e più lungo",
         ranks: [[Cone(every: 10, half_angle_deg: 22.0, range: 64.0, damage_pct: 40.0)]]),
    ]),
    (name: "Supporto", theme: Support, nodes: [
        (id: "ember_su_a1", name: "Calore amico", col: 0, row: 0, requires: [],
         description: "HP massimi +2 a grado",
         ranks: [[PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)]]),
        (id: "ember_su_a2", name: "Fiamma nel petto", col: 0, row: 1, requires: ["ember_su_a1"],
         description: "HP massimi +1 a grado",
         ranks: [[PlayerMaxHp(1.0)], [PlayerMaxHp(1.0)], [PlayerMaxHp(1.0)]]),
        (id: "ember_su_a3", name: "Lume", col: 0, row: 2, requires: ["ember_su_a2"],
         description: "Raggio di raccolta +3%",
         ranks: [[PlayerPickupPct(3.0)]]),
        (id: "ember_su_b1", name: "Luccichio", col: 1, row: 0, requires: [],
         description: "Monete +2% a grado",
         ranks: [[PlayerCoinPct(2.0)], [PlayerCoinPct(2.0)], [PlayerCoinPct(2.0)]]),
        (id: "ember_su_b2", name: "Fiuto per l'oro", col: 1, row: 1, requires: ["ember_su_b1"],
         description: "Monete +1% a grado",
         ranks: [[PlayerCoinPct(1.0)], [PlayerCoinPct(1.0)], [PlayerCoinPct(1.0)]]),
        (id: "ember_su_b3", name: "Occhio d'oro", col: 1, row: 2, requires: ["ember_su_b2"],
         description: "Monete +3%",
         ranks: [[PlayerCoinPct(3.0)]]),
        (id: "ember_su_c1", name: "Focolare", col: 2, row: 3, requires: ["ember_su_a3", "ember_su_b3"],
         description: "HP massimi +2",
         ranks: [[PlayerMaxHp(2.0)]]),
        (id: "ember_su_c2", name: "Fiamma custode", col: 2, row: 4, requires: ["ember_su_c1"],
         description: "Dopo un colpo resti invulnerabile 0,03 s in più",
         ranks: [[PlayerInvulnSecs(0.03)]]),
    ]),
    (name: "Evoluzione", theme: Evolution, nodes: [
        (id: "ember_ev_a1", name: "Artiglio caldo", col: 0, row: 0, requires: [],
         description: "Danno +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "ember_ev_a2", name: "Occhio di brace", col: 0, row: 1, requires: ["ember_ev_a1"],
         description: "Gittata +3% a grado",
         ranks: [[RangePct(3.0)], [RangePct(3.0)], [RangePct(3.0)]]),
        (id: "ember_ev_a3", name: "Fiato pronto", col: 0, row: 2, requires: ["ember_ev_a2"],
         description: "Intervallo tra gli attacchi -2%",
         ranks: [[IntervalPct(-2.0)]]),
        (id: "ember_ev_b1", name: "Sguardo lontano", col: 1, row: 0, requires: [],
         description: "Gittata +2% a grado",
         ranks: [[RangePct(2.0)], [RangePct(2.0)], [RangePct(2.0)]]),
        (id: "ember_ev_b2", name: "Scaglie calde", col: 1, row: 1, requires: ["ember_ev_b1"],
         description: "Danno +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "ember_ev_b3", name: "Cuore acceso", col: 1, row: 2, requires: ["ember_ev_b2"],
         description: "Intervallo tra gli attacchi -2%",
         ranks: [[IntervalPct(-2.0)]]),
        (id: "ember_ev_c1", name: "Fiammetto adulto", col: 2, row: 3, requires: ["ember_ev_a3", "ember_ev_b3"],
         description: "Danno +2%",
         ranks: [[DamagePct(2.0)]]),
        (id: "ember_ev_c2", name: "Pioggia di braci", col: 2, row: 4, requires: ["ember_ev_c1"],
         description: "Ogni 16 s una brace cade sul nemico più vicino (80% del colpo)",
         ranks: [[Strike(every_s: 16.0, range: 160.0, radius: 20.0, damage_pct: 80.0)]]),
    ]),
],

// tide (Ondino)
talents: [
    (name: "Onde", theme: Element, nodes: [
        (id: "tide_el_a1", name: "Getto dal profondo", col: 0, row: 0, requires: [],
         description: "Ogni 12 s un getto d'acqua sul nemico più vicino (60% del colpo)",
         ranks: [[Strike(every_s: 12.0, range: 120.0, radius: 20.0, damage_pct: 60.0)]]),
        (id: "tide_el_a2", name: "Acqua pesante", col: 0, row: 1, requires: ["tide_el_a1"],
         description: "Danno di Ondino +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "tide_el_a3", name: "Marea alta", col: 0, row: 2, requires: ["tide_el_a2"],
         description: "Onda più ampia: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "tide_el_b1", name: "Onda piena", col: 1, row: 0, requires: [],
         description: "Danno di Ondino +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "tide_el_b2", name: "Marea larga", col: 1, row: 1, requires: ["tide_el_b1"],
         description: "Onda più ampia: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "tide_el_b3", name: "Corrente", col: 1, row: 2, requires: ["tide_el_b2"],
         description: "Danno di Ondino +1%",
         ranks: [[DamagePct(1.0)]]),
        (id: "tide_el_c1", name: "Onda e getto", col: 2, row: 3, requires: ["tide_el_a3", "tide_el_b3"],
         description: "Onda più ampia: +1%",
         ranks: [[RangePct(1.0)]]),
        (id: "tide_el_c2", name: "Getto possente", col: 2, row: 4, requires: ["tide_el_c1"],
         description: "Getto ogni 10 s, più largo e più forte (70% del colpo)",
         ranks: [[Strike(every_s: 10.0, range: 130.0, radius: 22.0, damage_pct: 70.0)]]),
    ]),
    (name: "Supporto", theme: Support, nodes: [
        (id: "tide_su_a1", name: "Acqua che ristora", col: 0, row: 0, requires: [],
         description: "Ogni nemico colpito dall'onda ti cura 0,05–0,1 HP (max 0,15–0,3 a onda)",
         ranks: [[HealPerHit(hp: 0.05, cap: 0.15)], [HealPerHit(hp: 0.08, cap: 0.24)], [HealPerHit(hp: 0.1, cap: 0.3)]]),
        (id: "tide_su_a2", name: "Sorsi d'acqua", col: 0, row: 1, requires: ["tide_su_a1"],
         description: "Rigeneri 0,02 HP al secondo a grado",
         ranks: [[PlayerRegen(0.02)], [PlayerRegen(0.02)], [PlayerRegen(0.02)]]),
        (id: "tide_su_a3", name: "Velo d'acqua", col: 0, row: 2, requires: ["tide_su_a2"],
         description: "Danni subiti -1%",
         ranks: [[PlayerDamageTakenPct(-1.0)]]),
        (id: "tide_su_b1", name: "Bolle leggere", col: 1, row: 0, requires: [],
         description: "Raggio di raccolta +1% a grado",
         ranks: [[PlayerPickupPct(1.0)], [PlayerPickupPct(1.0)], [PlayerPickupPct(1.0)]]),
        (id: "tide_su_b2", name: "Pelle di rugiada", col: 1, row: 1, requires: ["tide_su_b1"],
         description: "HP massimi +2 a grado",
         ranks: [[PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)]]),
        (id: "tide_su_b3", name: "Acqua amica", col: 1, row: 2, requires: ["tide_su_b2"],
         description: "HP massimi +2",
         ranks: [[PlayerMaxHp(2.0)]]),
        (id: "tide_su_c1", name: "Fonte calma", col: 2, row: 3, requires: ["tide_su_a3", "tide_su_b3"],
         description: "HP massimi +2",
         ranks: [[PlayerMaxHp(2.0)]]),
        (id: "tide_su_c2", name: "Sorgente viva", col: 2, row: 4, requires: ["tide_su_c1"],
         description: "Rigeneri 0,04 HP al secondo",
         ranks: [[PlayerRegen(0.04)]]),
    ]),
    (name: "Evoluzione", theme: Evolution, nodes: [
        (id: "tide_ev_a1", name: "Acqua che trattiene", col: 0, row: 0, requires: [],
         description: "Il rallentamento dura 1,6–1,8 s invece di 1,5",
         ranks: [[SlowSecs(1.6)], [SlowSecs(1.7)], [SlowSecs(1.8)]]),
        (id: "tide_ev_a2", name: "Onda di ritorno", col: 0, row: 1, requires: ["tide_ev_a1"],
         description: "Poco dopo parte una seconda onda al 2–5%, che rallenta anche lei",
         ranks: [[Echo(delay_s: 0.6, damage_pct: 2.0)], [Echo(delay_s: 0.55, damage_pct: 3.0)], [Echo(delay_s: 0.5, damage_pct: 5.0)]]),
        (id: "tide_ev_a3", name: "Goccia dura", col: 0, row: 2, requires: ["tide_ev_a2"],
         description: "Danno +1%",
         ranks: [[DamagePct(1.0)]]),
        (id: "tide_ev_b1", name: "Acqua gelida", col: 1, row: 0, requires: [],
         description: "Chi è bagnato va al 52% della velocità invece del 55%",
         ranks: [[SlowFactor(0.52)]]),
        (id: "tide_ev_b2", name: "Risacca", col: 1, row: 1, requires: ["tide_ev_b1"],
         description: "Onda più ampia: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "tide_ev_b3", name: "Corrente fredda", col: 1, row: 2, requires: ["tide_ev_b2"],
         description: "Danno +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "tide_ev_c1", name: "Marea profonda", col: 2, row: 3, requires: ["tide_ev_a3", "tide_ev_b3"],
         description: "Onda più ampia: +1%",
         ranks: [[RangePct(1.0)]]),
        (id: "tide_ev_c2", name: "Marea che avvolge", col: 2, row: 4, requires: ["tide_ev_c1"],
         description: "Chi è bagnato va al 46% della velocità",
         ranks: [[SlowFactor(0.46)]]),
    ]),
],

// sprout (Foglin)
talents: [
    (name: "Foglie", theme: Element, nodes: [
        (id: "sprout_el_a1", name: "Linfa urticante", col: 0, row: 0, requires: [],
         description: "Le foglie lasciano una linfa che brucia: 3–5% del colpo al secondo per 2 s",
         ranks: [[Burn(dps_pct: 3.0, duration_s: 2.0)], [Burn(dps_pct: 4.0, duration_s: 2.0)], [Burn(dps_pct: 5.0, duration_s: 2.0)]]),
        (id: "sprout_el_a2", name: "Foglia robusta", col: 0, row: 1, requires: ["sprout_el_a1"],
         description: "Danno di Foglin +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "sprout_el_a3", name: "Giro largo", col: 0, row: 2, requires: ["sprout_el_a2"],
         description: "Orbita più ampia: +2%",
         ranks: [[RangePct(2.0)]]),
        (id: "sprout_el_b1", name: "Radici che afferrano", col: 1, row: 0, requires: [],
         description: "Ogni 12 s spuntano radici sotto il nemico più vicino (60% del colpo)",
         ranks: [[Strike(every_s: 12.0, range: 130.0, radius: 20.0, damage_pct: 60.0)]]),
        (id: "sprout_el_b2", name: "Vento tra le foglie", col: 1, row: 1, requires: ["sprout_el_b1"],
         description: "Foglie più veloci: +2% a grado",
         ranks: [[AngularSpeedPct(2.0)], [AngularSpeedPct(2.0)], [AngularSpeedPct(2.0)]]),
        (id: "sprout_el_b3", name: "Rami lunghi", col: 1, row: 2, requires: ["sprout_el_b2"],
         description: "Orbita più ampia: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "sprout_el_c1", name: "Rovo e radice", col: 2, row: 3, requires: ["sprout_el_a3", "sprout_el_b3"],
         description: "Foglie più veloci: +2%",
         ranks: [[AngularSpeedPct(2.0)]]),
        (id: "sprout_el_c2", name: "Radici spinose", col: 2, row: 4, requires: ["sprout_el_c1"],
         description: "Radici ogni 10 s, più larghe e più forti (70% del colpo)",
         ranks: [[Strike(every_s: 10.0, range: 140.0, radius: 22.0, damage_pct: 70.0)]]),
    ]),
    (name: "Supporto", theme: Support, nodes: [
        (id: "sprout_su_a1", name: "Scorza di corteccia", col: 0, row: 0, requires: [],
         description: "Danni subiti -1% a grado",
         ranks: [[PlayerDamageTakenPct(-1.0)], [PlayerDamageTakenPct(-1.0)], [PlayerDamageTakenPct(-1.0)]]),
        (id: "sprout_su_a2", name: "Linfa che nutre", col: 0, row: 1, requires: ["sprout_su_a1"],
         description: "Rigeneri 0,02 HP al secondo a grado",
         ranks: [[PlayerRegen(0.02)], [PlayerRegen(0.02)], [PlayerRegen(0.02)]]),
        (id: "sprout_su_a3", name: "Radice salda", col: 0, row: 2, requires: ["sprout_su_a2"],
         description: "HP massimi +3",
         ranks: [[PlayerMaxHp(3.0)]]),
        (id: "sprout_su_b1", name: "Foglia larga", col: 1, row: 0, requires: [],
         description: "HP massimi +2 a grado",
         ranks: [[PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)]]),
        (id: "sprout_su_b2", name: "Rami che raccolgono", col: 1, row: 1, requires: ["sprout_su_b1"],
         description: "Raggio di raccolta +1% a grado",
         ranks: [[PlayerPickupPct(1.0)], [PlayerPickupPct(1.0)], [PlayerPickupPct(1.0)]]),
        (id: "sprout_su_b3", name: "Seme di riserva", col: 1, row: 2, requires: ["sprout_su_b2"],
         description: "Rigeneri 0,02 HP al secondo",
         ranks: [[PlayerRegen(0.02)]]),
        (id: "sprout_su_c1", name: "Bosco fitto", col: 2, row: 3, requires: ["sprout_su_a3", "sprout_su_b3"],
         description: "HP massimi +1",
         ranks: [[PlayerMaxHp(1.0)]]),
        (id: "sprout_su_c2", name: "Corteccia viva", col: 2, row: 4, requires: ["sprout_su_c1"],
         description: "Danni subiti -1%",
         ranks: [[PlayerDamageTakenPct(-1.0)]]),
    ]),
    (name: "Evoluzione", theme: Evolution, nodes: [
        (id: "sprout_ev_a1", name: "Foglie svelte", col: 0, row: 0, requires: [],
         description: "Foglie più veloci: +2% a grado",
         ranks: [[AngularSpeedPct(2.0)], [AngularSpeedPct(2.0)], [AngularSpeedPct(2.0)]]),
        (id: "sprout_ev_a2", name: "Linfa forte", col: 0, row: 1, requires: ["sprout_ev_a1"],
         description: "Danno +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "sprout_ev_a3", name: "Nervature", col: 0, row: 2, requires: ["sprout_ev_a2"],
         description: "Orbita più ampia: +1%",
         ranks: [[RangePct(1.0)]]),
        (id: "sprout_ev_b1", name: "Giro veloce", col: 1, row: 0, requires: [],
         description: "Foglie più veloci: +2% a grado",
         ranks: [[AngularSpeedPct(2.0)], [AngularSpeedPct(2.0)], [AngularSpeedPct(2.0)]]),
        (id: "sprout_ev_b2", name: "Orbita ampia", col: 1, row: 1, requires: ["sprout_ev_b1"],
         description: "Orbita più ampia: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "sprout_ev_b3", name: "Nodo del ramo", col: 1, row: 2, requires: ["sprout_ev_b2"],
         description: "Foglie più veloci: +2%",
         ranks: [[AngularSpeedPct(2.0)]]),
        (id: "sprout_ev_c1", name: "Foglin cresciuto", col: 2, row: 3, requires: ["sprout_ev_a3", "sprout_ev_b3"],
         description: "Foglie più veloci: +2%",
         ranks: [[AngularSpeedPct(2.0)]]),
        (id: "sprout_ev_c2", name: "Quarta foglia", col: 2, row: 4, requires: ["sprout_ev_c1"],
         description: "Una foglia in più in orbita",
         ranks: [[ExtraCount(1)]]),
    ]),
],

// spark (Tuonny)
talents: [
    (name: "Fulmini", theme: Element, nodes: [
        (id: "spark_el_a1", name: "Eco del lampo", col: 0, row: 0, requires: [],
         description: "Il fulmine riparte poco dopo al 2–4% e scuote di nuovo",
         ranks: [[Echo(delay_s: 0.45, damage_pct: 2.0)], [Echo(delay_s: 0.42, damage_pct: 3.0)], [Echo(delay_s: 0.4, damage_pct: 4.0)]]),
        (id: "spark_el_a2", name: "Lampo vivo", col: 0, row: 1, requires: ["spark_el_a1"],
         description: "Danno di Tuonny +1% a grado",
         ranks: [[DamagePct(1.0)], [DamagePct(1.0)], [DamagePct(1.0)]]),
        (id: "spark_el_a3", name: "Coda di stelle", col: 0, row: 2, requires: ["spark_el_a2"],
         description: "Fulmine e salti arrivano più lontano: +3%",
         ranks: [[RangePct(3.0)]]),
        (id: "spark_el_b1", name: "Lampo dal cielo", col: 1, row: 0, requires: [],
         description: "Ogni 12 s un lampo cade sul nemico più vicino (50% del colpo)",
         ranks: [[Strike(every_s: 12.0, range: 160.0, radius: 20.0, damage_pct: 50.0)]]),
        (id: "spark_el_b2", name: "Aria tesa", col: 1, row: 1, requires: ["spark_el_b1"],
         description: "Fulmine e salti più lontano: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "spark_el_b3", name: "Nube bassa", col: 1, row: 2, requires: ["spark_el_b2"],
         description: "Fulmine e salti più lontano: +2%",
         ranks: [[RangePct(2.0)]]),
        (id: "spark_el_c1", name: "Lampo e catena", col: 2, row: 3, requires: ["spark_el_a3", "spark_el_b3"],
         description: "Fulmine e salti più lontano: +2%",
         ranks: [[RangePct(2.0)]]),
        (id: "spark_el_c2", name: "Temporale", col: 2, row: 4, requires: ["spark_el_c1"],
         description: "Lampi ogni 10 s, più larghi e più forti (60% del colpo)",
         ranks: [[Strike(every_s: 10.0, range: 160.0, radius: 24.0, damage_pct: 60.0)]]),
    ]),
    (name: "Supporto", theme: Support, nodes: [
        (id: "spark_su_a1", name: "Passo elettrico", col: 0, row: 0, requires: [],
         description: "Velocità di movimento +1% a grado",
         ranks: [[PlayerSpeedPct(1.0)], [PlayerSpeedPct(1.0)], [PlayerSpeedPct(1.0)]]),
        (id: "spark_su_a2", name: "Pelo che attira", col: 0, row: 1, requires: ["spark_su_a1"],
         description: "Raggio di raccolta +2% a grado",
         ranks: [[PlayerPickupPct(2.0)], [PlayerPickupPct(2.0)], [PlayerPickupPct(2.0)]]),
        (id: "spark_su_a3", name: "Scossa amica", col: 0, row: 2, requires: ["spark_su_a2"],
         description: "HP massimi +2 a grado",
         ranks: [[PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)], [PlayerMaxHp(2.0)]]),
        (id: "spark_su_b1", name: "Monete attratte", col: 1, row: 0, requires: [],
         description: "Monete +2% a grado",
         ranks: [[PlayerCoinPct(2.0)], [PlayerCoinPct(2.0)], [PlayerCoinPct(2.0)]]),
        (id: "spark_su_b2", name: "Cuore che batte", col: 1, row: 1, requires: ["spark_su_b1"],
         description: "HP massimi +1 a grado",
         ranks: [[PlayerMaxHp(1.0)], [PlayerMaxHp(1.0)], [PlayerMaxHp(1.0)]]),
        (id: "spark_su_b3", name: "Calamita viva", col: 1, row: 2, requires: ["spark_su_b2"],
         description: "Raggio di raccolta +1% a grado",
         ranks: [[PlayerPickupPct(1.0)], [PlayerPickupPct(1.0)], [PlayerPickupPct(1.0)]]),
        (id: "spark_su_c1", name: "Corrente amica", col: 2, row: 3, requires: ["spark_su_a3", "spark_su_b3"],
         description: "Monete +3%",
         ranks: [[PlayerCoinPct(3.0)]]),
        (id: "spark_su_c2", name: "Lampo nei piedi", col: 2, row: 4, requires: ["spark_su_c1"],
         description: "Velocità di movimento +2%",
         ranks: [[PlayerSpeedPct(2.0)]]),
    ]),
    (name: "Evoluzione", theme: Evolution, nodes: [
        (id: "spark_ev_a1", name: "Catena salda", col: 0, row: 0, requires: [],
         description: "Ogni salto perde meno: 19–18% invece del 20%",
         ranks: [[ChainFalloff(0.81)], [ChainFalloff(0.815)], [ChainFalloff(0.82)]]),
        (id: "spark_ev_a2", name: "Salto lungo", col: 0, row: 1, requires: ["spark_ev_a1"],
         description: "Fulmine e salti più lontano: +2% a grado",
         ranks: [[RangePct(2.0)], [RangePct(2.0)], [RangePct(2.0)]]),
        (id: "spark_ev_a3", name: "Pelo carico", col: 0, row: 2, requires: ["spark_ev_a2"],
         description: "Danno +1%",
         ranks: [[DamagePct(1.0)]]),
        (id: "spark_ev_b1", name: "Coda tesa", col: 1, row: 0, requires: [],
         description: "Fulmine e salti più lontano: +2%",
         ranks: [[RangePct(2.0)]]),
        (id: "spark_ev_b2", name: "Scintille lontane", col: 1, row: 1, requires: ["spark_ev_b1"],
         description: "Fulmine e salti più lontano: +1% a grado",
         ranks: [[RangePct(1.0)], [RangePct(1.0)], [RangePct(1.0)]]),
        (id: "spark_ev_b3", name: "Pelo dritto", col: 1, row: 2, requires: ["spark_ev_b2"],
         description: "Danno +1%",
         ranks: [[DamagePct(1.0)]]),
        (id: "spark_ev_c1", name: "Tuonny cresciuto", col: 2, row: 3, requires: ["spark_ev_a3", "spark_ev_b3"],
         description: "Fulmine e salti più lontano: +2%",
         ranks: [[RangePct(2.0)]]),
        (id: "spark_ev_c2", name: "Salto in più", col: 2, row: 4, requires: ["spark_ev_c1"],
         description: "Il fulmine salta su un nemico in più",
         ranks: [[ExtraCount(1)]]),
    ]),
],
```

## 8. Test case per qa-performance

| # | Scenario | Seed | Atteso |
|---|---|---|---|
| T1 | Sonda giocatore medio, zero talenti | 1–8 × 4 compagni iniziali | ~84% (oggi 27/32): riferimento dei danni |
| T2 | Albero completo (48 punti) su tutti e quattro | 1–8 × 4 | 92–97% di vittorie; danni subiti ≥ 55% di T1 |
| T3 | Metà albero (24 punti: righe 0, poi i finali) | 1–8 × 4 | 88–94% di vittorie |
| T4 | Curva | unità | 19 → livello 1, 20 → 2, 69 → 4, 1308 → 25, 4344 → 49, oltre resta 49; ogni compagno ha 48 punti di nodi |
| T5 | Requisiti | unità | unione aperta solo con a3 e b3 ad almeno un grado; finale solo con l'unione |
| T6 | Meccaniche | unità | ogni meccanica e SlowFactor/SlowSecs/ChainFalloff nominate da un solo nodo, tranne le coppie 1/1 → finale del § 1; il finale sostituisce il valore |
| T7 | Effetti nuovi | unità | `PlayerDamageTakenPct(-4)`: un contatto da 10 toglie 9,6; `PlayerInvulnSecs(0.03)`: finestra di 0,78 s |
| T8 | Validazione con carte, reliquie e talenti al massimo | unità | player 116 px/s < 120; onda di Ondino 0,74 s < 0,76 s; foglie e salti nei limiti |

## 9. Rischi

- Le stime dipendono dalla calca di riferimento: la sonda misuri danno e cura per compagno
  (il riepilogo li ha già) prima di toccare i valori, e tagli nell'ordine del § 4.
- Foglin e Tuonny hanno rami ripetitivi (velocità delle foglie, portata del fulmine): le loro
  statistiche sono poche e quasi tutte pesano sul danno. Se l'utente li trova noiosi, servono
  statistiche nuove per l'orbita e la catena (per esempio la portata dei soli salti).
- La quarta foglia e la velocità delle foglie valgono di più con le carte di cadenza (il
  limite di un colpo ogni 0,6 s sullo stesso nemico si abbassa): misurare Foglin a fine run.
- Cura e rigenerazione dei Supporti sono piccole ma si sommano a Germoglio e Rugiada perenne:
  controllare le vittorie della sonda più che i danni.
