#!/usr/bin/env cbqn
lib ← •Import "lib.bqn"

# backward operations are mapped to `⁼`
NewVariable ⇐ {
  grad‿input ⇐ 1‿𝕩
  Val ⇐ { S n: input; S⁼ n: grad }
  Set ⇐ { S 𝕩: input ↩ 𝕩; S⁼ 𝕩: grad ↩ 𝕩 }
}

_function_ ⇐ {
  Fwd‿Bck‿input ⇐ 𝔽‿𝔾‿¯∞
  Set ⇐ { S 𝕩: Fwd ↩ 𝕏; S⁼ x: Bck ↩ 𝕏 }
  Propagate ⇐ {
      S  𝕩: input ↩ 𝕩, NewVariable Fwd 𝕩.Val @;
      S⁼ 𝕩: (𝕩.Val⁼ @) × Bck input.Val @;
    v S⁼ 𝕩: v.Set⁼ (𝕩.Val⁼ @) × Bck input.Val @, v
  }
  NumericalDiff ⇐ { Diff x: 0.0001 Diff x; e Diff x: x.input (+((2×e)⊸(÷˜)∘-○Fwd)-) e }
}

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

x ← NewVariable 0
x10 ← NewVariable 10

"Square x10"                 lib.Debug (square.Propagate x10).Val @
"Exp x"                      lib.Debug (exp.Propagate x).Val @
"Ln x10"                     lib.Debug (ln.Propagate x10).Val @
"Ln∘Exp x10"                 lib.Debug (f.Propagate x10).Val @
"Square∘Square x10"          lib.Debug (g.Propagate x10).Val @
"A∘B∘C 0.5 should be 1.648"  lib.Debug (chain.Propagate NewVariable 0.5).Val @
"num. diff of Square at 2.0" lib.Debug square.NumericalDiff NewVariable 2
"num. diff of A∘B∘C at 0.5"  lib.Debug chain.NumericalDiff NewVariable 0.5

## step 6.4
FA ← 2⊸(⋆˜) _function_ (2⊸×)
FB ← ⋆_function_⋆
FC ← 2⊸(⋆˜) _function_ (2⊸×)
x ↩ NewVariable 0.5
y ← fc.Propagate b ← fb.Propagate a ← fa.Propagate x
x fa.Propagate⁼ a fb.Propagate⁼ b fc.Propagate⁼ y
"back prop. C→B→A at 0.5" lib.Debug x.Val⁼ @
