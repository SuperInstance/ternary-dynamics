//! # Ternary Dynamics
//!
//! Dynamic markings for agent resource control. Inspired by musical dynamics (pp, p, mp, mf,
//! f, ff), this crate maps abstract dynamic markings to concrete resource allocations for
//! ternary agents. Agents can swell, diminish, and accent their resource usage over time
//! using musical metaphors.
//!
//! ## Core Concepts
//!
//! - **DynamicMark**: Static intensity levels (pp → ff) with resource fractions.
//! - **DynamicCurve**: Shape dynamics over time — crescendo, diminuendo, sforzando.
//! - **DynamicContext**: Interpret dynamics based on surrounding context.
//! - **DynamicBalance**: Balance resource allocation across multiple agents.
//! - **DynamicInterpreter**: Map abstract dynamics to concrete resource values.

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::fmt;

// ── DynamicMark ──────────────────────────────────────────────────────────────

/// A musical dynamic marking mapped to a resource fraction [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DynamicMark {
    Ppp,
    Pp,
    P,
    Mp,
    Mf,
    F,
    Ff,
    Fff,
}

impl DynamicMark {
    pub fn all() -> &'static [DynamicMark; 8] {
        &[DynamicMark::Ppp, DynamicMark::Pp, DynamicMark::P, DynamicMark::Mp,
          DynamicMark::Mf, DynamicMark::F, DynamicMark::Ff, DynamicMark::Fff]
    }

    pub fn intensity(&self) -> f64 {
        match self {
            DynamicMark::Ppp => 0.0625,
            DynamicMark::Pp => 0.125,
            DynamicMark::P => 0.25,
            DynamicMark::Mp => 0.375,
            DynamicMark::Mf => 0.5,
            DynamicMark::F => 0.75,
            DynamicMark::Ff => 0.875,
            DynamicMark::Fff => 1.0,
        }
    }

    pub fn resource_amount(&self, total_budget: f64) -> f64 {
        self.intensity() * total_budget
    }

    pub fn notation(&self) -> &'static str {
        match self {
            DynamicMark::Ppp => "ppp", DynamicMark::Pp => "pp", DynamicMark::P => "p",
            DynamicMark::Mp => "mp", DynamicMark::Mf => "mf", DynamicMark::F => "f",
            DynamicMark::Ff => "ff", DynamicMark::Fff => "fff",
        }
    }

    pub fn default_mark() -> Self { DynamicMark::Mf }

    pub fn louder(&self) -> Self {
        match self {
            DynamicMark::Ppp => DynamicMark::Pp, DynamicMark::Pp => DynamicMark::P,
            DynamicMark::P => DynamicMark::Mp, DynamicMark::Mp => DynamicMark::Mf,
            DynamicMark::Mf => DynamicMark::F, DynamicMark::F => DynamicMark::Ff,
            DynamicMark::Ff => DynamicMark::Fff, DynamicMark::Fff => DynamicMark::Fff,
        }
    }

    pub fn softer(&self) -> Self {
        match self {
            DynamicMark::Ppp => DynamicMark::Ppp, DynamicMark::Pp => DynamicMark::Ppp,
            DynamicMark::P => DynamicMark::Pp, DynamicMark::Mp => DynamicMark::P,
            DynamicMark::Mf => DynamicMark::Mp, DynamicMark::F => DynamicMark::Mf,
            DynamicMark::Ff => DynamicMark::F, DynamicMark::Fff => DynamicMark::Ff,
        }
    }
}

impl fmt::Display for DynamicMark {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.notation()) }
}

// ── DynamicCurve ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum CurveShape {
    Flat(DynamicMark),
    Crescendo { from: DynamicMark, to: DynamicMark },
    Diminuendo { from: DynamicMark, to: DynamicMark },
    Sforzando { base: DynamicMark, accent: DynamicMark, position: usize },
    Swell { base: DynamicMark, peak: DynamicMark },
    Custom(Vec<DynamicMark>),
}

#[derive(Debug, Clone)]
pub struct DynamicCurve {
    shape: CurveShape,
    length: usize,
}

