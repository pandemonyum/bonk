# GDD — Pixel Monster Survivor (MVP)

Documento pratico del game-designer. Fonte dei requisiti: handoff §1 e §7. I valori vivono in
`game-data/*.ron`; questo documento ne spiega il perché e registra le misure.

## Fantasia e loop

Un giovane evocatore corre nella "Prateria dei Germogli" con compagni mostriciattolo
originali che attaccano da soli. Run di 10 minuti: muoviti, raccogli XP, scegli 1 carta su 3
a ogni livello, sopravvivi alle orde, batti il boss. Fine run: vittoria o morte, risultati,
restart immediato.

## Comandi

Menu: 1/2/3, frecce o clic per il compagno, Invio per iniziare. WASD/frecce: movimento 8
direzioni. Esc o P: pausa. F5: ricomincia. Sui risultati: Invio o F5 nuova run con lo stesso
compagno, M menu. 1/2/3 o clic: carta del draft. F3: overlay debug. F12: screenshot. Nessun tasto d'attacco: gli
attacchi sono automatici. In gioco l'HUD ricorda solo "Esc: pausa e comandi": l'elenco dei
comandi è nella schermata di pausa (con i potenziamenti presi) e in breve nel menu.

## HUD (P0.7)

Tutta l'interfaccia è disegnata per 960×540 e cresce con la scala intera del canvas, con lo
stesso pixel del mondo (D-045): a 1920×1080 è doppia, mai sfocata.

| Dove | Cosa | Perché |
|---|---|---|
| In alto a sinistra | cuori (uno ogni 20 HP) e HP in numeri; barra XP; fila delle abilità: un posto per ogni posto della squadra, con l'icona dell'elemento e la barra di ricarica dell'attacco, il lucchetto sui posti liberi | vita, crescita e squadra in un colpo d'occhio; il lucchetto ricorda che un compagno può arrivare dal draft |
| Sotto il player | barra degli HP di 16×2 texel, sempre visibile | si legge la vita senza staccare gli occhi dalla calca |
| In alto al centro | tempo che manca al boss (rosso negli ultimi 30 s), poi "BOSS" con nome e barra degli HP | l'arco della run |
| In alto a destra | livello, nemici sconfitti | progressi della run |
| Bordo dello schermo | teschio su un riquadro rosso nella direzione del boss, finché il boss non è a schermo | il boss nasce fuori vista: senza indicatore lo si cercava a caso |
| In basso a sinistra | "Esc: pausa e comandi" | unico promemoria fisso |

Testi in Tiny5 (numeri, etichette) e Pixelify Sans (titoli, frasi), D-044.

## Valori attuali

| Parametro | Valore | Motivo |
|---|---|---|
| Evocatore | 75 px/s, 100 HP, 0,75 s di invulnerabilità dopo un colpo | attraversa lo schermo in ~6 s; il tempo di uscire dalla calca (P0.6: da 0,6) |
| Arena | 4096×4096 px, bordo di siepe | ~55 s per attraversarla: spazio per fuggire senza perdersi |
| Simulazione | 60 Hz fissi | riproducibile con seed |
| Brasa (fuoco, iniziale) | proiettile: 12 danni, 1 colpo ogni 1,1 s, gittata 150 px, 220 px/s per 0,85 s | valori dell'handoff §7: il bersaglio singolo più lontano |
| Gocci (acqua) | onda: 8 danni a tutti entro 56 px ogni 2 s, fronte a 160 px/s, nemici al 55% di velocità per 1,5 s | difesa della calca: meno danno, tanti bersagli, rallenta chi ti stringe |
| Frond (natura) | 3 foglie a 34 px, mezzo giro al secondo, 6 danni, stesso nemico al massimo ogni 0,6 s | danno continuo appena fuori dal contatto: tiene lontani i più vicini |
| Squadra | 3 posti (`player.ron`): dietro, dietro-sinistra e dietro-destra a ±55°; scia 18–22 px, max 120 px/s | tre compagni visibili senza sovrapporsi né coprire il player |
| Ostacoli | densità 1,2%, zona libera 128 px, 60% alberi sull'erba | ~650 per arena, mai un vicolo cieco |

