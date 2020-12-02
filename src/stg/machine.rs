use std::rc::Rc;

use super::heap::Heap;
use super::ast::*;

const DEBUG: bool = true;

#[derive(Debug, Clone, Copy)]
pub enum Value {
    Addr(usize),
    Int(usize),
}

#[derive(Debug, Clone)]
pub struct Closure(pub LambdaForm, pub Vec<Value>);

impl Closure {
    pub fn is_updatable(&self) -> bool {
        let Closure(LambdaForm(_vs, pi, _xs, _e), _ws) = self;
        *pi
    }
}

pub type Addr = usize;

#[derive(Debug, Clone)]
pub struct Context(Vec<(Var, Value)>);

// TODO move this into its own module
impl Context {
    pub fn empty() -> Self {
        Context(vec![])
    }

    pub fn from(vars: Vec<Var>, values: Vec<Value>) -> Self {
        Context::empty().extend_many(vars, values)
    }

    pub fn extend_many(&self, vars: Vec<Var>, values: Vec<Value>) -> Context {
        let Context(mut bindings) = self.clone();

        assert_eq!(vars.len(), values.len());
        for (var, value) in vars.iter().zip(values) {
            bindings.push((var.to_owned(), value));
        }
        Context(bindings)
    }

    pub fn concat(&self, other: &Context) -> Context {
        let mut bindings = self.0.clone();

        for binding in other.0.iter() {
            bindings.push(binding.clone());
        }

        Context(bindings)
    }

    pub fn lookup(&self, name: &Var) -> Value {
        for (n, v) in &self.0 {
            if n == name {
                return v.clone();
            }
        }
        panic!("Could not find name {} in context {}", name, self);
    }

    pub fn lookup_many(&self, names: &[Var]) -> Vec<Value> {
        let mut values = vec![];
        for name in names {
            values.push(self.lookup(name));
        }
        values
    }

    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, (Var, Value)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
pub struct UpdateFrame(ArgStack, RetStack, Addr);

#[derive(Debug, Clone)]
pub struct Continuation(Alts, Context);

pub type ArgStack = Vec<Value>;
pub type RetStack = Vec<Continuation>;
pub type UpdStack = Vec<UpdateFrame>;

#[derive(Debug, Clone)]
pub enum Instr {
    Eval(Expr, Context),
    Enter(Addr),
    RetCtor(Ctor, Vec<Value>),
    RetInt(usize),
}

type PrimFn = Rc<dyn Fn(&[usize]) -> usize>;

#[derive(Clone)]
pub struct PrimOp {
    pub name: String,
    pub arg_count: usize,
    pub op: PrimFn,
 }


 impl std::fmt::Debug for PrimOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PRIMOP({})", self.name)
    }
}

impl PrimOp {
    pub fn new(name: &str, arg_count: usize, op: Rc<dyn Fn(&[usize]) -> usize>) -> Self {
        PrimOp {
            name: name.to_string(),
            arg_count,
            op,
        }
    }

    pub fn apply(&self, args: &[usize]) -> usize {
        assert_eq!(args.len(), self.arg_count);
        (*self.op)(args)
    }
}

#[derive(Debug, Clone)]
pub struct StgMachine {
    pub instr: Option<Instr>,
//    pub ctors: Vec<Var>
    pub primops: Vec<PrimOp>,
    pub globals: Context,
    pub arg_stack: ArgStack,
    pub ret_stack: RetStack,
    pub upd_stack: UpdStack,
    pub heap: Heap,
}

