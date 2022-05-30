/*
	V(x): value at register `x`
	IC  : instruction counter
*/
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
	Const,        // CONST [I] [L]  I: Index of the first byte L: The number of bytes taken
	Load,         // LOAD  [I] [L] [A]  Load value at index I of length L into A
	Add,          // ADD   [A] [B] [C]  C = V(A) + V(B)
	Sub,		  // SUB   [A] [B] [C]  C = V(A) - V(B)
	Mul,          // MUL   [A] [B] [C]  C = V(A) * V(B)
	Div,          // DIV   [A] [B] [C]  C = V(A) / V(B)
	Eq,           // EQ    [A] [B]      if V(A) == V(B) then IC++
	Lt,           // LT    [A] [B]      if V(A) < V(B) then IC++
	Le,           // LE    [A] [B]      if V(A) <= V(B) then IC++
	Not,          // NOT   [A] [B]      B = !V(A)
	Neg,          // NEG   [A] [B]      B = -V(A)
}