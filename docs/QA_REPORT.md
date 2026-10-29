# QA REPORT

## 2026-10-29 — Più mappa invece delle bande nere (D-062)

| Controllo | Esito | Prova |
|---|---|---|
| Finestre | 960×540 intera; 1280×720 tutta piena (640×360 di mondo); 1366×768 bande sottili oltre il limite | `docs/evidence/piu-mappa-1280x720.png` |
| Browser | 1280×720 piena, nessun errore; asset e dati in cartelle con la versione (niente cache vecchia) | `docs/evidence/web-piu-mappa.jpg` |
| Spawn | mai nella vista più grande; angoli e orde con la fascia di riserva (orda intera nel test) | test di `spawning` e `tests/sim/hordes.rs` |
| Bilanciamento | 27/32 (84%), danni medi 173: come prima | GDD, D-062 |
| Gate | 216 unit + 101 integrazione, 193 PNG | `./scripts/check.ps1` |

## 2026-10-29 — P1, il gioco nel browser (D-061)

| Controllo | Esito | Prova |
|---|---|---|
| Build | `dist/` da script, `.wasm` 22,5 MiB (5,9 gzip), nessun file oltre 25 MiB | `scripts/build-web.ps1` |
| Avvio | pagina, "Clicca per giocare", download con avanzamento, menu con ritratti, font e dati via HTTP | `docs/evidence/web-menu.jpg` |
| Gioco | run con la tastiera, terreno, nemici, HUD, draft | `docs/evidence/web-run.jpg` |
| Fine run e salvataggio | sconfitta, risultati con l'esperienza, profilo nel `localStorage` | `docs/evidence/web-risultati.jpg` |
| Opzioni dall'indirizzo | `?autoplay&seed=3`: la run parte da sola | `docs/evidence/web-autoplay.jpg` |
| Console | nessun errore del gioco né richieste fallite; un avviso WebGL all'avvio di Bevy | log del browser |
| Terreno nativo | stesso disegno di prima al pixel (0,17% di pixel diversi, sprite in movimento) | confronto prima/dopo |
| Test | 215 unit + 101 integrazione (`web::query_args`, pezzi del terreno) | `./scripts/check.ps1` |

## 2026-10-28 — Alberi grandi e schermata dei talenti (D-059)

| Controllo | Esito | Prova |
|---|---|---|
| Gradi e requisiti | un id per grado nel profilo, nodi aperti dai requisiti, statistiche sommate e meccaniche sostituite grado per grado, punti per livello | test di `talents`, `profile`, `talents_menu`, `talent_text` |
| Supporto | danni subiti, invulnerabilità e HP massimi del player seguono i talenti del compagno in squadra | `tests/sim/talents.rs` |
| Dati | 96 nodi validati (id unici, requisiti più in alto e nello stesso ramo, griglia, testi, valori, punti contro livelli, limiti del Supporto) | `validate/talents.rs` |
| Schermata | compagni, albero con tre colonne colorate, collegamenti, gradi, dettaglio con prossimo grado; icone del pixel-art-director | `docs/evidence/talenti-v2.png` |
| Bilanciamento | alberi completi: 32/32 con il 53% dei danni (guardia 97%/55% superata) | GDD, D-059 |

## 2026-10-28 — Riepilogo, squadra per livelli, orde, forzieri, talenti

