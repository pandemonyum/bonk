//! Testi dei talenti per la schermata: cosa fa un nodo al suo grado (statistiche sommate,
//! meccaniche con i loro valori), in italiano e con i soli caratteri dei font (niente meno
//! tipografico né frecce).

use crate::data::schema::TalentNode;
use crate::gameplay::talents::TalentKit;

/// Numero con la virgola dei decimali e senza zeri inutili: 0.4 → "0,4", 24.0 → "24".
pub fn num(value: f32) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if (rounded - rounded.round()).abs() < 1e-4 {
        format!("{}", rounded.round() as i64)
    } else {
        format!("{rounded}").replace('.', ",")
    }
}

fn signed(value: f32) -> String {
    if value >= 0.0 {
        format!("+{}", num(value))
    } else {
        format!("-{}", num(-value))
    }
}

/// Le righe di un kit di talenti: una per statistica o meccanica presente.
pub fn kit_lines(kit: &TalentKit) -> Vec<String> {
    let s = &kit.stats;
    let mut out = Vec::new();
    if s.damage_pct != 0.0 {
        out.push(format!("Danno {}%", signed(s.damage_pct)));
    }
    if s.interval_pct != 0.0 {
        out.push(format!("Tempo di ricarica {}%", signed(s.interval_pct)));
    }
    if s.range_pct != 0.0 {
        out.push(format!("Portata {}%", signed(s.range_pct)));
    }
    if s.extra_count > 0 {
        out.push(format!("+{} proiettili, foglie o salti", s.extra_count));
    }
    if let Some(f) = s.slow_factor {
        out.push(format!("Rallenta al {}%", num(f * 100.0)));
    }
    if let Some(secs) = s.slow_secs {
        out.push(format!("Rallentamento per {} s", num(secs)));
    }
    if let Some(f) = s.falloff {
        out.push(format!("Calo per salto {}%", num((1.0 - f) * 100.0)));
    }
    if s.angular_pct != 0.0 {
        out.push(format!("Velocità delle foglie {}%", signed(s.angular_pct)));
    }
    if let Some(e) = kit.explode {
        out.push(format!(
            "Esplosione di {} px al {}%",
            num(e.radius),
            num(e.damage_pct)
        ));
    }
    if let Some(c) = kit.cone {
        out.push(format!(
            "Soffio ogni {} attacchi, {} px al {}%",
            c.every,
            num(c.range),
            num(c.damage_pct)
        ));
    }
    if let Some(h) = kit.heal {
        out.push(format!(
            "Cura {} HP per colpito, fino a {}",
            num(h.hp),
            num(h.cap)
        ));
    }
    if let Some(e) = kit.echo {
        out.push(format!(
            "Eco dopo {} s al {}%",
            num(e.delay),
            num(e.damage_pct)
        ));
    }
    if let Some(st) = kit.strike {
        out.push(format!(
            "Ogni {} s un colpo di {} px al {}%",
            num(st.every),
            num(st.radius),
            num(st.damage_pct)
        ));
    }
    if let Some(b) = kit.burn {
        out.push(format!(
            "Brucia il {}% al secondo per {} s",
            num(b.dps_pct),
            num(b.duration)
        ));
    }
    let p = &kit.player;
    if p.max_hp != 0.0 {
        out.push(format!("HP massimi {}", signed(p.max_hp)));
    }
    if p.regen != 0.0 {
        out.push(format!("Rigeneri {} HP al secondo", num(p.regen)));
    }
    if p.speed_pct != 0.0 {
        out.push(format!("Velocità {}%", signed(p.speed_pct)));
    }
    if p.pickup_pct != 0.0 {
        out.push(format!("Raggio di raccolta {}%", signed(p.pickup_pct)));
    }
    if p.coin_pct != 0.0 {
        out.push(format!("Monete {}%", signed(p.coin_pct)));
    }
    if p.damage_taken_pct != 0.0 {
        out.push(format!("Danni subiti {}%", signed(p.damage_taken_pct)));
    }
    if p.invuln_secs != 0.0 {
        out.push(format!("Invulnerabilità {} s", signed(p.invuln_secs)));
    }
    out
}

/// Il kit dei soli primi `rank` gradi di un nodo (senza gli altri nodi).
pub fn node_kit(node: &TalentNode, rank: u32) -> TalentKit {
    let branch = crate::data::schema::TalentBranch {
        name: String::new(),
        theme: Default::default(),
        nodes: vec![TalentNode {
            requires: Vec::new(),
            ..node.clone()
        }],
    };
    let learned = vec![node.id.clone(); rank as usize];
    TalentKit::resolve(std::slice::from_ref(&branch), &learned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::data::schema::TalentEffect;

    #[test]
    fn numbers_use_the_decimal_comma() {
        assert_eq!(num(24.0), "24");
        assert_eq!(num(0.4), "0,4");
        assert_eq!(num(0.25), "0,25");
        assert_eq!(signed(-12.0), "-12");
        assert_eq!(signed(5.0), "+5");
    }

    #[test]
    fn every_node_of_the_data_says_what_it_does() {
        let config = GameConfig::embedded();
        for c in &config.companions {
            for node in c.talents.iter().flat_map(|b| b.nodes.iter()) {
                let lines = kit_lines(&node_kit(node, node.max_rank()));
                assert!(!lines.is_empty(), "{}: nessuna riga", node.id);
                for line in lines {
                    assert!(
                        line.chars().all(crate::data::ui_drawable),
                        "{}: {line}",
                        node.id
                    );
                }
            }
        }
    }

    #[test]
    fn ranks_add_up_in_the_summary() {
        let node = TalentNode {
            id: "x".into(),
            name: "X".into(),
            description: String::new(),
            col: 0,
            row: 0,
            requires: vec![],
            ranks: vec![vec![TalentEffect::DamagePct(4.0)]; 3],
        };
        assert_eq!(kit_lines(&node_kit(&node, 2)), ["Danno +8%"]);
        assert_eq!(kit_lines(&node_kit(&node, 0)), Vec::<String>::new());
    }
}
