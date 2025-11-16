use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

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

    pub const GEN_MIN_LEN: usize = 10;
    pub const GEN_MAX_LEN: usize = 30;

    pub const CHAIN_MIN_STEPS: usize = 1;
    pub const CHAIN_MAX_STEPS: usize = 12;

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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Inv {
    q_count: usize,
    q_skeleton: String,
    parity_lin_1: u8,
}

fn counts(w: &str) -> HashMap<char, usize> {
    let mut c = HashMap::new();
    for ch in w.chars() {
        *c.entry(ch).or_insert(0) += 1;
    }
    c
}
fn parity_linear_1(cnt: &HashMap<char, usize>) -> u8 {
    let a = *cnt.get(&'A').unwrap_or(&0);
    let b = *cnt.get(&'B').unwrap_or(&0);
    let c = *cnt.get(&'C').unwrap_or(&0);
    let d = *cnt.get(&'D').unwrap_or(&0);
    let j = *cnt.get(&'J').unwrap_or(&0);
    ((a + b + c + d + j) % 2) as u8
}
fn q_count(w: &str) -> usize {
    w.bytes().filter(|&b| b == b'Q').count()
}
fn q_order_signature(w: &str) -> String {
    w.chars().filter(|&ch| ch == 'Q').collect()
}

fn invariants(w: &str) -> Inv {
    let cnt = counts(w);
    Inv {
        q_count: q_count(w),
        q_skeleton: q_order_signature(w),
        parity_lin_1: parity_linear_1(&cnt),
    }
}

fn neighbors_fwd(word: &str, rules: &Rules) -> Vec<String> {
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
    let mut v: Vec<String> = outs.into_iter().collect();
    v.sort_unstable();
    v.dedup();
    v
}

fn random_word(alphabet: &[char], min_len: usize, max_len: usize, rng: &mut impl Rng) -> String {
    let l = rng.gen_range(min_len..=max_len);
    let mut s = String::with_capacity(l);
    for _ in 0..l {
        s.push(*alphabet.choose(rng).unwrap());
    }
    s
}

fn random_chain_fwd(
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
        let neigh = neighbors_fwd(&w, rules);
        if neigh.is_empty() {
            break;
        }
        w = neigh.choose(rng).unwrap().clone();
        chain.push(w.clone());
    }
    chain
}

fn check_chain_preserves_invariants(chain: &[String]) -> bool {
    let base = invariants(&chain[0]);
    for w in &chain[1..] {
        if invariants(w) != base {
            return false;
        }
    }
    true
}

fn fuzz_test_forward(
    rules: &Rules,
    trials: usize,
    min_len: usize,
    max_len: usize,
    min_steps: usize,
    max_steps: usize,
    seed: u64,
    alphabet: &[char],
) -> (usize, usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut ok = 0usize;

    for _ in 0..trials {
        let w0 = random_word(alphabet, min_len, max_len, &mut rng);
        let chain = random_chain_fwd(&w0, rules, min_steps, max_steps, &mut rng);
        if check_chain_preserves_invariants(&chain) {
            ok += 1;
        }
    }
    (ok, trials - ok)
}

fn main() {
    let t = cfg::rules_t();
    let tprime = cfg::build_tprime();

    println!("Fuzz invariants (forward-only), unified config & StdRng");

    let (ok_t, fail_t) = fuzz_test_forward(
        &t,
        cfg::NUM_TRIALS,
        cfg::GEN_MIN_LEN,
        cfg::GEN_MAX_LEN,
        cfg::CHAIN_MIN_STEPS,
        cfg::CHAIN_MAX_STEPS,
        cfg::RNG_SEED,
        cfg::ALPHABET,
    );
    println!(
        "T:   {{ trials: {}, ok: {}, fail: {} }}",
        cfg::NUM_TRIALS,
        ok_t,
        fail_t
    );

    let (ok_tp, fail_tp) = fuzz_test_forward(
        &tprime,
        cfg::NUM_TRIALS,
        cfg::GEN_MIN_LEN,
        cfg::GEN_MAX_LEN,
        cfg::CHAIN_MIN_STEPS,
        cfg::CHAIN_MAX_STEPS,
        cfg::RNG_SEED,
        cfg::ALPHABET,
    );
    println!(
        "T′:  {{ trials: {}, ok: {}, fail: {} }}",
        cfg::NUM_TRIALS,
        ok_tp,
        fail_tp
    );
}
