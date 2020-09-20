use alloc::string::{String, ToString};

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
	Undo,
	Neg,
	Modes,
	E,
	Disp,
	Backspace,
	Clear,
	Up,
	ShiftUp,
	Solver,
	Integrate,
	Matrix,
	Div,
	Stat,
	Down,
	ShiftDown,
	Base,
	Convert,
	Logic,
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

impl InputEvent {
	pub fn to_string(&self) -> String {
		match self {
			InputEvent::Character(ch) => {
				let mut result = String::new();
				result.push(*ch);
				result
			}
			InputEvent::FunctionKey(idx, shift) => {
				let mut result = String::new();
				if *shift {
					result.push('⬏');
				}
				result.push('F');
				result.push(char::from_u32('0' as u32 + *idx as u32).unwrap());
				result
			}
			InputEvent::SigmaPlus => "Σ+".to_string(),
			InputEvent::SigmaMinus => "Σ-".to_string(),
			InputEvent::Recip => "1/x".to_string(),
			InputEvent::Pow => "pow".to_string(),
			InputEvent::Sqrt => "sqrt".to_string(),
			InputEvent::Square => "x^2".to_string(),
			InputEvent::Log => "log".to_string(),
			InputEvent::TenX => "10^x".to_string(),
			InputEvent::Ln => "ln".to_string(),
			InputEvent::EX => "e^x".to_string(),
			InputEvent::Xeq => "xeq".to_string(),
			InputEvent::Gto => "gto".to_string(),
			InputEvent::Sto => "sto".to_string(),
			InputEvent::Complex => "y+xi".to_string(),
			InputEvent::Rcl => "rcl".to_string(),
			InputEvent::Percent => "%".to_string(),
			InputEvent::RotateDown => "R↓".to_string(),
			InputEvent::Pi => "π".to_string(),
			InputEvent::Sin => "sin".to_string(),
			InputEvent::Asin => "asin".to_string(),
			InputEvent::Cos => "cos".to_string(),
			InputEvent::Acos => "acos".to_string(),
			InputEvent::Tan => "tan".to_string(),
			InputEvent::Atan => "atan".to_string(),
			InputEvent::Enter => "↵".to_string(),
			InputEvent::Swap => "swap".to_string(),
			InputEvent::Undo => "Undo".to_string(),
			InputEvent::Neg => "±".to_string(),
			InputEvent::Modes => "Modes".to_string(),
			InputEvent::E => "ᴇ".to_string(),
			InputEvent::Disp => "Disp".to_string(),
			InputEvent::Backspace => "←".to_string(),
			InputEvent::Clear => "Clear".to_string(),
			InputEvent::Up => "↑".to_string(),
			InputEvent::ShiftUp => "⬏↑".to_string(),
			InputEvent::Solver => "Solver".to_string(),
			InputEvent::Integrate => "∫".to_string(),
			InputEvent::Matrix => "Matrix".to_string(),
			InputEvent::Div => "÷".to_string(),
			InputEvent::Stat => "Stat".to_string(),
			InputEvent::Down => "↓".to_string(),
			InputEvent::ShiftDown => "⬏↓".to_string(),
			InputEvent::Base => "Base".to_string(),
			InputEvent::Convert => "Convert".to_string(),
			InputEvent::Logic => "Logic".to_string(),
			InputEvent::Mul => "×".to_string(),
			InputEvent::Prob => "Prob".to_string(),
			InputEvent::Assign => "Assign".to_string(),
			InputEvent::Custom => "Custom".to_string(),
			InputEvent::ProgramFunc => "PrgFn".to_string(),
			InputEvent::Sub => "-".to_string(),
			InputEvent::Print => "Print".to_string(),
			InputEvent::Exit => "Exit".to_string(),
			InputEvent::Off => "Off".to_string(),
			InputEvent::Setup => "Setup".to_string(),
			InputEvent::Show => "Show".to_string(),
			InputEvent::Run => "Run".to_string(),
			InputEvent::Program => "Prgm".to_string(),
			InputEvent::Add => "+".to_string(),
			InputEvent::Catalog => "Catalog".to_string(),
			InputEvent::Screenshot => "Screenshot".to_string(),
		}
	}
}

