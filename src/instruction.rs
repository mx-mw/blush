/*
    V(x): value at register `x`
    IC  : instruction counter
*/
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Const, // 0  CONST [I] [L]  I: Index of the first byte L: The number of bytes taken
    Load,  // 1  LOAD  [I] [L] [A]  Load value at index I of length L into A
    Add,   // 2  ADD   [A] [B] [C]  C = V(A) + V(B)
    Sub,   // 3  SUB   [A] [B] [C]  C = V(A) - V(B)
    Mul,   // 4  MUL   [A] [B] [C]  C = V(A) * V(B)
    Div,   // 5  DIV   [A] [B] [C]  C = V(A) / V(B)
    Eq,    // 6  EQ    [A] [B]      if V(A) == V(B) then IC++
    Lt,    // 7  LT    [A] [B]      if V(A) < V(B) then IC++
    Le,    // 8  LE    [A] [B]      if V(A) <= V(B) then IC++
    Not,   // 9  NOT   [A] [B]      B = !V(A)
    Neg,   // 10 NEG   [A] [B]      B = -V(A)
}
