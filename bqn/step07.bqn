#!/usr/bin/env cbqn
lib ← •Import "lib.bqn"

# backward operations are mapped to `⁼`
NewVariable ⇐ {
    F 𝕩: ∞ F 𝕩;
  c F 𝕩:
    grad‿input‿creator ⇐ 1‿𝕩‿c
    Val ⇐ {
      S  n: input;
      S⁼ n: grad
    }
    Set ⇐ {
      S  𝕩: input ↩ 𝕩;
      S⁼ 𝕩: grad ↩ 𝕩
    }
}

_functionBuilder_ ⇐ {
  Self ⇐ { self ↩ 𝕩 }
  Fwd‿Bck‿input‿output ⇐ 𝔽‿𝔾‿¯∞‿∞
  Set ⇐ {
    S  𝕩: Fwd ↩ 𝕏;
    S⁼ x: Bck ↩ 𝕏
  }
  Propagate ⇐ {
      S  𝕩: input ↩ 𝕩, output ↩ self NewVariable Fwd 𝕩.Val @;
      S⁼ 𝕩: (𝕩.Val⁼ @) × Bck input.Val @;
    v S⁼ 𝕩: v.Set⁼ (𝕩.Val⁼ @) × Bck input.Val @, v
  }
  BackPropagate ⇐ {
      Rec 𝕩: input.creator Rec self;
    ∞ Rec f:
      f.input f.Propagate⁼ f.output
      f.input.grad;
    c Rec f:
      c.output f.Propagate⁼ f.output
      ! c.output.grad=f.input.grad
      c.input.creator Rec c
  }
  NumericalDiff ⇐ {
      Diff x: 0.0001 Diff x;
    e Diff x: x.input (+((2×e)⊸(÷˜)∘-○Fwd)-) e
  }
}

_function_ ⇐ { C ← 𝔽 _functionBuilder_ 𝔾, c.Self c }
_asFunction ⇐ { 𝔽 _function_ { F 𝕩: !𝕩≠𝕩 }  }
_followedBy_ ⇐ { 𝕘.Fwd∘𝕗.Fwd _asFunction }

# some function examples
Square ← 2⊸(⋆˜) _function_ (2⊸×)
Exp ← ⋆_function_⋆
Ln ← ⋆⁼_function_ (÷˜)
# composited functions
F ← Exp _followedBy_ Ln
G ← Square _followedBy_ Square
Chain ← Square _followedBy_ Exp _followedBy_ Square

# variables
x ← NewVariable 0
x10 ← NewVariable 10

"(Step02)Square x10 returns"         lib.Debug (square.Propagate x10).Val @
"(Step02)Exp x returns"              lib.Debug (exp.Propagate x).Val @
"(Step02)Ln x10 returns"             lib.Debug (ln.Propagate x10).Val @
"(Step02)Ln∘Exp x10 returns"         lib.Debug (f.Propagate x10).Val @
"(Step03)Square∘Square x10"          lib.Debug (g.Propagate x10).Val @
"(Step03)A∘B∘C 0.5 should be 1.648"  lib.Debug (chain.Propagate NewVariable 0.5).Val @
"(Step04)num. diff of Square at 2.0" lib.Debug square.NumericalDiff NewVariable 2
"(Step04)num. diff of A∘B∘C at 0.5"  lib.Debug chain.NumericalDiff NewVariable 0.5
## step 6.4
F06A ← 2⊸(⋆˜) _function_ (2⊸×)
F06B ← ⋆_function_⋆
F06C ← 2⊸(⋆˜) _function_ (2⊸×)
x06 ← NewVariable 0.5
y06 ← f06c.Propagate b06 ← f06b.Propagate a06 ← f06a.Propagate x06
x06 f06a.Propagate⁼ a06 f06b.Propagate⁼ b06 f06c.Propagate⁼ y06
"(Step06)back prop. C→B→A at 0.5"    lib.Debug x06.Val⁼ @
## step 7
"(Step07)Each function has self"     lib.Debug square=square.self
F07A ← 2⊸(⋆˜) _function_ (2⊸×)
F07B ← ⋆_function_⋆
F07C ← 2⊸(⋆˜) _function_ (2⊸×)
x07 ← NewVariable 0.5
y07 ← f07C.Propagate b07 ← f07B.Propagate a07 ← f07A.Propagate x07
f07c.BackPropagate @
"(Step07)auto back prop chain"       lib.Debug x07.Val⁼ @
