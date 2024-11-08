use crate::htn_vm::{Num, Operation};

use super::{tokens::VmValue, HtnInstr, ParseValue};

pub fn emit(instrs: Vec<HtnInstr>) -> Vec<Operation> {
	let mut ops = vec![];

	for instr in instrs {
		match instr {
			HtnInstr::Assign(key, value) => {
				ops.append(&mut value.resolve());
				ops.push(Operation::SetBlackBoard { key });
			}
			HtnInstr::Value(value) => {
				ops.append(&mut value.resolve());
				ops.push(Operation::Pop { count: 1 });
			}
			HtnInstr::Exit(state) => ops.push(Operation::EndTick(state)),
			HtnInstr::Noop => {},
			HtnInstr::If(cond, if_branch, else_branch) => {
				let mut if_branch = emit(if_branch);
				let else_branch = else_branch.map(emit);
				ops.append(&mut cond.resolve());

				let len = if else_branch.is_none() {
					if_branch.len()
				} else {
					if_branch.len() + 1
				} as u32;

				ops.push(Operation::SkipIf { skip: len as u32, invert: true });
				ops.append(&mut if_branch);

				if let Some(mut else_branch) = else_branch {
					ops.push(Operation::Skip { skip: else_branch.len() as u32 });
					ops.append(&mut else_branch);
				}
			}
		}
	}

	ops
}

trait Emit {
	fn resolve(self) -> Vec<Operation>;
}

impl Emit for ParseValue {
	fn resolve(self) -> Vec<Operation> {
		match self {
			ParseValue::Access(target, key) => {
				let mut ops = target.resolve();
				ops.push(Operation::Access { key });
				ops
			}
			ParseValue::Literal(value) => vec![Operation::Push { value: value }],
			ParseValue::Expression(lhs, op, rhs) => {
				let mut ops = lhs.resolve();
				ops.extend(rhs.resolve());
				ops.push(match op {
					super::BinaryOp::Add => Operation::Add,
					super::BinaryOp::Sub => Operation::Sub,
					super::BinaryOp::Mul => Operation::Mul,
					super::BinaryOp::Div => Operation::Div,
					super::BinaryOp::Mod => Operation::Mod,
					super::BinaryOp::And => Operation::And,
					super::BinaryOp::Or => Operation::Or,
					super::BinaryOp::Eq => Operation::Eq,
					super::BinaryOp::Neq => Operation::Neq,
					super::BinaryOp::Lt => Operation::Lt,
					super::BinaryOp::Lte => Operation::Lte,
					super::BinaryOp::Gt => Operation::Gt,
					super::BinaryOp::Gte => Operation::Gte,
				});
				ops
			}
			ParseValue::Index(target, key) => {
				todo!()
			}
			ParseValue::NullCoalesce(lhs, rhs) => {
				let mut ops = lhs.resolve();
				let rhs_ops = rhs.resolve();
				ops.push(Operation::IsNull);
				ops.push(Operation::SkipIf { skip: rhs_ops.len() as u32 + 1, invert: false });
				ops.push(Operation::Pop { count: 1 });
				ops.extend(rhs_ops);
				ops
			}
			ParseValue::Unary(op, target) => {
				let mut ops = target.resolve();
				ops.extend_from_slice(match op {
					super::UnaryOp::Not => &[ Operation::Not ],
					super::UnaryOp::Neg => &[
						Operation::Push { value: VmValue::Number(Num::Int(-1)) },
						Operation::Mul,
					],
					super::UnaryOp::Dbg => &[ Operation::Dbg ],
				});
				ops
			}
			ParseValue::Variable(name) => {
				vec![Operation::GetBlackBoard { key: name }]
			}
			ParseValue::Call(target, args) => {
				let mut ops = vec![];
				let mut method = false;

				let arg_count = args.len();
				for arg in args {
					ops.extend(arg.resolve());
				}

				if let ParseValue::Access(target, _) = target.as_ref() {
					ops.extend(target.clone().resolve());
					method = true;
				}

				ops.extend(target.resolve());
				ops.push(Operation::Call { args: arg_count as u32, method });
				ops
			}
			ParseValue::Object(fields) => {
				let mut keys = Vec::with_capacity(fields.len());
				let mut ops = Vec::with_capacity(fields.len());
				for (key, value) in fields {
					ops.append(&mut value.resolve());
					keys.push(key);
				}
				ops.push(Operation::DefineObj { fields: keys });

				ops
			}
		}
	}
}
