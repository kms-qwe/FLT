use rand::{seq::SliceRandom, Rng, SeedableRng};
use std::collections::{HashMap, HashSet, VecDeque};

type Rule = (String, String);
type Rules = Vec<Rule>;

fn kb_completion_rules(max_n: usize) -> Rules {
    let mut out = Vec::new();
    for n in 1..=max_n {
        let lhs = format!("G{}G", "F".repeat(n));
        let rhs = format!("G{}", "F".repeat(n + 1));
        out.push((lhs, rhs));
    }
    out
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

fn collect_alphabet(systems: &[&Rules]) -> Vec<char> {
    let mut set: HashSet<char> = HashSet::new();
    for rules in systems {
        for (lhs, rhs) in rules.iter() {
            for c in lhs.chars() {
                set.insert(c);
            }
            for c in rhs.chars() {
                set.insert(c);
            }
        }
    }
    let mut v: Vec<char> = set.into_iter().collect();
    v.sort_unstable();
    v
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
        let ch = alphabet.choose(rng).unwrap();
        s.push(*ch);
    }
    s
}

fn random_chain(word: &str, rules: &Rules, min_steps: usize, max_steps: usize, rng: &mut impl Rng) -> Vec<String> {
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

fn reachable_unidirectional(src: &str, dst: &str, rules: &Rules, max_depth: usize, max_nodes: usize) -> (bool, Vec<String>) {
    if src == dst {
        return (true, vec![src.to_string()]);
    }
    let mut q: VecDeque<(String, usize)> = VecDeque::new();
    let mut parents: HashMap<String, Option<String>> = HashMap::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut expanded = 0usize;
    q.push_back((src.to_string(), 0));
    parents.insert(src.to_string(), None);
    visited.insert(src.to_string());
    while let Some((u, d)) = q.pop_front() {
        if expanded >= max_nodes {
            break;
        }
        if d >= max_depth {
            continue;
        }
        let neigh = neighbors_fwd(&u, rules);
        for v in neigh {
            if expanded >= max_nodes {
                break;
            }
            expanded += 1;
            if visited.contains(&v) {
                continue;
            }
            visited.insert(v.clone());
            parents.insert(v.clone(), Some(u.clone()));
            if v == dst {
                let mut path = Vec::new();
                let mut x = Some(v.clone());
                while let Some(cur) = x {
                    path.push(cur.clone());
                    x = parents.get(&cur).cloned().unwrap_or(None);
                }
                path.reverse();
                return (true, path);
            }
            q.push_back((v, d + 1));
        }
    }
    (false, Vec::new())
}

fn fuzz_forward_only(
    t: &Rules,
    tprime: &Rules,
    num_trials: usize,
    gen_min_len: usize,
    gen_max_len: usize,
    chain_min_steps: usize,
    chain_max_steps: usize,
    bfs_max_depth: usize,
    bfs_max_nodes: usize,
    seed: u64,
) -> (usize, usize, usize) {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let t = dedup_rules(t);
    let tprime = dedup_rules(tprime);
    let alphabet = collect_alphabet(&[&t, &tprime]);
    let mut ok = 0usize;
    let mut fail = 0usize;
    for _ in 0..num_trials {
        let w0 = rand_word(&alphabet, gen_min_len, gen_max_len, &mut rng);
        let chain = random_chain(&w0, &t, chain_min_steps, chain_max_steps, &mut rng);
        let w1 = chain.last().unwrap().clone();
        let (hit, _) = reachable_unidirectional(&w0, &w1, &tprime, bfs_max_depth, bfs_max_nodes);
        if hit { ok += 1; } else { fail += 1; }
    }
    (ok, fail, num_trials)
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
    let tprime_base: Rules = vec![
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
    let kb_max_n = Some(12usize);
    let tprime_extra = kb_max_n.map(kb_completion_rules).unwrap_or_default();
    let mut tprime = tprime_base.clone();
    tprime.extend(tprime_extra);
    let (ok, fail, trials) = fuzz_forward_only(&t, &tprime, 100, 1, 5, 1, 5, 10_000, 6_000_000, 7);
    println!("T ⊆ → T': {}/{} ok, {} fail", ok, trials, fail);
}