impl DynamicCurve {
    pub fn flat(mark: DynamicMark, length: usize) -> Self {
        Self { shape: CurveShape::Flat(mark), length: length.max(1) }
    }

    pub fn crescendo(from: DynamicMark, to: DynamicMark, length: usize) -> Self {
        Self { shape: CurveShape::Crescendo { from, to }, length: length.max(2) }
    }

    pub fn diminuendo(from: DynamicMark, to: DynamicMark, length: usize) -> Self {
        Self { shape: CurveShape::Diminuendo { from, to }, length: length.max(2) }
    }

    pub fn sforzando(base: DynamicMark, accent: DynamicMark, position: usize, length: usize) -> Self {
        let length = length.max(position + 1).max(2);
        Self { shape: CurveShape::Sforzando { base, accent, position }, length }
    }

    pub fn swell(base: DynamicMark, peak: DynamicMark, length: usize) -> Self {
        Self { shape: CurveShape::Swell { base, peak }, length: length.max(3) }
    }

    pub fn custom(marks: Vec<DynamicMark>) -> Self {
        let length = marks.len().max(1);
        Self { shape: CurveShape::Custom(marks), length }
    }

    pub fn evaluate(&self) -> Vec<f64> {
        match &self.shape {
            CurveShape::Flat(mark) => vec![mark.intensity(); self.length],
            CurveShape::Crescendo { from, to } | CurveShape::Diminuendo { from, to } => {
                let from_idx = Self::mark_index(from);
                let to_idx = Self::mark_index(to);
                Self::interpolate_marks(from_idx, to_idx, self.length)
            }
            CurveShape::Sforzando { base, accent, position } => {
                let mut result = vec![base.intensity(); self.length];
                if *position < self.length { result[*position] = accent.intensity(); }
                result
            }
            CurveShape::Swell { base, peak } => {
                let base_idx = Self::mark_index(base);
                let peak_idx = Self::mark_index(peak);
                let mid = self.length / 2;
                (0..self.length).map(|i| {
                    if i <= mid {
                        let t = if mid == 0 { 0.0 } else { i as f64 / mid as f64 };
                        Self::intensity_at_index(base_idx as f64 + t * (peak_idx as f64 - base_idx as f64))
                    } else {
                        let t = (i - mid) as f64 / (self.length - 1 - mid) as f64;
                        Self::intensity_at_index(peak_idx as f64 + t * (base_idx as f64 - peak_idx as f64))
                    }
                }).collect()
            }
            CurveShape::Custom(marks) => marks.iter().map(|m| m.intensity()).collect(),
        }
    }

    pub fn len(&self) -> usize { self.length }
    pub fn is_empty(&self) -> bool { self.length == 0 }
    pub fn shape(&self) -> &CurveShape { &self.shape }

    fn mark_index(mark: &DynamicMark) -> usize {
        DynamicMark::all().iter().position(|m| m == mark).unwrap()
    }

    fn interpolate_marks(from_idx: usize, to_idx: usize, steps: usize) -> Vec<f64> {
        (0..steps).map(|i| {
            let t = if steps <= 1 { 0.0 } else { i as f64 / (steps - 1) as f64 };
            Self::intensity_at_index(from_idx as f64 + t * (to_idx as f64 - from_idx as f64))
        }).collect()
    }

    fn intensity_at_index(idx: f64) -> f64 {
        let all = DynamicMark::all();
        let low = (idx.floor() as usize).min(all.len() - 1);
        let high = (low + 1).min(all.len() - 1);
        let frac = idx - idx.floor();
        all[low].intensity() + frac * (all[high].intensity() - all[low].intensity())
    }
}

// ── DynamicContext ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DynamicContext {
    adjustments: HashMap<String, f64>,
}

impl DynamicContext {
    pub fn new() -> Self { Self { adjustments: HashMap::new() } }

