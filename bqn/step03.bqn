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
}

_followedBy_ ⇐ { w _s_ x: (x.Forward∘w.Forward) _asFunction }

x ← NewVariable 1
Square ← 2⊸(⋆˜)_asFunction
x10 ← NewVariable 10
"Square x10"            lib.Debug (square.Apply x10).Data @
Exp ← ⋆_asFunction
"Exp x"                 lib.Debug (exp.Apply x).Data @
Ln ← ⋆⁼_asFunction
"Ln x10"                lib.Debug (ln.Apply x10).Data @
F ← Exp _followedBy_ Ln
"Ln∘Exp x10"            lib.Debug (f.Apply x10).Data @
G ← Square _followedBy_ Square
"Square∘Square x10"     lib.Debug (g.Apply x10).Data @
Chain ← Square _followedBy_ Exp _followedBy_ Square
"A∘B∘C 0.5 should be 1.648"     lib.Debug (chain.Apply NewVariable 0.5).Data @
