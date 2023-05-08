#!/usr/bin/env cbqn
lib ← •Import "lib.bqn"

NewVariable ⇐ {F init: 
  val ⇐ init
  Data ⇐ { 𝕤 ⋄ val }
  Set ⇐ { F v: val ↩ v }
}

_asFunction ⇐ { w _s:
  test ⇐ 3
  Fn ⇐ W
  Set ⇐ { Fn ↩ 𝕏 }
  Apply ⇐ { Fn 𝕩 }
}

x ← NewVariable 1
F ← (1⊸+)_asFunction 
"F or f is a namespace"             lib.Debug f
"f contains a function → f.Apply 3" lib.Debug f.Apply 3
"f has a secret store → f.test"     lib.Debug f.test
