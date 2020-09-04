#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
	Sigma,
	Recip,
	Sqrt,
	Log,
	Ln,
	Xeq,
	Sto,
	Rcl,
	RotateDown,
	Sin,
	Cos,
	Tan,
	Enter,
	Swap,
	Neg,
	E,
	Backspace,
	Up,
	Seven,
	Eight,
	Nine,
	Div,
	Down,
	Four,
	Five,
	Six,
	Mul,
	Shift,
	One,
	Two,
	Three,
	Sub,
	Exit,
	Zero,
	Dot,
	Run,
	Add,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	Screenshot,
	ShiftUp,
	ShiftDown,
	DoubleRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvent {
	Press(Key),
	Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
	Normal,
	UpperAlpha,
	LowerAlpha,
}

pub struct InputMode {
	pub shift: bool,
	pub alpha: AlphaMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
	ModeChange,
	Character(char),
	FunctionKey(u8, bool),
	SigmaPlus,
	SigmaMinus,
	Recip,
	Pow,
	Sqrt,
	Square,
	Log,
	TenX,
	Ln,
	EX,
	Xeq,
	Gto,
	Sto,
	Complex,
	Rcl,
	Percent,
	RotateDown,
	Pi,
	Sin,
	Asin,
	Cos,
	Acos,
	Tan,
	Atan,
	Enter,
	Swap,
	LastX,
	Neg,
	Modes,
	E,
	Disp,
	Backspace,
	Clear,
	Up,
	Bst,
	Solver,
	Integrate,
	Matrix,
	Div,
	Stat,
	Down,
	Sst,
	Base,
	Convert,
	Flags,
	Mul,
	Prob,
	Assign,
	Custom,
	ProgramFunc,
	Sub,
	Print,
	Exit,
	Off,
	Setup,
	Show,
	Run,
	Program,
	Add,
	Catalog,
	Screenshot,
}

pub trait InputQueue {
	fn has_input(&self) -> bool;
	fn pop_raw(&mut self) -> Option<KeyEvent>;
	fn wait_raw(&mut self) -> KeyEvent;

