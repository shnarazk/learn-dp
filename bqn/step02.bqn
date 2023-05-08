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
"f is a namespace"                    lib.Debug f
"f is a sort of function `f.Apply 3`" lib.Debug f.Apply 4
"f has a secret store `f.test`"       lib.Debug f.test
