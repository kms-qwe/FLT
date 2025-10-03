use std::collections::{HashMap, HashSet};

type Rule = (String, String);
type Rules = Vec<Rule>;

#[derive(Clone)]
struct Rng64 { state: u64 }
impl Rng64 {
    fn new(seed: u64) -> Self { Self { state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed } } }
    fn next_u64(&mut self) -> u64 { let mut x = self.state; x ^= x >> 12; x ^= x << 25; x ^= x >> 27; self.state = x; x.wrapping_mul(2685821657736338717) }
    fn gen_range_usize(&mut self, lo: usize, hi: usize) -> usize { if lo == hi { lo } else { lo + (self.next_u64() as usize % (hi - lo + 1)) } }
    fn choose_idx(&mut self, len: usize) -> usize { self.gen_range_usize(0, len - 1) }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Inv { q_count: usize, q_skeleton: String, parity_lin_1: u8 }
impl Inv {
    fn to_debug_map(&self) -> String {
        format!("{{'Q_count': {}, 'Q_skeleton': '{}', 'parity_lin_1': {}}}", self.q_count, self.q_skeleton, self.parity_lin_1)
    }
}

fn collect_alphabet(systems: &[&Rules]) -> Vec<char> {
    let mut set: HashSet<char> = HashSet::new();
    for rules in systems {
        for (a, b) in rules.iter() {
            for c in a.chars() { set.insert(c); }
            for c in b.chars() { set.insert(c); }
        }
    }
    let mut v: Vec<char> = set.into_iter().collect();
    v.sort_unstable();
    v
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
            } else { break; }
        }
    }
    outs.remove(word);
    outs.into_iter().collect()
}

fn random_word(alphabet: &[char], min_len: usize, max_len: usize, rng: &mut Rng64) -> String {
    let l = rng.gen_range_usize(min_len, max_len);
    let mut s = String::with_capacity(l);
    for _ in 0..l {
        let idx = rng.choose_idx(alphabet.len());
        s.push(alphabet[idx]);
    }
    s
}