impl StgMachine {
    pub fn new(program: &Program, as_main: Option<&str>) -> Self {
        let mut heap = Heap::new();
        let mut global_bindings = Vec::new();

        // initialize global_bindings and heap
        for Binding(name, lf) in &program.0 {
            let closure = Closure(lf.clone(), vec![]);
            let a = heap.alloc(closure);
            global_bindings.push((name.clone(), Value::Addr(a)));
        }

        let globals = Context(global_bindings);

        // initial instruction is "Eval (main {}) {}"
        let instr = match as_main {
            None => None,
            Some(main_name) => Some(Instr::Eval(
                ExprNode::App(
                    AppType::Fun,
                    main_name.to_owned(),
                    vec![],
                ).into(),
                Context::empty(),
            )),
        };

        let primops = vec![
            PrimOp::new("+1", 1, Rc::new(|vs| vs[0] + 1)),
            PrimOp::new("-1", 1, Rc::new(|vs| vs[0] - 1)),
            PrimOp::new("+", 2, Rc::new(|vs| vs[0] + vs[1])),
            PrimOp::new("halt", 0, Rc::new(|_vs| panic!("HALT"))),
        ];

        StgMachine {
            arg_stack: vec![],
            ret_stack: vec![],
            upd_stack: vec![],
            primops,
            globals,
            heap,
            instr,
        }
    }

    pub fn is_halted(&self) -> bool {
        self.instr.is_none()
    }

    pub fn step(&mut self) {
        debug("*******************************************************************************");
        if let Some(instr) = self.instr.clone() {
            debug(&format!("INSTR: {}", &instr));
            match instr {
                Instr::Eval(e, p) => self.step_eval(e, p),
                Instr::Enter(addr) => self.step_enter(addr),
                Instr::RetCtor(c, vs) => self.step_retctor(&c, vs.as_slice()),
                Instr::RetInt(k) => self.step_retint(k),
            }
        }
    }

    fn step_eval(&mut self, e: Expr, p: Context) {
        // handles cases: 1 3 4 5 9 10 14
        self.instr = match e.as_ref() {
            ExprNode::App(AppType::Fun, f, vs) => {
                match self.lookup_var(&f, &p) {
                    Value::Addr(a) => {
                        // case 1
                        debug(&format!("LOOKING UP ATOMS {:?} in CONTEXT {:?}", &vs, &p));
                        let mut args = self.lookup_atoms(&vs, &p);
                        args.reverse();
                        debug(&format!("PUSHING ARGUMENTS {:?}", &args));
                        self.arg_stack.extend(args);
                        Some(Instr::Enter(a))
                    },
                    Value::Int(k) => {
                        // case 10
                        assert!(vs.is_empty(), "Cannot Eval a boxed value with parameters or something");
                        Some(Instr::RetInt(k))
                    },
                }
            },
            ExprNode::App(AppType::Ctor, c, vs) => {
                // case 5
                debug(&format!("LOOKING UP ATOMS {:?} in CONTEXT {:?}", &vs, &p));
                Some(Instr::RetCtor(c.clone(), self.lookup_atoms(&vs, &p)))
            },
            ExprNode::App(AppType::Prim, f, vs) => {
                // case 14
                debug(&format!("LOOKING UP PRIMOP {:?}", &f));
                let primop = self.lookup_prim(&f).expect(&format!("Unknonwn primop referenced: {:?}.", f));

                debug(&format!("LOOKING UP ATOMS {:?} in CONTEXT {:?}", &vs, &p));
                let vals = self.lookup_atoms(&vs, &p)
                    .iter()
                    .map(|val| {
                        match val {
                            Value::Int(k) => *k,
                            Value::Addr(a) => panic!("Unexpected address {} found as argument to primop {:?}", a, primop)
                        }
                    }).collect::<Vec<_>>();
                let k = primop.apply(&vals);
                debug(&format!("RESULT OF PRIMOP: {}", k));
                Some(Instr::RetInt(k))
            },
            ExprNode::Let(let_type, bindings, e) => {
                // case 3
                let mut vars = Vec::new();
                let mut addrs = Vec::new();

                for Binding(var, lf) in bindings {
                    let closure = Closure(lf.clone(), vec![]);

                    debug(&format!("ALLOCATING CLOSURE {:?}", &closure));
                    let a = self.heap.alloc(closure);
                    debug(&format!("ADDRESS: {}", a));
                    addrs.push(a);
                    vars.push(var.to_string());
                }

                let values = addrs.iter().map(|a| Value::Addr(*a)).collect();
                let p_prime = p.extend_many(vars, values);

                let p_rhs = match let_type {
                    LetType::NonRecursive => p,
                    LetType::Recursive => p_prime.clone(),
                };

                for a in addrs.iter() {
                    let Closure(LambdaForm(vs, _pi, _xs, _e), _ws_empty) = self.heap.lookup(*a);
                    let vs: Vec<&Var> = vs.iter().map(|v| v).collect();

                    debug(&format!("LOOKING UP {:?} in CONTEXT {:?}", &vs, &p_rhs));
                    let new_ws = self.lookup_vars(&vs, &p_rhs);
                    debug(&format!("CLOSURE CLOSES OVER {:?}", &new_ws));

                    let Closure(LambdaForm(_vs, _pi, _xs, _e), ref mut ws) = self.heap.lookup_mut(*a);
                    *ws = new_ws;
                }

                Some(Instr::Eval(e.clone(), p_prime))
            },
            ExprNode::Case(e, alts) => {
                // case 4
                debug(&format!("PUSHING CONTINUATION ONTO STACK {:?} {}", &alts, &p));
                self.ret_stack.push(Continuation(alts.clone(), p.clone()));
                Some(Instr::Eval(e.clone(), p.clone()))
            },
            ExprNode::Lit(k) => {
                // case 9
                Some(Instr::RetInt(*k))
            },
        }
    }

