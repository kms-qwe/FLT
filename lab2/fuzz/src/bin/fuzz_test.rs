use fancy_regex::Regex as FancyRegex;
use rand::Rng;
use regex::Regex;

// Регулярки

const REGEX_PATTERN: &str = r"^((aa|bb)*(ab|ba)(aa|bb)*(ab|ba))*(ab|(bc|cb)(bb)*(cb|bc))*$";

const EXTENDED_REGEX_PATTERN: &str = r"(?x)
^(
  (
    ((aa|bb)+)?
    ((?=(ab|ba))..)
    ((aa|bb)+)?
    ((?=(ab|ba))..)
  )+
)?(
  (
    ((?=ab)..)
    |
    ( ((?=(bc|cb))..) ((bb)+)? ((?=(bc|cb))..) )
  )+
)?$
";

fn regex_accepts(re: &Regex, s: &str) -> bool {
    re.is_match(s)
}

fn extended_regex_accepts(re: &FancyRegex, s: &str) -> bool {
    re.is_match(s).expect("fancy-regex execution error")
}

// НКА

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum NfaState {
    Start,
    AIn,
    AInAa,
    AInAb,
    AInBa,
    AInBb,
    AMid2,
    AMid2Ab,
    AMid2Ba,
    AEndOrBStart,
    BMid,
    BInAb,
    BInBc,
    BInCb,
    BMidBc,
}

#[inline]
fn nfa_bit(s: NfaState) -> u32 {
    1 << (s as u32)
}

#[inline]
fn nfa_has(states: u32, s: NfaState) -> bool {
    states & nfa_bit(s) != 0
}

#[inline]
fn nfa_add(states: &mut u32, s: NfaState) {
    *states |= nfa_bit(s);
}

fn nfa_epsilon_closure(mut states: u32) -> u32 {
    if nfa_has(states, NfaState::Start) {
        nfa_add(&mut states, NfaState::AIn);
        nfa_add(&mut states, NfaState::AEndOrBStart);
    }
    states
}

fn nfa_step(states: u32, ch: char) -> u32 {
    use NfaState::*;
    let mut next: u32 = 0;

    if nfa_has(states, AIn) {
        match ch {
            'a' => {
                nfa_add(&mut next, AInAa);
                nfa_add(&mut next, AInAb);
            }
            'b' => {
                nfa_add(&mut next, AInBb);
                nfa_add(&mut next, AInBa);
            }
            _ => {}
        }
    }

    if nfa_has(states, AInAa) {
        if ch == 'a' {
            nfa_add(&mut next, AIn);
        }
    }

    if nfa_has(states, AInBb) {
        if ch == 'b' {
            nfa_add(&mut next, AIn);
        }
    }

    if nfa_has(states, AInAb) {
        if ch == 'b' {
            nfa_add(&mut next, AMid2);
        }
    }

    if nfa_has(states, AInBa) {
        if ch == 'a' {
            nfa_add(&mut next, AMid2);
        }
    }

    if nfa_has(states, AMid2) {
        match ch {
            'a' => {
                nfa_add(&mut next, AMid2Ab);
                nfa_add(&mut next, AInBa);
            }
            'b' => {
                nfa_add(&mut next, AInAb);
                nfa_add(&mut next, AMid2Ba);
            }
            _ => {}
        }
    }

    if nfa_has(states, AMid2Ab) {
        if ch == 'b' {
            nfa_add(&mut next, Start);
        }
    }

    if nfa_has(states, AMid2Ba) {
        if ch == 'a' {
            nfa_add(&mut next, Start);
        }
    }

    if nfa_has(states, AEndOrBStart) {
        match ch {
            'a' => nfa_add(&mut next, BInAb),
            'b' => nfa_add(&mut next, BInBc),
            'c' => nfa_add(&mut next, BInCb),
            _ => {}
        }
    }

    if nfa_has(states, BInAb) {
        if ch == 'b' {
            nfa_add(&mut next, AEndOrBStart);
        }
    }

    if nfa_has(states, BInBc) {
        if ch == 'c' {
            nfa_add(&mut next, BMid);
        }
    }

    if nfa_has(states, BInCb) {
        if ch == 'b' {
            nfa_add(&mut next, BMid);
        }
    }

    if nfa_has(states, BMid) {
        match ch {
            'b' => {
                nfa_add(&mut next, BInCb);
                nfa_add(&mut next, BMidBc);
            }
            'c' => {
                nfa_add(&mut next, BInAb);
            }
            _ => {}
        }
    }

    if nfa_has(states, BMidBc) {
        if ch == 'c' {
            nfa_add(&mut next, AEndOrBStart);
        }
    }

    next
}

