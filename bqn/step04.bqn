#!/usr/bin/env cbqn
lib ← •Import "lib.bqn"

NewVariable ⇐ {F init:
  val ⇐ init
  Data ⇐ { 𝕤 ⋄ val }
  Set ⇐ { F v: val ↩ v }
}

_asFunction ⇐ { w _s:
  Forward ⇐ W
  Set ⇐ { Forward ↩ 𝕏 }
  Apply ⇐ { NewVariable Forward 𝕩.val }
  NumericalDiff ⇐ {
      S x: 0.0001 S x;
    e S x: x.val (+((2×e)⊸(÷˜)∘-○Forward)-) e
  }
}

_followedBy_ ⇐ { w _s_ x: (x.Forward∘w.Forward) _asFunction }

x ← NewVariable 1
Square ← 2⊸(⋆˜)_asFunction
x10 ← NewVariable 10
"Square x10"                    lib.Debug (square.Apply x10).Data @
Exp ← ⋆_asFunction
"Exp x"                         lib.Debug (exp.Apply x).Data @
Ln ← ⋆⁼_asFunction
"Ln x10"                        lib.Debug (ln.Apply x10).Data @
F ← Exp _followedBy_ Ln
"Ln∘Exp x10"                    lib.Debug (f.Apply x10).Data @
G ← Square _followedBy_ Square
"Square∘Square x10"             lib.Debug (g.Apply x10).Data @
Chain ← Square _followedBy_ Exp _followedBy_ Square
"A∘B∘C 0.5 should be 1.648"     lib.Debug (chain.Apply NewVariable 0.5).Data @
"num. diff of Square at 2.0"    lib.Debug square.NumericalDiff NewVariable 2
"num. diff of A∘B∘C at 0.5"     lib.Debug chain.NumericalDiff NewVariable 0.5