    fn step_enter(&mut self, addr: Addr)  {
        // handles cases: 2 15 17
        let closure = self.heap.lookup(addr);
        let global_name = self.lookup_global_name_from_addr(addr).unwrap_or_default();
        debug(&format!("LOOKING UP IN HEAP AT {} [{}]", &addr, &global_name));
        debug(&format!("FOUND: {:?}", &closure));

        if !closure.is_updatable() {
            debug(&format!("NOT UPDATABLE"));
            let Closure(LambdaForm(vs, _pi, xs, e), ws_f) = closure;

            debug(&format!("ARE THERE ENOUGH ARGS ON THE STACK?"));
            if  self.arg_stack.len() >= xs.len() {
                // enough args on arg stack
                // case 2
                debug(&format!("YES, THERE ARE ENOUGH ARGS"));

                let mut args = Vec::new();
                for _ in 0..xs.len() {
                    args.push(self.arg_stack.pop().unwrap()); // len() check above guarantees this is safe
                }
                debug(&format!("POP {} ARGS OFF THE ARG STACK: {:?}", xs.len(), &args));
                let p = Context::from(vs.clone(), ws_f.clone())
                    .extend_many(xs.clone(), args);

                self.instr = Some(Instr::Eval(e.clone(), p));
            } else {
                // not enough args on arg stack
                // case 17
                debug(&format!("NO, THERE ARE INSUFFICIENT ARGS"));
                let Closure(LambdaForm(vs, _pi, xs, _e), ws_f) = self.heap.lookup(addr).clone();
                assert!(xs.len() > 0); // only applies if the number of arguments #xs is greater than zero, so the closure being entered will be non-updatable
                let (xs1, _xs2) = xs.split_at(self.arg_stack.len());

                assert!(self.ret_stack.is_empty()); // TODO why?

                match self.upd_stack.pop() {
                    Some(UpdateFrame(as_u, rs_u, a_u)) => {
                        assert_ne!(addr, a_u); // TODO I'm just curious to see if this ever happens
                        let Closure(LambdaForm(ref mut closure_vs, _pi, _xs, _e), ref mut closure_ws) = self.heap.lookup_mut(a_u);

                        let mut new_arg_stack = self.arg_stack.clone();
                        new_arg_stack.extend(as_u.iter());
                        let orig_arg_stack = self.arg_stack.clone();

                        self.ret_stack = rs_u;
                        self.arg_stack = new_arg_stack;

                        *closure_vs = vs.iter().chain(xs1.iter()).map(|s| s.to_owned()).collect();
                        *closure_ws = ws_f.iter().chain(orig_arg_stack.iter()).cloned().collect();

                        closure_vs.extend(xs1.iter().map(|s| s.to_owned()));
                        // self.instr stays as Enter a
                    },
                    None => {
                        self.instr = None; // halt
                    }
                }
            }
        } else {
            // if updatable
            // case 15
            debug(&format!("UPDATABLE"));
            let Closure(LambdaForm(vs, _pi, _xs, e), ws_f) = closure;
            self.upd_stack.push(UpdateFrame(self.arg_stack.clone(), self.ret_stack.clone(), addr));
            self.arg_stack = vec![];
            self.ret_stack = vec![];
            let p = Context::from(vs.clone(), ws_f.clone());

            self.instr = Some(Instr::Eval(e.clone(), p));
        }
    }

