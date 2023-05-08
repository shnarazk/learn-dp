#!/usr/bin/env cbqn
lib ← •Import "lib.bqn"

NewVariable ⇐ {F init:
  val ⇐ init
  Data ⇐ { 𝕤 ⋄ val }
  Set ⇐ { F v: val ↩ v }
}

_asSimpleFunction ⇐ { w _s:
  test ⇐ 3
  Fn ⇐ W
  Set ⇐ { Fn ↩ 𝕏 }
  Apply ⇐ { Fn 𝕩 }
}

_asFunction ⇐ { w _s:
  test ⇐ 3
  Fn ⇐ W
  Set ⇐ { Fn ↩ 𝕏 }
  Apply ⇐ { NewVariable Fn 𝕩.val }
}

x ← NewVariable 1
F ← (1⊸+)_asSimpleFunction
"F or f is a namespace"             lib.Debug f
"f contains a function → f.Apply 3" lib.Debug f.Apply 3
"f has a secret store → f.test"     lib.Debug f.test
G ← (1⊸+)_asFunction
"G or g is a namespace"             lib.Debug g
"g contains a function → g.Apply x" lib.Debug g.Apply x
y ← g.Apply x
"The return value contains answer"  lib.Debug y.val