**Nemici (`enemies.ron`)**

| Archetipo | HP | Velocità | Danno a contatto | Ruolo |
|---|---|---|---|---|
| Gelatina | 30 (3 colpi) | 40 px/s | 2 | base, arriva in gruppo (P0.6: danno da 3) |
| Sciame | 12 (1 colpo) | 68 px/s | 2 | quasi veloce come il player: non si semina, va abbattuto |
| Corazzato | 120 (10 colpi) | 28 px/s | 6 | lento, blocca la strada mentre gli altri arrivano |
| Gelatina Gigante (élite) | 400 | 34 px/s | 8 | ogni 5 s, se sei entro 160 px, si ferma 0,8 s con la linea rossa a terra e scatta ~115 px; 25 XP |
| Gran Corazzato (boss) | 5500 (P0.7, da 3800) | 30 px/s | 15 | ogni 4 s segna un cerchio di 44 px dove sei: dopo 1 s colpisce per 25 chi è ancora dentro |

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
| Compagno garantito | nei draft dei livelli 2 e 3, con un posto libero, almeno una carta "nuovo compagno" (`companion_card_until_level: 3`, P0.7) | un'estrazione sfortunata lasciava Brasa da sola all'arrivo dello sciame: run deboli fino alla fine |

**Upgrade (`upgrades.ron`).** Le percentuali si sommano sul valore base (2 × +15% = +30%) e
tutto si ricalcola dai dati base più gli stack a ogni scelta.

| Carta | Effetto per stack | Rarità | Max | Requisiti |
|---|---|---|---|---|
| Fiamma viva | danno dei compagni +15% | comune | 5 | — |
| Respiro rapido | intervallo tra gli attacchi −10% | comune | 5 | — |
| Scintille gemelle | +1 proiettile a ventaglio, o +1 foglia | rara | 3 | Fiamma viva |
| Fuoco perforante | perforazione +1 | non comune | 3 | — |
| Vista lunga | gittata +20% (la durata del proiettile cresce con lei), area dell'onda e raggio delle foglie +20% | comune | 3 | — |
| Vampa veloce | velocità dei proiettili +20% | comune | 2 | — |
| Scorza robusta | HP massimi +20, cura 20 | comune | 5 | — |
| Germoglio | rigenera 0,3 HP/s (P0.7, da 0,5) | non comune | 3 | — |
| Passo leggero | movimento +10% | non comune | 3 | — |
| Richiamo | raggio di raccolta +25% | comune | 3 | — |
| Nuovo compagno: Gocci | Gocci entra nel primo posto libero, con gli upgrade già presi | non comune | 1 | un posto libero |
| Nuovo compagno: Frond | Frond entra nel primo posto libero, con gli upgrade già presi | non comune | 1 | un posto libero |
| Merenda | cura 30 HP, nessuno stack permanente | comune | 99 | — (il draft non resta vuoto) |

Danno e cadenza valgono per tutti i pattern (per le foglie la "cadenza" è l'intervallo tra due
colpi sullo stesso nemico); velocità dei proiettili e perforazione solo per i proiettili. La
validazione controlla i limiti fisici
con tutti gli upgrade al massimo (proiettili che attraversano i nemici, corpi che saltano oltre
un tronco, cadenza sotto un passo, gemme più lente del player).

**Ondate (`stages.ron`, ritmo = base + crescita × minuti dall'inizio dell'ondata)**

| Ondata | Da | Base/min | Crescita/min² | Max vivi |
|---|---|---|---|---|
| Gelatina | 0:00 | 24 | 12 | 90 |
| Sciame | 1:40 | 16 | 15 | 110 |
| Corazzato | 2:30 | 4 | 2 | 25 |

5 gelatine all'avvio; al massimo 220 nemici insieme (obiettivo prestazioni P0); nascita
nell'anello 290–360 px dal player, sempre fuori vista. Lo sciame arriva a 1:40 (P0.6, da 1:00):
quando di solito la squadra ha un secondo compagno.

