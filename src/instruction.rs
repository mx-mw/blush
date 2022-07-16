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
	# Grammar
    R(x) : value at register `x`
    IC   : instruction counter (starting at 0)
    V(x) : Variable x
	Vv(x): Value of V(x)
	L(x) : Local at index x

	# Boolean Operations (Eq, Lt, etc.)
    The instruction for the comparison is followed by a move instruction for the false case.
    Ex:
    1 Lt 0 1    If R(0) < R(1), IC becomes 5 (current position + 2 + increment in main loop).
                Else, IC is incremented in the main VM loop
    2 Move 9    Move to 9 if the expression is false
    3 Add 0 2 0 Add R(2) to R(0) if the expression is true
    4 ...       rest of program
*/
#[allow(unused)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    Const, // 0  CONST I L A  Load value at index I of length L into A
    Add,   // 1  ADD   A B C  R(C) = R(A) + R(B)
    Sub,   // 2  SUB   A B C  R(C) = R(A) - R(B)
    Mul,   // 3  MUL   A B C  R(C) = R(A) * R(B)
    Div,   // 4  DIV   A B C  R(C) = R(A) / R(B)
    Eq,    // 5  EQ    A B    if R(A) == R(B) then IC+=2
    Ne,    // 6  NE    A B    if R(A) != R(B) then IC+=2
    Lt,    // 7  LT    A B    if R(A) <  R(B) then IC+=2
    Le,    // 8  LE    A B    if R(A) <= R(B) then IC+=2
    Not,   // 9  NOT   A B    R(B) = !R(A)
    Neg,   // 10 NEG   A B    R(B) = -R(A)
    Let,   // 11 LET   L A    Vv(L) = R(A)
    Read,  // 12 READ  I A    R(A) = V(R(I))
    Set,   // 13 SET   I A    V(I) = R(A)
    Move,  // 14 Move  T      IC = T
}