	fn wait(&mut self, mode: &mut InputMode) -> InputEvent {
		loop {
			match self.wait_raw() {
				KeyEvent::Press(key) => {
					let shift = mode.shift;
					mode.shift = false;
					match key {
						Key::Sigma => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('A'),
							AlphaMode::LowerAlpha => return InputEvent::Character('a'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::SigmaMinus;
								} else {
									return InputEvent::SigmaPlus;
								}
							}
						},
						Key::Recip => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('B'),
							AlphaMode::LowerAlpha => return InputEvent::Character('b'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Pow;
								} else {
									return InputEvent::Recip;
								}
							}
						},
						Key::Sqrt => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('C'),
							AlphaMode::LowerAlpha => return InputEvent::Character('c'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Square;
								} else {
									return InputEvent::Sqrt;
								}
							}
						},
						Key::Log => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('D'),
							AlphaMode::LowerAlpha => return InputEvent::Character('d'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::TenX;
								} else {
									return InputEvent::Log;
								}
							}
						},
						Key::Ln => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('E'),
							AlphaMode::LowerAlpha => return InputEvent::Character('e'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::EX;
								} else {
									return InputEvent::Ln;
								}
							}
						},
						Key::Xeq => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('F'),
							AlphaMode::LowerAlpha => return InputEvent::Character('f'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Gto;
								} else {
									return InputEvent::Xeq;
								}
							}
						},
						Key::Sto => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('G'),
							AlphaMode::LowerAlpha => return InputEvent::Character('g'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Complex;
								} else {
									return InputEvent::Sto;
								}
							}
						},
						Key::Rcl => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('H'),
							AlphaMode::LowerAlpha => return InputEvent::Character('h'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Percent;
								} else {
									return InputEvent::Rcl;
								}
							}
						},
						Key::RotateDown => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('I'),
							AlphaMode::LowerAlpha => return InputEvent::Character('i'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Pi;
								} else {
									return InputEvent::RotateDown;
								}
							}
						},
						Key::Sin => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('J'),
							AlphaMode::LowerAlpha => return InputEvent::Character('j'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Asin;
								} else {
									return InputEvent::Sin;
								}
							}
						},
						Key::Cos => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('K'),
							AlphaMode::LowerAlpha => return InputEvent::Character('k'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Acos;
								} else {
									return InputEvent::Cos;
								}
							}
						},
						Key::Tan => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('L'),
							AlphaMode::LowerAlpha => return InputEvent::Character('l'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Atan;
								} else {
									return InputEvent::Tan;
								}
							}
						},
						Key::Enter => {
							if shift {
								mode.alpha = match mode.alpha {
									AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
										AlphaMode::Normal
									}
									AlphaMode::Normal => AlphaMode::UpperAlpha,
								};
								return InputEvent::ModeChange;
							} else {
								return InputEvent::Enter;
							}
						}
						Key::Swap => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('M'),
							AlphaMode::LowerAlpha => return InputEvent::Character('m'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::LastX;
								} else {
									return InputEvent::Swap;
								}
							}
						},
						Key::Neg => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('N'),
							AlphaMode::LowerAlpha => return InputEvent::Character('n'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Modes;
								} else {
									return InputEvent::Neg;
								}
							}
						},
						Key::E => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('O'),
							AlphaMode::LowerAlpha => return InputEvent::Character('o'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Disp;
								} else {
									return InputEvent::E;
								}
							}
						},
						Key::Backspace => {
							if shift {
								return InputEvent::Clear;
							} else {
								return InputEvent::Backspace;
							}
						}
						Key::Up => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Up;
								} else {
									mode.alpha = AlphaMode::UpperAlpha;
									return InputEvent::ModeChange;
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Bst;
								} else {
									return InputEvent::Up;
								}
							}
						},
						Key::Seven => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('7');
								} else {
									return InputEvent::Character('P');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('7');
								} else {
									return InputEvent::Character('p');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Solver;
								} else {
									return InputEvent::Character('7');
								}
							}
						},
						Key::Eight => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('8');
								} else {
									return InputEvent::Character('Q');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('8');
								} else {
									return InputEvent::Character('q');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Integrate;
								} else {
									return InputEvent::Character('8');
								}
							}
						},
						Key::Nine => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('9');
								} else {
									return InputEvent::Character('R');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('9');
								} else {
									return InputEvent::Character('r');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Matrix;
								} else {
									return InputEvent::Character('9');
								}
							}
						},
						Key::Div => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('S'),
							AlphaMode::LowerAlpha => return InputEvent::Character('s'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Stat;
								} else {
									return InputEvent::Div;
								}
							}
						},
						Key::Down => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Down;
								} else {
									mode.alpha = AlphaMode::LowerAlpha;
									return InputEvent::ModeChange;
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Sst;
								} else {
									return InputEvent::Down;
								}
							}
						},
						Key::Four => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('4');
								} else {
									return InputEvent::Character('T');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('4');
								} else {
									return InputEvent::Character('t');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Base;
								} else {
									return InputEvent::Character('4');
								}
							}
						},
						Key::Five => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('5');
								} else {
									return InputEvent::Character('U');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('5');
								} else {
									return InputEvent::Character('u');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Convert;
								} else {
									return InputEvent::Character('5');
								}
							}
						},
						Key::Six => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('6');
								} else {
									return InputEvent::Character('V');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('6');
								} else {
									return InputEvent::Character('v');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Flags;
								} else {
									return InputEvent::Character('6');
								}
							}
						},
						Key::Mul => match mode.alpha {
							AlphaMode::UpperAlpha => return InputEvent::Character('S'),
							AlphaMode::LowerAlpha => return InputEvent::Character('s'),
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Stat;
								} else {
									return InputEvent::Mul;
								}
							}
						},
						Key::Shift => {
							mode.shift = !shift;
							return InputEvent::ModeChange;
						}
						Key::One => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('1');
								} else {
									return InputEvent::Character('X');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('1');
								} else {
									return InputEvent::Character('x');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Assign;
								} else {
									return InputEvent::Character('1');
								}
							}
						},
						Key::Two => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('2');
								} else {
									return InputEvent::Character('Y');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('2');
								} else {
									return InputEvent::Character('y');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Custom;
								} else {
									return InputEvent::Character('2');
								}
							}
						},
						Key::Three => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return InputEvent::Character('3');
								} else {
									return InputEvent::Character('Z');
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('3');
								} else {
									return InputEvent::Character('z');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::ProgramFunc;
								} else {
									return InputEvent::Character('3');
								}
							}
						},
						Key::Sub => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								return InputEvent::Character('-');
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Print;
								} else {
									return InputEvent::Sub;
								}
							}
						},
						Key::Exit => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								mode.alpha = AlphaMode::Normal;
								return InputEvent::ModeChange;
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Off;
								} else {
									return InputEvent::Exit;
								}
							}
						},
						Key::Zero => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('0');
								} else {
									return InputEvent::Character(':');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Setup;
								} else {
									return InputEvent::Character('0');
								}
							}
						},
						Key::Dot => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								return InputEvent::Character('.');
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Show;
								} else {
									return InputEvent::Character('.');
								}
							}
						},
						Key::Run => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								return InputEvent::Character('?');
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Program;
								} else {
									return InputEvent::Run;
								}
							}
						},
						Key::Add => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return InputEvent::Character('+');
								} else {
									return InputEvent::Character(' ');
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Catalog;
								} else {
									return InputEvent::Add;
								}
							}
						},
						Key::F1 => return InputEvent::FunctionKey(1, shift),
						Key::F2 => return InputEvent::FunctionKey(2, shift),
						Key::F3 => return InputEvent::FunctionKey(3, shift),
						Key::F4 => return InputEvent::FunctionKey(4, shift),
						Key::F5 => return InputEvent::FunctionKey(5, shift),
						Key::F6 => return InputEvent::FunctionKey(6, shift),
						Key::Screenshot => return InputEvent::Screenshot,
						Key::ShiftUp => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									mode.alpha = AlphaMode::UpperAlpha;
									return InputEvent::ModeChange;
								} else {
									return InputEvent::Up;
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Up;
								} else {
									return InputEvent::Bst;
								}
							}
						},
						Key::ShiftDown => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									mode.alpha = AlphaMode::LowerAlpha;
									return InputEvent::ModeChange;
								} else {
									return InputEvent::Down;
								}
							}
							AlphaMode::Normal => {
								if shift {
									return InputEvent::Down;
								} else {
									return InputEvent::Sst;
								}
							}
						},
						Key::DoubleRelease => (),
					}
				}
				KeyEvent::Release => (),
			}
		}
	}

	fn suspend(&self);
}