**Arco della run (`stages.ron: run`, P0.6)**

| Parametro | Valore | Motivo |
|---|---|---|
| Durata | 10:00, poi le ondate si fermano ed entra il boss; sconfiggerlo è la vittoria | handoff: run di 10 minuti con boss finale |
| Difficoltà | nemici di ondate ed élite +15% HP e +12% danno per minuto, alla nascita (a 10:00 ×2,5 e ×2,2; P0.7, danno da +8%) | la squadra cresce, le orde anche; la seconda metà deve far paura |
| Élite | Gelatina Gigante a 2:30, poi ogni 75 s (~6 a run) | un pericolo da leggere, non da subire: si ferma e mostra la carica |
| Boss | Gran Corazzato, non scalato | tarato per la fine della run |
| Preavvisi | almeno 0,5 s (validato), lampeggio sempre più rapido | "telegraph chiaro" dell'handoff |
| Carte compagno | comuni (P0.6, prima non comuni) | la squadra arriva nei primi livelli |

## Misure

- **ART-P2, nemici in proporzione al player (`print_survival_medium_player`, 24 run):** con
  hurtbox, separazione e contatto alzati per tutti 16/24 vittorie (morti fra 4:17 e 7:24); con
  il contatto dei nemici piccoli come prima (gelatina 10, sciame 8, corazzato 13) e più grande
  solo per élite (20) e boss (36): **24/24**, boss in 46–81 s (mediana ~61 s), danni 89–416.

- **P0.7, "giocatore medio" (`print_survival_medium_player`, `tests/sim/bot.rs`, 8 seed per
  ognuno dei tre compagni iniziali, run riproducibili dopo D-047).** Il bot si allontana dai
  nemici girando loro attorno, esce dai segni di pericolo, prende le gemme più convenienti e
  sceglie compagni, poi danno e cadenza. Passi:
  1. *Prima versione del bot* (fuga dritta): 22/24 vittorie ma crescita bimodale (livello
     14–20 o 36–37 a 10:00). La traccia minuto per minuto mostra la causa: inseguito
     dall'orda attraversava l'arena e non raccoglieva più (XP ferma da 6:59, 300 gemme a
     terra). Con Brasa iniziale pesava anche la sfortuna del draft: 3 run su 8 con 1–2
     compagni a 2:00 → compagno garantito nei draft dei livelli 2 e 3.
  2. *Bot che gira attorno e raccoglie di passaggio*, valori della P0.6: 24/24 vittorie,
     livello 36–37 a 10:00, boss in 39–79 s (mediana ~50 s), mai vicino a morire: la
     rigenerazione (fino a 1,5 HP/s, ~90 HP al minuto) copriva quasi tutti i danni.
  3. *Valori della P0.7* (rigenerazione 0,3 HP/s per stack, danno +12% al minuto, boss 5500
     HP): **23/24 vittorie** (Brasa 7/8 con una sconfitta a 9:08, Gocci 8/8, Frond 8/8),
     primo level-up 0:19–0:44, livello 14–18 a 5:00, **boss in 50–95 s (mediana ~66 s)**,
     danni subiti 155–508 a run.
  Il bot reagisce a ogni passo a tutti i nemici vicini: un giocatore vero schiva peggio e
  sceglie meglio. Il prossimo passo è una prova umana, non un'altra taratura sul bot.
- **P0.7, autoplay cieco e fermi (`print_survival_with_upgrades`, 5 seed):** fermi si muore a
  0:18–0:19; l'autoplay con la scelta automatica arriva a 2:39–3:11, sempre con tre compagni
  (prima 1:37–2:53). Run completa con l'autoplay invulnerabile (gioco vero, seed 5, ×10):
  vittoria a 12:01, livello 37.

- **P0.2, colpi (`tests/sim/balance.rs`, 60 s, 5 seed):** 100% a segno, 14–18 nemici sconfitti
  al minuto con un compagno. La mira predittiva non serve.
