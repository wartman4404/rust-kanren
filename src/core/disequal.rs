use std::any::TypeId;
use std::fmt::{self, Debug, Formatter};
use core::{ConstraintResult, StateProxy, UntypedVar, Constraint, ToConstraint, State, Var, ToVar, FollowRef, Unifier, VarMap};
use core::ConstraintResult::*;
use core::VarRef::*;

///! The Disequal constraint enforces that its arguments will never have equal values.
#[derive(Clone)]
pub struct Disequal {
    pairs: Vec<(UntypedVar, UntypedVar, TypeId)>,
}

///! Builder for Disequal, which enforces that its arguments will never have equal values.
pub struct ToDisequal<A, B, C>
where A: ToVar<VarType=C>, B: ToVar<VarType=C>, C: ToVar {
    a: A,
    b: B,
}

impl<A, B, C> ToConstraint for ToDisequal<A, B, C>
where A: ToVar<VarType=C>, B: ToVar<VarType=C>, C: ToVar {
    type ConstraintType = Disequal;
    fn into_constraint(self, state: &mut State) -> Disequal {
        let mut disequal = Disequal::new_empty();
        let a = state.make_var_of(self.a);
        let b = state.make_var_of(self.b);
        disequal.push(state, a, b);
        disequal
    }
}

impl Debug for Disequal {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        try!(write!(fmt, "Disequal ["));
        let mut pairs = self.pairs.iter().peekable();
        while let Some(&(a, b, _)) = pairs.next() {
            try!(write!(fmt, "{:?} != {:?}", a, b));
            if pairs.peek().is_some() { try!(write!(fmt, ", ")); }
        }
        write!(fmt, "]")
    }
}

impl Disequal {
    pub fn new<A, B, C>(a: A, b: B) -> ToDisequal<A, B, C>
    where A: ToVar<VarType=C>, B: ToVar<VarType=C>, C: ToVar {
        ToDisequal { a: a, b: b }
    }

    fn new_empty() -> Disequal {
        Disequal { pairs: Vec::new() }
    }
    pub fn push<A>(&mut self, state: &State, a: Var<A>, b: Var<A>) where A: ToVar {
        let a = state.follow_id(a.var);
        let b = state.follow_id(b.var);
        self.pairs.push((a, b, TypeId::of::<A>()));
    }

    fn disequal_update_vars(&self, state: &State) -> ConstraintResult<Disequal> {
        let needs_update = self.pairs.iter().any(|&(a, b, _)| {
            if let Some(&EqualTo(..)) = state.eqs.get(&a) {
                true
            } else if let Some(&EqualTo(..)) = state.eqs.get(&b) {
                true
            } else {
                false
            }
        });
        if needs_update {
            println!("updating vars for {:?}", self);
            let pairs = self.pairs.iter().map(|&(olda, oldb, ty)| {
                let a = match state.eqs.get(&olda) {
                    Some(&EqualTo(x)) => x,
                    _ => olda,
                };
                let b = match state.eqs.get(&oldb) {
                    Some(&EqualTo(x)) => x,
                    _ => oldb,
                };
                println!("replacing {:?} != {:?} with {:?} != {:?}", olda, oldb, a, b);
                (a, b, ty)
            });
            Updated(Disequal { pairs: pairs.collect() })
        } else {
            Unchanged
        }
    }
    fn relevant(&self, var: UntypedVar) -> bool {
        let ret = self.pairs.iter().any(|&(a, b, _)| a == var || b == var);
        println!("{:?} relevant to {:?}: {}", var, self, ret);
        ret
    }
}


impl Constraint for Disequal {
        
    fn test(&self, proxy: &mut StateProxy) -> ConstraintResult<Disequal> {
        println!("updating {:?}", self);
        for &(a, b, ty) in self.pairs.iter() {
            proxy.untyped_unify(a, b, ty);
            if !proxy.ok() {
                println!("failed unification, disequality constraint satisfied");
                return Irrelevant;
            }
        }
        if proxy.eqs.eqs.is_empty() {
            println!("succeeded unification with no additions, disequality constraint failed");
            return Failed;
        }

        let mut all_unchanged = true;
        let mut updated = Disequal::new_empty();
        for &(k, _) in proxy.eqs.eqs.iter() {
            let eqvar = match proxy.get_ref(k) {
                // We don't care about values added by overwrite() -- they indicate that something
                // had to be updated (so it passes) but the value will be gone when we roll back
                &Exactly(..) => { continue; },
                &EqualTo(eqvar) => eqvar,
            };

            if all_unchanged && self.relevant(k) {
                continue;
            }
            all_unchanged = false;
            let newtype = proxy.follow_ref(eqvar).2;
            updated.pairs.push((k, eqvar, newtype));
        }
        if all_unchanged {
            println!("returning disequal {:?} unchanged", self);
            return self.disequal_update_vars(proxy.parent)
        }

        println!("returning updated disequal {:?}", updated);
        return Updated(updated);
    }

    fn relevant(&self, proxy: &VarMap) -> bool {
        self.pairs.iter().any(|&(ref a, ref b, _)| proxy.contains_key(a) || proxy.contains_key(b))
    }
    fn update_vars(&mut self, proxy: &State) {
        for &mut (ref mut a, ref mut b, _) in self.pairs.iter_mut() {
            proxy.update_var(a);
            proxy.update_var(b);
        }
    }
}

