use glpk_sys::*;
use std::ffi::CString;
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut, Mul};
use std::os::raw::{c_int, c_uint, c_void};

use log::*;

#[derive(Copy, Clone, Debug)]
pub struct VarRef(c_int);
impl Mul<f64> for VarRef {
    type Output = Term;
    fn mul(self, coef: f64) -> Term {
        Term(self, coef)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Term(VarRef, f64);

#[derive(Copy, Clone, Debug)]
pub struct VarRefs {
    first: c_int,
    len: c_int,
}
impl VarRefs {
    pub fn get(&self, index: usize) -> VarRef {
        debug_assert!(
            index < self.len as usize,
            "index {} out of bounds {}",
            index,
            self.len
        );
        VarRef(self.first + index as c_int)
    }
    pub fn iter(&self) -> impl Iterator<Item = VarRef> {
        (self.first..self.first + self.len).map(VarRef)
    }
}

trait IntoGlp {
    type Output;
    fn into_glp(self) -> Self::Output;
}
impl IntoGlp for String {
    type Output = CString;
    fn into_glp(self) -> Self::Output {
        CString::new(self).expect("CString::new failed")
    }
}
impl IntoGlp for usize {
    type Output = c_int;
    fn into_glp(self) -> Self::Output {
        debug_assert!(self <= c_int::MAX as usize, "size {} is too big", self);
        self as c_int
    }
}
impl IntoGlp for Vec<Term> {
    type Output = (c_int, Vec<c_int>, Vec<f64>);
    fn into_glp(self) -> Self::Output {
        let len = self.len().into_glp();
        let mut vars = Vec::with_capacity(self.len() + 1);
        let mut coeffs = Vec::with_capacity(self.len() + 1);
        // GLPK doesn't believe in 0 indicies for some reason
        vars.push(0);
        coeffs.push(0.0);
        for Term(var, coef) in self.into_iter() {
            vars.push(var.0);
            coeffs.push(coef);
        }
        (len, vars, coeffs)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Maximize,
    Minimize,
}
impl IntoGlp for Direction {
    type Output = c_int;
    fn into_glp(self) -> Self::Output {
        match self {
            Self::Maximize => GLP_MAX as c_int,
            Self::Minimize => GLP_MIN as c_int,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Kind {
    Float,
    Int,
}
impl IntoGlp for Kind {
    type Output = c_int;
    fn into_glp(self) -> Self::Output {
        match self {
            Self::Float => GLP_CV as c_int,
            Self::Int => GLP_IV as c_int,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Bounds {
    Free,
    Lower(f64),
    Upper(f64),
    Double(f64, f64),
    Fixed(f64),
}
impl IntoGlp for Bounds {
    type Output = (c_int, f64, f64);
    fn into_glp(self) -> Self::Output {
        match self {
            Self::Free => (GLP_FR as c_int, 0.0, 0.0),
            Self::Lower(lower) => (GLP_LO as c_int, lower, 0.0),
            Self::Upper(upper) => (GLP_UP as c_int, 0.0, upper),
            Self::Double(lower, upper) => (GLP_DB as c_int, lower, upper),
            Self::Fixed(value) => (GLP_FX as c_int, value, 0.0),
        }
    }
}
#[derive(Copy, Clone, Debug)]
pub enum LoggingLevel {
    Off,
    Error,
    Info,
    Verbose,
}
impl IntoGlp for LoggingLevel {
    type Output = c_int;
    fn into_glp(self) -> Self::Output {
        match self {
            Self::Off => GLP_MSG_OFF as c_int,
            Self::Error => GLP_MSG_ERR as c_int,
            Self::Info => GLP_MSG_ON as c_int,
            Self::Verbose => GLP_MSG_ALL as c_int,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Var {
    pub name: String,
    pub kind: Kind,
    pub bounds: Bounds,
    pub objective: f64,
}

pub struct Expr {
    pub name: String,
    pub bounds: Bounds,
    pub terms: Vec<Term>,
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    /// double must have correct order, integers must have integer bounds
    InvalidBounds,
    /// only if no presolver
    NoInitSolution,
    /// only if presolver
    NotPrimalFeasible,
    /// only if presolver, aka unbounded solution
    NotDualFeasible,
    /// generic solver failure
    SolverFailed,
    /// relative mip gap tolerance was reached
    HitMipGapLimit,
    /// time limit was reached
    Timeout,
    /// stopped by the callback
    Stopped,
    /// glpk didn't follow docs
    Unknown,
}

pub enum Reason<'p> {
    AddLazyExprs(&'p mut Prob),
    NewBestSolution(&'p Prob),
}

// TODO split into Owned and Ref types to get rid of extra deref on methods?
//      that is the methods should be on `&mut Problem(glp_prob)` not `&mut Problem(*mut glp_prob)`
pub struct Problem(*mut Prob);
impl Problem {
    pub fn new() -> Self {
        Problem(unsafe { glp_create_prob() } as *mut Prob)
    }
}
impl Default for Problem {
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for Problem {
    fn clone(&self) -> Self {
        let clone = Self::new();
        unsafe {
            glp_copy_prob(
                clone.0 as *mut glp_prob,
                self.0 as *mut glp_prob,
                GLP_ON as c_int,
            )
        };
        clone
    }
}
impl Drop for Problem {
    fn drop(&mut self) {
        unsafe { glp_delete_prob(self.0 as *mut glp_prob) }
    }
}
impl Deref for Problem {
    type Target = Prob;
    fn deref(&self) -> &Prob {
        unsafe { &*self.0 }
    }
}
impl DerefMut for Problem {
    fn deref_mut(&mut self) -> &mut Prob {
        unsafe { &mut *self.0 }
    }
}
// TODO could implement Borrow, ToOwned, and AsRef to support generic programing

pub struct Prob(glp_prob);
impl Prob {
    fn as_ptr(&self) -> *mut glp_prob {
        self as *const Prob as *mut glp_prob
    }
    pub fn set_name(&mut self, name: String) {
        unsafe { glp_set_prob_name(self.as_ptr(), name.into_glp().as_ptr()) }
    }
    pub fn set_direction(&mut self, dir: Direction) {
        unsafe { glp_set_obj_dir(self.as_ptr(), dir.into_glp()) };
    }

    pub fn add_vars(&mut self, specs: Vec<Var>) -> VarRefs {
        let vars = self.alloc_vars(specs.len().into_glp());
        vars.iter()
            .zip(specs.into_iter())
            .for_each(|(var, spec)| self.init_var(var, spec));
        vars
    }
    pub fn add_var(&mut self, spec: Var) -> VarRef {
        let var = self.alloc_vars(1).get(0);
        self.init_var(var, spec);
        var
    }
    fn alloc_vars(&mut self, len: c_int) -> VarRefs {
        let first = unsafe {
            if len == 0 {
                glp_get_num_cols(self.as_ptr())
            } else {
                glp_add_cols(self.as_ptr(), len)
            }
        };
        VarRefs { first, len }
    }
    fn init_var(&mut self, var: VarRef, spec: Var) {
        let name = spec.name.into_glp();
        let kind = spec.kind.into_glp();
        let (bounds, lower, upper) = spec.bounds.into_glp();
        let objective = spec.objective;
        unsafe {
            glp_set_col_name(self.as_ptr(), var.0, name.as_ptr());
            #[allow(clippy::float_cmp)]
            // if kind == GLP_IV as c_int && bounds == GLP_DB as c_int && lower == 0.0 && upper == 1.0
            // {
            //     glp_set_col_kind(self.0, var.0, GLP_BV as c_int);
            // } else {
            glp_set_col_kind(self.as_ptr(), var.0, kind);
            glp_set_col_bnds(self.as_ptr(), var.0, bounds, lower, upper);
            // }
            glp_set_obj_coef(self.as_ptr(), var.0, objective);
        }
    }
    pub fn get_value(&self, var: VarRef) -> f64 {
        unsafe { glp_get_col_prim(self.as_ptr(), var.0) }
    }
    pub fn get_int_value(&self, var: VarRef) -> f64 {
        unsafe { glp_mip_col_val(self.as_ptr(), var.0) }
    }

    pub fn add_exprs(&mut self, specs: Vec<Expr>) {
        let exprs = self.alloc_exprs(specs.len().into_glp());
        exprs
            .iter()
            .zip(specs.into_iter())
            .for_each(|(var, spec)| self.init_expr(var, spec));
    }
    pub fn add_expr(&mut self, spec: Expr) {
        let var = self.alloc_exprs(1).get(0);
        self.init_expr(var, spec);
    }
    fn alloc_exprs(&mut self, len: c_int) -> VarRefs {
        let first = unsafe {
            if len == 0 {
                glp_get_num_rows(self.as_ptr())
            } else {
                glp_add_rows(self.as_ptr(), len)
            }
        };
        VarRefs { first, len }
    }
    fn init_expr(&mut self, var: VarRef, spec: Expr) {
        let name = spec.name.into_glp();
        let (bounds, lower, upper) = spec.bounds.into_glp();
        let (terms_len, vars, coeffs) = spec.terms.into_glp();
        unsafe {
            glp_set_row_name(self.as_ptr(), var.0, name.as_ptr());
            glp_set_row_bnds(self.as_ptr(), var.0, bounds, lower, upper);
            glp_set_mat_row(
                self.as_ptr(),
                var.0,
                terms_len,
                vars.as_ptr(),
                coeffs.as_ptr(),
            );
        }
    }

    pub fn optimize_mip<T: FnMut(Reason<'_>)>(&mut self, callback: T) -> Result<(), Error> {
        let mut options = MaybeUninit::uninit();
        unsafe { glp_init_iocp(options.as_mut_ptr()) };
        let mut options = unsafe { options.assume_init() };
        options.presolve = GLP_ON as c_int;
        options.binarize = GLP_ON as c_int;
        // docs say this is good for binary problems
        // being lazy and setting this here instead of exposing
        options.ps_heur = GLP_ON as c_int;

        assert_eq!(
            std::mem::size_of::<*mut c_void>(),
            std::mem::size_of::<&mut T>(),
            "cannot cast callback to a C pointer"
        );

        #[deny(unsafe_op_in_unsafe_fn)]
        unsafe extern "C" fn c_callback<T: FnMut(Reason<'_>)>(
            tree: *mut glp_tree,
            callback: *mut c_void,
        ) {
            let mip_callback = unsafe { &mut *(callback as *mut T) };
            match unsafe { glp_ios_reason(tree) } as c_uint {
                // request for lazy exprs to be added
                GLP_IROWGEN => {
                    let problem = unsafe { &mut *(glp_ios_get_prob(tree) as *mut Prob) };
                    mip_callback(Reason::AddLazyExprs(problem));
                }
                // request for cuts to be added
                // remember that cuts cannot remove integral solutions
                // they are instead for cutting a fractional corner into multiple (hopefully) integral corners
                // GLP_ICUTGEN => {
                // }
                GLP_IBINGO => {
                    let problem = unsafe { &*(glp_ios_get_prob(tree) as *mut Prob) };
                    mip_callback(Reason::NewBestSolution(problem));
                }
                _ => {}
            }
        }
        options.cb_func = Some(c_callback::<T>);
        options.cb_info = &callback as *const T as *mut c_void;

        let err = unsafe { glp_intopt(self.as_ptr(), &options as *const glp_iocp) };
        match err as c_uint {
            0 => Ok(()),
            GLP_EBOUND => Err(Error::InvalidBounds),
            GLP_EROOT => Err(Error::NoInitSolution),
            GLP_ENOPFS => Err(Error::NotPrimalFeasible),
            GLP_ENODFS => Err(Error::NotDualFeasible),
            GLP_EFAIL => Err(Error::SolverFailed),
            GLP_EMIPGAP => Err(Error::HitMipGapLimit),
            GLP_ETMLIM => Err(Error::Timeout),
            GLP_ESTOP => Err(Error::Stopped),
            _ => {
                warn!("Unknown intopt error {}", err);
                Err(Error::Unknown)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_set_name() {
        let mut problem = Problem::new();
        problem.set_name("abc".to_owned());
        // TODO verify name
    }
}