    pub fn interpret(&self, current: DynamicMark, prev: Option<DynamicMark>, next: Option<DynamicMark>) -> f64 {
        let base = current.intensity();
        let mut adj = 0.0;
        if let Some(p) = prev {
            let contrast = p.intensity() - base;
            if contrast > 0.3 { adj -= 0.05; } else if contrast < -0.3 { adj += 0.05; }
        }
        if let Some(n) = next {
            if n.intensity() - base > 0.3 { adj -= 0.03; }
        }
        (base + adj).clamp(0.0, 1.0)
    }

    pub fn interpret_sequence(&self, marks: &[DynamicMark]) -> Vec<f64> {
        marks.iter().enumerate().map(|(i, &m)| {
            let prev = if i > 0 { Some(marks[i - 1]) } else { None };
            let next = if i + 1 < marks.len() { Some(marks[i + 1]) } else { None };
            self.interpret(m, prev, next)
        }).collect()
    }

    pub fn add_adjustment(&mut self, name: impl Into<String>, delta: f64) {
        self.adjustments.insert(name.into(), delta);
    }
    pub fn get_adjustment(&self, name: &str) -> Option<f64> { self.adjustments.get(name).copied() }
}

impl Default for DynamicContext { fn default() -> Self { Self::new() } }

// ── DynamicBalance ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DynamicBalance {
    agents: HashMap<String, DynamicMark>,
    total_budget: f64,
}

impl DynamicBalance {
    pub fn new(total_budget: f64) -> Self { Self { agents: HashMap::new(), total_budget } }

    pub fn set_agent(&mut self, name: impl Into<String>, mark: DynamicMark) {
        self.agents.insert(name.into(), mark);
    }
    pub fn remove_agent(&mut self, name: &str) { self.agents.remove(name); }
    pub fn get_agent(&self, name: &str) -> Option<DynamicMark> { self.agents.get(name).copied() }
    pub fn agent_count(&self) -> usize { self.agents.len() }

    pub fn allocate(&self) -> Vec<(String, f64)> {
        let total_intensity: f64 = self.agents.values().map(|m| m.intensity()).sum();
        if total_intensity == 0.0 {
            return self.agents.keys().map(|k| (k.clone(), 0.0)).collect();
        }
        let mut results: Vec<_> = self.agents.iter().map(|(name, mark)| {
            (name.clone(), (mark.intensity() / total_intensity) * self.total_budget)
        }).collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        results
    }

    pub fn total_allocated(&self) -> f64 { self.allocate().iter().map(|(_, v)| v).sum() }
    pub fn is_balanced(&self) -> bool { (self.total_allocated() - self.total_budget).abs() < 1e-9 }
}

// ── DynamicInterpreter ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DynamicInterpreter {
    budget: f64,
    overrides: HashMap<String, f64>,
}

impl DynamicInterpreter {
    pub fn new(budget: f64) -> Self { Self { budget, overrides: HashMap::new() } }

    pub fn set_override(&mut self, name: impl Into<String>, intensity: f64) {
        self.overrides.insert(name.into(), intensity.clamp(0.0, 1.0));
    }

    pub fn interpret(&self, mark: DynamicMark) -> f64 { mark.intensity() * self.budget }

    pub fn interpret_with_context(&self, mark: DynamicMark, context: &str) -> f64 {
        let base = self.overrides.get(context).copied().unwrap_or_else(|| mark.intensity());
        base * self.budget
    }

    pub fn interpret_curve(&self, curve: &DynamicCurve) -> Vec<f64> {
        curve.evaluate().iter().map(|i| i * self.budget).collect()
    }

    pub fn budget(&self) -> f64 { self.budget }
    pub fn available_at(&self, mark: DynamicMark) -> f64 { self.budget - self.interpret(mark) }
}

