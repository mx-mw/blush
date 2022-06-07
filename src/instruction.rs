/*
--------- no syntax trees? ---------
⠀⣞⢽⢪⢣⢣⢣⢫⡺⡵⣝⡮⣗⢷⢽⢽⢽⣮⡷⡽⣜⣜⢮⢺⣜⢷⢽⢝⡽⣝
⠸⡸⠜⠕⠕⠁⢁⢇⢏⢽⢺⣪⡳⡝⣎⣏⢯⢞⡿⣟⣷⣳⢯⡷⣽⢽⢯⣳⣫⠇
⠀⠀⢀⢀⢄⢬⢪⡪⡎⣆⡈⠚⠜⠕⠇⠗⠝⢕⢯⢫⣞⣯⣿⣻⡽⣏⢗⣗⠏⠀
⠀⠪⡪⡪⣪⢪⢺⢸⢢⢓⢆⢤⢀⠀⠀⠀⠀⠈⢊⢞⡾⣿⡯⣏⢮⠷⠁⠀⠀
⠀⠀⠀⠈⠊⠆⡃⠕⢕⢇⢇⢇⢇⢇⢏⢎⢎⢆⢄⠀⢑⣽⣿⢝⠲⠉⠀⠀⠀⠀
⠀⠀⠀⠀⠀⡿⠂⠠⠀⡇⢇⠕⢈⣀⠀⠁⠡⠣⡣⡫⣂⣿⠯⢪⠰⠂⠀⠀⠀⠀
⠀⠀⠀⠀⡦⡙⡂⢀⢤⢣⠣⡈⣾⡃⠠⠄⠀⡄⢱⣌⣶⢏⢊⠂⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⢝⡲⣜⡮⡏⢎⢌⢂⠙⠢⠐⢀⢘⢵⣽⣿⡿⠁⠁⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠨⣺⡺⡕⡕⡱⡑⡆⡕⡅⡕⡜⡼⢽⡻⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⣼⣳⣫⣾⣵⣗⡵⡱⡡⢣⢑⢕⢜⢕⡝⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⣴⣿⣾⣿⣿⣿⡿⡽⡑⢌⠪⡢⡣⣣⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⡟⡾⣿⢿⢿⢵⣽⣾⣼⣘⢸⢸⣞⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
⠀⠀⠀⠀⠁⠇⠡⠩⡫⢿⣝⡻⡮⣒⢽⠋
-----------------------------------
 */

/*
    R(x): value at register `x`
    IC  : instruction counter
    V(x): value of variable x
*/
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Const, // 0  CONST  [I] [L] [A] Load value at index I of length L into A
    Add,   // 1  ADD   [A] [B] [C]  R(C) = R(A) + R(B)
    Sub,   // 2  SUB   [A] [B] [C]  R(C) = R(A) - R(B)
    Mul,   // 3  MUL   [A] [B] [C]  R(C) = R(A) * R(B)
    Div,   // 4  DIV   [A] [B] [C]  R(C) = R(A) / R(B)
    Eq,    // 5  EQ    [A] [B]      if R(A) == R(B) then IC++
	Ne,    // 6  NE    [A] [B]      if R(A) != R(B) then IC++
    Lt,    // 7  LT    [A] [B]      if R(A) <  R(B) then IC++
    Le,    // 8  LE    [A] [B]      if R(A) <= R(B) then IC++
    Not,   // 9  NOT   [A] [B]      R(B) = !R(A)
    Neg,   // 10 NEG   [A] [B]      R(B) = -R(A)
	Let,   // 11 LET   [I] [A]      V(I) = R(A)
    Read,  // 12 READ  [I] [A]      R(A) = V(R(I))
    Set,   // 13 SET   [I] [A]      V(I) = R(A)
}
