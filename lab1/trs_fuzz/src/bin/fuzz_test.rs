use rand::{Rng, SeedableRng, seq::SliceRandom};
use std::collections::{HashSet, VecDeque};

type Rule = (String, String);
type Rules = Vec<Rule>;

mod cfg {
    use super::Rules;

    pub const ALPHABET: &[char] = &['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'Q'];

    pub const T_BASE: &[(&str, &str)] = &[
        ("JFF", "EID"),
        ("CFG", "IC"),
        ("GF", "GG"),
        ("IC", "JI"),
        ("AD", "BD"),
        ("FFI", "HGH"),
        ("FHDQ", "FHEDQ"),
        ("HA", "JH"),
        ("HE", "HDD"),
        ("GGA", "HCE"),
        ("IBB", "DDI"),
    ];

    pub const TPRIME_BASE: &[(&str, &str)] = &[
        ("JFF", "EID"),
        ("CFG", "IC"),
        ("GG", "GF"),
        ("JI", "IC"),
        ("BD", "AD"),
        ("HGH", "FFI"),
        ("FHEDQ", "FHDQ"),
        ("JH", "HA"),
        ("HDD", "HE"),
        ("HCE", "GGA"),
        ("IBB", "DDI"),
    ];

    pub const KB_MAX_N: Option<usize> = Some(12);

    pub const NUM_TRIALS: usize = 500;

    pub const GEN_MIN_LEN: usize = 1;
    pub const GEN_MAX_LEN: usize = 10;

    pub const CHAIN_MIN_STEPS: usize = 1;
    pub const CHAIN_MAX_STEPS: usize = 10;

    pub const NF_MAX_DEPTH: usize = 200;
    pub const NF_MAX_NODES: usize = 200_000;

    pub const RNG_SEED: u64 = 7;

    pub fn rules_from(slice: &[(&str, &str)]) -> Rules {
        slice
            .iter()
            .map(|(l, r)| (l.to_string(), r.to_string()))
            .collect()
    }

    pub fn rules_t() -> Rules {
        rules_from(T_BASE)
    }

    pub fn kb_completion_rules(max_n: usize) -> Rules {
        let mut out = Vec::new();
        for n in 1..=max_n {
            let lhs = format!("G{}G", "F".repeat(n));
            let rhs = format!("G{}", "F".repeat(n + 1));
            out.push((lhs, rhs));
        }
        out
    }

    pub fn build_tprime() -> Rules {
        let mut tprime = rules_from(TPRIME_BASE);
        let extra = KB_MAX_N.map(kb_completion_rules).unwrap_or_default();
        tprime.extend(extra);
        tprime
    }
}

fn dedup_rules(rules: &Rules) -> Rules {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut out = Vec::new();
    for (l, r) in rules.iter() {
        let key = (l.clone(), r.clone());
        if seen.insert(key.clone()) {
            out.push((key.0, key.1));
        }
    }
    out
}

fn neighbors_fwd(word: &str, rules: &Rules) -> HashSet<String> {
    let mut outs: HashSet<String> = HashSet::new();
    for (lhs, rhs) in rules.iter() {
        let mut start = 0usize;
        while start <= word.len() {
            if let Some(rel) = word[start..].find(lhs) {
                let i = start + rel;
                let mut s = String::with_capacity(word.len() - lhs.len() + rhs.len());
                s.push_str(&word[..i]);
                s.push_str(rhs);
                s.push_str(&word[i + lhs.len()..]);
                outs.insert(s);
                start = i + 1;
            } else {
                break;
            }
        }
    }
    outs.remove(word);
    outs
}

fn rand_word(alphabet: &[char], min_len: usize, max_len: usize, rng: &mut impl Rng) -> String {
    let l = rng.gen_range(min_len..=max_len);
    let mut s = String::with_capacity(l);
    for _ in 0..l {
        s.push(*alphabet.choose(rng).unwrap());
    }
    s
}

fn random_chain(
    word: &str,
    rules: &Rules,
    min_steps: usize,
    max_steps: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut w = word.to_string();
    let mut chain = vec![w.clone()];
    let steps = rng.gen_range(min_steps..=max_steps);
    for _ in 0..steps {
        let neigh: Vec<String> = neighbors_fwd(&w, rules).into_iter().collect();
        if neigh.is_empty() {
            break;
        }
        w = neigh.choose(rng).unwrap().clone();
        chain.push(w.clone());
    }
    chain
}

fn normal_forms_all(
    start: &str,
    rules: &Rules,
    max_depth: usize,
    max_nodes: usize,
) -> (HashSet<String>, bool) {
    let mut q: VecDeque<(String, usize)> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut nforms: HashSet<String> = HashSet::new();

    q.push_back((start.to_string(), 0));
    visited.insert(start.to_string());

    let mut expanded = 0usize;
    let mut cut = false;

    while let Some((u, d)) = q.pop_front() {
        if expanded >= max_nodes {
            cut = true;
            break;
        }
        let outs = neighbors_fwd(&u, rules);
        if outs.is_empty() {
            nforms.insert(u);
            continue;
        }
        if d >= max_depth {
            cut = true;
            continue;
        }
        for v in outs {
            if expanded >= max_nodes {
                cut = true;
                break;
            }
            expanded += 1;
            if visited.insert(v.clone()) {
                q.push_back((v, d + 1));
            }
        }
        if cut {
            break;
        }
    }
    (nforms, cut)
}

fn fuzz_nf_compare(
    t: &Rules,
    tprime: &Rules,
    num_trials: usize,
    gen_min_len: usize,
    gen_max_len: usize,
    chain_min_steps: usize,
    chain_max_steps: usize,
    nf_max_depth: usize,
    nf_max_nodes: usize,
    seed: u64,
) -> (usize, usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let t = dedup_rules(t);
    let tprime = dedup_rules(tprime);

    let mut success = 0usize;
    let mut fail = 0usize;

    for _ in 0..num_trials {
        let w = rand_word(cfg::ALPHABET, gen_min_len, gen_max_len, &mut rng);
        let chain = random_chain(&w, &t, chain_min_steps, chain_max_steps, &mut rng);
        let w_prime = chain.last().unwrap().clone();

        let (w0_set, _cut1) = normal_forms_all(&w, &tprime, nf_max_depth, nf_max_nodes);
        let (w0p_set, _cut2) = normal_forms_all(&w_prime, &tprime, nf_max_depth, nf_max_nodes);

        let hit = w0_set.iter().any(|x| w0p_set.contains(x));
        if hit {
            success += 1;
        } else {
            fail += 1;
        }
    }

    (success, fail)
}

fn main() {
    let t = cfg::rules_t();
    let tprime = cfg::build_tprime();

    let (success, fail) = fuzz_nf_compare(
        &t,
        &tprime,
        cfg::NUM_TRIALS,
        cfg::GEN_MIN_LEN,
        cfg::GEN_MAX_LEN,
        cfg::CHAIN_MIN_STEPS,
        cfg::CHAIN_MAX_STEPS,
        cfg::NF_MAX_DEPTH,
        cfg::NF_MAX_NODES,
        cfg::RNG_SEED,
    );

    println!("NF-compare(T vs T'): success={}, fail={}", success, fail);
}