// ══════════════════════════════════════════════════════════════════════════════
// Tests
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_intensity_ordering() {
        let all = DynamicMark::all();
        for w in all.windows(2) {
            assert!(w[0].intensity() < w[1].intensity());
        }
    }

    #[test]
    fn mark_mf_is_half() { assert!((DynamicMark::Mf.intensity() - 0.5).abs() < 1e-12); }

    #[test]
    fn mark_ppp_is_minimal() { assert!((DynamicMark::Ppp.intensity() - 0.0625).abs() < 1e-12); }

    #[test]
    fn mark_fff_is_max() { assert!((DynamicMark::Fff.intensity() - 1.0).abs() < 1e-12); }

    #[test]
    fn mark_resource_amount() { assert!((DynamicMark::Mf.resource_amount(100.0) - 50.0).abs() < 1e-12); }

    #[test]
    fn mark_notation() {
        assert_eq!(DynamicMark::Pp.notation(), "pp");
        assert_eq!(DynamicMark::Ff.notation(), "ff");
    }

    #[test]
    fn mark_display() { assert_eq!(DynamicMark::F.to_string(), "f"); }

    #[test]
    fn mark_louder() {
        assert_eq!(DynamicMark::P.louder(), DynamicMark::Mp);
        assert_eq!(DynamicMark::Fff.louder(), DynamicMark::Fff);
    }

    #[test]
    fn mark_softer() {
        assert_eq!(DynamicMark::F.softer(), DynamicMark::Mf);
        assert_eq!(DynamicMark::Ppp.softer(), DynamicMark::Ppp);
    }

    #[test]
    fn mark_all_count() { assert_eq!(DynamicMark::all().len(), 8); }

    #[test]
    fn mark_default_is_mf() { assert_eq!(DynamicMark::default_mark(), DynamicMark::Mf); }

    #[test]
    fn curve_flat() {
        let curve = DynamicCurve::flat(DynamicMark::F, 5);
        let vals = curve.evaluate();
        assert_eq!(vals.len(), 5);
        for v in &vals { assert!((v - DynamicMark::F.intensity()).abs() < 1e-12); }
    }

    #[test]
    fn curve_crescendo() {
        let curve = DynamicCurve::crescendo(DynamicMark::P, DynamicMark::F, 5);
        let vals = curve.evaluate();
        assert_eq!(vals.len(), 5);
        assert!(vals[0] < vals[4]);
    }

    #[test]
    fn curve_diminuendo() {
        let curve = DynamicCurve::diminuendo(DynamicMark::F, DynamicMark::P, 5);
        let vals = curve.evaluate();
        assert!(vals[0] > vals[4]);
    }

    #[test]
    fn curve_sforzando_accent() {
        let curve = DynamicCurve::sforzando(DynamicMark::P, DynamicMark::Fff, 2, 5);
        let vals = curve.evaluate();
        assert_eq!(vals.len(), 5);
        assert!((vals[2] - DynamicMark::Fff.intensity()).abs() < 1e-12);
        assert!((vals[0] - DynamicMark::P.intensity()).abs() < 1e-12);
    }

    #[test]
    fn curve_swell() {
        let curve = DynamicCurve::swell(DynamicMark::P, DynamicMark::F, 7);
        let vals = curve.evaluate();
        assert_eq!(vals.len(), 7);
        let mid = vals.len() / 2;
        assert!(vals[mid] >= vals[0]);
        assert!(vals[mid] >= vals[6]);
    }

    #[test]
    fn curve_custom() {
        let curve = DynamicCurve::custom(vec![DynamicMark::Pp, DynamicMark::Mf, DynamicMark::Ff]);
        let vals = curve.evaluate();
        assert_eq!(vals.len(), 3);
        assert!((vals[0] - DynamicMark::Pp.intensity()).abs() < 1e-12);
    }

    #[test]
    fn curve_length() { assert_eq!(DynamicCurve::flat(DynamicMark::Mf, 10).len(), 10); }

    #[test]
    fn curve_not_empty() { assert!(!DynamicCurve::flat(DynamicMark::Mf, 1).is_empty()); }

    #[test]
    fn curve_shape_accessor() {
        let curve = DynamicCurve::crescendo(DynamicMark::P, DynamicMark::F, 4);
        match curve.shape() {
            CurveShape::Crescendo { from, to } => { assert_eq!(*from, DynamicMark::P); assert_eq!(*to, DynamicMark::F); }
            _ => panic!("expected crescendo"),
        }
    }

    #[test]
    fn context_solo_mark() {
        let ctx = DynamicContext::new();
        assert!((ctx.interpret(DynamicMark::Mf, None, None) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn context_after_loud_feels_softer() {
        let ctx = DynamicContext::new();
        let solo = ctx.interpret(DynamicMark::P, None, None);
        let after_loud = ctx.interpret(DynamicMark::P, Some(DynamicMark::Fff), None);
        assert!(after_loud < solo, "p after fff should feel softer");
    }

    #[test]
    fn context_after_quiet_feels_louder() {
        let ctx = DynamicContext::new();
        let solo = ctx.interpret(DynamicMark::F, None, None);
        let after_quiet = ctx.interpret(DynamicMark::F, Some(DynamicMark::Ppp), None);
        assert!(after_quiet > solo, "f after ppp should feel louder");
    }

    #[test]
    fn context_sequence() {
        let ctx = DynamicContext::new();
        let seq = vec![DynamicMark::Pp, DynamicMark::Mf, DynamicMark::Ff];
        let interpreted = ctx.interpret_sequence(&seq);
        assert_eq!(interpreted.len(), 3);
        assert!(interpreted.iter().all(|&v| v >= 0.0 && v <= 1.0));
    }

    #[test]
    fn context_adjustments() {
        let mut ctx = DynamicContext::new();
        ctx.add_adjustment("night", -0.1);
        assert_eq!(ctx.get_adjustment("night"), Some(-0.1));
        assert_eq!(ctx.get_adjustment("day"), None);
    }

    #[test]
    fn balance_two_agents() {
        let mut bal = DynamicBalance::new(100.0);
        bal.set_agent("alice", DynamicMark::F);
        bal.set_agent("bob", DynamicMark::P);
        let alloc = bal.allocate();
        assert_eq!(alloc.len(), 2);
        assert!(alloc[0].1 > alloc[1].1, "louder agent gets more");
        assert!(bal.is_balanced());
    }

    #[test]
    fn balance_equal_agents() {
        let mut bal = DynamicBalance::new(100.0);
        bal.set_agent("a", DynamicMark::Mf);
        bal.set_agent("b", DynamicMark::Mf);
        let alloc = bal.allocate();
        assert!((alloc[0].1 - 50.0).abs() < 1e-9);
        assert!((alloc[1].1 - 50.0).abs() < 1e-9);
    }

    #[test]
    fn balance_remove_agent() {
        let mut bal = DynamicBalance::new(100.0);
        bal.set_agent("a", DynamicMark::Mf);
        bal.remove_agent("a");
        assert_eq!(bal.agent_count(), 0);
    }

    #[test]
    fn balance_empty() {
        let bal = DynamicBalance::new(100.0);
        assert_eq!(bal.allocate().len(), 0);
        assert!(bal.is_balanced() || bal.total_allocated() == 0.0);
    }

    #[test]
    fn interpreter_basic() {
        let interp = DynamicInterpreter::new(200.0);
        assert!((interp.interpret(DynamicMark::Mf) - 100.0).abs() < 1e-12);
    }

    #[test]
    fn interpreter_budget() { assert_eq!(DynamicInterpreter::new(42.0).budget(), 42.0); }

    #[test]
    fn interpreter_available() {
        let interp = DynamicInterpreter::new(100.0);
        assert!((interp.available_at(DynamicMark::Fff) - 0.0).abs() < 1e-12);
    }

    #[test]
    fn interpreter_with_override() {
        let mut interp = DynamicInterpreter::new(100.0);
        interp.set_override("emergency", 1.0);
        let normal = interp.interpret_with_context(DynamicMark::P, "normal");
        let emergency = interp.interpret_with_context(DynamicMark::P, "emergency");
        assert!(emergency > normal);
    }

    #[test]
    fn interpreter_curve() {
        let interp = DynamicInterpreter::new(100.0);
        let curve = DynamicCurve::flat(DynamicMark::Mf, 3);
        let resources = interp.interpret_curve(&curve);
        assert_eq!(resources.len(), 3);
        for r in &resources { assert!((r - 50.0).abs() < 1e-12); }
    }
}