- **P0.3, sopravvivenza senza upgrade (5 seed):** prima taratura (danni 5/3/10, invulnerabilità
  0,5 s, 8 nemici iniziali) troppo dura: fermi 0:10, autoplay 0:28–0:43. Dopo la taratura
  attuale: **fermi 0:13**, **autoplay 1:06–1:16**. L'autoplay è un movimento cieco (curva di
  Lissajous): un giocatore che schiva dura di più. La run completa di 10 minuti richiede gli
  upgrade della P0.4.
- **P0.3, carico:** 220 nemici a 59 FPS / 16,9 ms nel gioco (build dev, vsync); simulazione
  0,45–0,62 ms per passo con 50–500 nemici.
- **P0.6, run completa (`tests/sim/run.rs`, seed 5, autoplay invulnerabile):** boss a 10:00,
  vittoria a 11:49 con 3800 HP (combattimento 109 s), livello 37, 1592 sconfitti. Con 5000 HP il
  gioco vero (seed 5, ×10) aveva dato gli stessi livello e sconfitti del headless (vittoria a
  12:15).
- **P0.6, sopravvivenza vera (`print_survival_with_upgrades`, 5 seed):** da fermi 0:18–0:19;
  con l'autoplay e la scelta automatica che preferisce i compagni **1:37–2:53**, livello 3–9,
  2–3 compagni (P0.5: 1:08–1:22). Passi: sciame a 1:40, invulnerabilità 0,75 s, gelatina 2
  danni, carte compagno comuni.
- **P0.6, bot che schiva (`print_survival_kiting`, 5 seed):** sopravvive tutti i 13 minuti ma
  resta al livello 4–6 e non batte il boss (scappa da tutto, raccoglie poco). Un giocatore vero
  sta nel mezzo: la difficoltà va tarata con una prova umana (P0.7).
- **P0.5, squadra (5 seed, prima carta automatica):** con il player invulnerabile il livello
  finale a 10:00 passa da 14–25 a **33–36** (livello 10 a 2:47–5:01): tre compagni sconfiggono
  molti più nemici. Con l'autoplay e i danni la run dura ancora **1:08–1:22** (livello 2–3): si
  muore all'arrivo dello sciame (1:00), prima che i compagni nuovi arrivino o pesino. Il primo
  minuto va ribilanciato con la difficoltà della P0.6. Nel gioco, seed 20 con autoplay: Frond e
  Gocci ai livelli 2 e 3, a 2:41 livello 9 e 155 sconfitti con 13 nemici a schermo.
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

Élite e boss della P0.6 sono in `enemies.ron` (sprite segnaposto in attesa dell'art pass).

**Upgrade (~10, tutti con effetto runtime testabile, P0.4):** danno +15%, cadenza −10%,
proiettili +1, area +20%, raggio raccolta +25%, velocità +10%, HP massimi +20, rigenerazione,
perforazione +1, nuovo slot compagno. Ogni upgrade: ID, rarità, prerequisiti, max stack,
formula, testo UI.

## Obiettivi di bilanciamento

- Senza upgrade, il giocatore che resta fermo muore entro 90 s (non invincibile). **Oggi: 18–19 s.**
- Il primo level-up arriva entro 30–45 s (progressione percepibile). **Oggi: 0:30–0:41 con
  l'autoplay, 0:19–0:44 con il giocatore medio** (Gocci iniziale è il più rapido).
- A metà run almeno due build elementali diverse sono praticabili (scelte significative).
  **Oggi: tutti e tre i compagni iniziali portano il giocatore medio alla vittoria (7–8 su 8).**
- Il boss è sconfiggibile da una build media in 60–120 s. **Oggi: mediana ~66 s (50–95 s) con
  il giocatore medio e 5500 HP; 121 s con l'autoplay invulnerabile.**
- Il giocatore medio vince quasi sempre, ma la seconda metà della run lo mette in difficoltà.
  **Oggi: 23/24 vittorie, danni subiti fino a 508.** Da verificare con una prova umana.
