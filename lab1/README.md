# Анализ системы переписываний **T**

---

\[
T = \left\{
\begin{aligned}
&\mathtt{JFF}   \to \mathtt{EID}\\
&\mathtt{CFG}   \to \mathtt{IC}\\
&\mathtt{GF}    \to \mathtt{GG}\\
&\mathtt{IC}    \to \mathtt{JI}\\
&\mathtt{AD}    \to \mathtt{BD}\\
&\mathtt{FFI}   \to \mathtt{HGH}\\
&\mathtt{FHDQ}  \to \mathtt{FHEDQ}\\
&\mathtt{HA}    \to \mathtt{JH}\\
&\mathtt{HE}    \to \mathtt{HDD}\\
&\mathtt{GGA}   \to \mathtt{HCE}\\
&\mathtt{IBB}   \to \mathtt{DDI}
\end{aligned}
\right.
\]

Алфавит:
\(\Sigma = \{A,B,C,D,E,F,G,H,I,J,Q\}\)

---

## 1. Доказательство **завершаемости**

Введём право-лексикографический порядок \(\succ_{\mathrm{rlex}}\) над алфавитом \(\Sigma\) по шкале:
\[
A \succ F \succ G \succ C \succ B \succ I \succ H \succ E \succ D \succ J \succ Q.
\]
Слова сравниваются справа налево, при первом различии берётся сравнение по шкале, при равенстве суффиксов больше то, что длиннее.
Этот совместим с контекстами: \(u \succ_{\mathrm{rlex}} v \Rightarrow L,u,R \succ_{\mathrm{rlex}} L,v,R\).

**Анализ правил:**

\[
\begin{aligned}
&\mathtt{JFF} \succ \mathtt{EID} &&(F \succ D)\\
&\mathtt{CFG} \succ \mathtt{IC}  &&(G \succ C)\\
&\mathtt{GF}  \succ \mathtt{GG}  &&(F \succ G)\\
&\mathtt{IC}  \succ \mathtt{JI}  &&(C \succ I)\\
&\mathtt{AD}  \succ \mathtt{BD}  &&(D{=}D,\ A \succ B)\\
&\mathtt{FFI} \succ \mathtt{HGH} &&(I \succ H)\\
&\mathtt{FHDQ}\succ \mathtt{FHEDQ}&&(Q{=}Q,\ D{=}D,\ H \succ E)\\
&\mathtt{HA}  \succ \mathtt{JH}  &&(A \succ H)\\
&\mathtt{HE}  \succ \mathtt{HDD} &&(E \succ D)\\
&\mathtt{GGA} \succ \mathtt{HCE} &&(A \succ E)\\
&\mathtt{IBB} \succ \mathtt{DDI} &&(B \succ I)
\end{aligned}
\]

Каждое применение правила строго уменьшает слово в \(\succ_{\mathrm{rlex}}\).
Следовательно, бесконечных цепочек переписывания нет.

\[
\boxed{\text{Система T завершима.}}
\]

---

## 2. Доказательство **бесконечности множества нормальных форм**

Нормальная форма — слово, не содержащее ни одной левой части правил как подстроки.

Запрещённые подстроки:
`JFF`, `CFG`, `GF`, `IC`, `AD`, `FFI`, `FHDQ`, `HA`, `HE`, `GGA`, `IBB`.

Рассмотрим слова вида:
\[
C^n = \underbrace{CCCC\ldots C}_{n\text{ раз}}, \quad n \ge 1.
\]

В них отсутствуют все запрещённые шаблоны.

Следовательно, каждое \(C^n\) — нормальная форма, и таких слов бесконечно много.

\[
\boxed{\text{Множество нормальных форм бесконечно.}}
\]

---

## 3. Проверка **локальной конфлюэнтности**

Рассмотрим пересекающиеся редексы (критические пары).

**Пример:**
\[
l_1 = GF \to GG, \quad l_2 = FFI \to HGH.
\]
Возможное перекрытие: слово `GFFI`.

\[
GFFI \xrightarrow{GF} GGFI,\qquad GFFI \xrightarrow{FFI} GHGH.
\]

- `GGFI` не содержит ни одного редекса (нормальная форма).
- `GHGH` тоже нормальная форма.
Обе редукции заканчиваются различными словами:
\[
GGFI \ne GHGH.
\]

Критическая пара не замыкается.

\[
\boxed{\text{T не локально конфлюэнтна (и, следовательно, не конфлюэнтна).}}
\]

---

## 4. Проверка на пополняемость по Кнуту–Бендиксу

Введём порядок: shortlex — сперва по длине, затем по алфавиту
(A < B < C < D < E < F < G < H < I < J < Q).

Ориентируем так, чтобы \(l \succ r\) в этом порядке.

Получаем систему T′:

\[
T' = \left\{
\begin{aligned}
&\mathtt{JFF}   \to \mathtt{EID}\\
&\mathtt{CFG}   \to \mathtt{IC}\\
&\mathtt{GG}    \to \mathtt{GF}\\
&\mathtt{JI}    \to \mathtt{IC}\\
&\mathtt{BD}    \to \mathtt{AD}\\
&\mathtt{HGH}   \to \mathtt{FFI}\\
&\mathtt{FHEDQ} \to \mathtt{FHDQ}\\
&\mathtt{JH}    \to \mathtt{HA}\\
&\mathtt{HDD}   \to \mathtt{HE}\\
&\mathtt{HCE}   \to \mathtt{GGA}\\
&\mathtt{IBB}   \to \mathtt{DDI}
\end{aligned}
\right.
\]

Рассмотрим критические пары.

Уже правило `GG → GF` создаёт бесконечную семью перекрытий:
\[
GGG \Rightarrow GFG \text{ и } GGF \Rightarrow GFF,
\]
которые требуют добавки `GFG → GFF`.

Для `GGGGG` потребуется `GFFG → GFFF`,
для `GGGGGGG` — `GFFFG → GFFFF`, и так далее.

Таким образом, процедура Кнута–Бендикса будет порождать
бесконечную цепочку правил
\[
\boxed{GF^{\,n}G \to GF^{\,n+1}}, \quad n \ge 1,
\]
и никогда не завершится.

\[
\boxed{\text{Система T' не пополняема по алгоритму Кнута–Бендикса.}}
\]

---

## 5. Инварианты


1. Количество `Q` неизменно

2. Относительный порядок `Q` сохраняется

3. Паритетный линейный инвариант №1
   `(#A + #B + #C + #D + #J) mod 2` не меняется (`#` - количество вхождений буквы)
