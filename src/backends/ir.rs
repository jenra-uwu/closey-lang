use std::collections::HashMap;
use std::fmt::Display;

use super::super::frontend::ir::{self, SExpr};

#[derive(Copy, Clone)]
pub enum IrInstruction {
    Ret,
    Load,
    Apply,
    Call(bool)
}

impl Display for IrInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IrInstruction::*;
        match self {
            Ret  => write!(f, "ret" ),
            Load => write!(f, "load"),
            Apply => write!(f, "apply"),
            Call(known_arity) => write!(f, "call{}", if *known_arity { "" } else { "?" }),
        }
    }
}

#[derive(Clone)]
pub enum IrArgument {
    Local(usize),
    Argument(usize),
    Function(String)
}

impl Display for IrArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use IrArgument::*;
        match self {
            Local(l) => write!(f, "%{}", l),
            Argument(a) => write!(f, "${}", a),
            Function(g) => write!(f, "{}", g)
        }
    }
}

pub struct IrSsa {
    pub local: Option<usize>,
    pub local_lifetime: usize,
    pub local_register: usize,
    pub instr: IrInstruction,
    pub args: Vec<IrArgument>
}

impl Display for IrSsa {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(l) = self.local {
            write!(f, "%{} = ", l)?;
        }

        write!(f, "{}", self.instr)?;
        for a in self.args.iter() {
            write!(f, " {}", a)?;
        }
        Ok(())
    }
}

pub struct IrFunction {
    pub name: String,
    pub argc: usize,
    pub ssas: Vec<IrSsa>
}

impl Display for IrFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}):", self.name, self.argc)?;
        for ssa in self.ssas.iter() {
            write!(f, "\n    {}", ssa)?;
        }
        Ok(())
    }
}

impl IrFunction {
    fn get_last_local(&self) -> Option<usize> {
        for ssa in self.ssas.iter().rev() {
            if let Some(l) = ssa.local {
                return Some(l);
            }
        }
        None
    }

    fn get_next_local(&self) -> usize {
        for ssa in self.ssas.iter().rev() {
            if let Some(l) = ssa.local {
                return l + 1;
            }
        }
        0
    }
}

pub struct IrModule {
    pub funcs: Vec<IrFunction>
}

impl Display for IrModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for func in self.funcs.iter() {
            write!(f, "{}\n\n", func)?;
        }
        Ok(())
    }
}

fn conversion_helper(args_map: &HashMap<String, usize>, func: &mut IrFunction, sexpr: SExpr) -> Option<usize> {
    match sexpr {
        SExpr::Empty(_) => todo!(),
        SExpr::TypeAlias(_, _) => todo!(),

        SExpr::Symbol(_, s) => {
            if let Some(a) = args_map.get(&s) {
                let local = Some(func.get_next_local());
                func.ssas.push(IrSsa {
                    local,
                    local_lifetime: 0,
                    local_register: 0,
                    instr: IrInstruction::Load,
                    args: vec![IrArgument::Argument(*a)]
                });
                local
            } else {
                todo!("symbols that aren't arguments");
            }
        }

        SExpr::Function(_, f) => {
            let local = Some(func.get_next_local());
            func.ssas.push(IrSsa {
                local,
                local_lifetime: 0,
                local_register: 0,
                instr: IrInstruction::Load,
                args: vec![IrArgument::Function(f)]
            });
            local
        }

        SExpr::ExternalFunc(_, _, _) => todo!(),
        SExpr::Chain(_, _, _) => todo!(),

        SExpr::Application(m, mut f, a) => {
            let mut stack = vec![(m.arity, *a)];

            while let SExpr::Application(m, func, a) = *f {
                stack.push((m.arity, *a));
                f = func;
            }

            let mut last_arity = f.get_metadata().arity;
            let mut f = conversion_helper(args_map, func, *f).unwrap();
            let mut args = vec![];
            let mut local = None;
            while let Some((arity, a)) = stack.pop() {
                args.push(conversion_helper(args_map, func, a).unwrap());
                if arity == 0 {
                    use std::iter::once;
                    local = Some(func.get_next_local());
                    func.ssas.push(IrSsa {
                        local,
                        local_lifetime: 0,
                        local_register: 0,
                        instr: IrInstruction::Call(last_arity != 0),
                        args: once(IrArgument::Local(f)).chain(args.into_iter().map(IrArgument::Local)).collect()
                    });
                    f = local.unwrap();
                    args = vec![];
                }
                last_arity = arity;
            }

            if m.arity != 0 {
                use std::iter::once;
                local = Some(func.get_next_local());
                func.ssas.push(IrSsa {
                    local,
                    local_lifetime: 0,
                    local_register: 0,
                    instr: IrInstruction::Apply,
                    args: once(IrArgument::Local(f)).chain(args.into_iter().map(IrArgument::Local)).collect()
                });
            }

            local
        }

        SExpr::Assign(_, _, _) => todo!(),
        SExpr::With(_, _, _) => todo!(),
        SExpr::Match(_, _, _) => todo!(),
    }
}

fn calculate_lifetimes(func: &mut IrFunction) {
    let mut iter = func.ssas.iter_mut();
    let mut i = 0;
    while let Some(ssa) = iter.next() {
        if ssa.local.is_none() {
            continue;
        }
        let local = ssa.local.unwrap();

        let mut j = i + 1;
        for next in iter.as_slice() {
            for arg in next.args.iter() {
                if let IrArgument::Local(l) = arg {
                    if *l == local {
                        ssa.local_lifetime = j - i + 1;
                        break;
                    }
                }
            }

            j += 1;
        }

        i += 1;
    }
}

// convert_frontend_ir_to_backend_ir(ir::IrModule) -> IrModule
// Converts the frontend IR language to the backend IR language.
pub fn convert_frontend_ir_to_backend_ir(module: ir::IrModule) -> IrModule {
    let mut new = IrModule { funcs: vec![] };

    for func in module.funcs {
        let mut f = IrFunction {
            name: func.1.name,
            argc: func.1.args.len() + func.1.captured.len(),
            ssas: vec![]
        };
        let args_map: HashMap<String, usize> = func.1.captured_names.into_iter().enumerate().chain(func.1.args.into_iter().map(|v| v.0).enumerate()).map(|v| (v.1, v.0)).collect();

        conversion_helper(&args_map, &mut f, func.1.body);
        f.ssas.push(IrSsa {
            local: None,
            local_lifetime: 0,
            local_register: 0,
            instr: IrInstruction::Ret,
            args: if let Some(l) = f.get_last_local() {
                vec![IrArgument::Local(l)]
            } else {
                vec![]
            }
        });

        calculate_lifetimes(&mut f);

        new.funcs.push(f);
    }

    new
}