fn nfa_accepts(s: &str) -> bool {
    use NfaState::*;

    let mut states: u32 = 0;
    nfa_add(&mut states, Start);
    states = nfa_epsilon_closure(states);

    for ch in s.chars() {
        states = nfa_step(states, ch);
        if states == 0 {
            break;
        }
        states = nfa_epsilon_closure(states);
    }

    nfa_has(states, AEndOrBStart)
}

// ДКА

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DfaState {
    Q1,
    Q2,
    Q3,
    Q4,
    Q5,
    Q6,
    Q7,
    Q8,
    Q9,
    Q10,
    Q11,
    Q12,
    Q13,
    Q14,
    Q15,
    Q16,
    Q17,
    Dead,
}

fn dfa_step(state: DfaState, ch: char) -> DfaState {
    use DfaState::*;
    match (state, ch) {
        (Q1, 'a') => Q5,
        (Q2, 'a') => Q3,
        (Q5, 'a') => Q8,
        (Q6, 'a') => Q10,
        (Q8, 'a') => Q12,
        (Q9, 'a') => Q14,
        (Q10, 'a') => Q14,
        (Q12, 'a') => Q8,
        (Q13, 'a') => Q10,
        (Q14, 'a') => Q10,
        (Q15, 'a') => Q1,
        (Q16, 'a') => Q1,

        (Q1, 'b') => Q6,
        (Q2, 'b') => Q4,
        (Q3, 'b') => Q2,
        (Q5, 'b') => Q9,
        (Q6, 'b') => Q8,
        (Q7, 'b') => Q11,
        (Q8, 'b') => Q13,
        (Q9, 'b') => Q15,
        (Q10, 'b') => Q16,
        (Q11, 'b') => Q17,
        (Q12, 'b') => Q10,
        (Q13, 'b') => Q8,
        (Q14, 'b') => Q1,
        (Q15, 'b') => Q10,
        (Q16, 'b') => Q10,
        (Q17, 'b') => Q11,

        (Q1, 'c') => Q7,
        (Q2, 'c') => Q7,
        (Q4, 'c') => Q11,
        (Q6, 'c') => Q11,
        (Q9, 'c') => Q7,
        (Q11, 'c') => Q3,
        (Q15, 'c') => Q11,
        (Q17, 'c') => Q2,

        _ => Dead,
    }
}

fn dfa_accepts(s: &str) -> bool {
    use DfaState::*;
    let mut state = Q1;
    for ch in s.chars() {
        state = dfa_step(state, ch);
    }
    matches!(state, Q1 | Q2 | Q9)
}

// ДКА: чётная длина слова

#[derive(Copy, Clone)]
enum ELenState {
    EEven,
    EOdd,
}

fn dfa_even_length_accepts(s: &str) -> bool {
    use ELenState::*;
    let mut st = EEven;

    for ch in s.chars() {
        match st {
            EEven => match ch {
                'a' | 'b' | 'c' => st = EOdd,
                _ => {}
            },
            EOdd => match ch {
                'a' | 'b' | 'c' => st = EEven,
                _ => {}
            },
        }
    }

    matches!(st, EEven)
}

// ДКА: чётное число букв 'c'

#[derive(Copy, Clone)]
enum CCountState {
    CEven,
    COdd,
}

fn dfa_even_c_count_accepts(s: &str) -> bool {
    use CCountState::*;
    let mut st = CEven;

    for ch in s.chars() {
        st = match (st, ch) {
            (CEven, 'c') => COdd,
            (COdd, 'c') => CEven,
            (CEven, 'a' | 'b') => CEven,
            (COdd, 'a' | 'b') => COdd,
            (other, _) => other,
        };
    }

    matches!(st, CEven)
}

// ДКА: непустое слово содержит хотя бы одну 'b'

#[derive(Copy, Clone)]
enum BState {
    B0,
    B1,
    B2,
}

fn dfa_contains_b_accepts(s: &str) -> bool {
    use BState::*;
    let mut st = B0;

    for ch in s.chars() {
        st = match (st, ch) {
            (B0, 'b') => B2,
            (B0, 'a' | 'c') => B1,

            (B1, 'a' | 'c') => B1,
            (B1, 'b') => B2,

            (B2, 'a' | 'b' | 'c') => B2,
            (other, _) => other,
        };
    }

    matches!(st, B0 | B2)
}

// ДКА: слово не оканчивается на суффикс bb

#[derive(Copy, Clone)]
enum BBState {
    BB0,
    BB1,
    BB2,
}

fn dfa_no_bb_suffix_accepts(s: &str) -> bool {
    use BBState::*;
    let mut st = BB0;

    for ch in s.chars() {
        st = match (st, ch) {
            (BB0, 'b') => BB1,
            (BB0, 'a' | 'c') => BB0,

            (BB1, 'b') => BB2,
            (BB1, 'a' | 'c') => BB0,

            (BB2, 'b') => BB2,
            (BB2, 'a' | 'c') => BB0,

            (other, _) => other,
        };
    }

    matches!(st, BB0 | BB1)
}