    fn step_retctor(&mut self, c: &Ctor, ws: &[Value]) {
        // handles cases: 6 7 8 16
        // TODO case 8 (default, binding) is not yet implemented in the AST
        self.instr = match self.ret_stack.pop() {
            None => {
                // case 16
                assert!(self.arg_stack.is_empty()); // TODO explain why
                debug(&format!("POP FROM ARG STACK"));
                match self.upd_stack.pop() {
                    Some(UpdateFrame(as_u, rs_u, a_u)) => {
                        // replace the closure at a_u with a standard constructor closure
                        let closure = self.heap.lookup_mut(a_u);
                        debug(&format!("LOOKUP IN HEAP AT {} FOUND {:?}", &a_u, &closure));
                        debug(&format!("RESTORING ARG STACK {:?}", &as_u));
                        debug(&format!("RESTORING ARG STACK {:?}", &rs_u));
                        self.arg_stack = as_u;
                        self.ret_stack = rs_u;

                        let vs = (0..ws.len()).into_iter().map(|i| format!("gensym_v{}", i)).collect::<Vec<Var>>();
                        let pi = false;
                        let xs = vec![];
                        let e = ExprNode::App(
                            AppType::Ctor,
                            c.clone(),
                            vs.iter().map(|v| Atom::Var(v.clone())).collect(),
                        ).into();

                        *closure = Closure(LambdaForm(vs, pi, xs, e), ws.iter().cloned().collect());
                        debug(&format!("OVERWRITE CLOSURE WITH {:?}", &closure));

                        // no change to instruction
                        self.instr.clone()
                    },
                    None => None, // halt
                }
            },
            Some(Continuation(alts, ctx)) => {
                debug(&format!("POPPING CONTINUATION FROM RET STACK"));
                debug(&format!("CTX IS: {:?}", &ctx));
                debug(&format!("FINDING MATCHING ALT"));
                match alts.find_alt_for_ctor(c) {
                    Some(Alt::Ctor(ctor_tag, vars, e)) => {
                        // case 6
                        debug(&format!("USING {} {:?} => {}", &ctor_tag, &vars, &e));
                        let new_ctx = Context::from(vars.to_owned(), ws.to_owned()).concat(&ctx);
                        Some(Instr::Eval(e.clone(), new_ctx))
                    },
                    Some(Alt::Default(_var, e)) => {
                        // case 7
                        // TODO todo!() var isn't actually BOUND here!
                        debug(&format!("USING DEFAULT"));
                        Some(Instr::Eval(e.clone(), ctx))
                    }
                    _ => unreachable!(),
                }
            },
        }
    }

    fn step_retint(&mut self, k: usize) {
        // handles cases 11 12 13
        // TODO doesn't handle case 13 (non-binding default). This is unnecessary.
        self.instr = match self.ret_stack.pop() {
            None => None, // halt
            Some(Continuation(alts, ctx)) => {
                match alts.find_alt_for_int(k) {
                    Some(Alt::Lit(_n, e)) => {
                        // case 11
                        Some(Instr::Eval(e.clone(), ctx))
                    },
                    Some(Alt::Default(var, e)) => {
                        // case 12
                        let new_ctx = ctx.extend_many(vec![var.to_owned()], vec![Value::Int(k)]);
                        Some(Instr::Eval(e.clone(), new_ctx))
                    },
                    _ => unreachable!(),
                }
            },
        }
    }

