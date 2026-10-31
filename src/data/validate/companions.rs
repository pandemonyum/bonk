//! Regole per `companions.ron`: inseguimento, foglio, testi della selezione, attacco e sblocchi
//! (D-066: almeno un compagno `Start`, e lo è quello con cui parte lo stage).

use super::Report;
use crate::data::GameConfig;
use crate::data::schema::Unlock;

pub(super) fn check(r: &mut Report, config: &GameConfig) {
    r.unique_ids("companions.ron", config.companions.iter().map(|c| &c.id));
    for c in &config.companions {
        let at = |field: &str| format!("companions.ron: {}.{field}", c.id);
        r.non_negative(&at("follow.distance"), c.follow.distance);
        r.positive(&at("follow.max_speed"), c.follow.max_speed);
        r.positive(&at("follow.gain"), c.follow.gain);
        r.sprite(&at("sprite"), &c.sprite);
        r.sheet_frame(&at("frame"), c.frame);
        r.text(&at("display_name"), &c.display_name);
        r.text(&at("description"), &c.description);
        let exists = config.attack(&c.attack).is_some();
        r.reference(&at("attack"), &c.attack, exists, "attacks.ron");
    }
    if !config.companions.iter().any(|c| c.unlock == Unlock::Start) {
        r.0.push(
            "companions.ron: serve almeno un compagno con unlock: Start, altrimenti un profilo \
             nuovo non ha nessun compagno iniziale"
                .into(),
        );
    }
    // il riferimento mancante lo segnala già il controllo dello stage
    let s = &config.stage;
    if let Some(starter) = config.companion(&s.starting_companion)
        && starter.unlock != Unlock::Start
    {
        r.0.push(format!(
            "stages.ron: {}.starting_companion = \"{}\": serve un compagno con unlock: Start \
             (è quello delle run senza menu e il ripiego del menu)",
            s.id, starter.id
        ));
    }
}