pub trait InputQueue {
	fn has_input(&self) -> bool;
	fn pop_raw(&mut self) -> Option<KeyEvent>;
	fn wait_raw(&mut self) -> Option<KeyEvent>;

	fn wait(&mut self, mode: &mut InputMode) -> Option<InputEvent> {
		loop {
			match self.wait_raw() {
				Some(KeyEvent::Press(key)) => {
					let shift = mode.shift;
					mode.shift = false;
					match key {
						Key::Sigma => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('A')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('a')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::SigmaMinus);
								} else {
									return Some(InputEvent::SigmaPlus);
								}
							}
						},
						Key::Recip => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('B')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('b')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Pow);
								} else {
									return Some(InputEvent::Recip);
								}
							}
						},
						Key::Sqrt => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('C')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('c')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Square);
								} else {
									return Some(InputEvent::Sqrt);
								}
							}
						},
						Key::Log => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('D')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('d')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::TenX);
								} else {
									return Some(InputEvent::Log);
								}
							}
						},
						Key::Ln => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('E')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('e')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::EX);
								} else {
									return Some(InputEvent::Ln);
								}
							}
						},
						Key::Xeq => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('F')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('f')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Gto);
								} else {
									return Some(InputEvent::Xeq);
								}
							}
						},
						Key::Sto => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('G')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('g')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Complex);
								} else {
									return Some(InputEvent::Sto);
								}
							}
						},
						Key::Rcl => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('H')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('h')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Percent);
								} else {
									return Some(InputEvent::Rcl);
								}
							}
						},
						Key::RotateDown => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('I')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('i')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Pi);
								} else {
									return Some(InputEvent::RotateDown);
								}
							}
						},
						Key::Sin => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('J')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('j')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Asin);
								} else {
									return Some(InputEvent::Sin);
								}
							}
						},
						Key::Cos => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('K')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('k')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Acos);
								} else {
									return Some(InputEvent::Cos);
								}
							}
						},
						Key::Tan => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('L')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('l')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Atan);
								} else {
									return Some(InputEvent::Tan);
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
								return None;
							} else {
								return Some(InputEvent::Enter);
							}
						}
						Key::Swap => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('M')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('m')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Undo);
								} else {
									return Some(InputEvent::Swap);
								}
							}
						},
						Key::Neg => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('N')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('n')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Modes);
								} else {
									return Some(InputEvent::Neg);
								}
							}
						},
						Key::E => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('O')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('o')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Disp);
								} else {
									return Some(InputEvent::E);
								}
							}
						},
						Key::Backspace => {
							if shift {
								return Some(InputEvent::Clear);
							} else {
								return Some(InputEvent::Backspace);
							}
						}
						Key::Up => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Up);
								} else {
									mode.alpha = AlphaMode::UpperAlpha;
									return None;
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::ShiftUp);
								} else {
									return Some(InputEvent::Up);
								}
							}
						},
						Key::Seven => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('7'));
								} else {
									return Some(InputEvent::Character('P'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('7'));
								} else {
									return Some(InputEvent::Character('p'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Solver);
								} else {
									return Some(InputEvent::Character('7'));
								}
							}
						},
						Key::Eight => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('8'));
								} else {
									return Some(InputEvent::Character('Q'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('8'));
								} else {
									return Some(InputEvent::Character('q'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Integrate);
								} else {
									return Some(InputEvent::Character('8'));
								}
							}
						},
						Key::Nine => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('9'));
								} else {
									return Some(InputEvent::Character('R'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('9'));
								} else {
									return Some(InputEvent::Character('r'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Matrix);
								} else {
									return Some(InputEvent::Character('9'));
								}
							}
						},
						Key::Div => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('S')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('s')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Stat);
								} else {
									return Some(InputEvent::Div);
								}
							}
						},
						Key::Down => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Down);
								} else {
									mode.alpha = AlphaMode::LowerAlpha;
									return None;
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::ShiftDown);
								} else {
									return Some(InputEvent::Down);
								}
							}
						},
						Key::Four => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('4'));
								} else {
									return Some(InputEvent::Character('T'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('4'));
								} else {
									return Some(InputEvent::Character('t'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Base);
								} else {
									return Some(InputEvent::Character('4'));
								}
							}
						},
						Key::Five => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('5'));
								} else {
									return Some(InputEvent::Character('U'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('5'));
								} else {
									return Some(InputEvent::Character('u'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Convert);
								} else {
									return Some(InputEvent::Character('5'));
								}
							}
						},
						Key::Six => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('6'));
								} else {
									return Some(InputEvent::Character('V'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('6'));
								} else {
									return Some(InputEvent::Character('v'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Logic);
								} else {
									return Some(InputEvent::Character('6'));
								}
							}
						},
						Key::Mul => match mode.alpha {
							AlphaMode::UpperAlpha => return Some(InputEvent::Character('S')),
							AlphaMode::LowerAlpha => return Some(InputEvent::Character('s')),
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Stat);
								} else {
									return Some(InputEvent::Mul);
								}
							}
						},
						Key::Shift => {
							mode.shift = !shift;
							return None;
						}
						Key::One => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('1'));
								} else {
									return Some(InputEvent::Character('X'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('1'));
								} else {
									return Some(InputEvent::Character('x'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Assign);
								} else {
									return Some(InputEvent::Character('1'));
								}
							}
						},
						Key::Two => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('2'));
								} else {
									return Some(InputEvent::Character('Y'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('2'));
								} else {
									return Some(InputEvent::Character('y'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Custom);
								} else {
									return Some(InputEvent::Character('2'));
								}
							}
						},
						Key::Three => match mode.alpha {
							AlphaMode::UpperAlpha => {
								if shift {
									return Some(InputEvent::Character('3'));
								} else {
									return Some(InputEvent::Character('Z'));
								}
							}
							AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('3'));
								} else {
									return Some(InputEvent::Character('z'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::ProgramFunc);
								} else {
									return Some(InputEvent::Character('3'));
								}
							}
						},
						Key::Sub => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								return Some(InputEvent::Character('-'));
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Print);
								} else {
									return Some(InputEvent::Sub);
								}
							}
						},
						Key::Exit => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								mode.alpha = AlphaMode::Normal;
								return None;
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Off);
								} else {
									return Some(InputEvent::Exit);
								}
							}
						},
						Key::Zero => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('0'));
								} else {
									return Some(InputEvent::Character(':'));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Setup);
								} else {
									return Some(InputEvent::Character('0'));
								}
							}
						},
						Key::Dot => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								return Some(InputEvent::Character('.'));
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Show);
								} else {
									return Some(InputEvent::Character('.'));
								}
							}
						},
						Key::Run => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								return Some(InputEvent::Character('?'));
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Program);
								} else {
									return Some(InputEvent::Run);
								}
							}
						},
						Key::Add => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									return Some(InputEvent::Character('+'));
								} else {
									return Some(InputEvent::Character(' '));
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Catalog);
								} else {
									return Some(InputEvent::Add);
								}
							}
						},
						Key::F1 => return Some(InputEvent::FunctionKey(1, shift)),
						Key::F2 => return Some(InputEvent::FunctionKey(2, shift)),
						Key::F3 => return Some(InputEvent::FunctionKey(3, shift)),
						Key::F4 => return Some(InputEvent::FunctionKey(4, shift)),
						Key::F5 => return Some(InputEvent::FunctionKey(5, shift)),
						Key::F6 => return Some(InputEvent::FunctionKey(6, shift)),
						Key::Screenshot => return Some(InputEvent::Screenshot),
						Key::ShiftUp => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									mode.alpha = AlphaMode::UpperAlpha;
									return None;
								} else {
									return Some(InputEvent::Up);
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Up);
								} else {
									return Some(InputEvent::ShiftUp);
								}
							}
						},
						Key::ShiftDown => match mode.alpha {
							AlphaMode::UpperAlpha | AlphaMode::LowerAlpha => {
								if shift {
									mode.alpha = AlphaMode::LowerAlpha;
									return None;
								} else {
									return Some(InputEvent::Down);
								}
							}
							AlphaMode::Normal => {
								if shift {
									return Some(InputEvent::Down);
								} else {
									return Some(InputEvent::ShiftDown);
								}
							}
						},
						Key::DoubleRelease => continue,
					}
				}
				Some(KeyEvent::Release) => continue,
				None => return None,
			}
		}
	}

	fn suspend(&self);
}