| Controllo | Esito | Prova |
|---|---|---|
| Riepilogo | ogni sconfitto e ogni punto di danno al compagno giusto con i quattro pattern; cure delle carte senza eccesso | `tests/sim/recap.rs`; `docs/evidence/riepilogo-squadra.png`, `riepilogo-esperienza.png` |
| Squadra per livelli | ai livelli 3 e 9 solo compagni, altrove nessuno, squadra piena alla fine | `tests/sim/party.rs`, test di `upgrades` |
| Orde | nascono tutte, attorno al player e fuori vista, nella loro durata; stessa seed, stessa orda | `tests/sim/hordes.rs`; `docs/evidence/orda-annuncio.png`, `orda-in-corso.png` |
| Monete e forzieri | l'élite paga, la calamita le porta; forziere solo con le monete, reliquie, prezzo che sale, forziere nuovo, rinascita vicina; reliquie mai nel draft | `tests/sim/treasure.rs`; `docs/evidence/forziere-reliquie.png`, `monete-freccia-forziere.png` |
| Talenti | esplosione, soffio e bruciatura, cura con tetto ed eco, radici a tempo, statistiche dei rami; profilo su file (anche rovinato) | `tests/sim/talents.rs`, test di `profile`; `docs/evidence/talenti-effetti.png`, `talenti-menu.png` |
| Bilanciamento | 27/32 senza talenti; con tutti i talenti 32/32 anche dopo il taglio dei nodi 3, danni subiti 23–127 (~37% di quelli senza talenti): la guardia del 97%/50% resta superata, da rivedere con la prova umana o con l'albero nuovo | GDD, Misure |
| Avvio del gioco | si apriva e si chiudeva subito (risorsa mancante nell'HUD): corretto, menu e run controllati | screenshot del menu e delle run |

## 2026-10-28 — Compagni dell'utente e Tuonny

| Controllo | Esito | Prova |
|---|---|---|
| Fulmine a catena | colpisce il gruppo una volta ciascuno, −20% a salto, si ferma ai salti e alla portata, scossa sui colpiti, aspetta un bersaglio, "+1 proiettile" = +1 salto | `tests/sim/chain.rs`, test di `gameplay::chain` |
| Formula | danno per colpo e cadenza del fulmine con gli stack (istantaneo: +1 colpo all'istante 0) | `tests/sim/formulas.rs` |
| Validazione | salto, calo e scossa nei limiti | `validate/attacks.rs` |
| Menu e run | quattro carte (tasti 1–4), Tuonny scelto col 4, fulmini a zigzag tra i nemici | `docs/evidence/tuonny-menu.png`, `tuonny-fulmini.png` |
| Bilanciamento | Tuonny 6/8, Foglin 6/8 dopo la taratura (prima 4/8 e 3/8) | GDD, Misure |
| Gate | 174 unit + 86 integrazione, 48 PNG | `./scripts/check.ps1` |

## 2026-10-28 — Mappa nello stile nuovo

| Controllo | Esito | Prova |
|---|---|---|
| Tileset | stessi 42 indici e ruoli, test della generazione invariati e verdi | `./scripts/check.ps1` |
| Roccia e albero | impronte (7,4) e (4,4) misurate sui PNG nuovi: cadono su base e tronco | report del pixel-art-director |
| Gioco vero | seed 3 e 11: prato, terra, siepe, rocce, alberi con player e nemici | `docs/evidence/mappa-gioco.png` |

## 2026-10-28 — Nemici e boss animati

| Controllo | Esito | Prova |
|---|---|---|
| Contratto dei fogli | ogni PNG = 4 colonne × righe del nemico (2, o 4 con carica/colpo) per il suo `frame` | `enemy_sheets_match_their_frames`, `validate-assets.py` |
| Logica | verso specchiato, preparazione e scatto dell'élite, preparazione e colpo del boss, resti che cadono e svaniscono | test di `presentation::enemy_anim` |
| Validazione dei dati | `frame` non vuoto e largo un numero pari di texel | `enemy_sheet_frames_are_checked` |
| Nel gioco vero | orda, sciame, élite e boss animati, specchiati sul verso | `docs/evidence/nemici-animati-gioco.png`, `boss-animato-gioco.png` |
| Bilanciamento | 24/24 vittorie del giocatore medio, boss 46–81 s (prima della correzione del contatto 16/24) | GDD, Misure |
| Gate | 166 unit + 82 integrazione, 48 PNG | `./scripts/check.ps1` |

## 2026-10-28 — Player animato (sprite dell'utente)

| Controllo | Esito | Prova |
|---|---|---|
| Contratto del foglio | PNG 144×304 = 4×8 fotogrammi 36×38, palette propria, alpha 0/255, ogni fotogramma sul fondo | `the_player_sheet_matches_the_frame_grid`, `validate-assets.py` |
| Logica delle animazioni | direzione, specchio a sinistra, colpito, caduta a tempo reale tenuta a terra | test di `presentation::player_anim` |
| Cammino nel gioco vero | WASD reali: fronte, retro, profilo destro, sinistro specchiato | `docs/evidence/player-camminata-gioco.png` |
| Caduta e risultati | caduta in 4 tempi, a terra senza tinta, pannello dopo ~1,2 s | `docs/evidence/player-caduta-gioco.png` |
| Gate | fmt, clippy, 159 unit + 82 integrazione, asset | `./scripts/check.ps1` |
| Non verificato | prova umana; web | — |

## 2026-10-23 — P0.7 (HUD, formule, bilanciamento base)

| Controllo | Esito | Prova |
|---|---|---|
| Formule nella simulazione | danno per colpo e cadenza di proiettili, onda e foglie con 0/2/3 stack di danno e cadenza; livelli dall'XP con la curva dei dati | `tests/sim/formulas.rs` |
| Determinismo con la squadra intera in un'orda | il test nuovo fallisce con il codice vecchio (HashMap) e passa con il nuovo; seed 2 del giocatore medio ripetuta identica | `the_whole_party_in_a_crowd_is_deterministic` |
| Compagno garantito | 400 seed: sempre una carta compagno nei primi draft, cambia solo l'ultima carta e solo se serve; a squadra piena nessuna | `early_drafts_always_offer_a_companion_while_a_slot_is_free` |
| Gate | fmt, clippy, 153 unit + 82 integrazione, asset | `./scripts/check.ps1` |
| HUD a 960×540 | cuori, XP, abilità con ricarica, barra HP sotto il player, timer, livello, uccisioni | `docs/evidence/p07-hud.png` |
| HUD a 1440×810 (canvas ×3) | tutto ×1,5 con lo stesso pixel; tratti di Tiny5 a multipli di 6 px | `docs/evidence/p07-hud-1440x810.png` |
| Pausa | comandi e potenziamenti presi | `docs/evidence/p07-pausa.png` |
| Draft del livello 3 | carta "Nuovo compagno: Frond" garantita, lucchetto sul posto libero | `docs/evidence/p07-draft-compagno.png` |
| Boss fuori schermo | teschio al bordo in basso nella sua direzione; vittoria a 12:01 (autoplay ×10) | `docs/evidence/p07-boss-indicatore.png` |
| Restart con F5 a HP pieni | cuori e fila delle abilità ricostruiti | `docs/evidence/p07-restart.png` |
| Bilanciamento (giocatore medio, 24 run) | 23/24 vittorie, boss in 50–95 s, prima sconfitta a 9:08; fermi 0:18, autoplay 2:39–3:11 | GDD, Misure |
| Non verificato | prova umana; build web | — |

## 2026-10-23 — Font pixel (Tiny5, Pixelify Sans)

| Controllo | Esito | Prova |
|---|---|---|
| Copertura dei caratteri | entrambi: ASCII, Latin-1 tranne U+00AD (e U+00A6 in Pixelify), ‘ ’ “ ” – — … €; test sul cmap dei file | `presentation::fonts::tests` |
| Validazione dei testi | accentate accettate, simboli fuori dai font rifiutati con il codice U+ | test `card_texts_use_only_characters_the_fonts_draw`, `names_shown_on_screen_use_characters_the_fonts_draw` |
| Gate | fmt, clippy, 145 unit + 79 integrazione, 43 PNG e 2 font con licenza | `./scripts/check.ps1` |
| Menu (tastiera reale) | titolo e nomi in Pixelify Sans, tasti ed etichette in Tiny5, "1–3" | `docs/evidence/fonts-menu.png` |
| Draft (tastiera e mouse reali) | "Velocità dei proiettili", nome su due righe e descrizione su tre dentro la carta 212×252 | `docs/evidence/fonts-draft.png` |
| HUD e boss (run x10, seed 5) | Tiny5 con pixel 3×3 esatti a 24 px, ombra leggibile sull'erba, nome e barra del boss | `docs/evidence/fonts-hud-boss.png` |
| Pausa, sconfitta, vittoria | "PAUSA — Esc", "×1", "Gran Corazzato sconfitto: Prateria dei Germogli è salva!" | `docs/evidence/fonts-vittoria.png` |

## 2026-10-23 — Art pass v1 nel gioco

| Controllo | Esito | Prova |
|---|---|---|
| Unione del ramo grafico | 5 conflitti (sprite di élite/boss/segno, licenze, manifest) risolti; 43 PNG validi, nessun avviso | — |
| Test dopo l'unione e dopo i valori di STYLE_HANDOFF | 143 unit + 79 integrazione verdi (run completa compresa) | `./scripts/check.ps1` |
| Menu | ritratti 32×32 e icone degli elementi alla scala del canvas, colori v1 | `docs/evidence/v1-menu.png` |
| Draft con tastiera reale | icone degli upgrade sulle carte, titolo "LEVEL UP!", scelta col tasto 2 | `docs/evidence/v1-draft-icone.png` |
| HUD in gioco | cuori che seguono gli HP massimi (6 con 120), barra XP, icone della squadra, orologio, livello, teschio | `docs/evidence/v1-hud.png` |
| Run x10 fino al boss (seed 5) | boss v1, cerchio rosso, barra con teschio, vittoria a 11:43, livello 37, 1592 sconfitti | `docs/evidence/v1-boss-preavviso.png` (ancora col nome "Re Gelatina", poi rinominato), `v1-vittoria.png` |

## 2026-10-23 — P0.6 (arco della run, élite, boss, menu, vittoria e restart)

### Automatici

| Controllo | Esito |
|---|---|
| `./scripts/check.ps1` | OK: 218 test (139 unitari + 79 integrazione) + 6 sonde ignorate, 17 PNG, 13 sprite citati dai dati |
| Run completa headless (seed 5, autoplay invulnerabile) | boss a 10:00, vittoria a 11:49 (combattimento 109 s), livello 37, 1592 sconfitti; il restart coincide con una run nuova in tutti i conteggi |
| Sopravvivenza vera (`print_survival_with_upgrades`, 5 seed) | fermi 0:18–0:19; autoplay 1:37–2:53, 2–3 compagni |
| Bot che schiva (`print_survival_kiting`, 5 seed) | vivo a 12:59 in tutti i seed, livello 4–6, boss non sconfitto |

Controprove (regola rotta apposta → test che falliscono, poi ripristino e verde):

| Regola rotta | Test che falliscono |
|---|---|
| vittoria anche con il player morto | `dying_together_with_the_boss_is_a_defeat` |
| simulazione che continua dopo la caduta del boss | `the_boss_falling_is_the_last_step_even_with_accelerated_time` |
| preavviso non legato a chi lo lancia | `a_slam_vanishes_when_its_boss_falls` |
| perforazione e velocità anche senza chi spara | `the_companion_chosen_in_the_menu_starts_the_run`, `cards_only_appear_when_someone_can_use_them` |

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| Avvio senza autoplay | menu con i tre compagni, barre relative, mappa e obiettivo | `docs/evidence/p06-menu.png` |
| Tastiera reale: 2 + Invio | run con Gocci, conto alla rovescia 09:56 in alto | screenshot di controllo |
| `--seed 5 --autoplay --god --time-scale 10 --overlay` | élite a 7:15, timer 02:45; a 10:00 "BOSS" con barra HP; cerchio rosso dove stava il player; vittoria a 12:15 (5000 HP) | `docs/evidence/p06-elite-timer.png`, `p06-boss-telegraph.png`, `p06-vittoria.png` |

### Code review P0.6

Verdetto CHANGES REQUESTED, nessun bloccante:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Ricarica dei dati nel menu: carte vecchie, scelta per indice sulla lista nuova | anche dal menu si passa da `Loading` (si torna al menu); scelta per id | test del loader |
| 2 | Test "muoiono insieme" vacuo (il boss a 0 HP non veniva rimosso) | caso vero: boss tolto dal combattimento e player ucciso dal contatto nello stesso passo; vittoria ordinata dopo la sconfitta | controprova |
| 3 | Restart dopo la run completa controllato poco | censimento completo (entità P0.6 comprese) confrontato con una run nuova; restart con un colpo in arrivo | gate verde |
| 4 | Test del boss non scalato che non poteva fallire | uguaglianza su HP e danno dei dati | gate verde |
| 5 | Test della difficoltà lento e fragile (orologio letto prima del passo) | orologio dopo il passo, crescita alta e 20 s | gate verde |
| 6 | Validazione: nomi a schermo non ASCII, colpo non schivabile, carica oltre il player | `ui_text` su nomi di compagni, nemici e stage; uscita dal cerchio entro il preavviso; scatto per passo ≤ contatto | test di validazione |
| 7 | Test mancanti (`strike_player`, carta del compagno scelto, determinismo di élite e boss, flussi menu) | test aggiunti; avvio dal menu spostato in `app::start_run` | gate verde |
| 8 | Carte inutili senza chi spara | idoneità legata alla squadra (D-042) | controprova |
| 9 | Esito dopo la caduta del boss dipendente dagli FPS | `simulation_running` si ferma al passo della caduta | controprova |
| 10 | Preavvisi non legati a chi li lancia | `Telegraph::source`, despawn senza effetto se muore | controprova |
| 11 | Ordine ambiguo tra carica e inseguimento | `charge_step.before(chase_player)` | codice |

## 2026-10-22 — P0.5 (secondo e terzo compagno, slot da dati)

### Automatici

| Controllo | Esito |
|---|---|
| `./scripts/check.ps1` | OK: 186 test (121 unitari + 65 integrazione) + 4 sonde ignorate, 14 PNG, 11 sprite citati dai dati |
| Sonda livelli (5 seed, prima carta automatica) | primo level-up 0:30–0:41; invulnerabile: livello 10 a 2:47–5:01, finale 33–36 a 10:00 (P0.4: 14–25) |
| Sonda sopravvivenza (autoplay, danni attivi) | 1:08–1:22, livello 2–3: invariata, si muore all'arrivo dello sciame |

Controprove (regola rotta apposta → test che falliscono, poi ripristino e verde):

| Regola rotta | Test che falliscono |
|---|---|
| compagno nuovo senza upgrade | `a_new_companion_arrives_with_the_upgrades_already_taken` |
| onda esclusa dal ricalcolo degli upgrade | `upgrades_taken_later_reach_every_companion` |
| carte compagno senza posti liberi | `a_full_party_gets_no_more_companion_cards` |
| formazione senza angolo del posto | `companions_keep_separate_places_in_the_formation` |
| onda che colpisce più volte lo stesso nemico | `the_wave_hits_each_enemy_in_reach_once_and_slows_it` |
| inseguimento che ignora il rallentamento | `slowed_enemies_chase_slower_until_the_slow_wears_off` |
| foglie senza intervallo per nemico | `a_leaf_hits_the_same_enemy_at_most_once_per_interval` |
| compagno nuovo con direzione fissa | `a_new_companion_starts_behind_the_player` |
| carta per un compagno già in squadra | `companion_cards_need_a_free_slot` |
| foglie non controllate con l'orbita più larga | `leaves_are_checked_with_the_widest_orbit` |
| durata dell'onda non controllata | `a_wave_ends_before_the_next_one` |
| rallentamento che scade al doppio | `slowed_enemies_chase_slower_until_the_slow_wears_off` |
| durata del rallentamento sbagliata | `the_wave_hits_each_enemy_in_reach_once_and_slows_it` |

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| `--seed 20 --autoplay --god --time-scale 3 --overlay` | Frond (posto 2) e Gocci (posto 3) ai livelli 2 e 3, tre compagni in formazione distinti | `docs/evidence/p05-tre-compagni.png` |
| `--seed 20 --autoplay --god --time-scale 4 --overlay` | anello di gocce dell'onda, 4 foglie dopo "+1 proiettile", livello 9 e 155 sconfitti a 2:41, 60 FPS / 16,7 ms | `docs/evidence/p05-squadra-onda-foglie.png` |

Il seed 20 è stato scelto perché la prima carta automatica porta presto entrambi i compagni:
la sequenza dei draft dipende solo dal seed e dalle carte prese, quindi è la stessa in headless
e nel client.

### Code review P0.5

Verdetto APPROVE con correzioni consigliate, nessun bloccante:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Regola delle foglie con l'orbita più larga e tetto delle foglie senza test propri | test dedicati con messaggi distinti | controprova |
| 2 | Test del rallentamento debole (durata e valore scritto dall'onda non controllati) | durata al passo, `remaining` controllato dopo l'onda | 2 controprove |
| 3 | Compagno nuovo nato con direzione fissa: davanti al player che andava verso ovest | direzione di marcia della squadra | test nuovo con controprova |
| 4 | Intervallo delle foglie arrotondato per eccesso con gli upgrade (0,54 s → 33 passi) | arrotondato al passo più vicino | codice + test dell'intervallo |
| 5 | Posti liberi dagli stack: carta per il compagno iniziale sprecata (P0.6 con il menu) | `PartyView` dal compagno iniziale e dalle carte, esclusi i presenti | 2 test unitari, controprova |
| 6 | Due onde nello stesso passo: il rallentamento più forte poteva perdersi; onde lunghe senza limite | rallentamenti del passo combinati; onda più breve dell'intervallo, validato | test di validazione con controprova |
| 7 | Percentuale di colpi della sonda oltre il 100% con onde e foglie | `RunStats.projectile_hits` | sonda |
| 8 | Ricerca dei bersagli dell'onda anche in ricarica | uscita anticipata come i proiettili | test di cadenza |
| 9 | Campi validati senza test (`slow_s`, `center_height`, `radius`) | test di validazione | gate verde |
| 10 | Foglie ferme e congelamento vietati | restano vietati, motivo in D-035 | — |
| 11 | Determinismo non coperto per onda e foglie | test a parità di seed con nemici veri | gate verde |
| 12 | Restart senza controllo di foglie, onde e rallentamenti | test esteso | gate verde |

## 2026-10-22 — P0.4 (gemme XP, livelli, draft, upgrade a caldo)

### Automatici

| Controllo | Esito |
|---|---|
| `./scripts/check.ps1` | OK: 158 test (105 unitari + 53 integrazione) + 4 sonde ignorate, 10 PNG, 7 sprite citati dai dati |
| Sonda livelli (5 seed, prima carta automatica) | primo level-up con l'autoplay 0:30–0:41; da fermi morte a 0:13 al livello 1 |
| Sonda livelli con player invulnerabile (10 min) | livello 5 a ~2:20, livello 10 a 4:39–5:56, finale 14–25 |
| Sonda sopravvivenza con upgrade (autoplay) | 1:08–1:24, livello 2–3 (senza upgrade 1:06–1:16) |

Controprove (regola rotta apposta → test che falliscono, poi ripristino e verde):

| Regola rotta | Test che falliscono |
|---|---|
| il passo del livello non ferma la simulazione | `the_level_up_step_is_the_last_one_even_with_accelerated_time` |
| `+1 proiettile` ignorato nel ricalcolo | `the_multishot_upgrade_fires_one_more_projectile_per_attack` |
| nessun limite alle gemme a terra | `beyond_the_cap_new_xp_merges_into_gems_on_the_field` |
| `refresh_modifiers` anche in `Loading` (panic del gioco vero) | `the_app_waits_for_the_data_without_panicking_then_starts_the_run` |
| validazione carta per carta / somme che si compensano | `cards_with_the_same_effect_add_up`, `negative_cards_do_not_hide_the_upper_limits` |
| un solo messaggio letto per fotogramma | `extra_choices_in_the_same_frame_are_dropped` |
| carte estratte alla richiesta del draft | `a_pause_on_the_level_up_frame_does_not_change_the_cards` |
| attrazione che non dura | `an_attracted_gem_keeps_chasing_a_player_who_leaves_the_radius` |

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| Player fermo, `--god --time-scale 3`, input reale | draft al livello 2, tasto 2 sceglie la seconda carta, livello 3 dopo | log `upgrade scelto: ember_speed` |
| Mouse reale sulle carte | carta evidenziata al passaggio, clic sulla prima e sulla terza scelgono quelle | `docs/evidence/p04-draft.png` |
| `--autoplay --god --time-scale 4 --overlay` | livello 4 a 2:00, barra XP e conteggio gemme, draft aperto 0,6 s poi scelta automatica | `docs/evidence/p04-hud-xp-autoplay.png` |

Problemi trovati provando il gioco vero (invisibili in headless):
- panic all'avvio: `refresh_modifiers` chiedeva la `GameConfig` in `Loading`; corretto e coperto
  da un test che simula l'attesa asincrona dei dati;
- "più" disegnato come "pi" + quadratino: il font provvisorio non ha le accentate; testi
  riformulati e validazione ASCII (D-033).

### Code review P0.4

Verdetto CHANGES REQUESTED, nessun bloccante:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Validazione carta per carta: due carte di cadenza valide da sole portavano l'intervallo a 0 (60 raffiche/s); positive e negative si compensavano | somme separate per segno, fattore minimo > 0 e limiti fisici sul massimo; HP massimi cumulati (D-034) | 2 test nuovi, falliscono con la vecchia logica |
| 2 | Overflow di `pierce`/`projectiles` (`pierce` "infinito" + carta = panic in dev); raffiche illimitate | somme saturanti; massimo 32 proiettili per raffica | test di validazione |
| 3 | `draft_size` oltre 3 senza tasto né spazio a schermo | validato da 1 a 3 | test di validazione |
| 4 | Due scelte nello stesso fotogramma: la seconda finiva nel draft successivo | vale la prima, le altre scartate | test nuovo con controprova |
| 5 | XP dei nemici senza limite (overflow, ciclo da 1e9 livelli) | massimo 1000 XP per nemico, somme saturanti | test di validazione |
| 6 | Test della gemma attirata vacuo | il player esce dal raggio dopo l'aggancio | controprova |
| 7 | Valori di gioco nei test (slot 1, 2 XP, 60 px, livello 3 dopo 90 s) | slot, distanze e XP ricavati dai dati; livelli iniziali dati da una gemma | gate verde |
| 8 | `tests/sim/progression.rs` oltre 400 righe; gemma assente da `ASSET_LICENSES.md` | diviso in `progression.rs` e `upgrades.rs`; voce aggiunta; `Modifiers` in `upgrades/modifiers.rs` | tutti i file sotto 400 righe |
| 9 | `MaxHp(inf)` accettato; compagno più lento del player al massimo; prerequisito con peso 0; a capo vietato | valori finiti; `follow.max_speed` ≥ velocità massima; requisiti mai proposti segnalati; a capo ammesso | test di validazione |
| 10 | Test mancanti | morte e livello nello stesso passo, restart dal draft, draft vuoto, stesso seed con livelli e carte | 4 test nuovi + confronto esteso |
| P1 | Pausa nel fotogramma del livello: carte estratte di nuovo alla ripresa | estrazione in `OnEnter(LevelUp)` | test nuovo con controprova |
| P2 | Ricarica dei dati con il draft aperto: indici vecchi sulla lista nuova | `get` invece dell'indice diretto | codice |
| P3 | `Modifiers` della run precedente nei primi passi | azzerati in `reset_upgrades` | codice |
| P4 | Compagni aggiunti a metà run senza modificatori | annotato nella P0.5 del backlog | — |

## 2026-10-22 — P0.3 (spawn director, danni al player, griglia spaziale)

### Automatici

| Controllo | Esito |
|---|---|
| `./scripts/check.ps1` | OK: 122 test (85 unitari + 34 integrazione) + 3 sonde ignorate, 9 PNG, 6 sprite citati dai dati |
| Sonda costo per passo (build dev, headless) | 0,45 ms (50 nemici) / 0,49 ms (200) / 0,62 ms (500) |
| Sonda sopravvivenza senza upgrade (5 seed) | fermi 0:13; autoplay 1:06–1:16 |

Controprove: un director che ignora la vista fa fallire i test di spawn ("nato in vista");
la vecchia logica della sconfitta fa fallire `a_pause_on_the_killing_frame_does_not_cancel_the_defeat`
e `no_simulation_step_runs_after_the_killing_one`.

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| 3 min di gioco a ×4 (`--god`) | 135 nemici dei 3 archetipi, 59 FPS / 16,8 ms | `docs/evidence/p03-orda-135-nemici.png` |
| ~7 min di gioco a ×8 (`--god`) | 220 nemici (tetto), 59 FPS / 16,9 ms, 8 passi di simulazione per fotogramma | `docs/evidence/p03-220-nemici-59fps.png` |
| Run vera fino alla sconfitta | 01:06, schermata con tempo e sconfitti, mondo fermo | `docs/evidence/p03-sconfitta.png` |
| Invio da tastiera reale da sconfitti | run nuova, statistiche azzerate | `docs/evidence/p03-riavvio-con-invio.png` |

### Code review P0.3

Verdetto CHANGES REQUESTED, 7 rilievi, tutti corretti:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Una pausa (Esc o perdita di focus) nel fotogramma del colpo mortale cancellava la sconfitta: player a 0 HP immortale; col tempo accelerato la simulazione continuava dopo la morte | `simulation_running` (Playing e player vivo) per tutti i sistemi del passo fisso; `declare_defeat` in `PostUpdate` dagli HP | 2 test nuovi, falliscono con la vecchia logica |
| 2 | Da sconfitti gli sprite tremavano di 1 px (interpolazione con tempo che scorre) | tempo virtuale fermo anche in `Defeated` | asserzione nel test di sconfitta e di restart |
| 3 | Due ondate dello stesso nemico superavano il limite nello stesso passo | conteggio dei nati nel passo condiviso tra ondate | test `two_waves_of_the_same_enemy_share_its_cap` |
| 4 | L'ondata iniziale ignorava limite e inizio della prima ondata | validazione: `initial_burst` ≤ `waves[0].max_alive` e `waves[0].start_s` = 0 | test di validazione |
| 5 | Test legati ai valori dei dati (id, indici, HP fissi) | id, indici e numeri ricavati da `GameConfig`; nemici dei test di danno creati dal test | gate verde con gli stessi dati; non dipendono più dai numeri |
| 6 | Anello di spawn tutto fuori dall'arena accettato (run vuota senza errori) | validazione dell'anello rispetto al raggio utile dell'arena | test di validazione |
| 7 | `combat.rs` oltre 400 righe | attacchi spostati in `gameplay/attacks.rs`; test di validazione in `data/validate/tests.rs` | tutti i file sotto 400 righe |

## 2026-10-22 — P0.2, P0.1, P0.0 e riduzione dello spazio su disco

Stessa macchina della S0. Build `dev` con il nuovo profilo (dipendenze senza simboli di debug).

### Automatici

| Controllo | Comando | Esito |
|---|---|---|
| Gate completo | `./scripts/check.ps1` | OK: fmt, clippy `-D warnings`, test, asset |
| Unit test | `cargo test --lib` | 58/58 OK |
| Integrazione headless | `cargo test --test sim` | 21/21 OK, 1 ignorato (sonda di bilanciamento) |
| Asset + sprite citati dai dati | `python scripts/validate-assets.py` | OK, 7 PNG, 4 sprite citati in `game-data/` |
| Sonda bilanciamento | `cargo test --test sim balance -- --ignored --nocapture` | 54–55 colpi/min, 100% a segno, 14–18 sconfitti/min |

Controprove (il test fallisce quando il comportamento si rompe): ripresa dalla pausa senza
`resume_time`; sprite mancante citato in `enemies.ron`; dati con velocità negativa e
riferimento inesistente (schermata d'errore con entrambi i messaggi).

### Prove sul gioco reale

| Prova | Osservato | Prova |
|---|---|---|
| Compagno che attacca (P0.2) | proiettile in partenza dal compagno; tinta di colpo visibile su una gelatina | `docs/evidence/p02-compagno-proiettile.png`, `p00-dati-caricati-e-colpo.png` |
| Ostacoli (P0.1) | rocce e alberi sparsi, chioma che copre per profondità, orde che li aggirano; 58–62 FPS con ~1150 entità | `docs/evidence/p01-ostacoli-alberi.png`, `p01-ostacoli-radura.png` |
| Dati da `game-data/` (P0.0) | log "dati di gioco caricati da game-data/" e run avviata | `docs/evidence/p00-dati-caricati-e-colpo.png` |
| Dati non validi | schermata con "enemies.ron: blob.speed deve essere >= 0, trovato -3" e "stages.ron: meadow.enemy = \"dragon\": non esiste in enemies.ron" | `docs/evidence/p00-errori-dati.png` |
| Ricarica a caldo | `stages.ron` modificato a gioco avviato (10 → 20 nemici): log "dati di gioco ricaricati", la run riparte con 20 nemici e statistiche azzerate | `docs/evidence/p00-ricarica-a-caldo.png` |

### Spazio su disco (D-027)

| Momento | `target/` | Disco C: libero |
|---|---|---|
| Prima | 21,7 GB (11 GB di copie non più usate, .pdb da ~0,75 GB per eseguibile) | 6,5 GB |
| Dopo `cargo clean` + nuova configurazione: solo `cargo build` | 2,62 GB (build completa 14 min invece di 23) | |
| + test compilati (un solo eseguibile d'integrazione) | 2,85 GB | |
| + clippy | 3,31 GB | 25,6 GB |

Rimossa anche la toolchain `stable` duplicata (1,3 GB); default rustup = 1.99.1.

### Code review P0.2/P0.1/P0.0 (ruolo code-reviewer, lettura indipendente)

Verdetto iniziale CHANGES REQUESTED, 7 rilievi, tutti corretti. Dopo le correzioni: 92 test
(70 unitari + 22 d'integrazione, 1 sonda ignorata), gate verde, gioco avviato con i dati nuovi.

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | Lungo la siepe nord i colpi partivano fuori dal rettangolo dei piedi e sparivano al primo passo: il compagno non colpiva | limite dei proiettili = intera arena (`world::inside_arena`) | test `shots_fired_along_the_north_hedge_still_hit`; col vecchio limite il nemico resta a 30 HP (fallisce) |
| 2 | Ricarica a caldo: dopo un primo caricamento fallito la versione corretta arriva come `Added`, ignorata; un file illeggibile durante la run non mostrava errori | conta `Added`/`Modified`/`LoadedWithDependencies`; legge `AssetLoadFailedEvent` | 5 test del loader con AssetServer vero; reintroducendo il bug `fixing_an_unreadable_file_after_a_failed_first_load_starts_the_game` fallisce |
| 3 | Test della griglia spaziale vacuo (sonde senza ostacoli vicini) | sonde su ostacoli veri ai bordi e al centro, con asserzione "non vuoto" | test aggiornato |
| 4 | `pierce` enorme in overflow; corpi veloci attraversano i tronchi | `saturating_add`; validazione velocità ≤ (4 + raggio) px per passo | test di validazione (600 px/s rifiutati, 540 accettati) |
| 5 | Commento falso: il clamp poteva riportare un corpo dentro una roccia della riga più a nord | margine di 2 tile dalla siepe | test `obstacle_exclusion_zones_stay_inside_the_walkable_area` su 10 seed |
| 6 | Valori di bilanciamento nel codice (`LIFETIME_MARGIN`, densità ostacoli, zona libera, quota alberi) | `projectile_lifetime_s` in `attacks.ron`, `obstacles` in `stages.ron`, validati | test di validazione e `parameters_change_the_field` |
| 7 | Cadenza arrotondata per eccesso (0,935 s → 0,95 s) | carica che conserva il resto (`attack_tick`) | test: 66 passi esatti per 1,1 s, media 56,1 passi per 0,935 s, nessuna raffica dopo attesa |

### Limiti aperti

- Compilazione WASM non riverificata dopo la P0.0 (loader dei dati, watcher solo nativo):
  la verifica completa è il primo passo della P1.
- Ricarica a caldo provata sui dati; sugli sprite (`assets/`) è attiva ma non provata.
- I rimpiazzi dei nemici possono comparire in vista vicino ai bordi dell'arena (P0.3).

## 2026-10-22 — S0

**Macchina:** Windows 11 Enterprise 26200, Intel Core Ultra 5 238V, 31,5 GiB, Intel Arc 130V
integrata (Vulkan). **Build:** profilo `dev` (codice opt-level 1, dipendenze opt-level 3).

### Automatici

| Controllo | Comando | Esito |
|---|---|---|
| Formattazione | `cargo fmt --all -- --check` | OK |
| Lint | `cargo clippy --all-targets --locked -- -D warnings` | OK, 0 warning |
| Unit test | `cargo test --lib` | 31/31 OK |
| Simulazione headless | `cargo test --test headless_sim` | 7/7 OK (~1 s) |
| Asset | `python scripts/validate-assets.py` | OK, 3 PNG, 0 errori |
| Griglia pixel | `python scripts/check-pixel-grid.py` | OK 100% a 1366×768, 1365×767, 1101×701 @1.25 e @1.5 |
| Compilazione WASM | `cargo check --lib --target wasm32-unknown-unknown --locked` | OK (eseguito dal code-reviewer, 7 min 27 s; build web completa: P1) |

I test headless coprono: spawn (1 player + 10 nemici); velocità 75 px/s su 60 passi; player
sempre dentro l'arena dopo 60 s di spinta in diagonale; inseguimento (distanza media −75% in
8 s) con nessun nemico dentro il player e coppie di nemici sempre oltre 7 px (metà del raggio
di separazione); nemici mai dentro il player quando è schiacciato contro un muro; stesso seed =
stessa simulazione; pausa che congela la simulazione e ripresa che la fa ripartire.

### Prove sul gioco reale

| Prova | Come | Osservato | Prova |
|---|---|---|---|
| Avvio e rendering | `cargo run -- --autoplay --overlay --screenshot ...` | finestra 960×540, arena, 10 nemici che inseguono, pixel netti, exit 0 | `docs/evidence/s0-autoplay-960x540.png` |
| Tastiera reale | `keybd_event` di Windows: D 1 s, W 1 s, Esc | player (76,75) vs atteso (75,75); pausa con banner, fase `Paused` | `docs/evidence/s0-keyboard-pausa.png` |
| Resize / letterbox | `--window 1365x767` + `check-pixel-grid.py` | scala 2×, bande nere, 100% texel uniformi | `docs/evidence/s0-letterbox-1365x767.png` |
| Controprova mezzo pixel | stessa finestra con correzione disattivata | 96,1% → il controllo fallisce (non vacuo) | log nel report S0 |
| DPI 125% / 150% | `--window 1101x701 --scale-factor 1.25` e `1.5` | testo UI ingrandito (override attivo), canvas 2×, 100% texel uniformi | `docs/evidence/s0-dpi150-1101x701.png` |

Nota: a 960×540 senza bande nere il promemoria comandi copre il canvas e `check-pixel-grid`
misura ~99% anche con scala corretta: il controllo va fatto con finestre letterbox.

### Prestazioni (indicative, NON benchmark)

Overlay F3, build dev, 10 nemici, vsync attivo: 64 FPS / 15,7 ms a 15 s (1366×768), 60 FPS /
16,5 ms a 9 s (960×540). Nei primi 6 s la media è più bassa (49 FPS) per l'avvio. Benchmark
50/200/500 nemici in release: P3.1.

### Code review S0 (ruolo code-reviewer, lettura indipendente)

Verdetto iniziale CHANGES REQUESTED, 7 rilievi, tutti corretti nella stessa tranche:

| # | Rilievo | Correzione | Verifica |
|---|---|---|---|
| 1 | `check.ps1` poteva dire OK senza eseguire un passo (comando mancante) | exit code azzerato per passo, `try/catch`, fallback `py` | passo con comando inesistente → exit 1 |
| 2 | Nessun test sulla ripresa dalla pausa | asserzioni di ripresa nel test di pausa | rimuovendo `resume_time` il test fallisce |
| 3 | Ordine indefinito tra `move_player` e `chase_player` | nuovo `SimSet::Player` prima di `Movement` | test di determinismo verdi |
| 4 | Nemici dentro il player contro un muro | respinta dal player come ultima regola | nuovo test `enemies_never_enter_the_player_against_a_wall` |
| 5 | Limiti arena pensati per sprite centrati | `ArenaBounds` con `min`/`max` sui piedi | test `whole_sprite_stays_inside_the_hedge` |
| 6 | Attesa screenshot calcolata da `--exit-after` | si attende fino a `max(exit, screenshot) + 10 s` | revisione del codice |
| 7 | Affermazioni non provate nei documenti (determinismo cross-platform, "non sovrapposti", DPI 125%) | testi corretti; test di separazione rafforzato; DPI provato con `--scale-factor` | sopra |

### Limiti aperti

- DPI provato solo con override (`--scale-factor`), non su un monitor reale al 125%.
- Testato solo con finestra in primo piano; minimizzazione e monitor multipli non provati.
- La pausa su perdita di focus è attiva ma non provata con un vero cambio di finestra.
- Replay identici tra nativo e WASM non garantiti (vedi D-005).
- Build release non ancora compilata né misurata.
- Nessuna prova umana di "sensazione" del movimento: serve un playtest dell'utente.