fn random_chain_fwd(word: &str, rules: &Rules, min_steps: usize, max_steps: usize, rng: &mut Rng64) -> Vec<String> {
    let mut w = word.to_string();
    let mut chain = vec![w.clone()];
    let steps = rng.gen_range_usize(min_steps, max_steps);
    for _ in 0..steps {
        let neigh = neighbors_fwd(&w, rules);
        if neigh.is_empty() { break; }
        let idx = rng.choose_idx(neigh.len());
        w = neigh[idx].clone();
        chain.push(w.clone());
    }
    chain
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

fn q_count(w: &str) -> usize { w.bytes().filter(|&b| b == b'Q').count() }

fn q_order_signature(w: &str) -> String { w.chars().filter(|&ch| ch == 'Q').collect() }

fn invariants(w: &str) -> Inv {
    let cnt = counts(w);
    Inv {
        q_count: q_count(w),
        q_skeleton: q_order_signature(w),
        parity_lin_1: parity_linear_1(&cnt),
    }
}

fn unit_check_rules_forward(rules: &Rules) -> (bool, Vec<(String, String, Inv, Inv)>) {
    let mut bad = Vec::new();
    for (lhs, rhs) in rules.iter() {
        let il = invariants(lhs);
        let ir = invariants(rhs);
        if il != ir {
            bad.push((lhs.clone(), rhs.clone(), il, ir));
        }
    }
    (bad.is_empty(), bad)
}

fn check_chain(chain: &[String]) -> (bool, Inv, Inv, Option<String>) {
    let base = invariants(&chain[0]);
    for w in &chain[1..] {
        let inv = invariants(w);
        if inv != base {
            return (false, base, inv, Some(w.clone()));
        }
    }
    (true, base.clone(), base, None)
}

fn fuzz_test_forward(rules: &Rules, trials: usize, min_len: usize, max_len: usize, min_steps: usize, max_steps: usize, show: usize, seed: u64) -> (HashMap<&'static str, usize>, Vec<HashMap<&'static str, String>>) {
    let mut rng = Rng64::new(seed);
    let alphabet = {
        let mut t = Vec::new();
        for (lhs, rhs) in rules.iter() {
            t.push((lhs.clone(), rhs.clone()));
        }
        collect_alphabet(&[&t])
    };
    let mut ok = 0usize;
    let mut report: Vec<(String, Vec<String>, Inv, Inv, Option<String>)> = Vec::new();
    for _ in 0..trials {
        let w0 = random_word(&alphabet, min_len, max_len, &mut rng);
        let chain = random_chain_fwd(&w0, rules, min_steps, max_steps, &mut rng);
        let (good, base, bad_inv, at) = check_chain(&chain);
        if good { ok += 1; } else { report.push((w0, chain, base, bad_inv, at)); }
    }
    let mut summary = HashMap::new();
    summary.insert("trials", trials);
    summary.insert("ok", ok);
    summary.insert("fail", trials - ok);
    let mut examples = Vec::new();
    for (w0, chain, base, bad_inv, at) in report.into_iter().take(show) {
        let mut m = HashMap::new();
        m.insert("w0", w0);
        m.insert("violation_at", at.unwrap_or_else(|| "".into()));
        let prefix = {
            let k = chain.len().min(6);
            let mut parts = Vec::new();
            for s in chain.into_iter().take(k) {
                parts.push(s);
            }
            parts.join(" => ")
        };
        m.insert("chain_prefix", prefix);
        m.insert("base_invariants", base.to_debug_map());
        m.insert("bad_invariants", bad_inv.to_debug_map());
        examples.push(m);
    }
    (summary, examples)
}

fn main() {
    let t: Rules = vec![
        ("JFF".into(), "EID".into()),
        ("CFG".into(), "IC".into()),
        ("GF".into(), "GG".into()),
        ("IC".into(), "JI".into()),
        ("AD".into(), "BD".into()),
        ("FFI".into(), "HGH".into()),
        ("FHDQ".into(), "FHEDQ".into()),
        ("HA".into(), "JH".into()),
        ("HE".into(), "HDD".into()),
        ("GGA".into(), "HCE".into()),
        ("IBB".into(), "DDI".into()),
    ];
    let t_prime: Rules = vec![
        ("JFF".into(), "EID".into()),
        ("CFG".into(), "IC".into()),
        ("GG".into(), "GF".into()),
        ("JI".into(), "IC".into()),
        ("BD".into(), "AD".into()),
        ("HGH".into(), "FFI".into()),
        ("FHEDQ".into(), "FHDQ".into()),
        ("JH".into(), "HA".into()),
        ("HDD".into(), "HE".into()),
        ("HCE".into(), "GGA".into()),
        ("IBB".into(), "DDI".into()),
    ];

    let (ok_t, bad_t) = unit_check_rules_forward(&t);
    let (ok_tp, bad_tp) = unit_check_rules_forward(&t_prime);

    println!("Unit check (lhs -> rhs keep invariants)");
    println!("T:   {}", if ok_t { "OK" } else { "FAIL" });
    if !ok_t {
        for (l, r, il, ir) in bad_t.iter().take(10) {
            println!("  Rule FAIL: {} -> {} | inv(lhs)= {} | inv(rhs)= {}", l, r, il.to_debug_map(), ir.to_debug_map());
        }
    }
    println!("T′:  {}", if ok_tp { "OK" } else { "FAIL" });
    if !ok_tp {
        for (l, r, il, ir) in bad_tp.iter().take(10) {
            println!("  Rule FAIL: {} -> {} | inv(lhs)= {} | inv(rhs)= {}", l, r, il.to_debug_map(), ir.to_debug_map());
        }
    }

    println!("\nFuzz on forward-only random chains");
    let (sum_t, ex_t) = fuzz_test_forward(&t, 3000, 10, 30, 1, 12, 5, 12345);
    println!("T: {:?}", sum_t);
    let (sum_tp, ex_tp) = fuzz_test_forward(&t_prime, 3000, 10, 30, 1, 12, 5, 54321);
    println!("T′: {:?}", sum_tp);

    if !ex_t.is_empty() || !ex_tp.is_empty() {
        println!("\nSample violations (up to 5 per system):");
        for ex in ex_t {
            println!("T: {:?}", ex);
        }
        for ex in ex_tp {
            println!("T′: {:?}", ex);
        }
    }
}