// ДКА: нет подслова 'ccc'

#[derive(Copy, Clone)]
enum TState {
    T0,
    T1,
    T2,
    T3,
}

fn dfa_no_ccc_accepts(s: &str) -> bool {
    use TState::*;
    let mut st = T0;

    for ch in s.chars() {
        st = match (st, ch) {
            (T0, 'c') => T1,
            (T0, 'a' | 'b') => T0,

            (T1, 'c') => T2,
            (T1, 'a' | 'b') => T0,

            (T2, 'c') => T3,
            (T2, 'a' | 'b') => T0,

            (T3, 'a' | 'b' | 'c') => T3,

            (other, _) => other,
        };
    }

    matches!(st, T0 | T1 | T2)
}

// ДКА: после первой 'c' любая 'a' должна идти только как 'ab'

#[derive(Copy, Clone)]
enum ACState {
    AC0,
    AC1,
    ACWait,
    ACDead,
}

fn dfa_ac_constraint_accepts(s: &str) -> bool {
    use ACState::*;
    let mut st = AC0;

    for ch in s.chars() {
        st = match (st, ch) {
            (AC0, 'a' | 'b') => AC0,
            (AC0, 'c') => AC1,

            (AC1, 'b' | 'c') => AC1,
            (AC1, 'a') => ACWait,

            (ACWait, 'b') => AC1,
            (ACWait, 'a' | 'c') => ACDead,

            (ACDead, 'a' | 'b' | 'c') => ACDead,

            (other, _) => other,
        };
    }

    matches!(st, AC0 | AC1)
}

// ПКА

fn pka_accepts(s: &str) -> bool {
    let dfa_ok = dfa_accepts(s);

    let even_len_ok = dfa_even_length_accepts(s);
    let even_c_ok = dfa_even_c_count_accepts(s);
    let contains_b_ok = dfa_contains_b_accepts(s);
    let no_bb_suffix_ok = dfa_no_bb_suffix_accepts(s);
    let no_ccc_ok = dfa_no_ccc_accepts(s);
    let ac_ok = dfa_ac_constraint_accepts(s);

    dfa_ok && even_len_ok && even_c_ok && contains_b_ok && no_bb_suffix_ok && no_ccc_ok && ac_ok
}

// Хелперы

fn all_equal(values: &[bool]) -> bool {
    if let Some((&first, rest)) = values.split_first() {
        rest.iter().all(|&v| v == first)
    } else {
        true
    }
}

fn random_word<R: Rng + ?Sized>(rng: &mut R, max_len: usize) -> String {
    let len = rng.gen_range(0..=max_len);
    let alphabet = ['a', 'b', 'c'];
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        let idx = rng.gen_range(0..alphabet.len());
        s.push(alphabet[idx]);
    }
    s
}

// Запуск

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iterations: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);
    let max_len: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);

    println!(
        "Fuzz: {} случайных слов над {{a,b,c}}, max_len = {}",
        iterations, max_len
    );

    let reg = Regex::new(REGEX_PATTERN).expect("failed to compile regex");
    let extended_reg =
        FancyRegex::new(EXTENDED_REGEX_PATTERN).expect("failed to compile extended regex");

    let mut rng = rand::thread_rng();

    let mut positive: u64 = 0;
    let mut negative: u64 = 0;

    for i in 0..iterations {
        let word = random_word(&mut rng, max_len);

        let reg_res = regex_accepts(&reg, &word);
        let extended_reg_res = extended_regex_accepts(&extended_reg, &word);
        let nfa_res = nfa_accepts(&word);
        let dfa_res = dfa_accepts(&word);
        let pka_res = pka_accepts(&word);

        if !all_equal(&[reg_res, extended_reg_res, nfa_res, dfa_res, pka_res]) {
            println!("=== НАЙДЕНО РАСХОЖДЕНИЕ после {} итераций ===", i + 1);
            println!("слово = {:?}", word);
            println!("1) reg_res: {}", reg_res);
            println!("2) extended_reg_res: {}", extended_reg_res);
            println!("3) NFA   : {}", nfa_res);
            println!("4) DFA   : {}", dfa_res);
            println!("5) PKA   : {}", pka_res);
            std::process::exit(1);
        }

        if reg_res {
            positive += 1;
        } else {
            negative += 1;
        }

        if (i + 1) % 10_000 == 0 {
            println!("... выполнено {} итераций", i + 1);
        }
    }

    println!(
        "OK: все {} тестов прошли, все пять определений согласованы",
        iterations
    );
    println!(
        "Итог: принятых слов = {}, отклонённых слов = {}",
        positive, negative
    );
}