    pub fn lookup_prim(&self, name: &str) -> Option<&PrimOp> {
        for primop in &self.primops {
            if &primop.name == name {
                return Some(&primop);
            }
        }
        None
    }

    // TODO careful that these handle both locals and globals
    // TODO move the loops to Context
    fn lookup_atom(&self, atom: &Atom, locals: &Context) -> Value {
        match atom {
            Atom::Var(var) => self.lookup_var(var, locals),
            Atom::Lit(k) => Value::Int(*k),
        }
    }

    fn lookup_atoms(&self, atoms: &[Atom], locals: &Context) -> Vec<Value> {
        atoms.iter().map(|a| self.lookup_atom(a, locals)).collect()
    }

    pub fn lookup_global_addr(&self, var: &str) -> Option<Addr> {
        if let Some(Value::Addr(a)) = self.lookup_var_global(&var.to_owned()) {
            Some(a)
        } else {
            None
        }
    }

    fn lookup_var_global(&self, var: &Var) -> Option<Value> {
        let Context(global_bindings) = &self.globals;
        for (global_var, value) in global_bindings {
            if global_var == var {
                return Some(*value);
            }
        }
        None
    }

    fn lookup_vars(&self, vars: &[&Var], locals: &Context) -> Vec<Value> {
        let mut values = vec![];
        for var in vars.iter() {
            values.push(self.lookup_var(var, locals));
        }
        values
    }

    pub fn lookup_var(&self, var: &Var, locals: &Context) -> Value {
        let Context(local_bindings) = locals;
        for (local_var, value) in local_bindings {
            if local_var == var {
                return *value;
            }
        }
        match self.lookup_var_global(var)
        {
            Some(value) => value,
            None => panic!("Could not find variable {:?} in local context {:?}, global context: {:?}", var, locals, &self.globals),
        }
    }

    fn lookup_global_name_from_addr(&self, a: Addr) -> Option<Var> {
        for (global_var, value) in self.globals.iter() {
            if let Value::Addr(addr) = value {
                if a == *addr {
                    return Some(global_var.clone());
                }
            }
        }
        None
    }

    pub fn seq(&mut self, a: Addr) -> &Closure {
        debug(&format!("SEQ ON {}", a));
        self.instr = Some(Instr::Enter(a));
        while !self.is_halted() {
            self.step();
        }
        self.heap.lookup(a)
    }

    pub fn deep_seq(&mut self, a: Addr) -> &Closure {
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        debug(&format!("DEEP SEQ ON {}", a));
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        let original_a = a;
        let mut seq_queue = vec![a];
        while let Some(a) = seq_queue.pop() {
            dbg!(&a);
            let closure = self.seq(a);
            debug(&format!("RESULT OF SEQ ON {}: {:?}", a, &closure));

            let Closure(_lf, ws) = self.seq(a);
            for w in ws {
                if let Value::Addr(wa) = w {
                    debug(&format!("QUEUEING {} FOR SEQ'ING", wa));
                    seq_queue.push(*wa);
                }
            }
        }
        dbg!();
        self.heap.lookup(original_a)
    }

}

fn debug(msg: &str) {
    if DEBUG {
        eprintln!("{}", msg);
    }
}

impl std::fmt::Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        writeln!(f, "{:?}", self.0)
    }
}

impl std::fmt::Display for Instr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Instr::Eval(e, env) => write!(f, "Eval {} {}", e, env),
            Instr::Enter(a) => write!(f, "Enter {}", a),
            Instr::RetCtor(c, ws) => write!(f, "ReturnCon {} {:?}", c, ws),
            Instr::RetInt(k) => write!(f, "ReturnInt {}", k),
        }
    }
}
